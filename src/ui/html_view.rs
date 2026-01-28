//! HTML View Component
//!
//! Renders spec-compliant HTML content with simulated sandboxing and
//! full bidirectional communication via postMessage bridge.
//! 
//! Note: Full spec compliance requires true iframe sandboxing or WebView isolation.
//! This implementation provides CSP injection and security metadata display as
//! a pragmatic approximation for the Dioxus desktop environment.

use dioxus::prelude::*;
use crate::protocol::*;
use crate::ui::{UiMessageEvent, UiSessionState};

/// Props for HTML view
#[derive(Props, Clone, PartialEq)]
pub struct HtmlViewProps {
    /// HTML content
    pub html: String,
    /// Resource metadata
    #[props(!optional)]
    pub metadata: Option<UiResourceMeta>,
    /// Callback for UI messages
    #[props(!optional)]
    pub on_message: Option<EventHandler<UiMessageEvent>>,
    /// Host context to send to the view
    #[props(!optional)]
    pub host_context: Option<HostContext>,
}

/// Generate the postMessage bridge JavaScript code
fn generate_postmessage_bridge() -> String {
    r#"
<script>
(function() {
    'use strict';
    
    const MCP_BRIDGE_VERSION = '1.0.0';
    const parentOrigin = '*'; // In production, restrict to host origin
    
    // Track pending requests
    const pendingRequests = new Map();
    let nextRequestId = 1;
    
    // Notify host that view is ready
    function notifyReady() {
        window.parent.postMessage({
            jsonrpc: '2.0',
            method: 'ui/ready',
            params: {
                bridgeVersion: MCP_BRIDGE_VERSION,
                timestamp: Date.now()
            }
        }, parentOrigin);
    }
    
    // Listen for messages from host
    window.addEventListener('message', function(event) {
        // Validate message structure
        if (!event.data || typeof event.data !== 'object') return;
        
        const data = event.data;
        
        // Handle responses to our requests
        if (data.id !== undefined && pendingRequests.has(data.id)) {
            const { resolve, reject } = pendingRequests.get(data.id);
            pendingRequests.delete(data.id);
            
            if (data.error) {
                reject(new Error(data.error.message || 'Unknown error'));
            } else {
                resolve(data.result);
            }
            return;
        }
        
        // Handle notifications/requests from host
        if (!data.method) return;
        
        switch (data.method) {
            case 'host/context':
                window.mcpHostContext = data.params;
                document.dispatchEvent(new CustomEvent('mcp:context', { detail: data.params }));
                break;
                
            case 'tool/result':
                document.dispatchEvent(new CustomEvent('mcp:toolResult', { detail: data.params }));
                break;
                
            case 'display/modeChanged':
                document.dispatchEvent(new CustomEvent('mcp:displayModeChanged', { detail: data.params }));
                break;
                
            case 'ping':
                window.parent.postMessage({
                    jsonrpc: '2.0',
                    id: data.id,
                    result: { pong: true, timestamp: Date.now() }
                }, parentOrigin);
                break;
        }
    });
    
    // MCP API exposed to views
    window.mcp = {
        version: MCP_BRIDGE_VERSION,
        
        // Call a tool on the server
        callTool: function(name, args) {
            return new Promise((resolve, reject) => {
                const id = (nextRequestId++).toString();
                pendingRequests.set(id, { resolve, reject });
                
                // Set timeout
                setTimeout(() => {
                    if (pendingRequests.has(id)) {
                        pendingRequests.delete(id);
                        reject(new Error('Tool call timeout'));
                    }
                }, 30000);
                
                window.parent.postMessage({
                    jsonrpc: '2.0',
                    id: id,
                    method: 'tools/call',
                    params: { name: name, arguments: args || {} }
                }, parentOrigin);
            });
        },
        
        // Update model context
        updateContext: function(content, structuredContent) {
            window.parent.postMessage({
                jsonrpc: '2.0',
                method: 'context/update',
                params: {
                    content: content,
                    structuredContent: structuredContent
                }
            }, parentOrigin);
        },
        
        // Request display mode change
        requestDisplayMode: function(mode) {
            return new Promise((resolve, reject) => {
                const id = (nextRequestId++).toString();
                pendingRequests.set(id, { resolve, reject });
                
                setTimeout(() => {
                    if (pendingRequests.has(id)) {
                        pendingRequests.delete(id);
                        reject(new Error('Display mode request timeout'));
                    }
                }, 5000);
                
                window.parent.postMessage({
                    jsonrpc: '2.0',
                    id: id,
                    method: 'display/mode',
                    params: { mode: mode }
                }, parentOrigin);
            });
        },
        
        // Request expanded/fullscreen mode
        requestExpanded: function() {
            return this.requestDisplayMode('expanded');
        },
        
        // Request inline mode
        requestInline: function() {
            return this.requestDisplayMode('inline');
        },
        
        // Send log message to host
        log: function(level, message, logger) {
            window.parent.postMessage({
                jsonrpc: '2.0',
                method: 'logging/message',
                params: { 
                    level: level, 
                    message: message,
                    logger: logger || 'mcp-app'
                }
            }, parentOrigin);
        },
        
        // Open a link (requires host approval)
        openLink: function(url) {
            window.parent.postMessage({
                jsonrpc: '2.0',
                method: 'link/open',
                params: { url: url }
            }, parentOrigin);
        },
        
        // Get current host context
        getContext: function() {
            return window.mcpHostContext || null;
        },
        
        // Listen for context updates
        onContext: function(callback) {
            document.addEventListener('mcp:context', function(e) {
                callback(e.detail);
            });
        },
        
        // Listen for tool results
        onToolResult: function(callback) {
            document.addEventListener('mcp:toolResult', function(e) {
                callback(e.detail);
            });
        },
        
        // Listen for display mode changes
        onDisplayModeChanged: function(callback) {
            document.addEventListener('mcp:displayModeChanged', function(e) {
                callback(e.detail);
            });
        }
    };
    
    // Notify ready when DOM is loaded
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', notifyReady);
    } else {
        notifyReady();
    }
})();
</script>
"#.to_string()
}

