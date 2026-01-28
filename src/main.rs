#![allow(non_snake_case)]

//! MCP Apps Host - Desktop Application
//!
//! A robust implementation of the MCP Apps specification (SEP-1865) using Dioxus.
//! This application acts as a host for MCP servers, enabling interactive UI apps.

use dioxus::prelude::*;

mod host;
mod protocol;
mod server;
mod ui;

use host::{ConnectionManager, ConnectionState, HostState};
use protocol::*;
use ui::{UiContent, UiContentRenderer, UiMessageEvent};

fn main() {
    dioxus::launch(App);
}

/// Main application component
#[component]
fn App() -> Element {
    rsx! {
        // Tailwind CDN as a reliable fallback
        script { src: "https://unpkg.com/@tailwindcss/browser@4" }
        // Link to local compiled tailwind using the asset! macro for Dioxus 0.7
        link { rel: "stylesheet", href: asset!("/assets/tailwind.css") }
        
        // Basic fallback styles to ensure layout works even without Tailwind
        style {
            r#"
            body, html {{ margin: 0; padding: 0; height: 100%; }}
            .flex {{ display: flex; }}
            .flex-col {{ flex-direction: column; }}
            .flex-1 {{ flex: 1 1 0%; }}
            .h-screen {{ height: 100vh; }}
            .w-64 {{ width: 16rem; }}
            .bg-white {{ background-color: #ffffff; }}
            .bg-gray-100 {{ background-color: #f3f4f6; }}
            .border-r {{ border-right: 1px solid #e5e7eb; }}
            .p-6 {{ padding: 1.5rem; }}
            .p-4 {{ padding: 1rem; }}
            .p-8 {{ padding: 2rem; }}
            .font-bold {{ font-weight: 700; }}
            .text-xl {{ font-size: 1.25rem; }}
            "#
        }
        McpHost {}
    }
}

/// Global application state
#[derive(Clone)]
struct AppState {
    /// Connection manager for MCP servers
    pub connection_manager: Signal<ConnectionManager>,
    /// Currently selected connection ID
    pub selected_connection: Signal<Option<String>>,
    /// Currently active UI session
    pub active_session: Signal<Option<ui::UiSessionState>>,
    /// UI content to display
    pub ui_content: Signal<UiContent>,
    /// Error message
    pub error_message: Signal<Option<String>>,
    /// Current display mode for the UI
    pub display_mode: Signal<DisplayMode>,
}

impl AppState {
    pub fn new() -> Self {
        let host_state = HostState::default();
        let connection_manager = ConnectionManager::new(host_state);
        
        Self {
            connection_manager: Signal::new(connection_manager),
            selected_connection: Signal::new(None),
            active_session: Signal::new(None),
            ui_content: Signal::new(UiContent::Loading),
            error_message: Signal::new(None),
            display_mode: Signal::new(DisplayMode::Inline),
        }
    }
}

/// Main MCP Host component
#[component]
fn McpHost() -> Element {
    // Initialize application state
    let app_state = use_context_provider(|| AppState::new());
    
    // Auto-connect to embedded server on mount
    let mut conn_signal = app_state.selected_connection;
    let mut err_signal = app_state.error_message;
    
    use_effect(move || {
        let manager = app_state.connection_manager.read().clone();
        spawn(async move {
            // Try connecting via stdio first
            match manager.connect_stdio("cargo", vec!["run", "--bin", "mcp-server"].into_iter().map(String::from).collect()).await {
                Ok(conn_id) => {
                    conn_signal.set(Some(conn_id));
                }
                Err(e) => {
                    log::warn!("Failed to connect via stdio: {}. Falling back to direct embedded connection.", e);
                    match manager.connect_embedded().await {
                        Ok(conn_id) => {
                            conn_signal.set(Some(conn_id));
                        }
                        Err(e) => {
                            err_signal.set(Some(format!("Failed to connect to embedded server: {}", e)));
                        }
                    }
                }
            }
        });
    });
    
    rsx! {
        div { class: "flex h-screen bg-gray-100 font-sans",
            // Sidebar
            Sidebar {}
            
            // Main Content
            MainContent {}
        }
    }
}

/// Sidebar component with tools list
#[component]
fn Sidebar() -> Element {
    let app_state = use_context::<AppState>();
    let mut tools = use_signal(Vec::new);
    
    // Refresh tools list periodically
    use_effect(move || {
        spawn(async move {
            loop {
                let manager = app_state.connection_manager.read().clone();
                let tools_with_ui = manager.get_tools_with_ui().await;
                
                let tool_list: Vec<(String, rmcp::model::Tool, String)> = tools_with_ui
                    .into_iter()
                    .map(|(conn_id, tool, uri)| (conn_id, tool, uri))
                    .collect();
                
                tools.set(tool_list);
                
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });
    });
    
    let tools_signal = tools.read();
    
    rsx! {
        div { class: "w-64 bg-white border-r border-gray-200 flex flex-col shadow-sm z-10",
            // Header
            div { class: "p-6 border-b border-gray-100",
                h1 { class: "text-xl font-bold text-gray-800 tracking-tight", "MCP Apps" }
                p { class: "text-xs text-gray-500 mt-1 font-medium", "Host Implementation" }
            }
            
            // Tools List
            div { class: "flex-1 overflow-y-auto p-4 space-y-2",
                if tools_signal.is_empty() {
                    div { class: "text-sm text-gray-400 text-center py-8",
                        "No tools with UI available"
                    }
                } else {
                    for (conn_id, tool, uri) in tools_signal.iter() {
                        ToolItem {
                            conn_id: conn_id.clone(),
                            tool: tool.clone(),
                            resource_uri: uri.clone(),
                        }
                    }
                }
            }
            
            // Footer
            div { class: "p-4 border-t border-gray-100 text-xs text-center text-gray-400",
                "MCP Apps Host v0.1.0"
            }
        }
    }
}

/// Tool item component
#[derive(Props, Clone, PartialEq)]
struct ToolItemProps {
    conn_id: String,
    tool: rmcp::model::Tool,
    resource_uri: String,
}

#[component]
fn ToolItem(props: ToolItemProps) -> Element {
    let app_state = use_context::<AppState>();
    let tool = props.tool.clone();
    let conn_id = props.conn_id.clone();
    let resource_uri = props.resource_uri.clone();
    let mut ui_content = app_state.ui_content;
    let mut active_session = app_state.active_session;
    
    let on_click = move |_| {
        let conn_id = conn_id.clone();
        let resource_uri = resource_uri.clone();
        let tool_name = tool.name.to_string();
        
        spawn(async move {
            // Set loading state
            ui_content.set(UiContent::Loading);
            
            // Create session
            let session = ui::UiSessionState::new(
                uuid::Uuid::new_v4().to_string(),
                conn_id.clone(),
                resource_uri.clone(),
            );
            active_session.set(Some(session));
            
            // Call the tool
            let manager = app_state.connection_manager.read().clone();
            let args = serde_json::json!({ "location": "San Francisco" });
            
            match manager.call_tool(&conn_id, &tool_name, args).await {
                Ok(result) => {
                    // Read the UI resource
                    match manager.read_ui_resource(&conn_id, &resource_uri).await {
                        Ok(resource_content) => {
                            let tool_result_json = serde_json::to_string(&result).unwrap_or_default();
                            let content = UiContent::from_resource_content(resource_content, Some(tool_result_json));
                            ui_content.set(content);
                        }
                        Err(e) => {
                            ui_content.set(UiContent::Error(format!("Failed to load UI: {}", e)));
                        }
                    }
                }
                Err(e) => {
                    ui_content.set(UiContent::Error(format!("Tool error: {}", e)));
                }
            }
        });
    };
    
    let name = props.tool.name.to_string();
    let description = props.tool.description.clone();
    
    rsx! {
        div {
            class: "p-3 rounded-lg hover:bg-indigo-50 cursor-pointer transition-all duration-200 border border-transparent hover:border-indigo-100 group",
            onclick: on_click,
            
            div { class: "font-semibold text-gray-700 group-hover:text-indigo-700", "{name}" }
            if let Some(desc) = description {
                div { class: "text-xs text-gray-400 mt-1 truncate", "{desc}" }
            }
        }
    }
}

/// Main content area
#[component]
fn MainContent() -> Element {
    let mut app_state = use_context::<AppState>();
    let ui_content = app_state.ui_content.read().clone();
    let mut display_mode = app_state.display_mode;
    let mut active_session = app_state.active_session;
    
    // Get host state and create host context
    let host_state = use_memo(move || {
        app_state.connection_manager.read().host_state.clone()
    });
    
    let host_context = use_memo(move || {
        host_state.read().to_host_context()
    });
    
    // Handle UI messages
    let handle_message = move |event: UiMessageEvent| {
        match event {
            UiMessageEvent::RequestDisplayMode { mode } => {
                log::info!("UI requested display mode: {:?}", mode);
                display_mode.set(mode.clone());
                // Update session display mode if active
                let session = active_session.read().as_ref().cloned();
                if let Some(mut session) = session {
                    session.display_mode = mode;
                    active_session.set(Some(session));
                }
            }
            UiMessageEvent::ToolCall { name, arguments } => {
                log::info!("UI requested tool call: {} with args {:?}", name, arguments);
                // Tool calls from UI would be handled here
                // This requires routing back to the connection manager
            }
            UiMessageEvent::UpdateModelContext { content, structured_content } => {
                log::info!("UI updated model context");
                // Handle context updates from UI
            }
            UiMessageEvent::Log { level, message } => {
                log::info!("[UI:{}] {}", level, message);
            }
            UiMessageEvent::OpenLink { url } => {
                log::info!("UI requested to open link: {}", url);
                // In a full implementation, this would open the link
                // with user confirmation based on capability negotiation
            }
            UiMessageEvent::SizeChanged { width, height } => {
                log::info!("UI size changed: {}x{}", width, height);
            }
            _ => {
                log::info!("UI Message: {:?}", event);
            }
        }
    };
    
    // Get display mode class
    let display_class = match display_mode.read().clone() {
        DisplayMode::Fullscreen => "fixed inset-0 z-50 bg-white",
        DisplayMode::Pip => "fixed bottom-4 right-4 w-96 h-64 z-50 bg-white shadow-2xl rounded-lg border border-gray-200",
        DisplayMode::Inline | _ => "",
    };
    
    let is_overlay = matches!(display_mode.read().clone(), DisplayMode::Fullscreen | DisplayMode::Pip);
    
    rsx! {
        div { class: "flex-1 flex flex-col overflow-hidden relative bg-white",
            // Content Area
            div { class: "flex-1 overflow-y-auto p-8 {display_class}",
                // Close button for expanded/fullscreen modes
                if is_overlay {
                    div { class: "absolute top-4 right-4 z-10",
                        button {
                            class: "p-2 bg-gray-100 hover:bg-gray-200 rounded-full text-gray-600 transition-colors",
                            onclick: move |_| display_mode.set(DisplayMode::Inline),
                            "✕"
                        }
                    }
                }
                
                match ui_content {
                    UiContent::Loading => {
                        rsx! {
                            div { class: "flex items-center justify-center h-full",
                                div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600" }
                            }
                        }
                    }
                    UiContent::Error(e) => {
                        rsx! {
                            div { class: "flex flex-col items-center justify-center h-full text-red-500",
                                div { class: "text-4xl mb-4", "⚠️" }
                                div { class: "text-lg font-medium", "Error" }
                                div { class: "text-sm mt-2", "{e}" }
                            }
                        }
                    }
                    _ => {
                        rsx! {
                            UiContentRenderer {
                                content: ui_content,
                                on_message: Some(EventHandler::new(handle_message)),
                                host_context: Some(host_context.read().clone()),
                            }
                        }
                    }
                }
            }
        }
    }
}
