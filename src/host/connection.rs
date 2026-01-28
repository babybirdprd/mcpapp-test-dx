//! MCP Server Connection
//!
//! Manages a single connection to an MCP server, handling protocol
//! negotiation, capability exchange, and message routing.

use crate::protocol::{
    capabilities::{McpUiAppCapabilities, ServerCapabilities, UiHostCapabilities, negotiate_capabilities, NegotiatedCapabilities},
    resources::{UiResource, UiResourceMeta},
    UI_EXTENSION_ID,
};
use rmcp::model::{Resource, Tool};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Connection to an MCP server
#[derive(Debug, Clone)]
pub struct McpServerConnection {
    /// Connection ID
    pub id: String,
    /// Server name (from server info)
    pub name: String,
    /// Server version (from server info)
    pub version: String,
    /// Server capabilities (raw)
    pub server_capabilities: Option<Value>,
    /// Parsed server capabilities
    pub parsed_capabilities: Option<ServerCapabilities>,
    /// Negotiated capabilities
    pub negotiated_capabilities: Option<NegotiatedCapabilities>,
    /// Whether the server supports MCP Apps
    pub supports_ui_extension: bool,
    /// Connection state
    pub state: ConnectionState,
    /// Available tools
    pub tools: Arc<RwLock<Vec<Tool>>>,
    /// Available resources
    pub resources: Arc<RwLock<Vec<Resource>>>,
    /// UI resources (filtered from resources)
    pub ui_resources: Arc<RwLock<Vec<UiResource>>>,
}

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// Connecting
    Connecting,
    /// Initializing (handshake in progress)
    Initializing,
    /// Ready
    Ready,
    /// Disconnected
    Disconnected,
    /// Error
    Error(String),
}

/// Server info from initialize response
#[derive(Debug, Clone, Default)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

impl McpServerConnection {
    /// Create a new connection
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            version: String::new(),
            server_capabilities: None,
            parsed_capabilities: None,
            negotiated_capabilities: None,
            supports_ui_extension: false,
            state: ConnectionState::Connecting,
            tools: Arc::new(RwLock::new(Vec::new())),
            resources: Arc::new(RwLock::new(Vec::new())),
            ui_resources: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Check if connection is ready
    pub fn is_ready(&self) -> bool {
        self.state == ConnectionState::Ready
    }
    
    /// Update connection state
    pub fn set_state(&mut self, state: ConnectionState) {
        self.state = state;
    }
    
    /// Set server capabilities from initialize response
    pub fn set_capabilities(&mut self, response: &Value) {
        let protocol_version = response
            .get("protocolVersion")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        let server_info = response
            .get("serverInfo")
            .map(|info| ServerInfo {
                name: info.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                version: info.get("version").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
            })
            .unwrap_or_default();
        
        let capabilities = response
            .get("capabilities")
            .cloned()
            .unwrap_or_default();
        
        // Parse server capabilities using our structured types
        let parsed_caps: ServerCapabilities = serde_json::from_value(capabilities.clone())
            .unwrap_or_default();
        
        // Check for UI extension support
        let supports_ui = parsed_caps.supports_ui_apps();
        
        self.server_capabilities = Some(capabilities);
        self.parsed_capabilities = Some(parsed_caps);
        self.supports_ui_extension = supports_ui;
        self.name = server_info.name;
        self.version = server_info.version;
    }
    
    /// Negotiate capabilities with the server
    pub fn negotiate_capabilities(&mut self, host_caps: &UiHostCapabilities, app_caps: Option<&McpUiAppCapabilities>) {
        if let Some(server_caps) = &self.parsed_capabilities {
            let negotiated = negotiate_capabilities(host_caps, server_caps, app_caps);
            self.negotiated_capabilities = Some(negotiated);
        }
    }
    
    /// Get negotiated capabilities
    pub fn get_negotiated_capabilities(&self) -> Option<&NegotiatedCapabilities> {
        self.negotiated_capabilities.as_ref()
    }
    
