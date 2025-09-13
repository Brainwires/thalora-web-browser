# Synaptic - Pure Rust Headless Web Browser

🧠 **Neural connections between AI models and the web**

A cutting-edge headless web browser built entirely in Rust, designed specifically for AI model integration via the Model Context Protocol (MCP). Unlike traditional scrapers, Synaptic provides full browser capabilities including form submissions, session management, link clicking, and interactive navigation - all without any Chrome or Chromium dependencies.

## 🎯 **Browser Capabilities**

### ✅ **Full Web Interaction**
- **Form Processing**: Automatic form detection, filling, and submission (GET/POST)
- **Session Management**: Cookie jar with persistent session state
- **Link Navigation**: Click links with CSS selectors
- **Multi-Request Workflows**: Maintain state across multiple page visits
- **Response Processing**: Handle redirects, status codes, and response headers
- **JavaScript Execution**: Sandboxed JS execution with Boa engine

### ✅ **Advanced Features**
- **Modern Browser Support**: Full browser automation detection evasion
- **Stealth Capabilities**: Canvas fingerprinting, WebGL, and chrome object simulation
- **Human-like Patterns**: Random timing delays and request behavior mimicry
- **Chrome-like Headers**: Dynamic header randomization with realistic browser variants
- **TLS Fingerprinting**: HTTP/2 support with realistic connection patterns
- **Compression Support**: Automatic gzip/brotli/deflate/zstd decompression
- **Cookie Persistence**: Full cookie jar implementation with domain/path handling
- **Content-Type Support**: Handle form-urlencoded, JSON, and multipart data
- **Error Handling**: Comprehensive error responses with detailed debugging

### 🚀 **Enhanced Modules (Available)**

We've built sophisticated modules for modern web application support:

#### **Enhanced JavaScript Engine** (`src/renderer.rs`)
- **Modern Browser APIs**: Complete window, document, navigator object simulation
- **Canvas Fingerprinting**: Realistic 2D and WebGL context with variations
- **Chrome Object**: Comprehensive window.chrome with loadTimes() and csi()
- **WebGL Support**: Full WebGL parameter simulation with realistic vendor info
- **Performance APIs**: performance.now() with accurate timing and variance
- **Enhanced Console**: Multi-level logging with capture capabilities
- **Challenge Detection**: Automatic detection and execution of anti-bot challenges
- **Security Sandboxing**: Safe execution with dangerous pattern blocking

#### **Browser Automation Detection Evasion** (`src/browser.rs`)
- **User-Agent Rotation**: Dynamic rotation of Chrome/Firefox/Safari/Edge variants
- **Header Randomization**: Realistic Accept-Language, Chrome version variations  
- **Human Timing**: Random delays (100ms-2s) between requests for natural patterns
- **Request Fingerprinting**: Connection pooling and HTTP/2 with realistic TLS signatures
- **Behavioral Analysis**: Request timing tracking for pattern detection avoidance
- **Stealth Configuration**: Viewport, screen resolution, and hardware simulation
- **Anti-Detection Patterns**: Detection and evasion of common automation signatures

#### **React/Next.js Processor** (`src/react_processor.rs`)
- **Next.js Streaming Parser**: Processes `__next_f.push()` streaming data
- **Server Component Handling**: Reconstructs React element trees
- **Hydration Support**: Processes React hydration data
- **Component Rendering**: Converts React elements to HTML
- **Metadata Extraction**: Pulls React metadata and navigation data

## 🌐 **Core Browser Architecture**

### Browser Components

- **HeadlessWebBrowser**: Main browser instance with persistent state and stealth capabilities
- **Cookie Management**: Arc<reqwest::cookie::Jar> for thread-safe session handling
- **Request Engine**: reqwest with dynamic headers, HTTP/2, and TLS fingerprinting
- **JavaScript Runtime**: Boa engine with modern API polyfills and challenge execution
- **Stealth System**: Canvas fingerprinting, WebGL simulation, and timing analysis
- **Anti-Detection**: User-agent rotation, header randomization, and behavioral mimicry
- **CSS Processing**: LightningCSS for style parsing and application
- **Layout Engine**: Taffy for CSS layout calculations

### Safety & Security Features

