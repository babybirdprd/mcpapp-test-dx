<!-- For full edit history, see specification/draft/apps.mdx -->



\# SEP-1865: MCP Apps: Interactive User Interfaces for MCP



\*\*Track:\*\* Extensions



\*\*Authors:\*\* Ido Salomon, Liad Yosef, Olivier Chafik, Jerome Swannack, Jonathan Hefner, Anton Pidkuiko, Nick Cooper, Bryan Ashley, Alexi Christakis



\*\*Status:\*\* Stable (2026-01-26)



\*\*Created:\*\* 2025-11-21



\## Abstract



This SEP proposes an extension (per SEP-1724) to MCP that enables servers to deliver interactive user interfaces to hosts. MCP Apps introduces a standardized pattern for declaring UI resources via the `ui://` URI scheme, associating them with tools through metadata, and facilitating bidirectional communication between the UI and the host using MCP's JSON-RPC base protocol. This extension addresses the growing community need for rich, interactive experiences in MCP-enabled applications, maintaining security, auditability, and alignment with MCP's core architecture. The initial specification focuses on HTML resources (`text/html;profile=mcp-app`) with a clear path for future extensions.



\## Motivation



MCP lacks a standardized way for servers to deliver rich, interactive user interfaces to hosts. This gap blocks many use cases that require visual presentation and interactivity that goes beyond plain text or structured data. As more hosts adopt this capability, there's a growing risk of fragmentation and interoperability challenges.



\[MCP-UI](https://mcpui.dev/) has demonstrated the viability and value of MCP apps built on UI resources and serves as a community playground for the UI spec and SDK. Fuelled by a dedicated community, it developed the bi-directional communication model and the HTML, external URL, and remote DOM content types. MCP-UI's adopters, including hosts and providers such as Postman, HuggingFace, Shopify, Goose, and ElevenLabs, have provided critical insights and contributions to the community.



OpenAI's Apps SDK, launched in November 2025, further validated the demand for rich UI experiences within conversational AI interfaces. The Apps SDK enables developers to build rich, interactive applications inside ChatGPT using MCP as its backbone.



The architecture of both the Apps SDK and MCP-UI have both significantly informed the design of this specification.



However, without formal standardization:



\- Servers cannot reliably expect UI support via MCP

\- Each host may implement slightly different behaviors

\- Security and auditability patterns are inconsistent

\- Developers must maintain separate implementations or adapters for different hosts (e.g., MCP-UI vs. Apps SDK)



This SEP addresses the current limitations through an optional, backwards-compatible extension that unifies the approaches pioneered by MCP-UI and the Apps SDK into a single, open standard.



\## Specification



\### Extension Identifier



This extension is identified as: `io.modelcontextprotocol/ui`



\### Overview



MCP Apps extends the Model Context Protocol to enable servers to deliver interactive user interfaces to hosts. This extension introduces:



\- \*\*UI Resources:\*\* Predeclared resources using the `ui://` URI scheme

\- \*\*Tool-UI Linkage:\*\* Tools reference UI resources via metadata

\- \*\*Bidirectional Communication:\*\* UI iframes communicate with hosts using standard MCP JSON-RPC protocol

\- \*\*Security Model:\*\* Mandatory iframe sandboxing with auditable communication



This specification focuses on HTML content (`text/html;profile=mcp-app`) as the initial content type, with extensibility for future formats.



As an extension, MCP Apps is optional and must be explicitly negotiated between clients and servers through the extension capabilities mechanism (see Capability Negotiation section).



\### UI Resource Format



UI resources are declared using the standard MCP resource pattern with specific conventions:



```typescript

interface UIResource {

&nbsp; /\*\*

&nbsp;  \* Unique identifier for the UI resource

&nbsp;  \*

&nbsp;  \* MUST use the `ui://` URI scheme to distinguish UI resources from other

&nbsp;  \* MCP resource types.

&nbsp;  \*

&nbsp;  \* @example

&nbsp;  \* "ui://weather-dashboard"

&nbsp;  \*/

&nbsp; uri: string;



&nbsp; /\*\*

&nbsp;  \* Human-readable display name for the UI resource

&nbsp;  \*

&nbsp;  \* Used for listing and identifying the resource in host interfaces.

&nbsp;  \*

&nbsp;  \* @example

&nbsp;  \* "Weather Dashboard"

&nbsp;  \*/

&nbsp; name: string;



&nbsp; /\*\*

&nbsp;  \* Optional description of the UI resource's purpose and functionality

&nbsp;  \*

&nbsp;  \* Provides context about what the UI does and when to use it.

&nbsp;  \*

&nbsp;  \* @example

&nbsp;  \* "Interactive weather visualization with real-time updates"

&nbsp;  \*/

&nbsp; description?: string;



&nbsp; /\*\*

&nbsp;  \* MIME type of the UI content

&nbsp;  \*

&nbsp;  \* SHOULD be `text/html;profile=mcp-app` for HTML-based UIs in the initial MVP.

&nbsp;  \* Other content types are reserved for future extensions.

&nbsp;  \*

&nbsp;  \* @example

&nbsp;  \* "text/html;profile=mcp-app"

&nbsp;  \*/

&nbsp; mimeType: string;



&nbsp; /\*\*

&nbsp;  \* Resource metadata for security and rendering configuration

&nbsp;  \*

&nbsp;  \* Includes Content Security Policy configuration, dedicated domain settings,

&nbsp;  \* and visual preferences.

&nbsp;  \*/

&nbsp; \_meta?: {

&nbsp;   ui?: UIResourceMeta;

&nbsp; }

}



interface McpUiResourceCsp {

&nbsp;   /\*\*

&nbsp;    \* Origins for network requests (fetch/XHR/WebSocket)

&nbsp;    \*

&nbsp;    \* - Empty or omitted = no external connections (secure default)

&nbsp;    \* - Maps to CSP `connect-src` directive

&nbsp;    \*

&nbsp;    \* @example

&nbsp;    \* \["https://api.weather.com", "wss://realtime.service.com"]

&nbsp;    \*/

&nbsp;   connectDomains?: string\[],

&nbsp;   /\*\*

&nbsp;    \* Origins for static resources (images, scripts, stylesheets, fonts, media)

&nbsp;    \*

&nbsp;    \* - Empty or omitted = no external resources (secure default)

&nbsp;    \* - Wildcard subdomains supported: `https://\*.example.com`

&nbsp;    \* - Maps to CSP `img-src`, `script-src`, `style-src`, `font-src`, `media-src` directives

&nbsp;    \*

&nbsp;    \* @example

&nbsp;    \* \["https://cdn.jsdelivr.net", "https://\*.cloudflare.com"]

&nbsp;    \*/

&nbsp;   resourceDomains?: string\[],

&nbsp;   /\*\*

&nbsp;    \* Origins for nested iframes

&nbsp;    \*

&nbsp;    \* - Empty or omitted = no nested iframes allowed (`frame-src 'none'`)

&nbsp;    \* - Maps to CSP `frame-src` directive

&nbsp;    \*

&nbsp;    \* @example

&nbsp;    \* \["https://www.youtube.com", "https://player.vimeo.com"]

&nbsp;    \*/

&nbsp;   frameDomains?: string\[],

&nbsp;   /\*\*

&nbsp;    \* Allowed base URIs for the document

&nbsp;    \*

&nbsp;    \* - Empty or omitted = only same origin allowed (`base-uri 'self'`)

&nbsp;    \* - Maps to CSP `base-uri` directive

&nbsp;    \*

&nbsp;    \* @example

&nbsp;    \* \["https://cdn.example.com"]

&nbsp;    \*/

&nbsp;   baseUriDomains?: string\[],

}



