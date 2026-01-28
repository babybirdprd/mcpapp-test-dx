//! Embedded MCP Server
//!
//! This module provides an embedded MCP server for demonstration and testing.
//! It can also be built as a standalone binary for stdio transport testing.

use crate::protocol::*;
use rmcp::model::{CallToolResult, Content, ListToolsResult, ListResourcesResult, ReadResourceResult, ResourceContents, Tool, Meta, RawResource, Annotated};
use serde_json::{json, Value};
use std::sync::Arc;

/// Embedded MCP server implementing the MCP Apps specification
#[derive(Clone)]
pub struct EmbeddedServer {
    server_info: ServerInfo,
}

#[derive(Clone, Default)]
struct ServerInfo {
    name: String,
    version: String,
}

impl EmbeddedServer {
    /// Create a new embedded server
    pub fn new() -> Self {
        Self {
            server_info: ServerInfo {
                name: "mcp-apps-embedded-server".to_string(),
                version: "0.1.0".to_string(),
            },
        }
    }
    
    /// Get server capabilities
    pub fn get_capabilities(&self) -> Value {
        json!({
            "experimental": {
                UI_EXTENSION_ID: {
                    "supportedDisplayModes": ["inline", "fullscreen"],
                    "supportsSandboxing": true
                }
            },
            "tools": {
                "listChanged": true
            },
            "resources": {
                "listChanged": true
            }
        })
    }
    
    /// Get server info
    pub fn get_server_info(&self) -> Value {
        json!({
            "name": self.server_info.name,
            "version": self.server_info.version
        })
    }
    