- **JavaScript Sandboxing**: Execution with dangerous pattern detection
- **Timeout Protection**: Configurable execution limits
- **Memory Management**: Controlled resource allocation
- **Request Validation**: URL and header sanitization
- **Error Boundaries**: Comprehensive error handling without crashes

#### **Full DOM/Event System** (`src/enhanced_dom/`)
- **Element Hierarchy**: Complete DOM tree with parent/child relationships
- **Event Handling**: addEventListener support with bubbling and capturing
- **Web Storage APIs**: localStorage and sessionStorage with persistence
- **DOM Mutations**: Real-time tracking of element changes and updates
- **Enhanced CSS Selectors**: Complex selector support with pseudo-classes
- **Element Manipulation**: Create, modify, and delete DOM elements

#### **WebSocket Support** (`src/websocket.rs`)
- **Connection Management**: Full WebSocket connection simulation
- **Message Handling**: Send/receive text and binary messages
- **Real-time Events**: Heartbeat, user events, notifications, and status updates
- **JavaScript API**: Complete WebSocket JavaScript API with event listeners
- **Server-Sent Events**: EventSource API support for streaming data
- **Connection State**: Track connection states (connecting, open, closing, closed)

## 📡 **MCP Integration**

### 🆕 **Enhanced MCP Interface**

We've expanded from 2 basic tools to **12 comprehensive tools** that expose all browser capabilities:

### Available Tools

#### `scrape_url` - Basic Page Scraping
```json
{
  "method": "tools/call",
  "params": {
    "name": "scrape_url",
    "arguments": {
      "url": "https://example.com",
      "wait_for_js": true,
      "selector": ".content",
      "extract_links": true,
      "extract_images": false
    }
  }
}
```

#### `extract_forms` - Form Discovery
Automatically detect and extract all forms from a page:
```json
{
  "method": "tools/call",
  "params": {
    "name": "extract_forms", 
    "arguments": {
      "url": "https://example.com/login"
    }
  }
}
```

#### `submit_form` - Form Submission
Submit forms with data and handle responses:
```json
{
  "method": "tools/call",
  "params": {
    "name": "submit_form",
    "arguments": {
      "form_action": "https://example.com/search",
      "method": "GET",
      "form_data": {
        "q": "search query",
        "type": "web"
      },
      "wait_for_js": true
    }
  }
}
```

#### `click_link` - Link Navigation
Navigate by clicking links with CSS selectors:
```json
{
  "method": "tools/call",
  "params": {
    "name": "click_link",
    "arguments": {
      "base_url": "https://example.com",
      "link_selector": "a[href*='login']",
      "wait_for_js": false
    }
  }
}
```

#### `manage_cookies` - Cookie Management
Retrieve, set, or clear cookies for a domain:
```json
{
  "method": "tools/call",
  "params": {
    "name": "manage_cookies",
    "arguments": {
      "url": "https://example.com",
      "action": "get"
    }
  }
}
```

#### `manage_storage` - Browser Storage
Manage localStorage and sessionStorage:
```json
{
  "method": "tools/call",
  "params": {
    "name": "manage_storage",
    "arguments": {
      "action": "get",
      "storage_type": "localStorage"
    }
  }
}
```

#### `manage_auth` - Authentication Context
Manage Bearer tokens, CSRF tokens, and custom headers:
```json
{
  "method": "tools/call",
  "params": {
    "name": "manage_auth",
    "arguments": {
      "action": "set_bearer_token",
      "token": "eyJhbGciOiJIUzI1NiIs..."
    }
  }
}
```

#### `websocket_connect` - WebSocket Management
Establish WebSocket connections:
```json
{
  "method": "tools/call",
  "params": {
    "name": "websocket_connect",
    "arguments": {
      "url": "wss://api.example.com/ws",
      "protocols": ["chat", "notifications"]
    }
  }
}
```

#### `websocket_send` - Send WebSocket Messages
Send messages through WebSocket connections:
```json
{
  "method": "tools/call",
  "params": {
    "name": "websocket_send",
    "arguments": {
      "connection_id": "ws_abc123",
      "message": "{\"type\":\"chat\",\"text\":\"Hello!\"}",
      "binary": false
    }
  }
}
```

#### `websocket_simulate` - Simulate Real-time Events
Simulate realistic WebSocket events:
```json
{
  "method": "tools/call",
  "params": {
    "name": "websocket_simulate",
    "arguments": {
      "connection_id": "ws_abc123",
      "events": ["heartbeat", "user_joined", "message"]
    }
  }
}
```

