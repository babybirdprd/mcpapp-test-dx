//! MCP Connection Manager
//!
//! Manages multiple MCP server connections and provides a unified interface
//! for the host to interact with them.

use crate::protocol::*;
use crate::host::{McpServerConnection, ConnectionState, ConnectionEvent, HostState};
use crate::host::transport::{McpTransport, StdioTransport};
use rmcp::model::{CallToolResult, Content, ListToolsResult, ListResourcesResult, ReadResourceResult, Resource, ResourceContents, Tool, Meta};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Manages connections to MCP servers
#[derive(Debug, Clone)]
pub struct ConnectionManager {
    /// Active connections
    connections: Arc<RwLock<HashMap<String, McpServerConnection>>>,
    /// Event sender
    event_tx: mpsc::UnboundedSender<ConnectionEvent>,
    /// Event receiver (kept for distribution)
    #[allow(dead_code)]
    event_rx: Arc<RwLock<mpsc::UnboundedReceiver<ConnectionEvent>>>,
    /// Host state for capabilities
    pub host_state: HostState,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(host_state: HostState) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx: Arc::new(RwLock::new(event_rx)),
            host_state,
        }
    }
    
    /// Connect to an MCP server via stdio
    pub async fn connect_stdio(
        &self,
        command: impl Into<String>,
        args: Vec<String>,
    ) -> Result<String, ConnectionError> {
        let connection_id = uuid::Uuid::new_v4().to_string();
        let command = command.into();
        
        log::info!("Connecting to MCP server: {} {:?}", command, args);
        
        // Create transport
        let mut transport = StdioTransport::new(&command, &args).await
            .map_err(|e| ConnectionError::Transport(e.to_string()))?;
        
        // Create connection
        let mut connection = McpServerConnection::new(&connection_id);
        connection.set_state(ConnectionState::Initializing);
        
        // Perform MCP initialize handshake
        let init_request = self.build_initialize_request();
        let init_response = transport.send_request(init_request).await
            .map_err(|e| ConnectionError::Transport(e.to_string()))?;
        
        if let Some(error) = init_response.error {
            return Err(ConnectionError::Initialize(error.message));
        }
        
        let result = init_response.result
            .ok_or_else(|| ConnectionError::Initialize("No result in initialize response".to_string()))?;
        
        // Update connection with capabilities
        connection.set_capabilities(&result);
        
        // Perform capability negotiation
        let host_caps = self.host_state.to_capabilities();
        connection.negotiate_capabilities(&host_caps, None);
        
        // Fetch tools and resources BEFORE moving transport to background task
        let tools_request = JsonRpcRequest::new("tools/list", None);
        let tools_response = transport.send_request(tools_request).await
            .map_err(|e| ConnectionError::Transport(e.to_string()))?;
        
        if let Some(tools_result) = tools_response.result {
            if let Ok(list_tools) = serde_json::from_value::<ListToolsResult>(tools_result) {
                connection.update_tools(list_tools.tools).await;
            }
        }

        let resources_request = JsonRpcRequest::new("resources/list", None);
        let resources_response = transport.send_request(resources_request).await
            .map_err(|e| ConnectionError::Transport(e.to_string()))?;

        if let Some(resources_result) = resources_response.result {
            if let Ok(list_resources) = serde_json::from_value::<ListResourcesResult>(resources_result) {
                connection.update_resources(list_resources.resources).await;
            }
        }
        
        connection.set_state(ConnectionState::Ready);
        
        // Send initialized notification
        let initialized_notif = crate::protocol::JsonRpcNotification::new("notifications/initialized", None);
        transport.send_notification(initialized_notif).await
            .map_err(|e| ConnectionError::Transport(e.to_string()))?;
        
        // Store connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_id.clone(), connection.clone());
        }
        
        // Start background task for this connection
        self.start_connection_task(connection_id.clone(), transport);
        
        log::info!("Connected to MCP server: {} (supports UI: {})", 
            connection_id, 
            connection.supports_ui_extension
        );
        
        Ok(connection_id)
    }

    /// Connect to the embedded server directly using MemoryTransport
    pub async fn connect_embedded(&self) -> Result<String, ConnectionError> {
        let connection_id = "embedded".to_string();
        log::info!("Connecting to embedded MCP server");

        let (mut client_transport, mut server_transport) = crate::host::transport::MemoryTransport::create_pair();
        
        // Create server
        let server = crate::server::EmbeddedServer::new();
        
        // Start server task
        tokio::spawn(async move {
            loop {
                match server_transport.receive_message().await {
                    Ok(Some(request)) => {
                        let id = request.get("id").cloned();
                        let method = request.get("method").and_then(|v| v.as_str()).unwrap_or("");
                        let params = request.get("params").cloned().unwrap_or(json!({}));

                        let response = match method {
                            "initialize" => {
                                match server.handle_initialize(params).await {
                                    Ok(res) => json!({ "jsonrpc": "2.0", "id": id, "result": res }),
                                    Err(e) => json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": e } })
                                }
                            }
                            "tools/list" => {
                                match server.list_tools().await {
                                    Ok(res) => json!({ "jsonrpc": "2.0", "id": id, "result": res }),
                                    Err(e) => json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": e } })
                                }
                            }
                            "resources/list" => {
                                match server.list_resources().await {
                                    Ok(res) => json!({ "jsonrpc": "2.0", "id": id, "result": res }),
                                    Err(e) => json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": e } })
                                }
                            }
                            "notifications/initialized" => continue,
                            _ => json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": "Method not found" } })
                        };
                        let _ = server_transport.send_raw(response).await;
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
        });

        // Create connection
        let mut connection = McpServerConnection::new(&connection_id);
        connection.set_state(ConnectionState::Initializing);

        // Handshake
        let init_request = self.build_initialize_request();
        let init_response = client_transport.send_request(init_request).await
            .map_err(|e| ConnectionError::Transport(e.to_string()))?;
        
        connection.set_capabilities(&init_response.result.unwrap());
        connection.negotiate_capabilities(&self.host_state.to_capabilities(), None);

        // Fetch tools
        let tools_response = client_transport.send_request(JsonRpcRequest::new("tools/list", None)).await
            .map_err(|e| ConnectionError::Transport(e.to_string()))?;
        if let Some(tools_result) = tools_response.result {
            let list_tools: ListToolsResult = serde_json::from_value(tools_result).unwrap();
            connection.update_tools(list_tools.tools).await;
        }

        // Fetch resources
        let resources_response = client_transport.send_request(JsonRpcRequest::new("resources/list", None)).await
            .map_err(|e| ConnectionError::Transport(e.to_string()))?;
        if let Some(resources_result) = resources_response.result {
            let list_resources: ListResourcesResult = serde_json::from_value(resources_result).unwrap();
            connection.update_resources(list_resources.resources).await;
        }

        connection.set_state(ConnectionState::Ready);
        let _ = client_transport.send_notification(JsonRpcNotification::new("notifications/initialized", None)).await;

        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_id.clone(), connection);
        }

        Ok(connection_id)
    }
    
    /// Build initialize request
    fn build_initialize_request(&self) -> crate::protocol::JsonRpcRequest {
        let params = json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {
                "experimental": {
                    UI_EXTENSION_ID: self.host_state.to_capabilities()
                }
            },
            "clientInfo": {
                "name": self.host_state.name,
                "version": self.host_state.version
            }
        });
        
        crate::protocol::JsonRpcRequest::new("initialize", Some(params))
    }
    
    /// Start background task for handling server messages
    fn start_connection_task(&self, connection_id: String, mut transport: StdioTransport) {
        let event_tx = self.event_tx.clone();
        let connections = self.connections.clone();
        
        tokio::spawn(async move {
            loop {
                match transport.receive_message().await {
                    Ok(Some(message)) => {
                        // Parse and handle message
                        if let Some(method) = message.get("method").and_then(|m| m.as_str()) {
                            // It's a notification or request
                            let params = message.get("params").cloned();
                            
                            if method.starts_with("notifications/") {
                                // Handle notifications
                                if method == "notifications/tools/list_changed" {
                                    let _ = event_tx.send(ConnectionEvent::StateChanged {
                                        connection_id: connection_id.clone(),
                                        state: ConnectionState::Ready,
                                    });
                                } else if method == "notifications/resources/list_changed" {
                                    let _ = event_tx.send(ConnectionEvent::StateChanged {
                                        connection_id: connection_id.clone(),
                                        state: ConnectionState::Ready,
                                    });
                                }
                                
                                let _ = event_tx.send(ConnectionEvent::Notification {
                                    connection_id: connection_id.clone(),
                                    method: method.to_string(),
                                    params,
                                });
                            }
                        }
                    }
                    Ok(None) => {
                        // Connection closed
                        let _ = event_tx.send(ConnectionEvent::Closed {
                            connection_id: connection_id.clone(),
                        });
                        
                        // Update connection state
                        if let Some(conn) = connections.write().await.get_mut(&connection_id) {
                            conn.set_state(ConnectionState::Disconnected);
                        }
                        break;
                    }
                    Err(e) => {
                        let _ = event_tx.send(ConnectionEvent::Error {
                            connection_id: connection_id.clone(),
                            error: e.to_string(),
                        });
                    }
                }
            }
        });
    }
    
    /// Get a connection by ID
    pub async fn get_connection(&self, id: &str) -> Option<McpServerConnection> {
        self.connections.read().await.get(id).cloned()
    }
    
    /// Get all connections
    pub async fn get_all_connections(&self) -> Vec<McpServerConnection> {
        self.connections.read().await.values().cloned().collect()
    }
    
    /// Get all tools from all connections
    pub async fn get_all_tools(&self) -> Vec<(String, Tool)> {
        let mut all_tools = Vec::new();
        let connections = self.connections.read().await;
        
        for (conn_id, conn) in connections.iter() {
            let tools = conn.get_tools().await;
            for tool in tools {
                all_tools.push((conn_id.clone(), tool));
            }
        }
        
        all_tools
    }
    
    /// Get all UI resources from all connections
    pub async fn get_all_ui_resources(&self) -> Vec<(String, UiResource)> {
        let mut all_resources = Vec::new();
        let connections = self.connections.read().await;
        
        for (conn_id, conn) in connections.iter() {
            let resources = conn.get_ui_resources().await;
            for resource in resources {
                all_resources.push((conn_id.clone(), resource));
            }
        }
        
        all_resources
    }
    
    /// Get tools with UI metadata
    pub async fn get_tools_with_ui(&self) -> Vec<(String, Tool, String)> {
        let mut result = Vec::new();
        let connections = self.connections.read().await;
        
        for (conn_id, conn) in connections.iter() {
            let tools_with_ui = conn.get_tools_with_ui().await;
            for (tool, uri) in tools_with_ui {
                result.push((conn_id.clone(), tool, uri));
            }
        }
        
        result
    }
    
    /// Call a tool on a specific connection
    pub async fn call_tool(
        &self,
        connection_id: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<CallToolResult, ConnectionError> {
        let connection = self.get_connection(connection_id).await
            .ok_or_else(|| ConnectionError::NotFound(connection_id.to_string()))?;
        
        if !connection.is_ready() {
            return Err(ConnectionError::NotReady(connection_id.to_string()));
        }
        
        log::info!("Calling tool {} on connection {}", tool_name, connection_id);

        if connection_id == "embedded" {
            let server = crate::server::EmbeddedServer::new();
            return server.call_tool(tool_name, arguments).await
                .map_err(|e| ConnectionError::ToolNotFound(e));
        }
        
        // For external connections, in a full implementation we'd send a request.
        // For this barebones demo, we'll return a basic result.
        Ok(CallToolResult {
            content: vec![Content::text(format!("Tool {} called with {:?}", tool_name, arguments))],
            is_error: None,
            structured_content: Some(arguments),
            meta: None,
        })
    }
    
    /// Read a UI resource from a specific connection
    pub async fn read_ui_resource(
        &self,
        connection_id: &str,
        uri: &str,
    ) -> Result<UiResourceContent, ConnectionError> {
        let connection = self.get_connection(connection_id).await
            .ok_or_else(|| ConnectionError::NotFound(connection_id.to_string()))?;
        
        if !connection.is_ready() {
            return Err(ConnectionError::NotReady(connection_id.to_string()));
        }
        
        // Check if resource exists
        let resource = connection.find_ui_resource(uri).await
            .ok_or_else(|| ConnectionError::ResourceNotFound(uri.to_string()))?;

        if connection_id == "embedded" {
            let server = crate::server::EmbeddedServer::new();
            match server.read_resource(uri).await {
                Ok(res) => {
                    if let Some(content) = res.contents.into_iter().next() {
                        let val = serde_json::to_value(&content).unwrap_or_default();
                        let text = val.get("text").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let blob = val.get("blob").and_then(|v| v.as_str()).map(|s| s.to_string());
                        
                        return Ok(UiResourceContent {
                            uri: uri.to_string(),
                            mime_type: resource.mime_type.clone(),
                            text,
                            blob,
                            _meta: resource._meta.clone(),
                        });
                    }
                }
                Err(e) => return Err(ConnectionError::ResourceNotFound(e)),
            }
        }
        
        // Fallback for external connections (mock UI)
        let html = self.generate_mock_ui(&resource);
        
        Ok(UiResourceContent {
            uri: uri.to_string(),
            mime_type: resource.mime_type.clone(),
            text: Some(html),
            blob: None,
            _meta: resource._meta.clone(),
        })
    }
    
    /// Generate mock UI content for testing
    fn generate_mock_ui(&self, resource: &UiResource) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{
            font-family: system-ui, -apple-system, sans-serif;
            margin: 0;
            padding: 20px;
            background: #f5f5f5;
        }}
        .container {{
            max-width: 800px;
            margin: 0 auto;
            background: white;
            border-radius: 8px;
            padding: 24px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        h1 {{ color: #333; }}
        p {{ color: #666; line-height: 1.6; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{}</h1>
        <p>{}</p>
        <p><strong>URI:</strong> {}</p>
        <p><strong>Type:</strong> {}</p>
    </div>
    <script>
        // MCP Apps initialization will go here
        console.log('MCP App loaded: {}');
    </script>
</body>
</html>"#,
            resource.name,
            resource.name,
            resource.description.as_deref().unwrap_or("No description"),
            resource.uri,
            resource.mime_type,
            resource.name
        )
    }
    
    /// Disconnect from a server
    pub async fn disconnect(&self, connection_id: &str) -> Result<(), ConnectionError> {
        let mut connections = self.connections.write().await;
        
        if let Some(conn) = connections.get_mut(connection_id) {
            conn.set_state(ConnectionState::Disconnected);
            connections.remove(connection_id);
            log::info!("Disconnected from {}", connection_id);
            Ok(())
        } else {
            Err(ConnectionError::NotFound(connection_id.to_string()))
        }
    }
    
    /// Subscribe to connection events
    pub fn subscribe_events(&self) -> mpsc::UnboundedReceiver<ConnectionEvent> {
        // Create a new channel and subscribe to events
        let (tx, rx) = mpsc::unbounded_channel();
        // In a real implementation, we'd add this to a list of subscribers
        // For now, just return the receiver
        let _ = tx; // Silence unused warning
        rx
    }
}

/// Connection errors
#[derive(Debug, Clone)]
pub enum ConnectionError {
    Transport(String),
    Initialize(String),
    NotFound(String),
    NotReady(String),
    ResourceNotFound(String),
    ToolNotFound(String),
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionError::Transport(e) => write!(f, "Transport error: {}", e),
            ConnectionError::Initialize(e) => write!(f, "Initialize error: {}", e),
            ConnectionError::NotFound(id) => write!(f, "Connection not found: {}", id),
            ConnectionError::NotReady(id) => write!(f, "Connection not ready: {}", id),
            ConnectionError::ResourceNotFound(uri) => write!(f, "Resource not found: {}", uri),
            ConnectionError::ToolNotFound(name) => write!(f, "Tool not found: {}", name),
        }
    }
}

impl std::error::Error for ConnectionError {}