interface UIResourceMeta {

&nbsp; /\*\*

&nbsp;  \* Content Security Policy configuration

&nbsp;  \*

&nbsp;  \* Servers declare which external origins their UI needs to access.

&nbsp;  \* Hosts use this to enforce appropriate CSP headers.

&nbsp;  \*/

&nbsp; csp?: McpUiResourceCsp,

&nbsp; /\*\*

&nbsp;  \* Sandbox permissions requested by the UI

&nbsp;  \*

&nbsp;  \* Servers declare which browser capabilities their UI needs.

&nbsp;  \* Hosts MAY honor these by setting appropriate iframe `allow` attributes.

&nbsp;  \* Apps SHOULD NOT assume permissions are granted; use JS feature detection as fallback.

&nbsp;  \*/

&nbsp; permissions?: {

&nbsp;   /\*\*

&nbsp;    \* Request camera access

&nbsp;    \*

&nbsp;    \* Maps to Permission Policy `camera` feature

&nbsp;    \*/

&nbsp;   camera?: {},

&nbsp;   /\*\*

&nbsp;    \* Request microphone access

&nbsp;    \*

&nbsp;    \* Maps to Permission Policy `microphone` feature

&nbsp;    \*/

&nbsp;   microphone?: {},

&nbsp;   /\*\*

&nbsp;    \* Request geolocation access

&nbsp;    \*

&nbsp;    \* Maps to Permission Policy `geolocation` feature

&nbsp;    \*/

&nbsp;   geolocation?: {},

&nbsp;   /\*\*

&nbsp;    \* Request clipboard write access

&nbsp;    \*

&nbsp;    \* Maps to Permission Policy `clipboard-write` feature

&nbsp;    \*/

&nbsp;   clipboardWrite?: {},

&nbsp; },

&nbsp; /\*\*

&nbsp;  \* Dedicated origin for view

&nbsp;  \*

&nbsp;  \* Optional domain for the view's sandbox origin. Useful when views need

&nbsp;  \* stable, dedicated origins for OAuth callbacks, CORS policies, or API key allowlists.

&nbsp;  \*

&nbsp;  \* \*\*Host-dependent:\*\* The format and validation rules for this field are

&nbsp;  \* determined by each host. Servers MUST consult host-specific documentation

&nbsp;  \* for the expected domain format. Common patterns include:

&nbsp;  \* - Hash-based subdomains (e.g., `{hash}.claudemcpcontent.com`)

&nbsp;  \* - URL-derived subdomains (e.g., `www-example-com.oaiusercontent.com`)

&nbsp;  \*

&nbsp;  \* If omitted, Host uses default sandbox origin (typically per-conversation).

&nbsp;  \*

&nbsp;  \* @example

&nbsp;  \* "a904794854a047f6.claudemcpcontent.com"

&nbsp;  \* @example

&nbsp;  \* "www-example-com.oaiusercontent.com"

&nbsp;  \*/

&nbsp; domain?: string,

&nbsp; /\*\*

&nbsp;  \* Visual boundary preference

&nbsp;  \*

&nbsp;  \* Boolean controlling whether a visible border and background is provided by the host. Specifying an

&nbsp;  \* explicit value for this is recommended because hosts' defaults may vary.

&nbsp;  \*

&nbsp;  \* - `true`: Request visible border + background

&nbsp;  \* - `false`: Request no visible border + background

&nbsp;  \* - omitted: host decides border

&nbsp;  \*/

&nbsp; prefersBorder?: boolean,

}

```



The resource content is returned via `resources/read`:



```typescript

// resources/read response for UI resource

{

&nbsp; contents: \[{

&nbsp;   uri: string;                  // Matching UI resource URI

&nbsp;   mimeType: "text/html;profile=mcp-app";  // MUST be "text/html;profile=mcp-app"

&nbsp;   text?: string;                // HTML content as string

&nbsp;   blob?: string;                // OR base64-encoded HTML

&nbsp;   \_meta?: {

&nbsp;     ui?: {

&nbsp;       csp?: {

&nbsp;         connectDomains?: string\[]; // Origins for network requests (fetch/XHR/WebSocket).

&nbsp;         resourceDomains?: string\[]; // Origins for static resources (scripts, images, styles, fonts).

&nbsp;         frameDomains?: string\[]; // Origins for nested iframes (frame-src directive).

&nbsp;         baseUriDomains?: string\[]; // Allowed base URIs for the document (base-uri directive).

&nbsp;       };

&nbsp;       permissions?: {

&nbsp;         camera?: {};           // Request camera access

&nbsp;         microphone?: {};       // Request microphone access

&nbsp;         geolocation?: {};      // Request geolocation access

&nbsp;         clipboardWrite?: {};   // Request clipboard write access

&nbsp;       };

&nbsp;       domain?: string;

&nbsp;       prefersBorder?: boolean;

&nbsp;     };

&nbsp;   };

&nbsp; }];

}

```



\#### Content Requirements:



\- URI MUST start with `ui://` scheme

\- `mimeType` MUST be `text/html;profile=mcp-app` (other types reserved for future extensions)

\- Content MUST be provided via either `text` (string) or `blob` (base64-encoded)

\- Content MUST be valid HTML5 document



\#### Host Behavior:



\- \*\*CSP Enforcement:\*\* Host MUST construct CSP headers based on declared domains

\- \*\*Restrictive Default:\*\* If `ui.csp` is omitted, Host MUST use:



&nbsp; ```

&nbsp; default-src 'none';

&nbsp; script-src 'self' 'unsafe-inline';

&nbsp; style-src 'self' 'unsafe-inline';

&nbsp; img-src 'self' data:;

&nbsp; media-src 'self' data:;

&nbsp; connect-src 'none';

&nbsp; ```



\- \*\*No Loosening:\*\* Host MAY further restrict but MUST NOT allow undeclared domains

\- \*\*Audit Trail:\*\* Host SHOULD log CSP configurations for security review



Example:



```json

// Resource declaration

{

&nbsp; "uri": "ui://weather-server/dashboard-template",

&nbsp; "name": "weather\_dashboard",

&nbsp; "description": "Interactive weather dashboard view",

&nbsp; "mimeType": "text/html;profile=mcp-app"

}



// Resource content with metadata

{

&nbsp; "contents": \[{

&nbsp;   "uri": "ui://weather-server/dashboard-template",

&nbsp;   "mimeType": "text/html;profile=mcp-app",

&nbsp;   "text": "<!DOCTYPE html><html>...</html>",

&nbsp;   "\_meta": {

&nbsp;     "ui" : {

&nbsp;       "csp": {

&nbsp;         "connectDomains": \["https://api.openweathermap.org"],

&nbsp;         "resourceDomains": \["https://cdn.jsdelivr.net"]

&nbsp;       },

&nbsp;       "prefersBorder": true

&nbsp;     }

&nbsp;   }

&nbsp; }]

}

```



\### Resource Discovery



Tools are associated with UI resources through the `\_meta.ui` field:



```typescript

interface McpUiToolMeta {

&nbsp; /\*\* URI of UI resource for rendering tool results \*/

&nbsp; resourceUri?: string;

&nbsp; /\*\*

&nbsp;  \* Who can access this tool. Default: \["model", "app"]

&nbsp;  \* - "model": Tool visible to and callable by the agent

&nbsp;  \* - "app": Tool callable by the app from this server only

&nbsp;  \*/

&nbsp; visibility?: Array<"model" | "app">;

}



interface Tool {

&nbsp; name: string;

&nbsp; description: string;

&nbsp; inputSchema: object;

&nbsp; \_meta?: {

&nbsp;   ui?: McpUiToolMeta;

&nbsp;   /\*\* @deprecated Use `ui.resourceUri` instead. Will be removed before GA. \*/

&nbsp;   "ui/resourceUri"?: string;

&nbsp; };

}

```



> \*\*Deprecation notice:\*\* The flat `\_meta\["ui/resourceUri"]` format is deprecated. Use `\_meta.ui.resourceUri` instead. The deprecated format will be removed before GA.



Example (tool visible to both model and app):



```json

{

&nbsp; "name": "get\_weather",

&nbsp; "description": "Get current weather for a location",

&nbsp; "inputSchema": {

&nbsp;   "type": "object",

&nbsp;   "properties": {

&nbsp;     "location": { "type": "string" }

&nbsp;   }

&nbsp; },

&nbsp; "\_meta": {

&nbsp;   "ui": {

&nbsp;     "resourceUri": "ui://weather-server/dashboard-template",

&nbsp;     "visibility": \["model", "app"]

&nbsp;   }

&nbsp; }

}

```



Example (app-only tool, hidden from model):



```json

{

&nbsp; "name": "refresh\_dashboard",

&nbsp; "description": "Refresh dashboard data",

&nbsp; "inputSchema": { "type": "object" },

&nbsp; "\_meta": {

&nbsp;   "ui": {

&nbsp;     "resourceUri": "ui://weather-server/dashboard-template",

&nbsp;     "visibility": \["app"]

&nbsp;   }

&nbsp; }

}

```



\#### Behavior:



\- If `ui.resourceUri` is present and host supports MCP Apps, host renders tool results using the specified UI resource

\- If host does not support MCP Apps, tool behaves as standard tool (text-only fallback)

\- Resource MUST exist on the server

\- Host MUST use `resources/read` to fetch the referenced resource URI

\- Host MAY prefetch and cache UI resource content for performance optimization

\- Since UI resources are primarily discovered through tool metadata, Servers MAY omit UI-only resources from `resources/list` and `notifications/resources/list\_changed`



\#### Visibility:



\- `visibility` defaults to `\["model", "app"]` if omitted

\- `"model"`: Tool is visible to and callable by the agent

\- `"app"`: Tool is callable by the app from the same server connection only

\- \*\*tools/list behavior:\*\* Host MUST NOT include tools in the agent's tool list when their visibility does not include `"model"` (e.g., `visibility: \["app"]`)