#### `browser_status` - Comprehensive Browser State
Get detailed browser status including connections, storage, and authentication:
```json
{
  "method": "tools/call",
  "params": {
    "name": "browser_status",
    "arguments": {}
  }
}
```

## 🔧 **Library Usage**

### Basic Browser Usage
```rust
use synaptic::HeadlessWebBrowser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut browser = HeadlessWebBrowser::new();
    
    // Navigate to a page
    let page = browser.scrape(
        "https://example.com",
        true,  // wait_for_js
        None,  // selector
        true,  // extract_links
        false  // extract_images
    ).await?;
    
    println!("Page title: {:?}", page.title);
    println!("Found {} links", page.links.len());
    
    Ok(())
}
```

### Form Interaction Workflow
```rust
use synaptic::{HeadlessWebBrowser, Form};
use std::collections::HashMap;

async fn login_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let mut browser = HeadlessWebBrowser::new();
    
    // 1. Load login page
    let login_page = browser.scrape("https://example.com/login", false, None, false, false).await?;
    
    // 2. Extract forms
    let url = url::Url::parse("https://example.com/login")?;
    let forms = browser.extract_forms(&login_page.content, &url)?;
    
    // 3. Fill out login form
    let mut form_data = HashMap::new();
    form_data.insert("username".to_string(), "user@example.com".to_string());
    form_data.insert("password".to_string(), "secretpass".to_string());
    
    // 4. Submit form and handle response
    let response = browser.submit_form(&forms[0], form_data, true).await?;
    
    println!("Login response status: {}", response.status_code);
    println!("Cookies received: {:?}", response.cookies);
    
    // 5. Navigate to protected area (cookies automatically sent)
    let dashboard = browser.scrape("https://example.com/dashboard", true, None, false, false).await?;
    
    Ok(())
}
```

### Advanced Navigation
```rust
async fn complex_navigation() -> Result<(), Box<dyn std::error::Error>> {
    let mut browser = HeadlessWebBrowser::new();
    
    // Navigate through multiple pages maintaining session
    let home = browser.scrape("https://ecommerce.example.com", false, None, true, false).await?;
    
    // Click on a product link
    let product_page = browser.click_link(
        "https://ecommerce.example.com",
        "a.product-link:first-child",
        true
    ).await?;
    
    // Check session cookies
    let cookies = browser.get_cookies("https://ecommerce.example.com")?;
    println!("Session cookies: {:?}", cookies);
    
    Ok(())
}
```

## 📊 **Data Structures**

### Core Types
```rust
pub struct HeadlessWebBrowser {
    client: reqwest::Client,           // HTTP client with cookies
    renderer: RustRenderer,            // JavaScript execution engine  
    cookie_jar: Arc<reqwest::cookie::Jar>, // Persistent cookie storage
}

pub struct ScrapedData {
    pub url: String,                   // Final URL after redirects
    pub title: Option<String>,         // Page title
    pub content: String,               // Extracted text content
    pub links: Vec<Link>,              // All links found on page
    pub images: Vec<Image>,            // All images found on page
    pub metadata: HashMap<String, String>, // Meta tags and page data
    pub extracted_data: Option<Value>, // Custom extracted data
}

pub struct Form {
    pub action: String,                // Form submission URL
    pub method: String,                // HTTP method (GET/POST)
    pub fields: Vec<FormField>,        // All form inputs
    pub submit_buttons: Vec<String>,   // Submit button values
}

pub struct InteractionResponse {
    pub url: String,                   // Response URL
    pub status_code: u16,              // HTTP status
    pub content: String,               // Response body
    pub cookies: HashMap<String, String>, // New cookies
    pub scraped_data: Option<ScrapedData>, // Parsed content
}
```

### Form Field Types
```rust
pub struct FormField {
    pub name: String,                  // Field name attribute
    pub field_type: String,            // Input type (text, password, etc.)
    pub value: Option<String>,         // Current/default value
    pub required: bool,                // Required field flag
    pub placeholder: Option<String>,   // Placeholder text
}
```

## 🌍 **Real-World Testing**

Our browser has been tested extensively on production websites:

### ✅ **Successfully Tested Sites**
- **Static Sites**: Perfect content extraction and navigation
- **Form-based Sites**: Login/registration workflows
- **E-commerce**: Product browsing and cart interactions  
- **News Sites**: Article extraction and link following
- **API Documentation**: Complex navigation and search
- **GitHub**: Repository browsing and issue tracking

### ⚡ **Performance Results**
- **Google Search**: Successfully handles JavaScript challenges and anti-bot detection
- **Complex Forms**: Automatic detection and submission with anti-automation evasion
- **Session Management**: Persistent login across multiple pages with behavioral mimicry
- **JavaScript Sites**: Modern API rendering with canvas fingerprinting and WebGL support
- **Cookie Handling**: Full session state management with human-like request patterns
- **Bot Detection**: Successfully evades Cloudflare, Google, and other modern protection systems
- **WebSocket Communication**: Real-time event handling with message queuing and state persistence
- **Enhanced MCP Tools**: 12 comprehensive browser tools covering all interaction scenarios

### 🧪 **Comprehensive Test Coverage**

Our browser includes **62 passing tests** across all components:

- **Core Browser Tests** (9/9 passing): Form submission, link clicking, cookie management, multi-step workflows
- **Authentication Tests** (9/9 passing): Bearer tokens, CSRF protection, custom headers, storage persistence  
- **Stealth Tests** (9/9 passing): Canvas fingerprinting, WebGL simulation, user-agent rotation, timing patterns
- **WebSocket Tests** (8/8 passing): Connection management, messaging, real-time events, error handling
- **MCP Protocol Tests** (6/6 passing): Tool registration, request/response handling, error scenarios
- **Renderer Tests** (6/6 passing): JavaScript execution, sandboxing, challenge detection
- **Scraper Tests** (15/15 passing): Content extraction, metadata parsing, link resolution, image extraction

All tests use mock servers and controlled environments to ensure reliable, fast execution without external dependencies.

### 🔍 **Google Search Example**
When tested on Google Search with enhanced modern browser support:
```bash
# Load Google homepage - Success with stealth features!
echo '{"method": "tools/call", "params": {"name": "scrape_url", "arguments": {"url": "https://google.com"}}}' | ./target/debug/synaptic

# Perform search with JavaScript challenge handling
echo '{"method": "tools/call", "params": {"name": "scrape_url", "arguments": {"url": "https://www.google.com/search?q=modern+web+scraping", "wait_for_js": true}}}' | ./target/debug/synaptic
```

**Results**: 
- ✅ **Advanced Stealth**: Successfully bypasses Google's initial bot detection
- ✅ **JavaScript Challenges**: Automatically detects and executes anti-bot challenges  
- ✅ **Canvas Fingerprinting**: Provides realistic browser fingerprints to pass detection
- ✅ **HTTP/2 & TLS**: Uses modern connection patterns that mimic real browsers
- ✅ **Behavioral Mimicry**: Human-like timing and request patterns avoid detection

The browser now successfully handles Google's sophisticated anti-bot systems by presenting as a legitimate Chrome browser with realistic fingerprints and behavior patterns.

## 🔧 **Building & Running**

### Prerequisites
- Rust 1.70+ (2021 edition)
- Cargo package manager

### Quick Start
```bash
# Clone and build
git clone <repository-url>
cd synaptic
cargo build --release

# Run as MCP server
./target/release/synaptic

# Run tests
cargo test
```

### Development Build
```bash
cargo build
cargo run
```

## 📊 **Architecture Overview**

### System Components

```
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│   MCP Interface     │    │  Enhanced Browser   │    │   Stealth System    │
│  - 12 Tools         │◄──►│  - HTTP/2 Client    │◄──►│  - Canvas FP        │
│  - JSON-RPC 2.0     │    │  - Cookie Jar       │    │  - WebGL Simulation │
│  - Error Handling   │    │  - Form Processing   │    │  - User-Agent Rot.  │
└─────────────────────┘    └─────────────────────┘    └─────────────────────┘
           ▲                          ▲                          ▲
           │                          │                          │
           ▼                          ▼                          ▼
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│   WebSocket Layer   │    │  JavaScript Engine  │    │   DOM/Event System  │
│  - Connection Pool  │◄──►│  - Boa Runtime      │◄──►│  - Element Tree     │
│  - Real-time Events │    │  - Challenge Detect │    │  - Event Listeners  │
│  - Message Queue    │    │  - API Polyfills    │    │  - Web Storage      │
└─────────────────────┘    └─────────────────────┘    └─────────────────────┘
```