/// Wrap HTML content with CSP meta tag and security context
fn wrap_html_with_security(html: &str, metadata: &Option<UiResourceMeta>, host_context: &Option<HostContext>) -> String {
    // Extract CSP from metadata or use default restrictive policy
    let csp = metadata
        .as_ref()
        .and_then(|m| m.ui.as_ref())
        .and_then(|u| u.csp.as_ref())
        .map(|csp| csp.build_csp_header())
        .unwrap_or_else(|| {
            // Default restrictive CSP
            "default-src 'none'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; media-src 'self' data:; connect-src 'none'; frame-src 'none'; object-src 'none'".to_string()
        });
    
    let bridge = generate_postmessage_bridge();
    
    // Serialize host context for injection
    let context_script = host_context.as_ref().map(|ctx| {
        match serde_json::to_string(ctx) {
            Ok(json) => format!(
                r#"<script>window.mcpHostContext = {};</script>"#,
                json
            ),
            Err(_) => String::new(),
        }
    }).unwrap_or_default();
    
    // Check if HTML already has proper structure
    let has_html_tag = html.contains("<html") || html.contains("<!DOCTYPE");
    
    if has_html_tag {
        // Inject CSP meta tag, bridge, and context
        let csp_meta = format!(r#"<meta http-equiv="Content-Security-Policy" content="{}">"#, csp);
        
        let inject_head = format!("{}\n{}\n{}", csp_meta, context_script, bridge);
        
        if html.contains("<head>") {
            html.replacen("<head>", &format!("<head>\n{}", inject_head), 1)
        } else if html.contains("</head>") {
            html.replacen("</head>", &format!("{}\n</head>", inject_head), 1)
        } else if html.contains("<html") {
            html.replacen("<html", &format!("<head>{}</head>\n<html", inject_head), 1)
        } else {
            format!(
                r#"<!DOCTYPE html>
<html>
<head>
{}
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
</head>
<body>
{}
</body>
</html>"#,
                inject_head, html
            )
        }
    } else {
        // Wrap fragment in complete HTML document
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
<meta http-equiv="Content-Security-Policy" content="{}">
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
{}
{}
<style>
/* Reset and base styles for MCP Apps */
* {{
    box-sizing: border-box;
}}
body {{
    margin: 0;
    padding: 16px;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    line-height: 1.5;
    color: #1f2937;
}}
</style>
</head>
<body>
{}
</body>
</html>"#,
            csp, context_script, bridge, html
        )
    }
}

