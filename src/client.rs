use dioxus::prelude::*;
use crate::server::EmbeddedServer;
use rmcp::model::*;
use serde_json::Value;

#[derive(Clone, Copy)]
pub struct AppClient {
    server: Signal<EmbeddedServer>,
    pub tools: Signal<Vec<Tool>>,
    pub current_ui: Signal<Option<String>>,
    pub current_data: Signal<Option<String>>,
}

impl AppClient {
    pub fn new() -> Self {
        Self {
            server: Signal::new(EmbeddedServer::new()),
            tools: Signal::new(Vec::new()),
            current_ui: Signal::new(None),
            current_data: Signal::new(None),
        }
    }

    pub async fn load_tools(mut self) {
        let srv = self.server.read().clone();
        if let Ok(res) = srv.list_tools().await {
             self.tools.set(res.tools);
        }
    }

    pub async fn run_tool(mut self, name: String) {
        let srv = self.server.read().clone();

        // Mock args
        let args = if name == "get_weather" {
            serde_json::json!({ "location": "San Francisco" })
        } else {
            serde_json::json!({})
        };

        // Reset UI while loading
        self.current_ui.set(None);
        self.current_data.set(None);

        if let Ok(tool_res) = srv.call_tool(&name, args).await {
            // Find UI URI
            let tools = self.tools.read();
            let uri = tools.iter()
                .find(|t| t.name == name)
                .and_then(|t| t.meta.as_ref())
                .and_then(|m: &Meta| m.0.get("ui"))
                .and_then(|u: &Value| u.get("resourceUri"))
                .and_then(|s: &Value| s.as_str())
                .map(|s| s.to_string());

            if let Some(uri_str) = uri {
                if let Ok(res) = srv.read_resource(&uri_str).await {
                    if let Some(content) = res.contents.first() {
                         // Matching struct variant
                         if let ResourceContents::TextResourceContents { text, .. } = content {
                             self.current_ui.set(Some(text.clone()));
                             self.current_data.set(Some(serde_json::to_string(&tool_res).unwrap()));
                         }
                    }
                }
            } else {
                // Fallback: just show the JSON result
                 self.current_data.set(Some(serde_json::to_string_pretty(&tool_res).unwrap()));
                 self.current_ui.set(Some(r#"
                    el("div", #{ "class": "p-4 bg-gray-100 rounded" }, [
                        el("pre", #{ "class": "whitespace-pre-wrap font-mono text-sm" }, [ text(data) ])
                    ])
                 "#.to_string()));
            }
        }
    }
}
