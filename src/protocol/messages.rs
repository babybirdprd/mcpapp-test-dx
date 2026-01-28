//! MCP Apps JSON-RPC Messages
//!
//! Predefined message builders for common MCP Apps operations.

use serde_json::json;
use serde_json::Value;

use super::*;
use lifecycle::*;
use resources::*;
use capabilities::*;

/// Build a ui/initialize request
pub fn ui_initialize_request(
    app_name: impl Into<String>,
    app_version: impl Into<String>,
    capabilities: McpUiAppCapabilities,
) -> JsonRpcRequest {
    let params = json!({
        "protocolVersion": PROTOCOL_VERSION,
        "appInfo": {
            "name": app_name.into(),
            "version": app_version.into(),
        },
        "appCapabilities": capabilities,
    });
    
    JsonRpcRequest::new("ui/initialize", Some(params))
}

/// Build a ui/initialize success response
pub fn ui_initialize_response(
    id: Value,
    host_name: impl Into<String>,
    host_version: impl Into<String>,
    capabilities: UiHostCapabilities,
    context: Option<HostContext>,
) -> JsonRpcResponse {
    let mut result = json!({
        "protocolVersion": PROTOCOL_VERSION,
        "hostCapabilities": capabilities,
        "hostInfo": {
            "name": host_name.into(),
            "version": host_version.into(),
        },
    });
    
    if let Some(ctx) = context {
        result["hostContext"] = serde_json::to_value(ctx).unwrap_or_default();
    }
    
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(id),
        result: Some(result),
        error: None,
    }
}

/// Build an initialized notification
pub fn initialized_notification() -> JsonRpcNotification {
    JsonRpcNotification::new("ui/notifications/initialized", Some(json!({})))
}

/// Build a tool-input notification
pub fn tool_input_notification(arguments: Value) -> JsonRpcNotification {
    JsonRpcNotification::new(
        "ui/notifications/tool-input",
        Some(json!({ "arguments": arguments })),
    )
}

/// Build a tool-input-partial notification
pub fn tool_input_partial_notification(arguments: Value) -> JsonRpcNotification {
    JsonRpcNotification::new(
        "ui/notifications/tool-input-partial",
        Some(json!({ "arguments": arguments })),
    )
}

/// Build a tool-result notification
pub fn tool_result_notification(result: Value) -> JsonRpcNotification {
    JsonRpcNotification::new(
        "ui/notifications/tool-result",
        Some(result),
    )
}

/// Build a tool-cancelled notification
pub fn tool_cancelled_notification(reason: Option<&str>) -> JsonRpcNotification {
    let params = if let Some(r) = reason {
        json!({ "reason": r })
    } else {
        json!({})
    };
    JsonRpcNotification::new("ui/notifications/tool-cancelled", Some(params))
}

/// Build a resource-teardown request
pub fn resource_teardown_request(id: Value, reason: Option<&str>) -> JsonRpcRequest {
    let params = if let Some(r) = reason {
        json!({ "reason": r })
    } else {
        json!({})
    };
    JsonRpcRequest::new("ui/resource-teardown", Some(params)).with_id(id)
}

/// Build a size-changed notification
pub fn size_changed_notification(width: u32, height: u32) -> JsonRpcNotification {
    JsonRpcNotification::new(
        "ui/notifications/size-changed",
        Some(json!({
            "width": width,
            "height": height,
        })),
    )
}

/// Build a host-context-changed notification
pub fn host_context_changed_notification(context: Value) -> JsonRpcNotification {
    JsonRpcNotification::new(
        "ui/notifications/host-context-changed",
        Some(context),
    )
}

/// Build a sandbox-proxy-ready notification
pub fn sandbox_proxy_ready_notification() -> JsonRpcNotification {
    JsonRpcNotification::new("ui/notifications/sandbox-proxy-ready", Some(json!({})))
}