\- \*\*tools/call behavior:\*\* Host MUST reject `tools/call` requests from apps for tools that don't include `"app"` in visibility

\- Cross-server tool calls are always blocked for app-only tools



\#### Benefits:



\- \*\*Performance:\*\* Host can preload templates before tool execution

\- \*\*Security:\*\* Host can review UI templates during connection setup

\- \*\*Caching:\*\* Separate template (static) from data (dynamic)

\- \*\*Auditability:\*\* All UI resources are enumerable and inspectable



\### Communication Protocol



MCP Apps uses JSON-RPC 2.0 over `postMessage` for iframe-host communication. UI capabilities (e.g., tool call, prompt, etc.) will reuse MCP’s existing protocol. In the future, we may choose to enrich the communication protocol with additional MCP capabilities.



\### Transport Layer



Conceptually, UI iframes act as MCP clients, connecting to the host via a `postMessage` transport:



```typescript

// UI iframe initializes MCP client

const transport = new MessageTransport(window.parent);

const client = new Client({ name: "ui-view", version: "1.0.0" });

await client.connect(transport);

```



Note that you don’t need an SDK to “talk MCP” with the host:



```typescript

let nextId = 1;

function sendRequest(method: string, params: any) {

&nbsp; const id = nextId++;

&nbsp; window.parent.postMessage({ jsonrpc: "2.0", id, method, params }, '\*');

&nbsp; return new Promise((resolve, reject) => {

&nbsp;   window.addEventListener('message', function listener(event) {

&nbsp;     const data: JSONRPCMessage = event.data;

&nbsp;     if (event.data?.id === id) {

&nbsp;       window.removeEventListener('message', listener);

&nbsp;       if (event.data?.result) {

&nbsp;         resolve(event.data?.result);

&nbsp;       } else if (event.data?.error) {

&nbsp;         reject(new Error(event.data.error));

&nbsp;       }

&nbsp;     } else {

&nbsp;       reject(new Error(`Unsupported message: ${JSON.stringify(data)}`));

&nbsp;     }

&nbsp;   });

&nbsp; });

}

function sendNotification(method: string, params: any) {

&nbsp; window.parent.postMessage({ jsonrpc: "2.0", method, params }, '\*');

}

function onNotification(method: string, handler: (params: any) => void) {

&nbsp; window.addEventListener('message', function listener(event) {

&nbsp;   if (event.data?.method === method) {

&nbsp;     handler(event.data.params);

&nbsp;   }

&nbsp; });

}





const initializeResult = await sendRequest("initialize", {

&nbsp; capabilities: {},

&nbsp; clientInfo: {name: "My UI", version: "1.0.0"},

&nbsp; protocolVersion: "2026-01-26",

});

```



Hosts act as MCP servers (that can proxy the actual MCP server), receiving and handling requests from UI iframes.



\### Sandbox proxy



If the Host is a web page, it MUST wrap the View and communicate with it through an intermediate Sandbox proxy.



1\. The Host and the Sandbox MUST have different origins.

2\. The Sandbox MUST have the following permissions: `allow-scripts`, `allow-same-origin`.

3\. The Sandbox MUST send a `ui/notifications/sandbox-proxy-ready` notification to the host when it's ready to process an `ui/notifications/sandbox-resource-ready` notification.

4\. Once the Sandbox is ready, the Host MUST send the raw HTML resource to load in a `ui/notifications/sandbox-resource-ready` notification.

5\. The Sandbox MUST load the raw HTML of the View with CSP settings that:

&nbsp;  - Enforce the domains declared in `ui.csp` metadata

&nbsp;  - If `frameDomains` is provided, allow nested iframes from declared origins; otherwise use `frame-src 'none'`

&nbsp;  - If `baseUriDomains` is provided, allow base URIs from declared origins; otherwise use `base-uri 'self'`

&nbsp;  - Block dangerous features (`object-src 'none'`)

&nbsp;  - Apply restrictive defaults if no CSP metadata is provided

&nbsp;  - If `permissions` is declared, the Sandbox MAY set the inner iframe's `allow` attribute accordingly

6\. The Sandbox MUST forward messages sent by the Host to the View, and vice versa, for any method that doesn't start with `ui/notifications/sandbox-`. This includes lifecycle messages, e.g., `ui/initialize` request \& `ui/notifications/initialized` notification both sent by the View. The Host MUST NOT send any request or notification to the View before it receives an `initialized` notification.

7\. The Sandbox SHOULD NOT create/send any requests to the Host or to the View (this would require synthesizing new request ids).

8\. The Host MAY forward any message from the View (coming via the Sandbox) to the MCP Apps server, for any method that doesn't start with `ui/`. While the Host SHOULD ensure the View's MCP connection is spec-compliant, it MAY decide to block some messages or subject them to further user approval.



\### Standard MCP Messages



UI iframes can use the following subset of standard MCP protocol messages:



\*\*Tools:\*\*



\- `tools/call` - Execute a tool on the MCP server



\*\*Resources:\*\*



\- `resources/read` - Read resource content



\*\*Notifications:\*\*



\- `notifications/message` - Log messages to host



\*\*Lifecycle:\*\*



\- `ui/initialize` → `ui/notifications/initialized` - MCP-like handshake (replaces custom iframe-ready pattern in MCP-UI)

\- `ping` - Connection health check



\### App Capabilities in `ui/initialize`



When the View sends an `ui/initialize` request to the Host, it MUST include its capabilities in the `appCapabilities` field:



```typescript

interface McpUiAppCapabilities {

&nbsp; /\*\* Experimental features (structure TBD). \*/

&nbsp; experimental?: {};

&nbsp; /\*\* App exposes MCP-style tools that the host can call. \*/

&nbsp; tools?: {

&nbsp;   /\*\* App supports tools/list\_changed notifications. \*/

&nbsp;   listChanged?: boolean;

&nbsp; };

&nbsp; /\*\*

&nbsp;  \* Display modes the app supports. See Display Modes section for details.

&nbsp;  \* @example \["inline", "fullscreen"]

&nbsp;  \*/

&nbsp; availableDisplayModes?: Array<"inline" | "fullscreen" | "pip">;

}

```



\### Host Context in `McpUiInitializeResult`



When the View sends an `ui/initialize` request to the Host, the Host SHOULD include UI-specific context in the `McpUiInitializeResult`'s `hostContext` field:



```typescript

interface HostContext {

&nbsp; /\*\* Metadata of the tool call that instantiated the View \*/

&nbsp; toolInfo?: {

&nbsp;   /\*\* JSON-RPC id of the tools/call request \*/

&nbsp;   id?: RequestId,

&nbsp;   /\*\* Contains name, inputSchema, etc… \*/

&nbsp;   tool: Tool,

&nbsp; };

&nbsp; /\*\* Current color theme preference \*/

&nbsp; theme?: "light" | "dark";

&nbsp; /\*\* Style configuration for theming. See Theming section for standardized variable names. \*/

&nbsp; styles?: {

&nbsp;   /\*\* CSS variables for theming \*/

&nbsp;   variables?: Record<McpUiStyleVariableKey, string | undefined>;

&nbsp;   /\*\* CSS blocks that Views can inject \*/

&nbsp;   css?: {

&nbsp;     /\*\* CSS for font loading (@font-face rules or @import statements) \*/

&nbsp;     fonts?: string;

&nbsp;   };

&nbsp; };

&nbsp; /\*\* How the View is currently displayed \*/

&nbsp; displayMode?: "inline" | "fullscreen" | "pip";

&nbsp; /\*\* Display modes the host supports \*/

&nbsp; availableDisplayModes?: string\[];

&nbsp; /\*\* Container dimensions for the iframe. Specify either width or maxWidth, and either height or maxHeight. \*/

&nbsp; containerDimensions?: (

&nbsp;   | { height: number }      // If specified, container is fixed at this height

&nbsp;   | { maxHeight?: number }  // Otherwise, container height is determined by the View's height, up to this maximum height (if defined)

&nbsp; ) \& (

&nbsp;   | { width: number }       // If specified, container is fixed at this width

&nbsp;   | { maxWidth?: number }   // Otherwise, container width is determined by the View's width, up to this maximum width (if defined)

&nbsp; );

&nbsp; /\*\* User's language/region preference (BCP 47, e.g., "en-US") \*/

&nbsp; locale?: string;

&nbsp; /\*\* User's timezone (IANA, e.g., "America/New\_York") \*/

&nbsp; timeZone?: string;

&nbsp; /\*\* Host application identifier \*/

&nbsp; userAgent?: string;

&nbsp; /\*\* Platform type for responsive design \*/

&nbsp; platform?: "web" | "desktop" | "mobile";

&nbsp; /\*\* Device capabilities such as touch \*/

&nbsp; deviceCapabilities?: {

&nbsp;   touch?: boolean;

&nbsp;   hover?: boolean;

&nbsp; }

&nbsp; /\*\* Safe area boundaries in pixels \*/

&nbsp; safeAreaInsets?: {

&nbsp;   top: number;

&nbsp;   right: number;

&nbsp;   bottom: number;

&nbsp;   left: number;

&nbsp; };

}

```



