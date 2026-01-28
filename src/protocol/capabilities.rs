//! MCP Apps Capability Types
//!
//! Types for capability negotiation between hosts and MCP servers.
//! Based on SEP-1865 Section: Client<>Server Capability Negotiation

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Client (Host) capabilities for the MCP Apps extension
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiHostCapabilities {
    /// Experimental features
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<Value>,
    
    /// Host supports opening external URLs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_links: Option<Empty>,
    
    /// Host can proxy tool calls to the MCP server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_tools: Option<ServerToolsCapability>,
    
    /// Host can proxy resource reads to the MCP server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_resources: Option<ServerResourcesCapability>,
    
    /// Host accepts log messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<Empty>,
    
    /// Sandbox configuration applied by the host
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<SandboxCapability>,
}

impl UiHostCapabilities {
    /// Create capabilities with all features enabled
    pub fn full() -> Self {
        Self {
            experimental: None,
            open_links: Some(Empty {}),
            server_tools: Some(ServerToolsCapability { list_changed: Some(true) }),
            server_resources: Some(ServerResourcesCapability { list_changed: Some(true) }),
            logging: Some(Empty {}),
            sandbox: Some(SandboxCapability {
                permissions: Some(UiPermissions {
                    camera: Some(Empty {}),
                    microphone: Some(Empty {}),
                    geolocation: Some(Empty {}),
                    clipboard_write: Some(Empty {}),
                }),
                csp: Some(ApprovedCsp {
                    connect_domains: Some(vec!["*".to_string()]),
                    resource_domains: Some(vec!["*".to_string()]),
                    frame_domains: None,
                    base_uri_domains: None,
                }),
            }),
        }
    }
    
    /// Create minimal capabilities (safe defaults)
    pub fn minimal() -> Self {
        Self {
            experimental: None,
            open_links: Some(Empty {}),
            server_tools: Some(ServerToolsCapability { list_changed: Some(false) }),
            server_resources: Some(ServerResourcesCapability { list_changed: Some(false) }),
            logging: Some(Empty {}),
            sandbox: Some(SandboxCapability {
                permissions: Some(UiPermissions::default()),
                csp: Some(ApprovedCsp::default()),
            }),
        }
    }
    
    /// Check if host supports a specific feature
    pub fn supports_open_links(&self) -> bool {
        self.open_links.is_some()
    }
    
    pub fn supports_tool_notifications(&self) -> bool {
        self.server_tools.as_ref().map(|t| t.list_changed.unwrap_or(false)).unwrap_or(false)
    }
    
    pub fn supports_resource_notifications(&self) -> bool {
        self.server_resources.as_ref().map(|r| r.list_changed.unwrap_or(false)).unwrap_or(false)
    }
    
    pub fn supports_logging(&self) -> bool {
        self.logging.is_some()
    }
}