    /// List available tools
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
                            "location": { "type": "string", "description": "City name or location" }
                        },
                        "required": ["location"]
                    }).as_object().unwrap().clone()),
                    output_schema: None,
                    annotations: None,
                    icons: None,
                    meta: Some(Meta(json!({
                        "ui": {
                            "resourceUri": "ui://weather-server/dashboard",
                            "visibility": ["model", "app"]
                        }
                    }).as_object().unwrap().clone())),
                },
                Tool {
                    name: "get_portfolio".to_string().into(),
                    title: Some("Portfolio Gallery".to_string().into()),
                    description: Some("View professional portfolio".to_string().into()),
                    input_schema: Arc::new(json!({
                        "type": "object",
                        "properties": {},
                    }).as_object().unwrap().clone()),
                    output_schema: None,
                    annotations: None,
                    icons: None,
                    meta: Some(Meta(json!({
                        "ui": {
                            "resourceUri": "ui://portfolio-server/gallery",
                            "visibility": ["model", "app"]
                        }
                    }).as_object().unwrap().clone())),
                },
                Tool {
                    name: "get_system_status".to_string().into(),
                    title: Some("System Status".to_string().into()),
                    description: Some("Monitor system performance metrics".to_string().into()),
                    input_schema: Arc::new(json!({
                        "type": "object",
                        "properties": {},
                    }).as_object().unwrap().clone()),
                    output_schema: None,
                    annotations: None,
                    icons: None,
                    meta: Some(Meta(json!({
                        "ui": {
                            "resourceUri": "ui://system-server/status",
                            "visibility": ["model", "app"]
                        }
                    }).as_object().unwrap().clone())),
                },
                Tool {
                    name: "create_note".to_string().into(),
                    title: Some("Create Note".to_string().into()),
                    description: Some("Create a new sticky note".to_string().into()),
                    input_schema: Arc::new(json!({
                        "type": "object",
                        "properties": {
                            "title": { "type": "string" },
                            "content": { "type": "string" }
                        },
                        "required": ["title", "content"]
                    }).as_object().unwrap().clone()),
                    output_schema: None,
                    annotations: None,
                    icons: None,
                    meta: Some(Meta(json!({
                        "ui": {
                            "resourceUri": "ui://notes-server/editor",
                            "visibility": ["model", "app"]
                        }
                    }).as_object().unwrap().clone())),
                },
                Tool {
                    name: "refresh_weather".to_string().into(),
                    title: Some("Refresh Weather".to_string().into()),
                    description: Some("Refresh weather data (app-only)".to_string().into()),
                    input_schema: Arc::new(json!({
                        "type": "object",
                        "properties": {},
                    }).as_object().unwrap().clone()),
                    output_schema: None,
                    annotations: None,
                    icons: None,
                    meta: Some(Meta(json!({
                        "ui": {
                            "resourceUri": "ui://weather-server/dashboard",
                            "visibility": ["app"]  // App-only, hidden from model
                        }
                    }).as_object().unwrap().clone())),
                },
            ],
            next_cursor: None,
            meta: None,
        })
    }
    
    /// Call a tool
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<CallToolResult, String> {
        match name {
            "get_weather" => {
                let location = arguments
                    .get("location")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");
                
                let res = CallToolResult {
                    content: vec![
                        Content::text(format!("Sunny, 25°C in {}", location))
                    ],
                    is_error: None,
                    structured_content: Some(serde_json::Value::Object(json!({
                        "temp": 25,
                        "conditions": "Sunny",
                        "location": location,
                        "humidity": 45,
                        "wind_speed": 12,
                        "forecast": [
                            { "day": "Today", "high": 25, "low": 18, "conditions": "Sunny" },
                            { "day": "Tomorrow", "high": 23, "low": 17, "conditions": "Partly Cloudy" },
                            { "day": "Wednesday", "high": 22, "low": 16, "conditions": "Cloudy" },
                        ]
                    }).as_object().unwrap().clone())),
                    meta: None,
                };
                Ok(res)
            }
            "refresh_weather" => {
                // Same as get_weather but app-only - duplicate to avoid recursive async
                let location = arguments
                    .get("location")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");
                
                let res = CallToolResult {
                    content: vec![
                        Content::text(format!("Sunny, 25°C in {}", location))
                    ],
                    is_error: None,
                    structured_content: Some(serde_json::Value::Object(json!({
                        "temp": 25,
                        "conditions": "Sunny",
                        "location": location,
                        "humidity": 45,
                        "wind_speed": 12,
                        "forecast": [
                            { "day": "Today", "high": 25, "low": 18, "conditions": "Sunny" },
                            { "day": "Tomorrow", "high": 23, "low": 17, "conditions": "Partly Cloudy" },
                            { "day": "Wednesday", "high": 22, "low": 16, "conditions": "Cloudy" },
                        ]
                    }).as_object().unwrap().clone())),
                    meta: None,
                };
                Ok(res)
            }
            "get_portfolio" => {
                let res = CallToolResult {
                    content: vec![],
                    is_error: None,
                    structured_content: Some(serde_json::Value::Object(json!({
                        "owner": "Developer Portfolio",
                        "bio": "Full-stack developer specializing in Rust and web technologies",
                        "projects": [
                            { 
                                "name": "MCP-Rust", 
                                "desc": "A Rust implementation of the Model Context Protocol.",
                                "tech": ["Rust", "Tokio", "JSON-RPC"],
                                "stars": 128
                            },
                            { 
                                "name": "Dioxus Dashboard", 
                                "desc": "A high-performance dashboard using Dioxus.",
                                "tech": ["Rust", "Dioxus", "Tailwind"],
                                "stars": 256
                            },
                            { 
                                "name": "Generative UI", 
                                "desc": "Dynamic UI generation using Rhai scripts.",
                                "tech": ["Rhai", "Rust", "MCP"],
                                "stars": 64
                            },
                            { 
                                "name": "AI Agent", 
                                "desc": "An autonomous agent that builds applications.",
                                "tech": ["Rust", "OpenAI", "MCP"],
                                "stars": 512
                            }
                        ],
                        "skills": ["Rust", "TypeScript", "React", "Dioxus", "MCP", "AI/ML"]
                    }).as_object().unwrap().clone())),
                    meta: None,
                };
                Ok(res)
            }
            "get_system_status" => {
                let res = CallToolResult {
                    content: vec![],
                    is_error: None,
                    structured_content: Some(serde_json::Value::Object(json!({
                        "cpu_usage": 45,
                        "memory_usage": 62,
                        "disk_usage": 28,
                        "uptime": "14d 2h 15m",
                        "active_processes": 142,
                        "services": [
                            { "name": "Database", "status": "online", "latency": "12ms" },
                            { "name": "Web Server", "status": "online", "latency": "4ms" },
                            { "name": "Auth Service", "status": "online", "latency": "45ms" },
                            { "name": "Worker Node", "status": "maintenance", "latency": "--" }
                        ]
                    }).as_object().unwrap().clone())),
                    meta: None,
                };
                Ok(res)
            }
            "create_note" => {
                let title = arguments.get("title").and_then(|v| v.as_str()).unwrap_or("New Note");
                let content = arguments.get("content").and_then(|v| v.as_str()).unwrap_or("");
                
                let res = CallToolResult {
                    content: vec![Content::text("Note created")],
                    is_error: None,
                    structured_content: Some(serde_json::Value::Object(json!({
                        "title": title,
                        "content": content,
                        "created_at": "Just now",
                        "id": uuid::Uuid::new_v4().to_string()
                    }).as_object().unwrap().clone())),
                    meta: None,
                };
                Ok(res)
            }
            _ => Err(format!("Tool not found: {}", name)),
        }
    }
    
    /// List available resources
    pub async fn list_resources(&self) -> Result<ListResourcesResult, String> {
        let weather_resource = RawResource {
            uri: "ui://weather-server/dashboard".to_string(),
            name: "Weather Dashboard".to_string(),
            title: None,
            description: Some("Interactive weather visualization dashboard".to_string()),
            mime_type: Some("text/html;profile=mcp-app".to_string()),
            size: None,
            icons: None,
            meta: Some(Meta(json!({
                "ui": {
                    "csp": {
                        "connectDomains": ["https://api.openweathermap.org"]
                    },
                    "prefersBorder": true
                }
            }).as_object().unwrap().clone())),
        };
        
        let portfolio_resource = RawResource {
            uri: "ui://portfolio-server/gallery".to_string(),
            name: "Portfolio Gallery".to_string(),
            title: None,
            description: Some("Professional portfolio gallery view".to_string()),
            mime_type: Some("text/html;profile=mcp-app".to_string()),
            size: None,
            icons: None,
            meta: Some(Meta(json!({
                "ui": {
                    "prefersBorder": true
                }
            }).as_object().unwrap().clone())),
        };

        let system_resource = RawResource {
            uri: "ui://system-server/status".to_string(),
            name: "System Status".to_string(),
            title: None,
            description: Some("System performance monitor".to_string()),
            mime_type: Some("text/html;profile=mcp-app".to_string()),
            size: None,
            icons: None,
            meta: Some(Meta(json!({ "ui": { "prefersBorder": true } }).as_object().unwrap().clone())),
        };

        let notes_resource = RawResource {
            uri: "ui://notes-server/editor".to_string(),
            name: "Note Editor".to_string(),
            title: None,
            description: Some("Note creation interface".to_string()),
            mime_type: Some("text/html;profile=mcp-app".to_string()),
            size: None,
            icons: None,
            meta: Some(Meta(json!({ "ui": { "prefersBorder": true } }).as_object().unwrap().clone())),
        };
        
        Ok(ListResourcesResult {
            resources: vec![
                Annotated::new(weather_resource, None),
                Annotated::new(portfolio_resource, None),
                Annotated::new(system_resource, None),
                Annotated::new(notes_resource, None),
            ],
            next_cursor: None,
            meta: None,
        })
    }
    
    /// Read a resource
    pub async fn read_resource(&self, uri: &str) -> Result<ReadResourceResult, String> {
        match uri {
            "ui://weather-server/dashboard" => {
                // Return a Rhai script for native rendering
                // In production, this would be HTML with proper MCP Apps lifecycle
                let script = r#"
                    let content = if data.structured_content != () { data.structured_content } else { #{} };
                    let location = if "location" in content { content.location } else { "Loading..." };
                    let temp = if "temp" in content { content.temp.to_string() + "°" } else { "--°" };
                    let conditions = if "conditions" in content { content.conditions } else { "Please wait" };
                    let humidity = if "humidity" in content { content.humidity.to_string() + "%" } else { "--%" };
                    let wind = if "wind_speed" in content { content.wind_speed.to_string() + " km/h" } else { "-- km/h" };
                    let forecast_data = if "forecast" in content { content.forecast } else { [] };

                    let forecast_items = [];
                    for day in forecast_data {
                        forecast_items.push(el("div", #{ "class": "flex justify-between items-center bg-white/10 rounded px-3 py-2" }, [
                            el("span", #{ "class": "text-sm" }, [ text(day.day) ]),
                            el("span", #{ "class": "text-sm font-medium" }, [ text(day.high.to_string() + "° / " + day.low.to_string() + "°") ]),
                            el("span", #{ "class": "text-xs" }, [ text(day.conditions) ])
                        ]));
                    }

                    return el("div", #{ "class": "bg-gradient-to-br from-blue-400 to-blue-600 p-6 rounded-xl shadow-2xl text-white max-w-sm mx-auto transform transition-all hover:scale-105" }, [
                        el("div", #{ "class": "flex justify-between items-center mb-4" }, [
                            el("h2", #{ "class": "text-2xl font-bold" }, [ text(location) ]),
                            el("span", #{ "class": "bg-white/20 px-3 py-1 rounded-full text-sm" }, [ text("Now") ])
                        ]),
                        el("div", #{ "class": "flex flex-col items-center my-6" }, [
                             el("span", #{ "class": "text-6xl font-bold mb-2" }, [ text(temp) ]),
                             el("span", #{ "class": "text-xl font-medium tracking-wide" }, [ text(conditions) ])
                        ]),
                        el("div", #{ "class": "flex justify-between mt-6 text-blue-100" }, [
                            el("div", #{ "class": "flex flex-col items-center" }, [
                                el("span", #{ "class": "text-xs uppercase" }, [ text("Humidity") ]),
                                el("span", #{ "class": "font-bold" }, [ text(humidity) ])
                            ]),
                            el("div", #{ "class": "flex flex-col items-center" }, [
                                el("span", #{ "class": "text-xs uppercase" }, [ text("Wind") ]),
                                el("span", #{ "class": "font-bold" }, [ text(wind) ])
                            ])
                        ]),
                        el("div", #{ "class": "mt-6 pt-4 border-t border-white/20" }, [
                            el("h3", #{ "class": "text-sm font-semibold mb-3" }, [ text("3-Day Forecast") ]),
                            el("div", #{ "class": "space-y-2" }, forecast_items)
                        ])
                    ]);
                "#;

                Ok(ReadResourceResult {
                    contents: vec![
                        ResourceContents::text(script, uri)
                    ],
                })
            }
            "ui://portfolio-server/gallery" => {
                let script = r#"
                    let content = if data.structured_content != () { data.structured_content } else { #{} };
                    let owner = if "owner" in content { content.owner } else { "Portfolio" };
                    let bio = if "bio" in content { content.bio } else { "" };
                    let skills_data = if "skills" in content { content.skills } else { [] };
                    let projects_data = if "projects" in content { content.projects } else { [] };

                    let skill_chips = [];
                    for skill in skills_data {
                        skill_chips.push(el("span", #{ "class": "px-3 py-1 bg-indigo-100 text-indigo-700 rounded-full text-sm font-medium" }, [ text(skill) ]));
                    }

                    let project_cards = [];
                    for proj in projects_data {
                        let tech_stack = [];
                        let tech_list = if "tech" in proj { proj.tech } else { [] };
                        for t in tech_list {
                            tech_stack.push(el("span", #{ "class": "px-2 py-1 bg-gray-100 text-gray-700 rounded text-xs" }, [ text(t) ]));
                        }
                        
                        let stars = if "stars" in proj { proj.stars } else { 0 };

                        project_cards.push(el("div", #{ "class": "bg-white rounded-xl shadow-lg overflow-hidden hover:shadow-xl transition-shadow border border-gray-100" }, [
                            el("div", #{ "class": "h-32 bg-gradient-to-r from-indigo-500 to-purple-600 flex items-center justify-center" }, [
                                el("span", #{ "class": "text-white text-3xl font-mono" }, [ text("</>") ])
                            ]),
                            el("div", #{ "class": "p-6" }, [
                                el("div", #{ "class": "flex justify-between items-start mb-2" }, [
                                    el("h3", #{ "class": "text-xl font-bold text-gray-900" }, [ text(proj.name) ]),
                                    el("span", #{ "class": "text-sm text-yellow-600 font-medium" }, [ text("★ " + stars.to_string()) ])
                                ]),
                                el("p", #{ "class": "text-gray-600 mb-4" }, [ text(proj.desc) ]),
                                el("div", #{ "class": "flex flex-wrap gap-2" }, tech_stack)
                            ])
                        ]));
                    }

                    return el("div", #{ "class": "p-8 bg-gray-50 min-h-screen font-sans" }, [
                        el("div", #{ "class": "max-w-6xl mx-auto" }, [
                            el("div", #{ "class": "text-center mb-12" }, [
                                el("h1", #{ "class": "text-4xl font-extrabold text-gray-900 mb-2" }, [ text(owner) ]),
                                el("p", #{ "class": "text-lg text-gray-600 max-w-2xl mx-auto" }, [ text(bio) ]),
                                el("div", #{ "class": "flex flex-wrap justify-center gap-2 mt-4" }, skill_chips)
                            ]),
                            el("div", #{ "class": "grid grid-cols-1 md:grid-cols-2 gap-6" }, project_cards)
                        ])
                    ]);
                "#;
                Ok(ReadResourceResult { contents: vec![ResourceContents::text(script, uri)] })
            }
            "ui://system-server/status" => {
                let script = r#"
                    let content = if data.structured_content != () { data.structured_content } else { #{} };
                    let cpu = if "cpu_usage" in content { content.cpu_usage } else { 0 };
                    let mem = if "memory_usage" in content { content.memory_usage } else { 0 };
                    let disk = if "disk_usage" in content { content.disk_usage } else { 0 };
                    let uptime = if "uptime" in content { content.uptime } else { "--" };
                    let services = if "services" in content { content.services } else { [] };

                    let service_rows = [];
                    for svc in services {
                        let status_color = if svc.status == "online" { "text-green-600 bg-green-100" } else { "text-red-600 bg-red-100" };
                        service_rows.push(el("tr", #{ "class": "border-b border-gray-100 last:border-0" }, [
                            el("td", #{ "class": "py-3 px-4 font-medium text-gray-800" }, [ text(svc.name) ]),
                            el("td", #{ "class": "py-3 px-4" }, [
                                el("span", #{ "class": "px-2 py-1 rounded-full text-xs font-semibold " + status_color }, [ text(svc.status.to_string().to_upper_case()) ])
                            ]),
                            el("td", #{ "class": "py-3 px-4 text-gray-500 text-sm text-right" }, [ text(svc.latency) ])
                        ]));
                    }

                    return el("div", #{ "class": "p-6 bg-gray-50 min-h-full" }, [
                        el("h2", #{ "class": "text-2xl font-bold text-gray-800 mb-6" }, [ text("System Status") ]),
                        
                        // Metrics Grid
                        el("div", #{ "class": "grid grid-cols-3 gap-4 mb-8" }, [
                            el("div", #{ "class": "bg-white p-5 rounded-lg shadow-sm border border-gray-100" }, [
                                el("div", #{ "class": "text-sm text-gray-500 mb-1" }, [ text("CPU Usage") ]),
                                el("div", #{ "class": "text-3xl font-bold text-indigo-600" }, [ text(cpu.to_string() + "%") ]),
                                el("div", #{ "class": "w-full bg-gray-200 h-2 rounded-full mt-3 overflow-hidden" }, [
                                    el("div", #{ "class": "bg-indigo-600 h-full", "style": "width: " + cpu.to_string() + "%" }, [])
                                ])
                            ]),
                            el("div", #{ "class": "bg-white p-5 rounded-lg shadow-sm border border-gray-100" }, [
                                el("div", #{ "class": "text-sm text-gray-500 mb-1" }, [ text("Memory") ]),
                                el("div", #{ "class": "text-3xl font-bold text-purple-600" }, [ text(mem.to_string() + "%") ]),
                                el("div", #{ "class": "w-full bg-gray-200 h-2 rounded-full mt-3 overflow-hidden" }, [
                                    el("div", #{ "class": "bg-purple-600 h-full", "style": "width: " + mem.to_string() + "%" }, [])
                                ])
                            ]),
                            el("div", #{ "class": "bg-white p-5 rounded-lg shadow-sm border border-gray-100" }, [
                                el("div", #{ "class": "text-sm text-gray-500 mb-1" }, [ text("Disk") ]),
                                el("div", #{ "class": "text-3xl font-bold text-blue-600" }, [ text(disk.to_string() + "%") ]),
                                el("div", #{ "class": "w-full bg-gray-200 h-2 rounded-full mt-3 overflow-hidden" }, [
                                    el("div", #{ "class": "bg-blue-600 h-full", "style": "width: " + disk.to_string() + "%" }, [])
                                ])
                            ])
                        ]),

                        // Services Table
                        el("div", #{ "class": "bg-white rounded-lg shadow-sm border border-gray-100 overflow-hidden" }, [
                            el("div", #{ "class": "px-6 py-4 border-b border-gray-100 flex justify-between items-center" }, [
                                el("h3", #{ "class": "font-semibold text-gray-800" }, [ text("Active Services") ]),
                                el("span", #{ "class": "text-xs text-gray-500" }, [ text("Uptime: " + uptime) ])
                            ]),
                            el("table", #{ "class": "w-full" }, [
                                el("thead", #{ "class": "bg-gray-50 text-xs text-gray-500 uppercase" }, [
                                    el("tr", #{}, [
                                        el("th", #{ "class": "px-4 py-3 text-left font-medium" }, [ text("Service") ]),
                                        el("th", #{ "class": "px-4 py-3 text-left font-medium" }, [ text("Status") ]),
                                        el("th", #{ "class": "px-4 py-3 text-right font-medium" }, [ text("Latency") ])
                                    ])
                                ]),
                                el("tbody", #{}, service_rows)
                            ])
                        ])
                    ]);
                "#;
                Ok(ReadResourceResult { contents: vec![ResourceContents::text(script, uri)] })
            }
            "ui://notes-server/editor" => {
                let script = r#"
                    let content = if data.structured_content != () { data.structured_content } else { #{} };
                    let title = if "title" in content { content.title } else { "Untitled" };
                    let body = if "content" in content { content.content } else { "" };
                    let id = if "id" in content { content.id } else { "" };

                    return el("div", #{ "class": "p-8 max-w-2xl mx-auto font-sans" }, [
                        el("div", #{ "class": "bg-yellow-50 p-6 rounded-lg shadow-md border border-yellow-200 relative transform rotate-1 transition-transform hover:rotate-0" }, [
                             el("div", #{ "class": "absolute -top-3 left-1/2 transform -translate-x-1/2 w-32 h-6 bg-yellow-200/50 backdrop-blur-sm rounded-sm shadow-sm" }, []),
                             
                             el("h2", #{ "class": "text-2xl font-bold text-gray-800 mb-4 border-b border-yellow-200 pb-2" }, [ text(title) ]),
                             
                             el("p", #{ "class": "text-gray-700 whitespace-pre-wrap text-lg leading-relaxed" }, [ text(body) ]),
                             
                             el("div", #{ "class": "mt-8 flex justify-between items-center text-xs text-yellow-700" }, [
                                 el("span", #{}, [ text("ID: " + id.sub_string(0, 8)) ]),
                                 el("span", #{ "class": "italic" }, [ text("Created just now") ])
                             ])
                        ])
                    ]);
                "#;
                Ok(ReadResourceResult { contents: vec![ResourceContents::text(script, uri)] })
            }
            _ => Err(format!("Resource not found: {}", uri)),
        }
    }
    
    /// Handle initialize request
    pub async fn handle_initialize(&self, params: Value) -> Result<Value, String> {
        // Validate protocol version
        let client_version = params
            .get("protocolVersion")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        log::info!("Client initializing with protocol version: {}", client_version);
        
        Ok(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": self.get_capabilities(),
            "serverInfo": self.get_server_info()
        }))
    }
}

impl Default for EmbeddedServer {
    fn default() -> Self {
        Self::new()
    }
}

// Standalone server binary for stdio transport testing
#[cfg(feature = "server-binary")]
mod standalone {
    use super::*;
    use std::io::{self, BufRead, Write};
    
    #[tokio::main]
    async fn main() {
        env_logger::init();
        
        let server = EmbeddedServer::new();
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        
        log::info!("MCP Embedded Server started");
        
        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    log::error!("Error reading stdin: {}", e);
                    continue;
                }
            };
            
            let request: Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(e) => {
                    let error = json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": {
                            "code": -32700,
                            "message": format!("Parse error: {}", e)
                        }
                    });
                    writeln!(stdout, "{}", error).unwrap();
                    continue;
                }
            };
            
            let method = request.get("method").and_then(|v| v.as_str());
            let id = request.get("id").cloned();
            let params = request.get("params").cloned();
            
            let response = match method {
                Some("initialize") => {
                    match server.handle_initialize(params.unwrap_or(json!({}))).await {
                        Ok(result) => json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": result
                        }),
                        Err(e) => json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32603,
                                "message": e
                            }
                        })
                    }
                }
                Some("tools/list") => {
                    match server.list_tools().await {
                        Ok(result) => json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": result
                        }),
                        Err(e) => json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32603,
                                "message": e
                            }
                        })
                    }
                }
                Some("tools/call") => {
                    let params = params.unwrap_or(json!({}));
                    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
                    
                    match server.call_tool(name, arguments).await {
                        Ok(result) => json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": result
                        }),
                        Err(e) => json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32603,
                                "message": e
                            }
                        })
                    }
                }
                Some("resources/list") => {
                    match server.list_resources().await {
                        Ok(result) => json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": result
                        }),
                        Err(e) => json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32603,
                                "message": e
                            }
                        })
                    }
                }
                Some("resources/read") => {
                    let params = params.unwrap_or(json!({}));
                    let uri = params.get("uri").and_then(|v| v.as_str()).unwrap_or("");
                    
                    match server.read_resource(uri).await {
                        Ok(result) => json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": result
                        }),
                        Err(e) => json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32603,
                                "message": e
                            }
                        })
                    }
                }
                Some("notifications/initialized") => {
                    log::info!("Client initialized notification received");
                    continue; // No response for notifications
                }
                Some(method) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {}", method)
                    }
                }),
                None => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32600,
                        "message": "Invalid request: missing method"
                    }
                })
            };
            
            writeln!(stdout, "{}", response).unwrap();
            stdout.flush().unwrap();
        }
        
        log::info!("MCP Embedded Server stopped");
    }
}