    /// Check if a specific display mode is supported
    pub fn supports_display_mode(&self, mode: crate::protocol::DisplayMode) -> bool {
        self.negotiated_capabilities.as_ref()
            .map(|n| n.display_modes.contains(&mode))
            .unwrap_or(false)
    }
    
    /// Update tools list
    pub async fn update_tools(&self, tools: Vec<Tool>) {
        let mut guard = self.tools.write().await;
        *guard = tools;
    }
    
    /// Update resources list and extract UI resources
    pub async fn update_resources(&self, resources: Vec<Resource>) {
        let ui_resources: Vec<UiResource> = resources
            .iter()
            .filter(|r| UiResource::is_valid_uri(&r.uri))
            .filter_map(|r| Self::convert_to_ui_resource(r).ok())
            .collect();
        
        let mut guard = self.resources.write().await;
        *guard = resources;
        drop(guard);
        
        let mut ui_guard = self.ui_resources.write().await;
        *ui_guard = ui_resources;
    }
    
    /// Get tools list
    pub async fn get_tools(&self) -> Vec<Tool> {
        self.tools.read().await.clone()
    }
    
    /// Get UI resources list
    pub async fn get_ui_resources(&self) -> Vec<UiResource> {
        self.ui_resources.read().await.clone()
    }
    
    /// Find a UI resource by URI
    pub async fn find_ui_resource(&self, uri: &str) -> Option<UiResource> {
        self.ui_resources.read().await.iter()
            .find(|r| r.uri == uri)
            .cloned()
    }
    
    /// Find a tool by name
    pub async fn find_tool(&self, name: &str) -> Option<Tool> {
        self.tools.read().await.iter()
            .find(|t| t.name.as_ref() == name)
            .cloned()
    }
    
    /// Convert an MCP Resource to a UiResource
    fn convert_to_ui_resource(resource: &Resource) -> Result<UiResource, String> {
        if !UiResource::is_valid_uri(&resource.uri) {
            return Err("Not a UI resource URI".to_string());
        }
        
        let mime_type = resource.mime_type.clone()
            .map(|m| m.to_string())
            .unwrap_or_else(|| UiResource::recommended_mime_type().to_string());
        
        let _meta = resource.meta.as_ref()
            .and_then(|m| serde_json::from_value::<UiResourceMeta>(serde_json::Value::Object(m.0.clone())).ok());
        
        Ok(UiResource {
            uri: resource.uri.clone(),
            name: resource.name.clone(),
            description: resource.description.clone(),
            mime_type,
            _meta,
        })
    }
    
    /// Get tools that have UI metadata
    pub async fn get_tools_with_ui(&self) -> Vec<(Tool, String)> {
        let tools = self.tools.read().await;
        tools
            .iter()
            .filter_map(|t| {
                let uri = t.meta.as_ref()
                    .and_then(|m| m.0.get("ui"))
                    .and_then(|u| u.get("resourceUri"))
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| {
                        // Try deprecated format
                        t.meta.as_ref()
                            .and_then(|m| m.0.get("ui/resourceUri"))
                            .and_then(|s| s.as_str())
                            .map(|s| s.to_string())
                    });
                
                uri.map(|u| (t.clone(), u))
            })
            .collect()
    }
}

/// Connection manager event types
#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    /// Connection state changed
    StateChanged { connection_id: String, state: ConnectionState },
    /// Tools list updated
    ToolsUpdated { connection_id: String, tools: Vec<Tool> },
    /// Resources list updated
    ResourcesUpdated { connection_id: String, resources: Vec<Resource> },
    /// Server sent a notification
    Notification { connection_id: String, method: String, params: Option<Value> },
    /// Error occurred
    Error { connection_id: String, error: String },
    /// Connection closed
    Closed { connection_id: String },
}
