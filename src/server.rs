use rmcp::model::*;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Clone)]
pub struct EmbeddedServer;

impl EmbeddedServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn list_tools(&self) -> Result<ListToolsResult, String> {
        Ok(ListToolsResult {
            tools: vec![
                Tool {
                    name: "get_weather".to_string().into(),
                    title: Some("Get Weather".to_string().into()),
                    description: Some("Get current weather for a location".to_string().into()),
                    input_schema: Arc::new(json!({
                        "type": "object",
                        "properties": {
                            "location": { "type": "string" }
                        },
                        "required": ["location"]
                    }).as_object().unwrap().clone()),
                    output_schema: None,
                    annotations: None,
                    icons: None,
                    meta: Some(Meta(json!({
                        "ui": {
                            "resourceUri": "ui://weather"
                        }
                    }).as_object().unwrap().clone())),
                },
                Tool {
                    name: "get_portfolio".to_string().into(),
                    title: Some("Portfolio".to_string().into()),
                    description: Some("Get my professional portfolio".to_string().into()),
                    input_schema: Arc::new(json!({
                        "type": "object",
                        "properties": {},
                    }).as_object().unwrap().clone()),
                    output_schema: None,
                    annotations: None,
                    icons: None,
                    meta: Some(Meta(json!({
                        "ui": {
                            "resourceUri": "ui://portfolio"
                        }
                    }).as_object().unwrap().clone())),
                },
            ],
            next_cursor: None,
            meta: None,
        })
    }

    pub async fn call_tool(&self, name: &str, _arguments: Value) -> Result<CallToolResult, String> {
        match name {
            "get_weather" => {
                let res = CallToolResult {
                    content: vec![
                        Content::text("Sunny, 25°C")
                    ],
                    is_error: None,
                    structured_content: Some(serde_json::Value::Object(json!({
                        "temp": 25,
                        "conditions": "Sunny",
                        "location": "San Francisco", // Mock
                        "humidity": 45
                    }).as_object().unwrap().clone())),
                    meta: None,
                };
                Ok(res)
            },
            "get_portfolio" => {
                let res = CallToolResult {
                    content: vec![],
                    is_error: None,
                    structured_content: Some(serde_json::Value::Object(json!({
                        "projects": [
                            { "name": "MCP-Rust", "desc": "A Rust implementation of the Model Context Protocol." },
                            { "name": "Dioxus Dashboard", "desc": "A high-performance dashboard using Dioxus and Tailwind." },
                            { "name": "Generative UI", "desc": "Dynamic UI generation using Rhai scripts." },
                            { "name": "AI Agent", "desc": "An autonomous agent that builds apps." }
                        ]
                    }).as_object().unwrap().clone())),
                    meta: None,
                };
                Ok(res)
            },
            _ => Err(format!("Tool not found: {}", name)),
        }
    }

    pub async fn read_resource(&self, uri: &str) -> Result<ReadResourceResult, String> {
        match uri {
            "ui://weather" => {
                let script = r#"
                    el("div", #{ "class": "bg-gradient-to-br from-blue-400 to-blue-600 p-6 rounded-xl shadow-2xl text-white max-w-sm mx-auto transform transition-all hover:scale-105" }, [
                        el("div", #{ "class": "flex justify-between items-center mb-4" }, [
                            el("h2", #{ "class": "text-2xl font-bold" }, [ text(data.structured_content.location) ]),
                            el("span", #{ "class": "bg-white/20 px-3 py-1 rounded-full text-sm" }, [ text("Now") ])
                        ]),
                        el("div", #{ "class": "flex flex-col items-center my-6" }, [
                             el("span", #{ "class": "text-6xl font-bold mb-2" }, [ text(data.structured_content.temp.to_string() + "°") ]),
                             el("span", #{ "class": "text-xl font-medium tracking-wide" }, [ text(data.structured_content.conditions) ])
                        ]),
                        el("div", #{ "class": "flex justify-between mt-6 text-blue-100" }, [
                            el("div", #{ "class": "flex flex-col items-center" }, [
                                el("span", #{ "class": "text-xs uppercase" }, [ text("Humidity") ]),
                                el("span", #{ "class": "font-bold" }, [ text(data.structured_content.humidity.to_string() + "%") ])
                            ]),
                            el("div", #{ "class": "flex flex-col items-center" }, [
                                el("span", #{ "class": "text-xs uppercase" }, [ text("Wind") ]),
                                el("span", #{ "class": "font-bold" }, [ text("12 km/h") ])
                            ])
                        ])
                    ])
                "#;

                Ok(ReadResourceResult {
                    contents: vec![
                        ResourceContents::text(script, uri)
                    ],
                })
            },
            "ui://portfolio" => {
                let script = r#"
                    el("div", #{ "class": "p-8 bg-gray-50 min-h-screen font-sans" }, [
                        el("div", #{ "class": "max-w-6xl mx-auto" }, [
                            el("div", #{ "class": "text-center mb-16" }, [
                                el("h1", #{ "class": "text-5xl font-extrabold text-gray-900 mb-4 tracking-tight" }, [ text("Creative Portfolio") ]),
                                el("p", #{ "class": "text-xl text-gray-500 max-w-2xl mx-auto" }, [ text("Showcasing the intersection of Rust, WebAssembly, and Generative UI.") ])
                            ]),
                            el("div", #{ "class": "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 gap-10" },
                                data.structured_content.projects.map(|proj| {
                                    el("div", #{ "class": "bg-white rounded-2xl shadow-xl overflow-hidden hover:shadow-2xl transition-all duration-300 border border-gray-100 group" }, [
                                        el("div", #{ "class": "h-48 bg-indigo-600 flex items-center justify-center group-hover:bg-indigo-700 transition-colors" }, [
                                            el("span", #{ "class": "text-white text-4xl font-mono opacity-50" }, [ text("</>") ])
                                        ]),
                                        el("div", #{ "class": "p-8" }, [
                                            el("h3", #{ "class": "text-2xl font-bold text-gray-900 mb-3" }, [ text(proj.name) ]),
                                            el("p", #{ "class": "text-gray-600 mb-6 leading-relaxed" }, [ text(proj.desc) ]),
                                            el("div", #{ "class": "flex items-center text-indigo-600 font-semibold" }, [
                                                text("View Project"),
                                                el("span", #{ "class": "ml-2 text-xl" }, [ text("→") ])
                                            ])
                                        ])
                                    ])
                                })
                            )
                        ])
                    ])
                "#;

                Ok(ReadResourceResult {
                    contents: vec![
                        ResourceContents::text(script, uri)
                    ],
                })
            },
            _ => Err(format!("Resource not found: {}", uri)),
        }
    }
}