Example:



```json

// Host responds with McpUiInitializeResult

{

&nbsp; "jsonrpc": "2.0",

&nbsp; "id": 1,

&nbsp; "result": {

&nbsp;   "protocolVersion": "2026-01-26",

&nbsp;   "hostCapabilities": { /\* ... \*/ },

&nbsp;   "hostInfo": { "name": "claude-desktop", "version": "1.0.0" },

&nbsp;   "hostContext": {

&nbsp;     "theme": "dark",

&nbsp;     "styles": {

&nbsp;       "variables": {

&nbsp;         "--color-background-primary": "light-dark(#ffffff, #171717)",

&nbsp;         "--color-text-primary": "light-dark(#171717, #fafafa)",

&nbsp;         "--font-sans": "Anthropic Sans, sans-serif",

&nbsp;         ...

&nbsp;       },

&nbsp;       "css": {

&nbsp;         "fonts": "@font-face { font-family: \\"Custom Font Name\\"; src: url(\\"https://...\\"); }"

&nbsp;       }

&nbsp;     },

&nbsp;     "displayMode": "inline",

&nbsp;     "containerDimensions": { "width": 400, "maxHeight": 600 }

&nbsp;   }

&nbsp; }

}

```



\### Host Capabilities



`HostCapabilities` are sent to the View as part of the response to `ui/initialize` (inside `McpUiInitializeResult`).

They describe the features and capabilities that the Host supports.



```typescript

interface HostCapabilities {

&nbsp; /\*\* Experimental features (structure TBD). \*/

&nbsp; experimental?: {};

&nbsp; /\*\* Host supports opening external URLs. \*/

&nbsp; openLinks?: {};

&nbsp; /\*\* Host can proxy tool calls to the MCP server. \*/

&nbsp; serverTools?: {

&nbsp;   /\*\* Host supports tools/list\_changed notifications. \*/

&nbsp;   listChanged?: boolean;

&nbsp; };

&nbsp; /\*\* Host can proxy resource reads to the MCP server. \*/

&nbsp; serverResources?: {

&nbsp;   /\*\* Host supports resources/list\_changed notifications. \*/

&nbsp;   listChanged?: boolean;

&nbsp; };

&nbsp; /\*\* Host accepts log messages. \*/

&nbsp; logging?: {};

&nbsp; /\*\* Sandbox configuration applied by the host. \*/

&nbsp; sandbox?: {

&nbsp;   /\*\* Permissions granted by the host (camera, microphone, geolocation, clipboard-write). \*/

&nbsp;   permissions?: {

&nbsp;     camera?: {};

&nbsp;     microphone?: {};

&nbsp;     geolocation?: {};

&nbsp;     clipboardWrite?: {};

&nbsp;   };

&nbsp;   /\*\* CSP domains approved by the host. \*/

&nbsp;   csp?: {

&nbsp;     /\*\* Approved origins for network requests (fetch/XHR/WebSocket). \*/

&nbsp;     connectDomains?: string\[];

&nbsp;     /\*\* Approved origins for static resources (scripts, images, styles, fonts). \*/

&nbsp;     resourceDomains?: string\[];

&nbsp;     /\*\* Approved origins for nested iframes (frame-src directive). \*/

&nbsp;     frameDomains?: string\[];

&nbsp;     /\*\* Approved base URIs for the document (base-uri directive). \*/

&nbsp;     baseUriDomains?: string\[];

&nbsp;   };

&nbsp; };

}

```



\### Container Dimensions



The `HostContext` provides sizing information via `containerDimensions`:



\- \*\*`containerDimensions`\*\*: The dimensions of the container that holds the View. This controls the actual space the View occupies within the host. Each dimension (height and width) operates independently and can be either \*\*fixed\*\* or \*\*flexible\*\*.



\#### Dimension Modes



| Mode | Dimensions Field | Meaning |

|------|-----------------|---------|

| Fixed | `height` or `width` | Host controls the size. View should fill the available space. |

| Flexible | `maxHeight` or `maxWidth` | View controls the size, up to the specified maximum. |

| Unbounded | Field omitted | View controls the size with no limit. |



These modes can be combined independently. For example, a host might specify a fixed width but flexible height, allowing the View to grow vertically based on content.



\#### View Behavior



Views should check the containerDimensions configuration and apply appropriate CSS:



```typescript

// In the View's initialization

const containerDimensions = hostContext.containerDimensions;



if (containerDimensions) {

&nbsp; // Handle height

&nbsp; if ("height" in containerDimensions) {

&nbsp;   // Fixed height: fill the container

&nbsp;   document.documentElement.style.height = "100vh";

&nbsp; } else if ("maxHeight" in containerDimensions \&\& containerDimensions.maxHeight) {

&nbsp;   // Flexible with max: let content determine size, up to max

&nbsp;   document.documentElement.style.maxHeight = `${containerDimensions.maxHeight}px`;

&nbsp; }

&nbsp; // If neither, height is unbounded



&nbsp; // Handle width

&nbsp; if ("width" in containerDimensions) {

&nbsp;   // Fixed width: fill the container

&nbsp;   document.documentElement.style.width = "100vw";

&nbsp; } else if ("maxWidth" in containerDimensions \&\& containerDimensions.maxWidth) {

&nbsp;   // Flexible with max: let content determine size, up to max

&nbsp;   document.documentElement.style.maxWidth = `${containerDimensions.maxWidth}px`;

&nbsp; }

&nbsp; // If neither, width is unbounded

}

```



\#### Host Behavior



When using flexible dimensions (no fixed `height` or `width`), hosts MUST listen for `ui/notifications/size-changed` notifications from the View and update the iframe dimensions accordingly:



```typescript

// Host listens for size changes from the View

bridge.onsizechange = ({ width, height }) => {

&nbsp; // Update iframe to match View's content size

&nbsp; if (width != null) {

&nbsp;   iframe.style.width = `${width}px`;

&nbsp; }

&nbsp; if (height != null) {

&nbsp;   iframe.style.height = `${height}px`;

&nbsp; }

};

```



Views using the SDK automatically send size-changed notifications via ResizeObserver when `autoResize` is enabled (the default). The notifications are debounced and only sent when dimensions actually change.



\### Display Modes



Views can be displayed in different modes depending on the host's capabilities and the view's declared support.



```typescript

type McpUiDisplayMode = "inline" | "fullscreen" | "pip";

```



\- \*\*inline\*\*: Default mode, embedded within the host's content flow

\- \*\*fullscreen\*\*: View takes over the full screen/window

\- \*\*pip\*\*: Picture-in-picture, floating overlay



\#### Declaring Support



\*\*View (`appCapabilities.availableDisplayModes`):\*\*



Views declare which display modes they support in the `ui/initialize` request via `appCapabilities.availableDisplayModes`. This allows hosts to only offer display mode options that the view can handle.



```typescript

// Example: View declares support for inline and fullscreen

{

&nbsp; method: "ui/initialize",

&nbsp; params: {

&nbsp;   appCapabilities: {

&nbsp;     availableDisplayModes: \["inline", "fullscreen"]

&nbsp;   },

&nbsp;   // ...

&nbsp; }

}

```



\*\*Host (`HostContext.availableDisplayModes`):\*\*



Hosts declare which display modes they support in `HostContext.availableDisplayModes`. Views should check this before requesting a mode change.



\#### Requesting Changes



Views request display mode changes via `ui/request-display-mode`. See the Requests section for message format.



\#### Notifying Changes



Hosts notify views of display mode changes via `ui/notifications/host-context-changed` with the `displayMode` field.



\#### Requirements



\*\*View behavior:\*\*

\- View MUST declare all display modes it supports in `appCapabilities.availableDisplayModes` during initialization.

\- View MUST check if the requested mode is in `availableDisplayModes` from host context before requesting a mode change.

\- View MUST handle the response mode differing from the requested mode.



\*\*Host behavior:\*\*

\- Host MUST NOT switch the View to a display mode that does not appear in its `appCapabilities.availableDisplayModes`, if set.

\- Host MUST return the resulting mode (whether updated or not) in the response to `ui/request-display-mode`.

\- If the requested mode is not available, Host SHOULD return the current display mode in the response.

\- Host MAY decline display mode requests from Views that did not declare said modes in their capabilities.



\### Theming



Hosts can optionally pass CSS custom properties via `HostContext.styles.variables` for visual cohesion with the host environment.



\#### Current Standardized Variables



