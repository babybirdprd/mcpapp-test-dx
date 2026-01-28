//! MCP Apps Bridge
//!
//! Handles bidirectional communication between the host and UI views
//! using JSON-RPC over postMessage (for WebView) or direct channels.

use crate::protocol::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification, JsonRpcError, error_codes, Message};
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;

/// Bridge for communicating with a UI view
#[derive(Clone)]
pub struct UiBridge {
    /// Bridge ID
    pub id: String,
    /// Session ID
    pub session_id: String,
    /// Outgoing message sender (Host → View)
    outgoing_tx: mpsc::UnboundedSender<Value>,
    /// Incoming message receiver (View → Host)
    incoming_rx: Arc<RwLock<mpsc::UnboundedReceiver<Value>>>,
    /// Request handlers
    request_handlers: Arc<RwLock<HashMap<String, Box<dyn Fn(Value) -> Result<Value, String> + Send + Sync>>>>,
    /// Notification handlers
    notification_handlers: Arc<RwLock<HashMap<String, Box<dyn Fn(Value) + Send + Sync>>>>,
    /// Next request ID
    next_id: Arc<RwLock<u64>>,
    /// Pending requests
    pending_requests: Arc<RwLock<HashMap<u64, mpsc::Sender<Result<Value, JsonRpcError>>>>>,
}

impl std::fmt::Debug for UiBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UiBridge")
            .field("id", &self.id)
            .field("session_id", &self.session_id)
            .finish_non_exhaustive()
    }
}

impl UiBridge {
    /// Create a new UI bridge
    pub fn new(session_id: impl Into<String>) -> (Self, mpsc::UnboundedReceiver<Value>, mpsc::UnboundedSender<Value>) {
        let id = uuid::Uuid::new_v4().to_string();
        let session_id = session_id.into();
        
        let (outgoing_tx, outgoing_rx) = mpsc::unbounded_channel();
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();
        
        let bridge = Self {
            id,
            session_id,
            outgoing_tx,
            incoming_rx: Arc::new(RwLock::new(incoming_rx)),
            request_handlers: Arc::new(RwLock::new(HashMap::new())),
            notification_handlers: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
        };
        
        (bridge, outgoing_rx, incoming_tx)
    }
    