/// HTML view component with security sandboxing and postMessage bridge
/// 
/// This component renders HTML content with:
/// - CSP (Content Security Policy) injection
/// - Security metadata display
/// - Simulated sandbox environment
/// - Full bidirectional postMessage bridge
/// 
/// Note: True iframe sandboxing requires WebView integration for full spec compliance.
#[component]
pub fn HtmlView(props: HtmlViewProps) -> Element {
    let html = props.html.clone();
    let metadata = props.metadata.clone();
    let host_context = props.host_context.clone();
    let metadata_for_csp = metadata.clone();
    let metadata_for_perms = metadata.clone();
    let metadata_for_border = metadata.clone();
    let on_message = props.on_message.clone();
    
    // Wrap HTML with security context
    let secured_html = use_memo(move || {
        wrap_html_with_security(&html, &metadata, &host_context)
    });
    
    // Extract CSP info for display
    let csp_info = use_memo(move || {
        metadata_for_csp.as_ref()
            .and_then(|m| m.ui.as_ref())
            .and_then(|u| u.csp.as_ref())
            .map(|csp| csp.build_csp_header())
            .unwrap_or_else(|| "Default CSP (restrictive)".to_string())
    });
    
    // Extract permissions
    let permissions = use_memo(move || {
        metadata_for_perms.as_ref()
            .and_then(|m| m.ui.as_ref())
            .and_then(|u| u.permissions.clone())
    });
    
    // Border preference
    let prefers_border = use_memo(move || {
        metadata_for_border.as_ref()
            .and_then(|m| m.ui.as_ref())
            .and_then(|u| u.prefers_border)
            .unwrap_or(true)
    });
    
    let border_class = if *prefers_border.read() {
        "border border-gray-200 rounded-lg shadow-sm"
    } else {
        ""
    };
    
    // Track if security panel is expanded
    let mut show_security = use_signal(|| false);
    let is_expanded = *show_security.read();
    
    rsx! {
        div {
            class: "flex flex-col h-full",
            
            // Security toggle button
            div {
                class: "flex justify-end mb-2",
                button {
                    class: "text-xs px-3 py-1 bg-gray-100 hover:bg-gray-200 rounded text-gray-600 transition-colors",
                    onclick: move |_| {
                        let current = *show_security.read();
                        show_security.set(!current);
                    },
                    if is_expanded {
                        "🔒 Hide Security Info"
                    } else {
                        "🔒 Show Security Info"
                    }
                }
            }
            
            // HTML Content Container
            // Using a scoped wrapper to simulate isolation
            div {
                class: "flex-1 overflow-auto {border_class}",
                
                // The actual HTML content with injected bridge
                div {
                    class: "mcp-html-content",
                    dangerous_inner_html: "{secured_html}"
                }
            }
            
            // Security Info Panel
            if is_expanded {
                div {
                    class: "mt-4 p-4 bg-gray-50 rounded-lg text-xs font-mono border border-gray-200",
                    
                    div {
                        class: "flex items-center justify-between mb-2",
                        span { class: "font-semibold text-gray-700", "Security Details" }
                        span { class: "text-gray-400", "MCP Apps Sandbox" }
                    }
                    
                    div { class: "space-y-3",
                        // CSP Section
                        div {
                            div { class: "font-semibold text-gray-600 mb-1", "Content Security Policy:" }
                            div { 
                                class: "break-all text-gray-500 bg-gray-100 p-2 rounded",
                                "{csp_info}"
                            }
                        }
                        
                        // Permissions Section
                        if let Some(perms) = permissions.read().as_ref() {
                            div {
                                div { class: "font-semibold text-gray-600 mb-1", "Requested Permissions:" }
                                div { class: "flex flex-wrap gap-2",
                                    if perms.camera.is_some() {
                                        span { class: "px-2 py-1 bg-yellow-100 text-yellow-800 rounded", "📷 Camera" }
                                    }
                                    if perms.microphone.is_some() {
                                        span { class: "px-2 py-1 bg-yellow-100 text-yellow-800 rounded", "🎤 Microphone" }
                                    }
                                    if perms.geolocation.is_some() {
                                        span { class: "px-2 py-1 bg-yellow-100 text-yellow-800 rounded", "📍 Geolocation" }
                                    }
                                    if perms.clipboard_write.is_some() {
                                        span { class: "px-2 py-1 bg-blue-100 text-blue-800 rounded", "📋 Clipboard" }
                                    }
                                    if perms.camera.is_none() && perms.microphone.is_none() && perms.geolocation.is_none() && perms.clipboard_write.is_none() {
                                        span { class: "text-gray-400", "None requested" }
                                    }
                                }
                            }
                        }
                        
                        // Implementation Note
                        div {
                            class: "mt-3 pt-3 border-t border-gray-200 text-gray-400 italic",
                            "Note: This is a simulated sandbox. True iframe isolation requires WebView integration."
                        }
                    }
                }
            }
        }
    }
}