```typescript

/\*\* CSS variable keys available to Views for theming. \*/

type McpUiStyleVariableKey =

&nbsp; // Background colors

&nbsp; | "--color-background-primary"

&nbsp; | "--color-background-secondary"

&nbsp; | "--color-background-tertiary"

&nbsp; | "--color-background-inverse"

&nbsp; | "--color-background-ghost"

&nbsp; | "--color-background-info"

&nbsp; | "--color-background-danger"

&nbsp; | "--color-background-success"

&nbsp; | "--color-background-warning"

&nbsp; | "--color-background-disabled"

&nbsp; // Text colors

&nbsp; | "--color-text-primary"

&nbsp; | "--color-text-secondary"

&nbsp; | "--color-text-tertiary"

&nbsp; | "--color-text-inverse"

&nbsp; | "--color-text-info"

&nbsp; | "--color-text-danger"

&nbsp; | "--color-text-success"

&nbsp; | "--color-text-warning"

&nbsp; | "--color-text-disabled"

&nbsp; | "--color-text-ghost"

&nbsp; // Border colors

&nbsp; | "--color-border-primary"

&nbsp; | "--color-border-secondary"

&nbsp; | "--color-border-tertiary"

&nbsp; | "--color-border-inverse"

&nbsp; | "--color-border-ghost"

&nbsp; | "--color-border-info"

&nbsp; | "--color-border-danger"

&nbsp; | "--color-border-success"

&nbsp; | "--color-border-warning"

&nbsp; | "--color-border-disabled"

&nbsp; // Ring colors

&nbsp; | "--color-ring-primary"

&nbsp; | "--color-ring-secondary"

&nbsp; | "--color-ring-inverse"

&nbsp; | "--color-ring-info"

&nbsp; | "--color-ring-danger"

&nbsp; | "--color-ring-success"

&nbsp; | "--color-ring-warning"

&nbsp; // Typography - Family

&nbsp; | "--font-sans"

&nbsp; | "--font-mono"

&nbsp; // Typography - Weight

&nbsp; | "--font-weight-normal"

&nbsp; | "--font-weight-medium"

&nbsp; | "--font-weight-semibold"

&nbsp; | "--font-weight-bold"

&nbsp; // Typography - Text Size

&nbsp; | "--font-text-xs-size"

&nbsp; | "--font-text-sm-size"

&nbsp; | "--font-text-md-size"

&nbsp; | "--font-text-lg-size"

&nbsp; // Typography - Heading Size

&nbsp; | "--font-heading-xs-size"

&nbsp; | "--font-heading-sm-size"

&nbsp; | "--font-heading-md-size"

&nbsp; | "--font-heading-lg-size"

&nbsp; | "--font-heading-xl-size"

&nbsp; | "--font-heading-2xl-size"

&nbsp; | "--font-heading-3xl-size"

&nbsp; // Typography - Text Line Height

&nbsp; | "--font-text-xs-line-height"

&nbsp; | "--font-text-sm-line-height"

&nbsp; | "--font-text-md-line-height"

&nbsp; | "--font-text-lg-line-height"

&nbsp; // Typography - Heading Line Height

&nbsp; | "--font-heading-xs-line-height"

&nbsp; | "--font-heading-sm-line-height"

&nbsp; | "--font-heading-md-line-height"

&nbsp; | "--font-heading-lg-line-height"

&nbsp; | "--font-heading-xl-line-height"

&nbsp; | "--font-heading-2xl-line-height"

&nbsp; | "--font-heading-3xl-line-height"

&nbsp; // Border radius

&nbsp; | "--border-radius-xs"

&nbsp; | "--border-radius-sm"

&nbsp; | "--border-radius-md"

&nbsp; | "--border-radius-lg"

&nbsp; | "--border-radius-xl"

&nbsp; | "--border-radius-full"

&nbsp; // Border width

&nbsp; | "--border-width-regular"

&nbsp; // Shadows

&nbsp; | "--shadow-hairline"

&nbsp; | "--shadow-sm"

&nbsp; | "--shadow-md"

&nbsp; | "--shadow-lg";

```



\#### Host Behavior



\- Hosts can provide any subset of standardized variables, or not pass `styles` at all

&nbsp; - However, unexpected clashes may occur if hosts pass some color variables but not others for example, since Views are instructed to fallback on their own default values for unspecified style variables

\- Hosts should use the CSS `light-dark()` function for theme-aware values (i.e. light mode and dark mode colors)



\#### View Behavior



\- Views should set default fallback values for the set of these variables that they use, to account for hosts who don't pass some or all style variables. This ensures graceful degradation when hosts omit `styles` or specific variables:

```

:root {

&nbsp; --color-text-primary: light-dark(#171717, #000000);

&nbsp; --border-radius-small: 8px;

&nbsp; ...

}

```

\- Views can use the `applyHostStyleVariables` utility (or `useHostStyleVariables` if they prefer a React hook) to easily populate the host-provided CSS variables into their style sheet

\- Views can use the `applyDocumentTheme` utility (or `useDocumentTheme` if they prefer a React hook) to easily respond to Host Context `theme` changes in a way that is compatible with the host's light/dark color variables 



Example usage of standardized CSS variables:



```css

.container {

&nbsp; background: var(--color-background-primary);

&nbsp; color: var(--color-text-primary);

&nbsp; font-family: var(--font-sans);

}

```



\#### Custom Fonts



Hosts can provide custom fonts via `styles.css.fonts`, which can contain `@font-face` rules for self-hosted fonts, `@import` statements for font services like Google Fonts, or both:



```typescript

hostContext.styles.variables\["--font-sans"] = '"Font Name", sans-serif';



// Self-hosted fonts

hostContext.styles.css.fonts = `

&nbsp; @font-face {

&nbsp;   font-family: "Font Name";

&nbsp;   src: url("https://url-where-font-is-hosted.com/.../Regular.otf") format("opentype");

&nbsp;   font-weight: 400;

&nbsp;   font-style: normal;

&nbsp;   font-display: swap;

&nbsp; }

&nbsp; @font-face {

&nbsp;   font-family: "Font Name";

&nbsp;   src: url("https://url-where-font-is-hosted.com/.../Medium.otf") format("opentype");

&nbsp;   font-weight: 500;

&nbsp;   font-style: medium;

&nbsp;   font-display: swap;

&nbsp; }

`;



// Google Fonts

hostContext.styles.css.fonts = `

&nbsp; @import url('https://fonts.googleapis.com/css2?family=Font+Name\&display=swap');

`;

```



Views can use the `applyHostFonts` utility to inject the font CSS into the document:



```typescript

if (hostContext.styles?.css?.fonts) {

&nbsp; applyHostFonts(hostContext.styles.css.fonts);

}

```



\### MCP Apps Specific Messages



MCP Apps introduces additional JSON-RPC methods for UI-specific functionality:



\#### Requests (View → Host)



`ui/open-link` - Request host to open external URL



```typescript

// Request

{

&nbsp; jsonrpc: "2.0",

&nbsp; id: 1,

&nbsp; method: "ui/open-link",

&nbsp; params: {

&nbsp;   url: string  // URL to open in host's browser

&nbsp; }

}



// Success Response

{

&nbsp; jsonrpc: "2.0",

&nbsp; id: 1,

&nbsp; result: {}  // Empty result on success

}



// Error Response (if denied or failed)

{

&nbsp; jsonrpc: "2.0",

&nbsp; id: 1,

&nbsp; error: {

&nbsp;   code: -32000,  // Implementation-defined error

&nbsp;   message: "Link opening denied by user" | "Invalid URL" | "Policy violation"

&nbsp; }

}

```



Host SHOULD open the URL in the user's default browser or a new tab.



`ui/message` - Send message content to the host's chat interface



```typescript

// Request

{

&nbsp; jsonrpc: "2.0",

&nbsp; id: 2,

&nbsp; method: "ui/message",

&nbsp; params: {

&nbsp;   role: "user",

&nbsp;   content: {

&nbsp;     type: "text",

&nbsp;     text: string

&nbsp;   }

&nbsp; }

}



// Success Response

{

&nbsp; jsonrpc: "2.0",

&nbsp; id: 2,

&nbsp; result: {}  // Empty result on success

}



// Error Response (if denied or failed)

{

&nbsp; jsonrpc: "2.0",

&nbsp; id: 2,

&nbsp; error: {

&nbsp;   code: -32000,  // Implementation-defined error

&nbsp;   message: "Message sending denied" | "Invalid message format"

&nbsp; }

}

```

Host behavior:

\* Host SHOULD add the message to the conversation context, preserving the specified role.

\* Host MAY request user consent.



`ui/request-display-mode` - Request host to change display mode



```typescript

// Request

{

&nbsp; jsonrpc: "2.0",

&nbsp; id: 3,

&nbsp; method: "ui/request-display-mode",

&nbsp; params: {

&nbsp;   mode: "inline" | "fullscreen" | "pip"  // Requested display mode

&nbsp; }

}



// Success Response

{

&nbsp; jsonrpc: "2.0",

&nbsp; id: 3,

&nbsp; result: {

&nbsp;   mode: "inline" | "fullscreen" | "pip"  // Actual display mode set

&nbsp; }

}

```



