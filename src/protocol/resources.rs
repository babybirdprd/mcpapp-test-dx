//! MCP Apps Resource Types
//!
//! Types for UI resources (ui:// scheme) and their metadata.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// UI Resource type as defined in the spec
/// 
/// URI MUST start with `ui://` scheme
/// mimeType MUST be `text/html;profile=mcp-app` (other types reserved for future extensions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiResource {
    /// Unique identifier for the UI resource (MUST use `ui://` URI scheme)
    pub uri: String,
    /// Human-readable display name for the UI resource
    pub name: String,
    /// Optional description of the UI resource's purpose
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type of the UI content (SHOULD be `text/html;profile=mcp-app`)
    pub mime_type: String,
    /// Resource metadata for security and rendering configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<UiResourceMeta>,
}

/// UI Resource metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UiResourceMeta {
    /// UI-specific metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui: Option<UiResourceDetails>,
}

/// UI Resource details within metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiResourceDetails {
    /// Content Security Policy configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csp: Option<McpUiResourceCsp>,
    /// Sandbox permissions requested by the UI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<UiResourcePermissions>,
    /// Dedicated origin for view
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Visual boundary preference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefers_border: Option<bool>,
}

/// Content Security Policy configuration for UI resources
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpUiResourceCsp {
    /// Origins for network requests (fetch/XHR/WebSocket)
    /// Maps to CSP `connect-src` directive
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_domains: Option<Vec<String>>,
    /// Origins for static resources (images, scripts, stylesheets, fonts, media)
    /// Maps to CSP `img-src`, `script-src`, `style-src`, `font-src`, `media-src` directives
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_domains: Option<Vec<String>>,
    /// Origins for nested iframes
    /// Maps to CSP `frame-src` directive
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_domains: Option<Vec<String>>,
    /// Allowed base URIs for the document
    /// Maps to CSP `base-uri` directive
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_uri_domains: Option<Vec<String>>,
}

/// UI Resource permissions
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiResourcePermissions {
    /// Request camera access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera: Option<super::capabilities::Empty>,
    /// Request microphone access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub microphone: Option<super::capabilities::Empty>,
    /// Request geolocation access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geolocation: Option<super::capabilities::Empty>,
    /// Request clipboard write access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clipboard_write: Option<super::capabilities::Empty>,
}

/// UI Resource content returned from resources/read
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiResourceContent {
    /// Matching UI resource URI
    pub uri: String,
    /// MIME type (MUST be "text/html;profile=mcp-app")
    pub mime_type: String,
    /// HTML content as string (either text or blob must be provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Base64-encoded HTML (alternative to text)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
    /// Resource metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<UiResourceMeta>,
}

/// Tool metadata linking to UI resources
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpUiToolMeta {
    /// URI of UI resource for rendering tool results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_uri: Option<String>,
    /// Who can access this tool. Default: ["model", "app"]
    /// - "model": Tool visible to and callable by the agent
    /// - "app": Tool callable by the app from this server only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Vec<ToolVisibility>>,
}

/// Tool visibility options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolVisibility {
    /// Tool visible to and callable by the agent
    Model,
    /// Tool callable by the app from this server only
    App,
}

/// Style configuration for theming
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiStyleConfig {
    /// CSS variables for theming
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Value>,
    /// CSS blocks that Views can inject
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css: Option<UiCssConfig>,
}

/// CSS configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UiCssConfig {
    /// CSS for font loading (@font-face rules or @import statements)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fonts: Option<String>,
}

impl UiResource {
    /// Check if the URI is a valid UI resource URI (starts with ui://)
    pub fn is_valid_uri(uri: &str) -> bool {
        uri.starts_with("ui://")
    }
    
    /// Get the recommended mime type for UI resources
    pub fn recommended_mime_type() -> &'static str {
        "text/html;profile=mcp-app"
    }
    
    /// Check if this is the standard HTML content type
    pub fn is_html_content_type(mime_type: &str) -> bool {
        mime_type == "text/html;profile=mcp-app" || mime_type.starts_with("text/html")
    }
}

impl McpUiResourceCsp {
    /// Build a CSP header string from the configuration
    pub fn build_csp_header(&self) -> String {
        let mut parts = Vec::new();
        
        // Default restrictive policy
        parts.push("default-src 'none'".to_string());
        parts.push("script-src 'self' 'unsafe-inline'".to_string());
        parts.push("style-src 'self' 'unsafe-inline'".to_string());
        parts.push("img-src 'self' data:".to_string());
        parts.push("media-src 'self' data:".to_string());
        
        // Connect-src
        if let Some(domains) = &self.connect_domains {
            if domains.is_empty() {
                parts.push("connect-src 'none'".to_string());
            } else {
                parts.push(format!("connect-src {}", domains.join(" ")));
            }
        } else {
            parts.push("connect-src 'none'".to_string());
        }
        
        // Frame-src
        if let Some(domains) = &self.frame_domains {
            if domains.is_empty() {
                parts.push("frame-src 'none'".to_string());
            } else {
                parts.push(format!("frame-src {}", domains.join(" ")));
            }
        } else {
            parts.push("frame-src 'none'".to_string());
        }
        
        // Object-src (always block)
        parts.push("object-src 'none'".to_string());
        
        parts.join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ui_uri_validation() {
        assert!(UiResource::is_valid_uri("ui://weather"));
        assert!(UiResource::is_valid_uri("ui://server/dashboard"));
        assert!(!UiResource::is_valid_uri("file://weather"));
        assert!(!UiResource::is_valid_uri("https://example.com"));
    }
    
    #[test]
    fn test_csp_header_building() {
        let csp = McpUiResourceCsp {
            connect_domains: Some(vec!["https://api.example.com".to_string()]),
            resource_domains: Some(vec!["https://cdn.example.com".to_string()]),
            frame_domains: None,
            base_uri_domains: None,
        };
        
        let header = csp.build_csp_header();
        assert!(header.contains("connect-src https://api.example.com"));
        assert!(header.contains("frame-src 'none'"));
        assert!(header.contains("object-src 'none'"));
    }
}