/// Server tools capability
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerToolsCapability {
    /// Host supports tools/list_changed notifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Server resources capability
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerResourcesCapability {
    /// Host supports resources/list_changed notifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Sandbox capability
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxCapability {
    /// Permissions granted by the host
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<UiPermissions>,
    /// CSP domains approved by the host
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csp: Option<ApprovedCsp>,
}

/// UI Permissions that can be granted by the host
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiPermissions {
    /// Camera access granted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera: Option<Empty>,
    /// Microphone access granted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub microphone: Option<Empty>,
    /// Geolocation access granted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geolocation: Option<Empty>,
    /// Clipboard write access granted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clipboard_write: Option<Empty>,
}

impl UiPermissions {
    /// Check if any permissions are granted
    pub fn has_any(&self) -> bool {
        self.camera.is_some() ||
        self.microphone.is_some() ||
        self.geolocation.is_some() ||
        self.clipboard_write.is_some()
    }
    
    /// Get list of granted permissions as strings
    pub fn granted(&self) -> Vec<&'static str> {
        let mut granted = Vec::new();
        if self.camera.is_some() { granted.push("camera"); }
        if self.microphone.is_some() { granted.push("microphone"); }
        if self.geolocation.is_some() { granted.push("geolocation"); }
        if self.clipboard_write.is_some() { granted.push("clipboard-write"); }
        granted
    }
}

/// Approved CSP domains by the host
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApprovedCsp {
    /// Approved origins for network requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_domains: Option<Vec<String>>,
    /// Approved origins for static resources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_domains: Option<Vec<String>>,
    /// Approved origins for nested iframes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_domains: Option<Vec<String>>,
    /// Approved base URIs for the document
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_uri_domains: Option<Vec<String>>,
}

impl ApprovedCsp {
    /// Check if a domain is approved for connections
    pub fn allows_connection(&self, domain: &str) -> bool {
        if let Some(domains) = &self.connect_domains {
            domains.iter().any(|d| d == "*" || domain.ends_with(d.trim_start_matches("*.")) || domain == d)
        } else {
            false
        }
    }
    
    /// Check if a domain is approved for resources
    pub fn allows_resource(&self, domain: &str) -> bool {
        if let Some(domains) = &self.resource_domains {
            domains.iter().any(|d| d == "*" || domain.ends_with(d.trim_start_matches("*.")) || domain == d)
        } else {
            false
        }
    }
}

/// App (View) capabilities sent during ui/initialize
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpUiAppCapabilities {
    /// Experimental features
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<Value>,
    
    /// App exposes MCP-style tools that the host can call
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<AppToolsCapability>,
    
    /// Display modes the app supports
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_display_modes: Option<Vec<DisplayMode>>,
}

impl McpUiAppCapabilities {
    /// Check if app supports a specific display mode
    pub fn supports_display_mode(&self, mode: DisplayMode) -> bool {
        self.available_display_modes.as_ref()
            .map(|modes| modes.contains(&mode))
            .unwrap_or(true) // Default to true if not specified
    }
    
    /// Check if app exposes tools
    pub fn exposes_tools(&self) -> bool {
        self.tools.is_some()
    }
    
    /// Check if app supports tool list changed notifications
    pub fn supports_tool_notifications(&self) -> bool {
        self.tools.as_ref().map(|t| t.list_changed.unwrap_or(false)).unwrap_or(false)
    }
}

/// App tools capability
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppToolsCapability {
    /// App supports tools/list_changed notifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Display modes supported by apps/hosts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DisplayMode {
    /// Default mode, embedded within the host's content flow
    Inline,
    /// View takes over the full screen/window
    Fullscreen,
    /// Picture-in-picture, floating overlay
    Pip,
}

impl std::fmt::Display for DisplayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisplayMode::Inline => write!(f, "inline"),
            DisplayMode::Fullscreen => write!(f, "fullscreen"),
            DisplayMode::Pip => write!(f, "pip"),
        }
    }
}

/// Empty struct for capability flags
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Empty {}

/// Wrapper for extension capabilities in MCP initialize
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperimentalCapabilities {
    #[serde(rename = "io.modelcontextprotocol/ui", skip_serializing_if = "Option::is_none")]
    pub ui: Option<UiHostCapabilities>,
    
    /// Other experimental extensions
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// Server capabilities (parsed from initialize response)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Experimental capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<ExperimentalCapabilities>,
    /// Tools capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsServerCapability>,
    /// Resources capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesServerCapability>,
    /// Other capabilities
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

impl ServerCapabilities {
    /// Check if server supports MCP Apps
    pub fn supports_ui_apps(&self) -> bool {
        self.experimental.as_ref()
            .and_then(|e| e.ui.as_ref())
            .is_some()
    }
    
    /// Get UI capabilities if supported
    pub fn ui_capabilities(&self) -> Option<&UiHostCapabilities> {
        self.experimental.as_ref().and_then(|e| e.ui.as_ref())
    }
    
    /// Check if server supports tools
    pub fn supports_tools(&self) -> bool {
        self.tools.is_some()
    }
    
    /// Check if server supports resources
    pub fn supports_resources(&self) -> bool {
        self.resources.is_some()
    }
    
    /// Check if server supports tool list changed notifications
    pub fn supports_tool_notifications(&self) -> bool {
        self.tools.as_ref().map(|t| t.list_changed.unwrap_or(false)).unwrap_or(false)
    }
    
    /// Check if server supports resource list changed notifications
    pub fn supports_resource_notifications(&self) -> bool {
        self.resources.as_ref().map(|r| r.list_changed.unwrap_or(false)).unwrap_or(false)
    }
}

/// Server tools capability (from server)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsServerCapability {
    pub list_changed: Option<bool>,
}

