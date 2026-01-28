//! MCP Apps Host Module
//!
//! This module manages connections to external MCP servers and handles
//! the host-side responsibilities of the MCP Apps specification.

pub mod connection;
pub mod manager;
pub mod transport;

pub use connection::*;
pub use manager::*;

use crate::protocol::{
    DisplayMode, McpUiAppCapabilities, UiHostCapabilities, ServerToolsCapability,
    ServerResourcesCapability, SandboxCapability, UiPermissions, HostContext, ToolInfo,
    ContainerDimensions, Platform, DeviceCapabilities, SafeAreaInsets, ApprovedCsp,
};
use crate::protocol::messages::Message;
use serde_json::Value;
use std::collections::HashMap;


/// Host state for MCP Apps
#[derive(Debug, Clone, PartialEq)]
pub struct HostState {
    /// Host name
    pub name: String,
    /// Host version
    pub version: String,
    /// Supported display modes
    pub supported_display_modes: Vec<DisplayMode>,
    /// Current theme
    pub theme: String,
    /// Platform type
    pub platform: Platform,
    /// Device capabilities
    pub device_capabilities: DeviceCapabilities,
    /// Container dimensions for the iframe
    pub container_dimensions: ContainerDimensions,
    /// User locale
    pub locale: String,
    /// User timezone
    pub time_zone: String,
}

impl Default for HostState {
    fn default() -> Self {
        Self {
            name: "mcp-apps-host".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            supported_display_modes: vec![DisplayMode::Inline],
            theme: "light".to_string(),
            platform: Platform::Desktop,
            device_capabilities: DeviceCapabilities {
                touch: Some(false),
                hover: Some(true),
            },
            container_dimensions: ContainerDimensions {
                height: None,
                max_height: Some(600),
                width: None,
                max_width: Some(800),
            },
            locale: "en-US".to_string(),
            time_zone: "UTC".to_string(),
        }
    }
}

impl HostState {
    /// Create host capabilities for MCP initialize
    pub fn to_capabilities(&self) -> UiHostCapabilities {
        UiHostCapabilities {
            experimental: None,
            open_links: Some(crate::protocol::capabilities::Empty {}),
            server_tools: Some(ServerToolsCapability { list_changed: Some(true) }),
            server_resources: Some(ServerResourcesCapability { list_changed: Some(true) }),
            logging: Some(crate::protocol::capabilities::Empty {}),
            sandbox: Some(SandboxCapability {
                permissions: Some(UiPermissions {
                    camera: None,
                    microphone: None,
                    geolocation: None,
                    clipboard_write: None,
                }),
                csp: Some(crate::protocol::ApprovedCsp::default()),
            }),
        }
    }
    
    /// Create full host capabilities (all features enabled)
    pub fn to_full_capabilities(&self) -> UiHostCapabilities {
        UiHostCapabilities::full()
    }
    
    /// Create minimal host capabilities (safe defaults)
    pub fn to_minimal_capabilities(&self) -> UiHostCapabilities {
        UiHostCapabilities::minimal()
    }
    
    /// Create host context for UI initialization
    pub fn to_host_context(&self) -> HostContext {
        HostContext {
            tool_info: None,
            theme: Some(self.theme.clone()),
            styles: None,
            display_mode: Some(DisplayMode::Inline),
            available_display_modes: Some(self.supported_display_modes.clone()),
            container_dimensions: Some(self.container_dimensions.clone()),
            locale: Some(self.locale.clone()),
            time_zone: Some(self.time_zone.clone()),
            user_agent: Some(format!("{}/{} (MCP Apps Host)", self.name, self.version)),
            platform: Some(self.platform),
            device_capabilities: Some(self.device_capabilities.clone()),
            safe_area_insets: None,
        }
    }
    
    /// Builder method: Set supported display modes
    pub fn with_display_modes(mut self, modes: Vec<DisplayMode>) -> Self {
        self.supported_display_modes = modes;
        self
    }
    
    /// Builder method: Set theme
    pub fn with_theme(mut self, theme: impl Into<String>) -> Self {
        self.theme = theme.into();
        self
    }
    
    /// Builder method: Set platform
    pub fn with_platform(mut self, platform: Platform) -> Self {
        self.platform = platform;
        self
    }
    
    /// Builder method: Set locale
    pub fn with_locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = locale.into();
        self
    }
    
    /// Builder method: Set timezone
    pub fn with_timezone(mut self, tz: impl Into<String>) -> Self {
        self.time_zone = tz.into();
        self
    }
}

/// Information about an active UI session
#[derive(Debug, Clone)]
pub struct UiSession {
    /// Session ID
    pub id: String,
    /// Associated server connection ID
    pub server_id: String,
    /// UI resource URI
    pub resource_uri: String,
    /// Tool that triggered this session (if any)
    pub tool_info: Option<ToolInfo>,
    /// Current state
    pub state: UiSessionState,
    /// App capabilities received during initialization
    pub app_capabilities: Option<McpUiAppCapabilities>,
    /// Session metadata
    pub metadata: HashMap<String, Value>,
}

/// UI session state
#[derive(Debug, Clone, PartialEq)]
pub enum UiSessionState {
    /// Initializing (waiting for ui/initialize)
    Initializing,
    /// Initialized and ready
    Ready,
    /// Loading tool data
    Loading,
    /// Active and interactive
    Active,
    /// Tearing down
    Teardown,
    /// Error state
    Error(String),
}

impl UiSession {
    pub fn new(id: impl Into<String>, server_id: impl Into<String>, resource_uri: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            server_id: server_id.into(),
            resource_uri: resource_uri.into(),
            tool_info: None,
            state: UiSessionState::Initializing,
            app_capabilities: None,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_tool_info(mut self, tool_info: ToolInfo) -> Self {
        self.tool_info = Some(tool_info);
        self
    }
}

/// Event types for UI session events
#[derive(Debug, Clone)]
pub enum UiSessionEvent {
    /// Session state changed
    StateChanged { session_id: String, state: UiSessionState },
    /// Message received from the view
    Message { session_id: String, message: Message },
    /// Tool input received
    ToolInput { session_id: String, arguments: Value },
    /// Tool result received
    ToolResult { session_id: String, result: Value },
    /// Tool cancelled
    ToolCancelled { session_id: String, reason: Option<String> },
    /// Display mode changed
    DisplayModeChanged { session_id: String, mode: DisplayMode },
    /// Size changed notification
    SizeChanged { session_id: String, width: u32, height: u32 },
    /// Session error
    Error { session_id: String, error: String },
    /// Session closed
    Closed { session_id: String },
}