See the Display Modes section for detailed behavior requirements.



`ui/update-model-context` - Update the model context



```typescript

// Request

{

&nbsp; jsonrpc: "2.0",

&nbsp; id: 3,

&nbsp; method: "ui/update-model-context",

&nbsp; params: {

&nbsp;   content?: ContentBlock\[],

&nbsp;   structuredContent?: Record<string, unknown>

&nbsp; }

}



// Success Response

{

&nbsp; jsonrpc: "2.0",

&nbsp; id: 3,

&nbsp; result: {}  // Empty result on success

}



// Error Response (if denied or failed)

{

&nbsp; jsonrpc: "2.0",

&nbsp; id: 3,

&nbsp; error: {

&nbsp;   code: -32000,  // Implementation-defined error

&nbsp;   message: "Context update denied" | "Invalid content format"

&nbsp; }

}

```



The View MAY send this request to update the Host's model context. This context will be used in future turns. Each request overwrites the previous context sent by the View.

This event serves a different use case from `notifications/message` (logging) and `ui/message` (which also trigger follow-ups).



Host behavior:

\- SHOULD provide the context to the model in future turns

\- MAY overwrite the previous model context with the new update

\- MAY defer sending the context to the model until the next user message (including `ui/message`)

\- MAY dedupe identical `ui/update-model-context` calls

\- If multiple updates are received before the next user message, Host SHOULD only send the last update to the model

\- MAY display context updates to the user



\#### Notifications (Host → View)



`ui/notifications/tool-input` - Host MUST send this notification with the complete tool arguments after the View's initialize request completes.



```typescript

{

&nbsp; jsonrpc: "2.0",

&nbsp; method: "ui/notifications/tool-input",

&nbsp; params: {

&nbsp;   arguments: Record<string, unknown>  // Tool input arguments

&nbsp; }

}

```



Host sends this notification after the View's initialize request completes, when tool arguments become available. This notification is sent at most once and is required before sending `ui/notifications/tool-result`.



`ui/notifications/tool-input-partial` - Host MAY send this notification zero or more times while the agent is streaming tool arguments, before `ui/notifications/tool-input` is sent.



```typescript

{

&nbsp; jsonrpc: "2.0",

&nbsp; method: "ui/notifications/tool-input-partial",

&nbsp; params: {

&nbsp;   arguments: Record<string, unknown>  // Tool input arguments

&nbsp; }

}

```



The arguments object represents best-effort recovery of incomplete JSON, with unclosed structures automatically closed to produce valid JSON. Host behavior (optional):



\- MAY parse the agent's partial JSON output by closing unclosed brackets/braces

\- MAY send recovered arguments as they become available during streaming

\- MUST stop sending once `ui/notifications/tool-input` is sent with complete arguments



View behavior (optional):



\- MAY ignore these notifications entirely

\- MAY render progressive loading/streaming states based on available fields

\- MUST NOT rely on partial arguments for critical operations

\- SHOULD gracefully handle missing or changing fields between notifications



`ui/notifications/tool-result` - Tool execution result



```typescript

{

&nbsp; jsonrpc: "2.0",

&nbsp; method: "ui/notifications/tool-result",

&nbsp; params: CallToolResult  // Standard MCP type

}

```



Host MUST send this notification when tool execution completes (if the View is displayed during tool execution).



`ui/notifications/tool-cancelled` - Tool execution was cancelled



```typescript

{

&nbsp; jsonrpc: "2.0",

&nbsp; method: "ui/notifications/tool-cancelled",

&nbsp; params: {

&nbsp;   reason: string

&nbsp; }

}

```



Host MUST send this notification if the tool execution was cancelled, for any reason (which can optionally be specified), including user action, sampling error, classifier intervention, etc.



`ui/resource-teardown` - Host notifies View before teardown



```typescript

{

&nbsp; jsonrpc: "2.0",

&nbsp; id: 1,

&nbsp; method: "ui/resource-teardown",

&nbsp; params: {

&nbsp;   reason: string

&nbsp; }

}



// Success Response

{

&nbsp; jsonrpc: "2.0",

&nbsp; id: 1,

&nbsp; result: {}  // Empty result on success

}



// Error Response (if failed)

{

&nbsp; jsonrpc: "2.0",

&nbsp; id: 1,

&nbsp; error: {

&nbsp;   code: -32000,  // Implementation-defined error

&nbsp;   message: "Teardown error"

&nbsp; }

}

```



Host MUST send this notification before tearing down the UI resource, for any reason, including user action, resource re-allocation, etc. The Host MAY specify the reason.

Host SHOULD wait for a response before tearing down the resource (to prevent data loss).



`ui/notifications/size-changed` - View's size changed



```typescript

{

&nbsp; jsonrpc: "2.0",

&nbsp; method: "ui/notifications/size-changed",

&nbsp; params: {

&nbsp;   width: number,   // Viewport width in pixels

&nbsp;   height: number   // Viewport height in pixels

&nbsp; }

}

```



The View SHOULD send this notification when rendered content body size changes (e.g. using ResizeObserver API to report up to date size).



`ui/notifications/host-context-changed` - Host context has changed



```typescript

{

&nbsp; jsonrpc: "2.0",

&nbsp; method: "ui/notifications/host-context-changed",

&nbsp; params: Partial<HostContext>  // See HostContext type above

}

```



Host MAY send this notification when any context field changes (e.g., theme toggle, display mode change, device orientation change, window/panel resize). This notification contains partial updates - the View SHOULD merge received fields with its current context state.



\#### Reserved Messages (Sandbox Proxy)



These messages are reserved for web-based hosts that implement the recommended double-iframe sandbox architecture:



`ui/notifications/sandbox-proxy-ready` (Sandbox Proxy → Host) - Sandbox proxy is ready



```typescript

{

&nbsp; jsonrpc: "2.0",

&nbsp; method: "ui/notifications/sandbox-proxy-ready",

&nbsp; params: {}

}

```



`ui/notifications/sandbox-resource-ready` (Host → Sandbox Proxy) - HTML resource ready to load



```typescript

{

&nbsp; jsonrpc: "2.0",

&nbsp; method: "ui/notifications/sandbox-resource-ready",

&nbsp; params: {

&nbsp;   html: string,        // HTML content to load

&nbsp;   sandbox?: string,    // Optional override for inner iframe `sandbox` attribute

&nbsp;   csp?: {              // CSP configuration from resource metadata

&nbsp;     connectDomains?: string\[],

&nbsp;     resourceDomains?: string\[],

&nbsp;     frameDomains?: string\[],

&nbsp;     baseUriDomains?: string\[],

&nbsp;   },

&nbsp;   permissions?: {      // Sandbox permissions from resource metadata

&nbsp;     camera?: {},

&nbsp;     microphone?: {},

&nbsp;     geolocation?: {},

&nbsp;     clipboardWrite?: {},

&nbsp;   }

&nbsp; }

}

```



These messages facilitate the communication between the outer sandbox proxy iframe and the host, enabling secure loading of untrusted HTML content. The `permissions` field maps to the inner iframe's `allow` attribute for Permission Policy features.



\### Lifecycle



The typical lifecycle for rendering a UI resource:



\#### 1. Connection \& Discovery



```mermaid

sequenceDiagram

&nbsp; participant H as Host

&nbsp; participant S as MCP Server



&nbsp; autonumber

&nbsp; S -->> H: resources/list (includes ui:// resources)

&nbsp; S -->> H: tools/list (includes tools with \_meta.ui metadata)

```



\#### 2. UI Initialization (Desktop/Native Hosts)



```mermaid

sequenceDiagram

&nbsp; participant H as Host

&nbsp; participant UI as View (iframe)

&nbsp; participant P as Sandbox Proxy

&nbsp; participant S as MCP Server



&nbsp; autonumber

&nbsp; par UI Tool call

&nbsp;   H ->> S: tools/call to Tool with \_meta.ui metadata

&nbsp; and UI initialization

&nbsp;   alt Desktop/Native hosts

&nbsp;     H ->> H: Render View in an iframe (HTML from the ui:// resource)

&nbsp;   else Web hosts

&nbsp;     H ->> H: Render Sandbox Proxy in an iframe (different origin)

&nbsp;     P ->> H: ui/notifications/sandbox-proxy-ready

&nbsp;     H -->> P: ui/notifications/sandbox-resource-ready (HTML content)

&nbsp;     P -> P: Render inner iframe with HTML



&nbsp;   end

&nbsp;   UI ->> H: ui/initialize

&nbsp;   H -->> UI: McpUiInitializeResult (e.g., host-context, capabilities, etc.)

&nbsp;   UI ->> H: ui/notifications/initialized

&nbsp;   opt Stream Tool input to UI

&nbsp;     H -->> UI: ui/notifications/tool-input-partial (0..n)

&nbsp;   end

&nbsp;   H -->> UI: ui/notifications/tool-input (complete)

&nbsp; end

&nbsp; alt Tool complete

&nbsp;   H -->> UI: ui/notifications/tool-result

&nbsp; else Tool cancelled

&nbsp;   H -->> UI: ui/notifications/tool-cancelled

&nbsp; end

```



