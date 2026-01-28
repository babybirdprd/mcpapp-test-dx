//! MCP Apps UI Module
//!
//! Handles rendering of UI content from MCP servers, including both
//! spec-compliant HTML content and the custom Rhai scripting extension.

pub mod rhai_renderer;
pub mod html_view;
pub mod bridge;

pub use rhai_renderer::*;
pub use html_view::*;
pub use bridge::*;

use crate::protocol::*;
use dioxus::prelude::*;

/// UI content types that can be rendered
#[derive(Debug, Clone, PartialEq)]
pub enum UiContent {
    /// Standard HTML content (spec-compliant)
    Html {
        /// HTML content
        content: String,
        /// Resource metadata
        metadata: Option<UiResourceMeta>,
    },
    /// Rhai script (custom extension)
    RhaiScript {
        /// Rhai script
        script: String,
        /// Context data
        context: String,
    },
    /// Error state
    Error(String),
    /// Loading state
    Loading,
}

impl UiContent {
    /// Create UI content from a UiResourceContent
    pub fn from_resource_content(content: UiResourceContent, tool_result: Option<String>) -> Self {
        let html = content.text.or(content.blob.map(|b| {
            // Decode base64 if needed
            String::from_utf8_lossy(&base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &b).unwrap_or_default()).to_string()
        }));
        
        if let Some(html) = html {
            // Check if it's actually a Rhai script (custom extension)
            if html.trim_start().starts_with("el(") || html.contains("data.") {
                // It's likely a Rhai script
                return UiContent::RhaiScript {
                    script: html,
                    context: tool_result.unwrap_or_else(|| "{}".to_string()),
                };
            }
            
            UiContent::Html {
                content: html,
                metadata: content._meta,
            }
        } else {
            UiContent::Error("No content in resource".to_string())
        }
    }
    
    /// Check if content is HTML
    pub fn is_html(&self) -> bool {
        matches!(self, UiContent::Html { .. })
    }
    
    /// Check if content is a Rhai script
    pub fn is_rhai(&self) -> bool {
        matches!(self, UiContent::RhaiScript { .. })
    }
}

/// Props for UI content renderer
#[derive(Props, Clone, PartialEq)]
pub struct UiContentProps {
    /// The content to render
    pub content: UiContent,
    /// Callback for UI messages
    #[props(!optional)]
    pub on_message: Option<EventHandler<UiMessageEvent>>,
    /// Host context to send to the view
    #[props(!optional)]
    pub host_context: Option<HostContext>,
}

/// UI message event from the view
#[derive(Debug, Clone)]
pub enum UiMessageEvent {
    /// Tool call request
    ToolCall { name: String, arguments: serde_json::Value },
    /// Message to host
    Message { role: String, content: serde_json::Value },
    /// Open link request
    OpenLink { url: String },
    /// Display mode change request
    RequestDisplayMode { mode: DisplayMode },
    /// Update model context
    UpdateModelContext { content: Option<Vec<serde_json::Value>>, structured_content: Option<serde_json::Value> },
    /// Log message
    Log { level: String, message: String },
    /// Size changed
    SizeChanged { width: u32, height: u32 },
    /// Generic JSON-RPC message
    JsonRpc(serde_json::Value),
}

/// Main UI content renderer component
#[component]
pub fn UiContentRenderer(props: UiContentProps) -> Element {
    match &props.content {
        UiContent::Html { content, metadata } => {
            rsx! {
                HtmlView {
                    html: content.clone(),
                    metadata: metadata.clone(),
                    on_message: props.on_message.clone(),
                    host_context: props.host_context.clone(),
                }
            }
        }
        UiContent::RhaiScript { script, context } => {
            rsx! {
                RhaiRenderer {
                    script: script.clone(),
                    context: context.clone(),
                }
            }
        }
        UiContent::Error(e) => {
            rsx! {
                div {
                    class: "p-4 bg-red-50 border border-red-200 rounded-lg text-red-700",
                    "Error: {e}"
                }
            }
        }
        UiContent::Loading => {
            rsx! {
                div {
                    class: "flex items-center justify-center h-full",
                    div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600" }
                }
            }
        }
    }
}

/// UI session state for tracking active sessions
#[derive(Debug, Clone, PartialEq)]
pub struct UiSessionState {
    /// Session ID
    pub session_id: String,
    /// Connection ID
    pub connection_id: String,
    /// Resource URI
    pub resource_uri: String,
    /// Current content
    pub content: UiContent,
    /// Display mode
    pub display_mode: DisplayMode,
    /// Tool info (if triggered by a tool)
    pub tool_info: Option<ToolInfo>,
}

impl UiSessionState {
    pub fn new(
        session_id: impl Into<String>,
        connection_id: impl Into<String>,
        resource_uri: impl Into<String>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            connection_id: connection_id.into(),
            resource_uri: resource_uri.into(),
            content: UiContent::Loading,
            display_mode: DisplayMode::Inline,
            tool_info: None,
        }
    }
}