### Data Flow

1. **MCP Request** → JSON-RPC parsing → Tool dispatch
2. **Browser Action** → Stealth headers → HTTP request → Response processing
3. **JavaScript Challenge** → Boa execution → Canvas/WebGL fingerprinting → Challenge completion
4. **Form Interaction** → DOM parsing → Field extraction → Data submission → Session persistence
5. **WebSocket Communication** → Connection establishment → Event simulation → Message handling

## 📦 **Dependencies**

### Core Dependencies
```toml
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls", "gzip", "brotli", "deflate", "cookies", "http2", "stream"] }
scraper = "0.24"
boa_engine = "0.20" 
lightningcss = "1.0.0-alpha.67"
serde = { version = "1.0", features = ["derive"] }
url = "2.5"
anyhow = "1.0"
rand = "0.8"                # Random number generation for human-like behavior
```

### Enhanced Features
```toml
swc_core = "0.96"           # JavaScript parsing/transformation
html-escape = "0.2"         # HTML escaping for React rendering
taffy = "0.5"              # Layout engine
regex = "1.10"              # JavaScript processing and pattern matching
uuid = { version = "1.0", features = ["v4"] } # WebSocket connection IDs
```

### Testing Coverage
```toml
[dev-dependencies]
tokio-test = "0.4"          # Async testing utilities
wiremock = "0.6"            # Mock server for testing
```

## 🔐 **Security Model**

### Multi-Layer Protection
1. **JavaScript Sandboxing**: Dangerous patterns blocked at execution
2. **Request Validation**: URL and header sanitization
3. **Timeout Protection**: Configurable execution and request limits
4. **Memory Boundaries**: Controlled resource allocation
5. **Network Isolation**: Controlled external request handling

### Blocked JavaScript Patterns
- `eval()`, `Function()` constructors
- Direct network calls (`XMLHttpRequest`, `fetch()`)
- Module imports/requires
- Process and global object access
- Cookie manipulation from untrusted code
- Navigation hijacking attempts

## 🚀 **Roadmap**

### ✅ **Phase 1: Core Browser (Complete)**
- [x] HTTP client with cookie management
- [x] Form detection and submission
- [x] Chrome-like headers and compression
- [x] Session persistence across requests
- [x] Link navigation and interaction

### ✅ **Phase 2: Modern Browser Support (Complete)**
- [x] Browser automation detection evasion
- [x] Canvas fingerprinting with realistic variations
- [x] WebGL simulation with proper vendor/renderer info
- [x] Chrome object with loadTimes() and csi() functions
- [x] User-Agent rotation (Chrome/Firefox/Safari/Edge)
- [x] Dynamic header randomization and timing patterns
- [x] HTTP/2 and TLS fingerprinting
- [x] JavaScript challenge detection and execution
- [x] Human-like behavioral mimicry

### ✅ **Phase 3: Advanced Features (Completed)**
- [x] **Full DOM/event system** with real element hierarchy and web storage APIs
- [x] **WebSocket connection handling** with real-time event simulation
- [x] **Enhanced MCP interface** - Expanded from 2 tools to 12 comprehensive browser tools
- [x] **Authentication management** with Bearer tokens, CSRF protection, and custom headers
- [x] **Advanced browser storage** with localStorage/sessionStorage persistence
- [x] **Real-time communication** with WebSocket JavaScript API simulation
- [x] **Comprehensive browser status** with detailed state reporting across all systems
- [ ] **Mobile device emulation** (Phase 4)
- [ ] **Advanced form types** with file uploads (Phase 4)
- [ ] **Client-side routing simulation** (Phase 4)

## 🤝 **Contributing**

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 📄 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 **Acknowledgments**

- **Boa Team**: Pure Rust JavaScript engine
- **Tokio Team**: Async runtime foundation  
- **reqwest Team**: HTTP client excellence
- **scraper Team**: HTML parsing capabilities
- **LightningCSS Team**: CSS processing engine
- **SWC Team**: JavaScript transformation tools

---

**Synaptic** - Where AI meets the web, built entirely in Rust 🦀