Note: when the View is rendered inside a sandbox, the sandbox transparently passes messages between the View and the Host, except for messages named `ui/notifications/sandbox-\*`.



\#### 3. Interactive Phase



```mermaid

sequenceDiagram

&nbsp; actor U as User / Agent

&nbsp; participant H as Host

&nbsp; participant UI as View (iframe)

&nbsp; participant S as MCP Server

&nbsp; loop Interactive phase

&nbsp;   U ->> UI: interaction (e.g., click)

&nbsp;   alt Tool call

&nbsp;     UI ->> H: tools/call

&nbsp;     H ->> S: tools/call

&nbsp;     opt Stream Tool input to UI

&nbsp;       H -->> UI: ui/notifications/tool-input-partial (0..n)

&nbsp;     end

&nbsp;     H -->> UI: ui/notifications/tool-input (complete)

&nbsp;     H-->>UI: ui/notifications/tool-result

&nbsp;   else Message

&nbsp;     UI ->> H: ui/message

&nbsp;     H -->> UI: ui/message response

&nbsp;     H -->> H: Process message and follow up

&nbsp;   else Context update

&nbsp;     UI ->> H: ui/update-model-context

&nbsp;     H ->> H: Store model context (overwrite existing)

&nbsp;     H -->> UI: ui/update-model-context response

&nbsp;   else Log

&nbsp;     UI ->> H: notifications/message

&nbsp;     H ->> H: Record log for debugging/telemetry

&nbsp;   else Resource read

&nbsp;     UI ->> H: resources/read

&nbsp;     H ->> S: resources/read

&nbsp;     S --> H: resources/read response

&nbsp;     H --> UI: resources/read response

&nbsp;   end

&nbsp;   opt View notifications

&nbsp;     UI ->> H: notifications (e.g., ui/notifications/size-changed)

&nbsp;   end

&nbsp;   opt Host notifications

&nbsp;     H ->> UI: notifications (e.g., ui/notifications/host-context-changed)

&nbsp;   end

&nbsp; end

```



\#### 4. Cleanup



```mermaid

sequenceDiagram

&nbsp; participant H as Host

&nbsp; participant UI as View (iframe)

&nbsp; H ->> UI: ui/resource-teardown

&nbsp; UI --> UI: Graceful termination

&nbsp; UI -->> H: ui/resource-teardown response

&nbsp; H -x H: Tear down iframe and listeners

```



Note: Cleanup may be triggered at any point in the lifecycle following View initialization.



\#### Key Differences from Pre-SEP MCP-UI:



\- Handshakes using MCP-like `ui/initialize` / `ui/notifications/initialized` messages instead of `iframe-ready`

\- Capabilities negotiated via `McpUiInitializeResult`

\- Tool data passed via notifications



\### Data Passing



Tool execution results are passed to the View through two mechanisms:



\#### 1. Tool Input (via `ui/notifications/tool-input` notification)



The original tool call arguments:



```typescript

// Tool was called with (JSON-RPC request from Host to Server):

{

&nbsp; jsonrpc: "2.0",

&nbsp; id: 1,

&nbsp; method: "tools/call",

&nbsp; params: {

&nbsp;   name: "get\_weather",

&nbsp;   arguments: {

&nbsp;     location: "San Francisco"

&nbsp;   }

&nbsp; }

}



// View receives (JSON-RPC notification from Host to View):

{

&nbsp; jsonrpc: "2.0",

&nbsp; method: "ui/notifications/tool-input",

&nbsp; params: {

&nbsp;   arguments: {

&nbsp;     location: "San Francisco"

&nbsp;   }

&nbsp; }

}

```



\#### 2. Tool Result (via `ui/notifications/tool-result` notification)



The tool's execution result:



```typescript

// Server returns from tool execution (JSON-RPC response):

{

&nbsp; jsonrpc: "2.0",

&nbsp; id: 1,  // Matches the tools/call request id

&nbsp; result: {

&nbsp;   content: \[

&nbsp;     { type: "text", text: "Current weather: Sunny, 72°F" }

&nbsp;   ],

&nbsp;   structuredContent: {

&nbsp;     temperature: 72,

&nbsp;     conditions: "sunny",

&nbsp;     humidity: 45

&nbsp;   },

&nbsp;   \_meta: {

&nbsp;     timestamp: "2025-11-10T15:30:00Z",

&nbsp;     source: "weather-api"

&nbsp;   }

&nbsp; }

}



// View receives (JSON-RPC notification):

{

&nbsp; jsonrpc: "2.0",

&nbsp; method: "ui/notifications/tool-result",

&nbsp; params: {

&nbsp;   content: \[

&nbsp;     { type: "text", text: "Current weather: Sunny, 72°F" }

&nbsp;   ],

&nbsp;   structuredContent: {

&nbsp;     temperature: 72,

&nbsp;     conditions: "sunny",

&nbsp;     humidity: 45

&nbsp;   },

&nbsp;   \_meta: {

&nbsp;     timestamp: "2025-11-10T15:30:00Z",

&nbsp;     source: "weather-api"

&nbsp;   }

&nbsp; }

}

```



\*\*Best Practices:\*\*



\- `content`: Text representation for model context and text-only hosts

\- `structuredContent`: Structured data optimized for UI rendering (not added to model context)

\- `\_meta`: Additional metadata (timestamps, version info, etc.) not intended for model context



\#### 3. Interactive Updates



Views can request fresh data by calling tools:



```typescript

// View requests updated data

await client.callTool("get\_weather", { location: "New York" });



// Result returned via standard tools/call response

```



This pattern enables interactive, self-updating views.



Note: Tools with `visibility: \["app"]` are hidden from the agent but remain callable by apps via `tools/call`. This enables UI-only interactions (refresh buttons, form submissions) without exposing implementation details to the model. See the Visibility section under Resource Discovery for details.



\### Client\\<\\>Server Capability Negotiation



Clients and servers negotiate MCP Apps support through the standard MCP extensions capability mechanism (defined in SEP-1724).



\#### Client (Host) Capabilities



Clients advertise MCP Apps support in the initialize request using the extension identifier `io.modelcontextprotocol/ui`:



```json

{

&nbsp; "method": "initialize",

&nbsp; "params": {

&nbsp;   "protocolVersion": "2024-11-05",

&nbsp;   "capabilities": {

&nbsp;     "extensions": {

&nbsp;       "io.modelcontextprotocol/ui": {

&nbsp;         "mimeTypes": \["text/html;profile=mcp-app"]

&nbsp;       }

&nbsp;     }

&nbsp;   },

&nbsp;   "clientInfo": {

&nbsp;     "name": "claude-desktop",

&nbsp;     "version": "1.0.0"

&nbsp;   }

&nbsp; }

}

```



\*\*Extension Settings:\*\*



\- `mimeTypes`: Array of supported content types (REQUIRED, e.g., `\["text/html;profile=mcp-app"]`)



Future versions may add additional settings:



\- `features`: Specific feature support (e.g., `\["streaming", "persistence"]`)

\- `sandboxPolicies`: Supported sandbox attribute configurations



\#### Server Behavior



Servers SHOULD check client capabilities before registering UI-enabled tools. The SDK provides the `getUiCapability` helper for this:



```typescript

import { getUiCapability, RESOURCE\_MIME\_TYPE } from "@modelcontextprotocol/ext-apps/server";



const uiCap = getUiCapability(clientCapabilities);

if (uiCap?.mimeTypes?.includes(RESOURCE\_MIME\_TYPE)) {

&nbsp; // Register tools with UI templates

&nbsp; server.registerTool("get\_weather", {

&nbsp;   description: "Get weather with interactive dashboard",

&nbsp;   inputSchema: { /\* ... \*/ },

&nbsp;   \_meta: {

&nbsp;     ui: { resourceUri: "ui://weather-server/dashboard" }

&nbsp;   }

&nbsp; });

} else {

&nbsp; // Register text-only version

&nbsp; server.registerTool("get\_weather", {

&nbsp;   description: "Get weather as text",

&nbsp;   inputSchema: { /\* ... \*/ }

&nbsp;   // No UI metadata

&nbsp; });

}

```



\*\*Graceful Degradation:\*\*



\- Servers SHOULD provide text-only fallback behavior for all UI-enabled tools

