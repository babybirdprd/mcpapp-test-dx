#![allow(non_snake_case)]
use dioxus::prelude::*;
use crate::client::AppClient;
use crate::rhai_ui::RhaiRenderer;

mod server;
mod client;
mod rhai_ui;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        // Tailwind is automatically handled by Dioxus 0.7 if tailwind.css exists
        MainLayout {}
    }
}

#[component]
fn MainLayout() -> Element {
    // Initialize the client state
    let mut client = use_context_provider(|| Signal::new(AppClient::new()));

    // Load tools on mount
    use_effect(move || {
        spawn(async move {
            client.write().load_tools().await;
        });
    });

    // Access signals
    let tools_signal = client.read().tools;
    let ui_script_signal = client.read().current_ui;
    let ui_data_signal = client.read().current_data;

    rsx! {
        div { class: "flex h-screen bg-gray-100 font-sans",
            // Sidebar
            div { class: "w-64 bg-white border-r border-gray-200 flex flex-col shadow-sm z-10",
                div { class: "p-6 border-b border-gray-100",
                    h1 { class: "text-xl font-bold text-gray-800 tracking-tight", "MCP Apps" }
                    p { class: "text-xs text-gray-500 mt-1 font-medium", "Dioxus + Rhai + MCP" }
                }
                div { class: "flex-1 overflow-y-auto p-4 space-y-2",
                    for tool in tools_signal.read().clone() {
                         div {
                            class: "p-3 rounded-lg hover:bg-indigo-50 cursor-pointer transition-all duration-200 border border-transparent hover:border-indigo-100 group",
                            onclick: move |_| {
                                let name = tool.name.clone();
                                spawn(async move {
                                    client.write().run_tool(name.to_string()).await;
                                });
                            },
                            div { class: "font-semibold text-gray-700 group-hover:text-indigo-700", "{tool.name}" }
                            if let Some(desc) = &tool.description {
                                div { class: "text-xs text-gray-400 mt-1 truncate", "{desc}" }
                            }
                         }
                    }
                }
                div { class: "p-4 border-t border-gray-100 text-xs text-center text-gray-400",
                    "Powered by rmcp"
                }
            }

            // Main Content
            div { class: "flex-1 flex flex-col overflow-hidden relative",
                // Content Area
                div { class: "flex-1 overflow-y-auto p-8",
                     if let Some(script) = ui_script_signal.read().as_ref() {
                        if let Some(data) = ui_data_signal.read().as_ref() {
                            RhaiRenderer { script: script.clone(), context: data.clone() }
                        } else {
                            div { class: "flex items-center justify-center h-full",
                                div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600" }
                            }
                        }
                     } else {
                         div { class: "h-full flex flex-col items-center justify-center text-gray-400 space-y-4",
                            div { class: "text-6xl filter grayscale opacity-50", "⚡" }
                            p { class: "text-lg font-medium", "Select a tool from the sidebar to launch an App." }
                         }
                     }
                }
            }
        }
    }
}