/// Server resources capability (from server)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesServerCapability {
    pub list_changed: Option<bool>,
}

/// Capability negotiation result
#[derive(Debug, Clone)]
pub struct NegotiatedCapabilities {
    /// Protocol version agreed upon
    pub protocol_version: String,
    /// Server supports MCP Apps
    pub supports_ui_apps: bool,
    /// Agreed display modes
    pub display_modes: Vec<DisplayMode>,
    /// Whether tool notifications are supported
    pub tool_notifications: bool,
    /// Whether resource notifications are supported
    pub resource_notifications: bool,
    /// Granted permissions
    pub permissions: UiPermissions,
}

/// Negotiate capabilities between host and server
pub fn negotiate_capabilities(
    host_caps: &UiHostCapabilities,
    server_caps: &ServerCapabilities,
    app_caps: Option<&McpUiAppCapabilities>,
) -> NegotiatedCapabilities {
    // Determine supported display modes
    let host_modes = vec![DisplayMode::Inline, DisplayMode::Fullscreen];
    let app_modes = app_caps.and_then(|a| a.available_display_modes.clone());
    
    let display_modes = match app_modes {
        Some(modes) => host_modes.into_iter().filter(|m| modes.contains(m)).collect(),
        None => host_modes,
    };
    
    NegotiatedCapabilities {
        protocol_version: super::PROTOCOL_VERSION.to_string(),
        supports_ui_apps: server_caps.supports_ui_apps(),
        display_modes,
        tool_notifications: host_caps.supports_tool_notifications() && server_caps.supports_tool_notifications(),
        resource_notifications: host_caps.supports_resource_notifications() && server_caps.supports_resource_notifications(),
        permissions: host_caps.sandbox.as_ref().and_then(|s| s.permissions.clone()).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ui_host_capabilities_full() {
        let caps = UiHostCapabilities::full();
        assert!(caps.supports_open_links());
        assert!(caps.supports_tool_notifications());
        assert!(caps.supports_resource_notifications());
        assert!(caps.supports_logging());
    }
    
    #[test]
    fn test_ui_host_capabilities_minimal() {
        let caps = UiHostCapabilities::minimal();
        assert!(caps.supports_open_links());
        assert!(!caps.supports_tool_notifications());
        assert!(!caps.supports_resource_notifications());
    }
    
    #[test]
    fn test_permissions_granted() {
        let perms = UiPermissions {
            camera: Some(Empty {}),
            microphone: None,
            geolocation: Some(Empty {}),
            clipboard_write: None,
        };
        
        assert!(perms.has_any());
        let granted = perms.granted();
        assert_eq!(granted.len(), 2);
        assert!(granted.contains(&"camera"));
        assert!(granted.contains(&"geolocation"));
    }
    
    #[test]
    fn test_approved_csp() {
        let csp = ApprovedCsp {
            connect_domains: Some(vec!["*.example.com".to_string(), "api.test.com".to_string()]),
            resource_domains: None,
            frame_domains: None,
            base_uri_domains: None,
        };
        
        assert!(csp.allows_connection("sub.example.com"));
        assert!(csp.allows_connection("api.test.com"));
        assert!(!csp.allows_connection("other.com"));
    }
    
    #[test]
    fn test_display_mode_serialization() {
        let mode = DisplayMode::Fullscreen;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"fullscreen\"");
        
        let parsed: DisplayMode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, DisplayMode::Fullscreen);
    }
}
