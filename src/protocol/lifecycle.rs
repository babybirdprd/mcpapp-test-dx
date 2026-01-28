//! MCP Apps Lifecycle Types
//!
//! Types for UI initialization, teardown, and lifecycle management.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::capabilities::{DisplayMode, McpUiAppCapabilities, UiHostCapabilities, Empty};
use super::resources::UiStyleConfig;

/// UI Initialize Request (View → Host)
/// 
/// Sent by the View to initialize the MCP Apps connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpUiInitializeRequest {
    /// Protocol version
    pub protocol_version: String,
    
    /// App information
    pub app_info: AppInfo,
    
    /// App capabilities
    pub app_capabilities: McpUiAppCapabilities,
}

/// App information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    /// App name
    pub name: String,
    /// App version
    pub version: String,
}

/// UI Initialize Result (Host → View)
/// 
/// Response to ui/initialize containing host capabilities and context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpUiInitializeResult {
    /// Protocol version
    pub protocol_version: String,
    
    /// Host capabilities
    pub host_capabilities: UiHostCapabilities,
    
    /// Host information
    pub host_info: HostInfo,
    
    /// Host context for the view
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_context: Option<HostContext>,
}

/// Host information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostInfo {
    /// Host name
    pub name: String,
    /// Host version
    pub version: String,
}

/// Host context provided to views
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HostContext {
    /// Metadata of the tool call that instantiated the View
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_info: Option<ToolInfo>,
    
    /// Current color theme preference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    
    /// Style configuration for theming
    #[serde(skip_serializing_if = "Option::is_none")]
    pub styles: Option<UiStyleConfig>,
    
    /// How the View is currently displayed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_mode: Option<DisplayMode>,
    
    /// Display modes the host supports
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_display_modes: Option<Vec<DisplayMode>>,
    
    /// Container dimensions for the iframe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_dimensions: Option<ContainerDimensions>,
    
    /// User's language/region preference (BCP 47, e.g., "en-US")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    
    /// User's timezone (IANA, e.g., "America/New_York")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
    
    /// Host application identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    
    /// Platform type for responsive design
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<Platform>,
    
    /// Device capabilities such as touch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_capabilities: Option<DeviceCapabilities>,
    
    /// Safe area boundaries in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_area_insets: Option<SafeAreaInsets>,
}

/// Tool info for host context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    /// JSON-RPC id of the tools/call request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    /// Contains name, inputSchema, etc.
    pub tool: Value,
}

/// Container dimensions for the iframe
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContainerDimensions {
    /// Fixed height (if specified, container is fixed at this height)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Maximum height (if specified, view controls height up to this max)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_height: Option<u32>,
    /// Fixed width (if specified, container is fixed at this width)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// Maximum width (if specified, view controls width up to this max)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_width: Option<u32>,
}

/// Platform types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Web,
    Desktop,
    Mobile,
}

/// Device capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCapabilities {
    /// Touch support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub touch: Option<bool>,
    /// Hover support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover: Option<bool>,
}

/// Safe area insets for notches, etc.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SafeAreaInsets {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

/// Initialized notification params (View → Host)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializedNotification {}

/// Tool input notification params (Host → View)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputNotification {
    /// Tool input arguments
    pub arguments: Value,
}

/// Tool input partial notification params (Host → View)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputPartialNotification {
    /// Partial tool input arguments
    pub arguments: Value,
}

/// Tool result notification params (Host → View)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultNotification {
    /// Tool execution result (CallToolResult)
    #[serde(flatten)]
    pub result: Value,
}

/// Tool cancelled notification params (Host → View)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCancelledNotification {
    /// Cancellation reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Resource teardown request params (Host → View)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTeardownRequest {
    /// Teardown reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Size changed notification params (View → Host)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeChangedNotification {
    /// Viewport width in pixels
    pub width: u32,
    /// Viewport height in pixels
    pub height: u32,
}

/// Host context changed notification params (Host → View)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostContextChangedNotification {
    /// Partial host context update
    #[serde(flatten)]
    pub context: Value,
}

/// Sandbox proxy ready notification (Sandbox → Host)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxProxyReadyNotification {}

/// Sandbox resource ready notification (Host → Sandbox)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxResourceReadyNotification {
    /// HTML content to load
    pub html: String,
    /// Optional override for inner iframe sandbox attribute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<String>,
    /// CSP configuration from resource metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csp: Option<CspConfig>,
    /// Sandbox permissions from resource metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<UiPermissionsConfig>,
}

/// CSP configuration for sandbox
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CspConfig {
    pub connect_domains: Option<Vec<String>>,
    pub resource_domains: Option<Vec<String>>,
    pub frame_domains: Option<Vec<String>>,
    pub base_uri_domains: Option<Vec<String>>,
}

/// UI permissions configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiPermissionsConfig {
    pub camera: Option<Empty>,
    pub microphone: Option<Empty>,
    pub geolocation: Option<Empty>,
    pub clipboard_write: Option<Empty>,
}

/// Request display mode request (View → Host)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestDisplayModeRequest {
    /// Requested display mode
    pub mode: DisplayMode,
}

/// Request display mode result (Host → View)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestDisplayModeResult {
    /// Actual display mode set
    pub mode: DisplayMode,
}

/// Update model context request (View → Host)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateModelContextRequest {
    /// Content blocks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<Value>>,
    /// Structured content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<Value>,
}

/// Open link request (View → Host)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenLinkRequest {
    /// URL to open
    pub url: String,
}

/// UI message request (View → Host)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiMessageRequest {
    /// Message role
    pub role: String,
    /// Message content
    pub content: Value,
}