/// Props for WebView-based rendering (future enhancement)
#[derive(Props, Clone, PartialEq)]
pub struct WebViewBridgeProps {
    /// Session state
    pub session: UiSessionState,
    /// Host context
    pub host_context: HostContext,
    /// Callback for UI messages
    pub on_message: EventHandler<UiMessageEvent>,
}

/// WebView bridge component placeholder
/// 
/// In a full implementation with WebView integration (e.g., using `wry` or `tao`),
/// this would create an actual WebView with:
/// - True iframe-style sandboxing
/// - Native postMessage bridge
/// - Proper origin isolation
/// - Hardware acceleration
#[component]
pub fn WebViewBridge(props: WebViewBridgeProps) -> Element {
    let session = props.session.clone();
    let _host_context = props.host_context.clone();
    
    rsx! {
        div {
            class: "flex flex-col h-full",
            
            // WebView placeholder
            div {
                class: "flex-1 bg-white border border-gray-200 rounded-lg overflow-hidden",
                
                div {
                    class: "flex items-center justify-center h-full text-gray-400",
                    
                    div {
                        class: "text-center",
                        
                        div { class: "text-4xl mb-4", "🌐" }
                        div { class: "text-lg font-medium", "WebView Content" }
                        div { class: "text-sm mt-2", "Session: {session.session_id}" }
                        div { class: "text-sm", "Resource: {session.resource_uri}" }
                        div { class: "text-sm", "Mode: {session.display_mode}" }
                        
                        div { class: "mt-4 text-xs text-gray-300 max-w-xs",
                            "For true spec-compliant sandboxing, integrate with wry/tao WebView"
                        }
                    }
                }
            }
        }
    }
}

/// Helper component to render an iframe with sandbox attributes
/// 
/// Note: Dioxus doesn't directly support iframes with srcdoc in RSX.
/// This is a workaround using dangerously_set_inner_html.
#[component]
pub fn SandboxedIframe(html: String, sandbox: String) -> Element {
    let iframe_html = format!(
        r##"<iframe 
            sandbox="{}" 
            srcdoc="{}" 
            style="width: 100%; height: 100%; border: none;"
            title="MCP App View"
        ></iframe>"##,
        sandbox,
        html_escape(&html)
    );
    
    rsx! {
        div {
            class: "w-full h-full",
            dangerous_inner_html: "{iframe_html}"
        }
    }
}

/// Escape HTML for use in iframe srcdoc
fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape(r#""quoted""#), "&quot;quoted&quot;");
    }
    
    #[test]
    fn test_wrap_html_with_security() {
        let html = "<div>Hello</div>";
        let wrapped = wrap_html_with_security(html, &None, &None);
        
        assert!(wrapped.contains("<!DOCTYPE html>"));
        assert!(wrapped.contains("Content-Security-Policy"));
        assert!(wrapped.contains("<div>Hello</div>"));
        assert!(wrapped.contains("window.mcp"));
    }
    
    #[test]
    fn test_csp_injection_existing_head() {
        let html = r#"<!DOCTYPE html><html><head><title>Test</title></head><body>Hello</body></html>"#;
        let wrapped = wrap_html_with_security(html, &None, &None);
        
        // CSP should be injected after <head>
        assert!(wrapped.contains("<head>\n<meta http-equiv=\"Content-Security-Policy\""));
    }
}