    /// Send a request to the view and wait for response
    pub async fn send_request(&self, method: impl Into<String>, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let id = {
            let mut next_id = self.next_id.write().await;
            let id = *next_id;
            *next_id += 1;
            id
        };
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::from(id)),
            method: method.into(),
            params,
        };
        
        // Create channel for response
        let (tx, mut rx) = mpsc::channel(1);
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(id, tx);
        }
        
        // Send request
        let value = serde_json::to_value(&request).unwrap();
        self.outgoing_tx.send(value).map_err(|_| {
            JsonRpcError::new(error_codes::INTERNAL_ERROR, "Failed to send request")
        })?;
        
        // Wait for response
        match tokio::time::timeout(tokio::time::Duration::from_secs(30), rx.recv()).await {
            Ok(Some(result)) => result,
            Ok(None) => Err(JsonRpcError::new(error_codes::INTERNAL_ERROR, "Channel closed")),
            Err(_) => Err(JsonRpcError::new(error_codes::INTERNAL_ERROR, "Request timeout")),
        }
    }
    
    /// Send a notification to the view
    pub fn send_notification(&self, method: impl Into<String>, params: Option<Value>) -> Result<(), String> {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        };
        
        let value = serde_json::to_value(&notification).map_err(|e| e.to_string())?;
        self.outgoing_tx.send(value).map_err(|_| "Failed to send notification".to_string())
    }
    
    /// Register a request handler
    pub async fn on_request<F>(&self, method: impl Into<String>, handler: F)
    where
        F: Fn(Value) -> Result<Value, String> + Send + Sync + 'static,
    {
        let mut handlers = self.request_handlers.write().await;
        handlers.insert(method.into(), Box::new(handler));
    }
    
    /// Register a notification handler
    pub async fn on_notification<F>(&self, method: impl Into<String>, handler: F)
    where
        F: Fn(Value) + Send + Sync + 'static,
    {
        let mut handlers = self.notification_handlers.write().await;
        handlers.insert(method.into(), Box::new(handler));
    }
    
    /// Process an incoming message from the view
    pub async fn process_message(&self, message: Value) -> Result<(), String> {
        // Check if it's a response
        if message.get("result").is_some() || message.get("error").is_some() {
            if let Some(id) = message.get("id").and_then(|v| v.as_u64()) {
                let mut pending = self.pending_requests.write().await;
                if let Some(tx) = pending.remove(&id) {
                    let result = if let Some(error) = message.get("error") {
                        Err(serde_json::from_value::<JsonRpcError>(error.clone())
                            .unwrap_or_else(|_| JsonRpcError::new(error_codes::INTERNAL_ERROR, "Unknown error")))
                    } else {
                        Ok(message.get("result").cloned().unwrap_or(Value::Object(serde_json::Map::new())))
                    };
                    let _ = tx.send(result).await;
                }
            }
            return Ok(());
        }
        
        // It's a request or notification
        let method = message.get("method")
            .and_then(|v| v.as_str())
            .ok_or("Missing method")?;
        
        let params = message.get("params").cloned();
        let id = message.get("id").cloned();
        
        if let Some(id) = id {
            // It's a request
            let handlers = self.request_handlers.read().await;
            if let Some(handler) = handlers.get(method) {
                match handler(params.unwrap_or(Value::Null)) {
                    Ok(result) => {
                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: Some(id),
                            result: Some(result),
                            error: None,
                        };
                        let value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
                        self.outgoing_tx.send(value).map_err(|_| "Failed to send response")?;
                    }
                    Err(e) => {
                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: Some(id),
                            result: None,
                            error: Some(JsonRpcError::new(error_codes::SERVER_ERROR, e)),
                        };
                        let value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
                        self.outgoing_tx.send(value).map_err(|_| "Failed to send error response")?;
                    }
                }
            } else {
                // Method not found
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Some(id),
                    result: None,
                    error: Some(JsonRpcError::new(
                        error_codes::METHOD_NOT_FOUND,
                        format!("Method not found: {}", method)
                    )),
                };
                let value = serde_json::to_value(&response).map_err(|e| e.to_string())?;
                self.outgoing_tx.send(value).map_err(|_| "Failed to send error response")?;
            }
        } else {
            // It's a notification
            let handlers = self.notification_handlers.read().await;
            if let Some(handler) = handlers.get(method) {
                handler(params.unwrap_or(Value::Null));
            }
        }
        
        Ok(())
    }
    
    /// Start processing messages from the view
    pub async fn start(&self) {
        let mut rx = self.incoming_rx.write().await;
        while let Some(message) = rx.recv().await {
            if let Err(e) = self.process_message(message).await {
                log::error!("Error processing message: {}", e);
            }
        }
    }
}

/// Bridge manager for multiple UI sessions
#[derive(Debug, Clone)]
pub struct BridgeManager {
    bridges: Arc<RwLock<HashMap<String, UiBridge>>>,
}

impl BridgeManager {
    pub fn new() -> Self {
        Self {
            bridges: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create a new bridge for a session
    pub async fn create_bridge(&self, session_id: impl Into<String>) -> UiBridge {
        let session_id = session_id.into();
        let (bridge, _outgoing_rx, _incoming_tx) = UiBridge::new(&session_id);
        
        let mut bridges = self.bridges.write().await;
        bridges.insert(session_id, bridge.clone());
        
        bridge
    }
    
    /// Get a bridge by session ID
    pub async fn get_bridge(&self, session_id: &str) -> Option<UiBridge> {
        self.bridges.read().await.get(session_id).cloned()
    }
    
    /// Remove a bridge
    pub async fn remove_bridge(&self, session_id: &str) {
        self.bridges.write().await.remove(session_id);
    }
}

impl Default for BridgeManager {
    fn default() -> Self {
        Self::new()
    }
}
