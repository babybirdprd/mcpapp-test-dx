//! MCP Apps Protocol Types
//! 
//! This module contains type definitions based on the MCP Apps Specification (SEP-1865).
//! These types implement the JSON-RPC protocol extensions for UI resources and
//! bidirectional communication between hosts and views.

pub mod capabilities;
pub mod lifecycle;
pub mod resources;
pub mod messages;

pub use capabilities::*;
pub use lifecycle::*;
pub use resources::*;
pub use messages::*;

// Re-export specific types that are commonly used
pub use capabilities::ApprovedCsp;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Extension identifier for MCP Apps
pub const UI_EXTENSION_ID: &str = "io.modelcontextprotocol/ui";

/// Protocol version
pub const PROTOCOL_VERSION: &str = "2026-01-26";

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// JSON-RPC 2.0 notification (request without id)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    pub fn new(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::from(1)), // TODO: Use proper ID generation
            method: method.into(),
            params,
        }
    }
    
    pub fn with_id(mut self, id: Value) -> Self {
        self.id = Some(id);
        self
    }
}

impl JsonRpcNotification {
    pub fn new(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        }
    }
}

impl JsonRpcError {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }
    
    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }
}

/// Standard error codes
pub mod error_codes {
    /// Parse error
    pub const PARSE_ERROR: i32 = -32700;
    /// Invalid request
    pub const INVALID_REQUEST: i32 = -32600;
    /// Method not found
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid params
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal error
    pub const INTERNAL_ERROR: i32 = -32603;
    /// Server error (implementation-defined)
    pub const SERVER_ERROR: i32 = -32000;
}