\- Tools MUST return meaningful content array even when UI is available

\- Servers MAY register different tool variants based on host capabilities



\### Extensibility



This specification defines the Minimum Viable Product (MVP) for MCP Apps. Future extensions may include:



\*\*Content Types (deferred from MVP):\*\*



\- `externalUrl`: Embed external web applications (e.g., `text/uri-list`)



\*\*Advanced Features (see Future Considerations):\*\*



\- Support multiple UI resources in a tool response

\- State persistence and restoration

\- Custom sandbox policies per resource

\- View-to-View communication

\- Screenshot/preview generation APIs



\## Rationale



This proposal synthesizes feedback from the UI CWG and MCP-UI community, host implementors, and lessons from similar solutions. The guiding principle of this proposal is to start lean and expand in the future. There are breaking changes from existing solutions, which will be addressed via the MCP-UI SDK during the migration period.



\### Design Decisions



\#### 1. Predeclared Resources vs. Inline Embedding



\*\*Decision:\*\* Require UI resources to be registered and referenced in tool metadata.



\*\*Rationale:\*\*



\- Enables hosts to prefetch templates before tool execution, improving performance

\- Separates presentation (template) from data (tool results), improving caching

\- Allows hosts to review UI templates

\- Aligns with MCP's resource discovery pattern



\*\*Alternatives considered:\*\*



\- \*\*Embedded resources:\*\* Current MCP-UI approach, where resources are returned in tool results. Although it's more convenient for server development, it was deferred due to the gaps in performance optimization and the challenges in the UI review process.

\- \*\*Resource links:\*\* Predeclare the resources but return links in tool results. Deferred due to the gaps in performance optimization.



\#### 2. Host communication via MCP Transport



\*\*Decision:\*\* Use MCP's JSON-RPC base protocol over postMessage instead of custom message format.



\*\*Rationale:\*\*



\- Reuses existing MCP infrastructure (type definitions, error handling, timeouts)

\- UI developers can use standard MCP SDK (`@modelcontextprotocol/sdk`) or alternatives

\- Automatic compatibility with future MCP features (long-running tools, sampling, etc.)

\- Better auditability through structured JSON-RPC messages

\- Reduces maintenance burden (no parallel protocol to evolve)



\*\*Alternatives considered:\*\*



\- \*\*Custom message protocol:\*\* Current MCP-UI approach with message types like tool, prompt, etc. These message types can be translated to a subset of the proposed JSON-RPC messages.

\- \*\*Global API object:\*\* Rejected because it requires host-specific injection and doesn't work with external iframe sources. Syntactic sugar may still be added on the server/UI side.



\#### 3. Support Raw HTML Content Type



\*\*Decision:\*\* MVP supports only `text/html;profile=mcp-app` (rawHtml), with other types explicitly deferred.



\*\*Rationale:\*\*



\- HTML is universally supported and well-understood

\- Simplest security model (standard iframe sandbox)

\- Allows screenshot/preview generation (e.g., via html2canvas)

\- Sufficient for most observed use cases

\- Provides clear baseline for future extensions



\*\*Alternatives considered:\*\*



\- \*\*Include external URLs in MVP:\*\* This is one of the easiest content types for servers to adopt, as it's possible to embed regular apps. However, it was deferred due to concerns around model visibility, inability to screenshot content, and review process.

\- \*\*Support multiple content types:\*\* Deferred to maintain a lean MVP.



\#### 4. Host Theming via CSS Variables



\*\*Decision:\*\* Provide a standardized set of CSS custom properties for visual cohesion.



\*\*Rationale:\*\*



\- CSS variables are universal, framework-agnostic, and require no runtime

\- Views apply styles via `var(--name)` with fallbacks for graceful degradation

\- Limited variable set (colors, typography, borders) ensures hosts can realistically provide all values

\- Spacing intentionally excluded—layouts break when spacing varies from original design

\- No UI component library—no single library works across all host environments



\*\*Alternatives considered:\*\*



\- \*\*Full design system:\*\* Rejected as too prescriptive; hosts have different aesthetics

\- \*\*Inline styles in tool results:\*\* Rejected; separating theming from data enables caching and updates

\- \*\*CSS-in-JS injection:\*\* Rejected; framework-specific and security concerns with injected code



\#### 5. Tool Visibility via Metadata



\*\*Decision:\*\* Use `\_meta.ui.visibility` array to control tool accessibility between model and app.



\*\*Rationale:\*\*



\- Nested `\_meta.ui` structure groups all UI-related metadata cleanly

\- Array format (`\["model", "app"]`) allows flexible combinations

\- Default `\["model", "app"]` allows both agent and app to access tools

\- `"app"` scope is per-server, preventing cross-server tool calls

\- Cleaner than OpenAI's two-field approach (`widgetAccessible` + `visibility`)



\*\*Alternatives considered:\*\*



\- \*\*Two separate fields:\*\* OpenAI uses `widgetAccessible` and `visibility` separately. Rejected as redundant; single `visibility` array covers all cases.

\- \*\*Boolean `private` flag:\*\* Simpler but less flexible; doesn't express model-only tools.

\- \*\*Flat `ui/visibility` key:\*\* Rejected in favor of nested structure for consistency with future `\_meta.ui` fields.



\### Backward Compatibility



The proposal builds on the existing core protocol. There are no incompatibilities.



\### Reference Implementation



The MCP-UI project serves as a reference implementation demonstrating the core concept, though it uses pre-SEP patterns.



Olivier Chafik has developed a prototype demonstrating the pattern described in this SEP.



\## Security Implications



Hosting interactive UI content from potentially untrusted MCP servers requires careful security consideration.



\### Threat Model



Attackers may use the embedded UI in different scenarios. For example:



\- Malicious server delivers harmful HTML content

\- Compromised View attempts to escape sandbox

\- View attempts unauthorized tool execution

\- View exfiltrates sensitive host data

\- View performs phishing or social engineering



\### Mitigations



\#### 1. Iframe Sandboxing



All View content MUST be rendered in sandboxed iframes with restricted permissions.



The sandbox limits the View from accessing the host or manipulating it. All communication with the host is done via `postMessage`, where the host is in control.



\#### 2. Auditable Communication



All View-to-host communication goes through auditable MCP JSON-RPC messages.



\*\*Host behavior:\*\*



\- Validate all incoming messages from UI iframes

\- Reject malformed message types

\- Log View-initiated RPC calls for security review



\#### 3. Predeclared Resource Review



Hosts receive UI templates during connection setup, before tool execution.



\*\*Host behavior:\*\*



\- Review HTML content for obvious malicious patterns

\- Generate hash/signature for resources

\- Warn users about suspicious content

\- Implement allowlists/blocklists based on resource hashes



\#### 4. Content Security Policy Enforcement



Hosts MUST enforce Content Security Policies based on resource metadata.



\*\*CSP Construction from Metadata:\*\*



```typescript

const csp = resource.\_meta?.ui?.csp; // `resource` is extracted from the `contents` of the `resources/read` result

const permissions = resource.\_meta?.ui?.permissions;



const cspValue = `

&nbsp; default-src 'none';

&nbsp; script-src 'self' 'unsafe-inline' ${csp?.resourceDomains?.join(' ') || ''};

&nbsp; style-src 'self' 'unsafe-inline' ${csp?.resourceDomains?.join(' ') || ''};

&nbsp; connect-src 'self' ${csp?.connectDomains?.join(' ') || ''};

&nbsp; img-src 'self' data: ${csp?.resourceDomains?.join(' ') || ''};

&nbsp; font-src 'self' ${csp?.resourceDomains?.join(' ') || ''};

&nbsp; media-src 'self' data: ${csp?.resourceDomains?.join(' ') || ''};

&nbsp; frame-src ${csp?.frameDomains?.join(' ') || "'none'"};

&nbsp; object-src 'none';

&nbsp; base-uri ${csp?.baseUriDomains?.join(' ') || "'self'"};

`;



// Permission Policy for iframe allow attribute

const allowList: string\[] = \[];

if (permissions?.camera) allowList.push('camera');

if (permissions?.microphone) allowList.push('microphone');

if (permissions?.geolocation) allowList.push('geolocation');

const allowAttribute = allowList.join(' ');

```



\*\*Security Requirements:\*\*



\- Host MUST block connections to undeclared domains

\- Host SHOULD warn users when UI requires external domain access

\- Host MAY implement global domain allowlists/blocklists



\### Other risks



\- \*\*Social engineering:\*\* UI can still display misleading content. Hosts should clearly indicate sandboxed UI boundaries.

\- \*\*Resource consumption:\*\* Malicious View can consume CPU/memory. Hosts should implement resource limits.



\## Reservations in MCP



\- The resource prefix `ui://` will be reserved for MCP Apps

\- The label `io.modelcontextprotocol/ui` is reserved

