# MCP Apps Demo - Dioxus Desktop Application

This is a demo application implementing the [MCP Apps specification (SEP-1865)](./MCPAPP_SPEC.md) using Dioxus 0.7, Rust, and Rhai scripting for dynamic UI generation.

## Project Overview

This project demonstrates an MCP (Model Context Protocol) host application with an embedded MCP server that serves interactive UI applications. It showcases:

- **MCP Apps Extension**: Implements the `io.modelcontextprotocol/ui` extension for interactive user interfaces
- **Dynamic UI Generation**: Uses Rhai scripts to generate Dioxus UI components at runtime
- **Multi-Platform Support**: Can build for desktop (default), web, or mobile using Dioxus

### What is MCP Apps?

MCP Apps enables MCP servers to deliver interactive user interfaces to hosts through:
- UI resources using the `ui://` URI scheme
- Tool-UI linkage via metadata
- Bidirectional JSON-RPC communication over postMessage
- Sandboxed iframe rendering with CSP enforcement

## Technology Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Frontend Framework | Dioxus | 0.7.1 |
| MCP Protocol | rmcp | 0.14.0 |
| Scripting Engine | Rhai | 1.19.0 |
| Async Runtime | Tokio | 1.x |
| Styling | Tailwind CSS | v4 (automatic) |
| Serialization | Serde | 1.0 |

## Project Structure

```
project/
├── Cargo.toml           # Rust dependencies and features
├── Dioxus.toml          # Dioxus application configuration
├── tailwind.css         # Tailwind CSS entry point (enables auto-Tailwind)
├── MCPAPP_SPEC.md       # Full MCP Apps specification (SEP-1865)
├── src/
│   ├── main.rs          # Application entry point and main layout
│   ├── client.rs        # AppClient - MCP client state management
│   ├── server.rs        # EmbeddedServer - mock MCP server with tools
│   └── rhai_ui.rs       # Rhai script engine and UI renderer
└── assets/              # Static assets (favicon, styles)
```

### Module Descriptions

#### `main.rs`
- **Entry point**: Launches the Dioxus application
- **MainLayout component**: Provides the application shell with:
  - Sidebar showing available MCP tools
  - Main content area for rendering Rhai-generated UIs
  - State management via Dioxus Signals and Context API

#### `client.rs`
- **AppClient struct**: Manages application state including:
  - `server`: Signal to the embedded MCP server
  - `tools`: List of available tools from the server
  - `current_ui`: Rhai script for the current UI
  - `current_data`: JSON data context for the UI
- **Methods**:
  - `load_tools()`: Fetches tools list from server on mount
  - `run_tool()`: Executes a tool and loads its associated UI resource

#### `server.rs`
- **EmbeddedServer**: Mock MCP server implementing:
  - `list_tools()`: Returns available tools with UI metadata
  - `call_tool()`: Executes tools (get_weather, get_portfolio)
  - `read_resource()`: Returns Rhai UI scripts for `ui://` resources
- **Tools**:
  - `get_weather`: Returns weather data with a weather dashboard UI
  - `get_portfolio`: Returns project portfolio with a portfolio gallery UI

#### `rhai_ui.rs`
- **UiNode enum**: Represents UI elements (Element, Text)
- **Rhai Engine**: Configured with helper functions:
  - `el(tag, props, children)`: Creates UI element maps
  - `text(string)`: Creates text nodes
  - `v(array)`: Vector helper for iteration
- **RhaiRenderer component**: Evaluates Rhai scripts and renders to Dioxus elements
- **RenderUiNode component**: Recursively renders UiNode to Dioxus RSX

## Build and Development

### Prerequisites

1. **Rust toolchain** (latest stable)
2. **Dioxus CLI** (`dx`):
   ```bash
   curl -sSL https://dioxuslabs.com/install.sh | sh
   ```

### Development Commands

```bash
# Serve the desktop application (default)
dx serve

# Serve for desktop explicitly
dx serve --platform desktop

# Serve for web (requires web feature)
dx serve --platform web

# Build release binary
cargo build --release
```

### Platform Features

The `Cargo.toml` defines three feature flags:
- `desktop` (default): Builds for desktop using `dioxus/desktop`
- `web`: Builds for web using `dioxus/web`
- `mobile`: Builds for mobile using `dioxus/mobile`

To build for a specific platform:
```bash
# Web build
cargo build --features web --no-default-features

# Desktop build (default)
cargo build
```