/// Build a sandbox-resource-ready notification
pub fn sandbox_resource_ready_notification(
    html: impl Into<String>,
    csp: Option<McpUiResourceCsp>,
    permissions: Option<UiResourcePermissions>,
) -> JsonRpcNotification {
    let mut params = json!({
        "html": html.into(),
    });
    
    if let Some(csp_config) = csp {
        params["csp"] = serde_json::to_value(csp_config).unwrap_or_default();
    }
    
    if let Some(perms) = permissions {
        params["permissions"] = serde_json::to_value(perms).unwrap_or_default();
    }
    
    JsonRpcNotification::new("ui/notifications/sandbox-resource-ready", Some(params))
}

/// Build a request-display-mode request
pub fn request_display_mode_request(id: Value, mode: DisplayMode) -> JsonRpcRequest {
    JsonRpcRequest::new(
        "ui/request-display-mode",
        Some(json!({ "mode": mode })),
    ).with_id(id)
}

/// Build a request-display-mode success response
pub fn request_display_mode_response(id: Value, mode: DisplayMode) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(id),
        result: Some(json!({ "mode": mode })),
        error: None,
    }
}

/// Build an update-model-context request
pub fn update_model_context_request(
    id: Value,
    content: Option<Vec<Value>>,
    structured_content: Option<Value>,
) -> JsonRpcRequest {
    let mut params = json!({});
    if let Some(c) = content {
        params["content"] = json!(c);
    }
    if let Some(s) = structured_content {
        params["structuredContent"] = s;
    }
    JsonRpcRequest::new("ui/update-model-context", Some(params)).with_id(id)
}

/// Build an open-link request
pub fn open_link_request(id: Value, url: impl Into<String>) -> JsonRpcRequest {
    JsonRpcRequest::new(
        "ui/open-link",
        Some(json!({ "url": url.into() })),
    ).with_id(id)
}

/// Build a ui/message request
pub fn ui_message_request(id: Value, role: impl Into<String>, content: Value) -> JsonRpcRequest {
    JsonRpcRequest::new(
        "ui/message",
        Some(json!({
            "role": role.into(),
            "content": content,
        })),
    ).with_id(id)
}

/// Build a success response
pub fn success_response(id: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(id),
        result: Some(json!({})),
        error: None,
    }
}

/// Build an error response
pub fn error_response(id: Value, code: i32, message: impl Into<String>) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(id),
        result: None,
        error: Some(JsonRpcError::new(code, message)),
    }
}

/// Parse a JSON-RPC message from JSON value
pub fn parse_message(value: Value) -> Result<Message, serde_json::Error> {
    // Check if it's a notification (no id) or request/response
    if value.get("id").is_none() {
        // Notification
        let notif: JsonRpcNotification = serde_json::from_value(value)?;
        Ok(Message::Notification(notif))
    } else if value.get("result").is_some() || value.get("error").is_some() {
        // Response
        let resp: JsonRpcResponse = serde_json::from_value(value)?;
        Ok(Message::Response(resp))
    } else {
        // Request
        let req: JsonRpcRequest = serde_json::from_value(value)?;
        Ok(Message::Request(req))
    }
}

/// Enum representing any JSON-RPC message type
#[derive(Debug, Clone)]
pub enum Message {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ui_initialize_request() {
        let req = ui_initialize_request(
            "test-app",
            "1.0.0",
            McpUiAppCapabilities::default(),
        );
        
        assert_eq!(req.method, "ui/initialize");
        assert!(req.id.is_some());
    }
    
    #[test]
    fn test_size_changed_notification() {
        let notif = size_changed_notification(800, 600);
        
        assert_eq!(notif.method, "ui/notifications/size-changed");
    }
    
    #[test]
    fn test_parse_message() {
        // Test request
        let req_json = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "ui/initialize",
            "params": {}
        });
        
        match parse_message(req_json).unwrap() {
            Message::Request(req) => {
                assert_eq!(req.method, "ui/initialize");
            }
            _ => panic!("Expected request"),
        }
        
        // Test notification
        let notif_json = json!({
            "jsonrpc": "2.0",
            "method": "ui/notifications/initialized",
            "params": {}
        });
        
        match parse_message(notif_json).unwrap() {
            Message::Notification(notif) => {
                assert_eq!(notif.method, "ui/notifications/initialized");
            }
            _ => panic!("Expected notification"),
        }
    }
}