### Tailwind CSS

As of Dioxus 0.7, Tailwind CSS is automatically handled when a `tailwind.css` file exists in the project root. No manual installation required.

To customize Tailwind configuration, edit `Dioxus.toml`:
```toml
[application]
tailwind_input = "my.css"
tailwind_output = "assets/out.css"
```

## Code Style Guidelines

### Rust

1. **Component Functions**: Use `#[component]` macro, capitalized names
   ```rust
   #[component]
   fn MyComponent() -> Element { ... }
   ```

2. **Signals**: Use `use_signal` for local state, `use_context_provider` for shared state
   ```rust
   let mut count = use_signal(|| 0);
   use_context_provider(|| Signal::new(AppClient::new()));
   ```

3. **Async Effects**: Use `use_effect` with `spawn` for async initialization
   ```rust
   use_effect(move || {
       spawn(async move {
           client.write().load_tools().await;
       });
   });
   ```

4. **RSX Formatting**: Prefer loops over iterators, use conditionals directly
   ```rust
   rsx! {
       for tool in tools {
           ToolItem { tool }
       }
       if condition {
           div { "Active" }
       }
   }
   ```

### Rhai UI Scripts

Rhai scripts define UI structures using the custom DSL:

```rhai
el("div", #{ "class": "container" }, [
    el("h1", #{}, [ text("Title") ]),
    el("p", #{ "class": "text-gray-600" }, [ text("Description") ])
])
```

**Supported HTML tags** (in `rhai_ui.rs`):
- `div`, `span`, `h1`/`h2`, `p`, `button`
- `ul`/`li`, `img`, `input`

**Data access**: Use `data` variable (JSON converted to Rhai Dynamic)
```rhai
el("span", #{}, [ text(data.structured_content.location) ])
```

## Testing

Run tests with Cargo:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_rhai_script_rendering
```

### Test Coverage

Current tests (in `rhai_ui.rs`):
- `test_rhai_script_rendering`: Validates Rhai script evaluation and UiNode parsing

## Architecture Patterns

### State Management

The application uses Dioxus Signals with Context API:

1. **Provide context** in parent component:
   ```rust
   let mut client = use_context_provider(|| Signal::new(AppClient::new()));
   ```

2. **Access context** in child components:
   ```rust
   let client = use_context::<Signal<AppClient>>();
   ```

### MCP Communication Flow

```
┌─────────────┐     list_tools()     ┌──────────────┐
│   Client    │ ◄──────────────────► │    Server    │
│  (Signals)  │     call_tool()      │ (Mock Tools) │
└──────┬──────┘                      └──────┬───────┘
       │                                    │
       │ read_resource(uri)                ▼
       │◄────────────────────────── Rhai UI Script
       │
       ▼
┌──────────────┐    evaluate()     ┌──────────────┐
│ RhaiRenderer │ ─────────────────►│  Rhai Engine │
└──────┬───────┘                   └──────────────┘
       │
       ▼
┌──────────────┐
│  Dioxus RSX  │
└──────────────┘
```

### UI Resource URI Scheme

Tools reference UI resources via `ui://` URIs in their metadata:

```rust
meta: Some(Meta(json!({
    "ui": {
        "resourceUri": "ui://weather"
    }
}).as_object().unwrap().clone()))
```

The server returns Rhai scripts when these resources are requested.

## Security Considerations

This demo implements security patterns from the MCP Apps spec:

1. **CSP Enforcement**: UI resources declare required domains in metadata
2. **Sandboxing**: (In production) UI runs in sandboxed iframes
3. **Auditability**: All tool calls and resource reads are traceable

See [MCPAPP_SPEC.md](./MCPAPP_SPEC.md) section "Security Implications" for full details.

## Dependencies Reference

### Core Dependencies

| Crate | Purpose |
|-------|---------|
| `dioxus` | UI framework with router feature |
| `rmcp` | MCP protocol implementation |
| `rhai` | Scripting engine with serde support |
| `serde`/`serde_json` | Serialization |
| `tokio` | Async runtime |
| `futures` | Async utilities |
| `anyhow` | Error handling |
| `dashmap` | Concurrent hash map |

## Additional Resources

- [Dioxus 0.7 Documentation](https://dioxuslabs.com/learn/0.7)
- [MCP Apps Specification](./MCPAPP_SPEC.md) (SEP-1865)
- [Rhai Scripting Book](https://rhai.rs/book/)
