# Brainwires Scraper

A pure Rust web scraper that works as both a standalone library and an MCP (Model Context Protocol) tool. Features a complete Rust-based HTML/CSS/JavaScript rendering engine without any Chrome or Chromium dependencies.

## 🎯 **Current Status & Capabilities**

### ✅ **Production Ready**
- **Pure Rust Implementation**: No external browser dependencies
- **MCP Protocol Support**: JSON/STDIO communication for AI model integration
- **Safe JavaScript Execution**: Sandboxed Boa engine with security patterns
- **Advanced CSS Processing**: Full CSS parsing and layout calculations with LightningCSS
- **Intelligent Data Extraction**: CSS selector-based structured data extraction
- **Dual Architecture**: Works as both library and MCP tool
- **Comprehensive Testing**: 23 passing tests across all components

### 🚀 **Enhanced Features (In Development)**

We've analyzed modern React/Next.js applications and designed enhanced modules for better scraping of dynamic content:

#### **Enhanced JavaScript Engine** (`src/enhanced_js.rs`)
- **Real Timer Functions**: setTimeout/setInterval with tokio async support
- **Enhanced Console**: Multi-level logging (log, error, warn)
- **Fetch API**: Network requests with reqwest backend
- **Performance APIs**: performance.now() with accurate timing
- **Promise Support**: Async/await handling for modern JS patterns
- **Module System**: ES6 import/export parsing with SWC
- **Modern JS Transform**: Arrow functions, template literals, let/const to var

#### **Advanced DOM Implementation** (`src/enhanced_dom.rs`)
- **Real DOM Tree**: Complete element hierarchy with event handling
- **Event System**: addEventListener with bubbling/capturing support
- **Web Storage**: localStorage/sessionStorage with persistence
- **Mutation Detection**: DOM change tracking and application
- **Enhanced Selectors**: Full CSS4 selector support
- **Attribute Management**: Dynamic attribute updates

#### **React/Next.js Processor** (`src/react_processor.rs`)
- **Next.js Streaming Parser**: Processes `__next_f.push()` streaming data
- **Server Component Handling**: Reconstructs React element trees
- **Hydration Support**: Processes React hydration data
- **Component Rendering**: Converts React elements to HTML
- **Metadata Extraction**: Pulls React metadata and navigation data

## Features

## Architecture

### Core Components

- **Boa Engine**: Pure Rust JavaScript execution engine
- **Scraper**: HTML parsing with CSS selector support
- **LightningCSS**: Pure Rust CSS parsing and processing
- **Taffy**: Rust-based CSS layout engine
- **Reqwest**: HTTP client with TLS support

### Safety Features

- JavaScript execution sandboxing
- Timeout protection for JS execution
- Safe DOM manipulation APIs
- Input validation and sanitization

## Usage

### As an MCP Tool

The scraper can be used as an MCP tool for AI models like Claude:

```bash
cargo build --release
./target/release/brainwires-scraper
```

The tool provides two main functions via MCP:

#### `scrape_url`
Scrapes content from a URL with optional JavaScript execution.

**Parameters:**
- `url` (required): The URL to scrape
- `wait_for_js` (optional, default: true): Execute JavaScript and wait for dynamic content
- `selector` (optional): CSS selector to extract specific elements
- `extract_links` (optional, default: false): Extract all links from the page
- `extract_images` (optional, default: false): Extract all images from the page

**Example:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "scrape_url",
    "arguments": {
      "url": "https://example.com",
      "wait_for_js": true,
      "selector": ".content",
      "extract_links": true
    }
  }
}
```

#### `extract_data`
Extracts structured data from HTML using CSS selectors.

**Parameters:**
- `html` (required): The HTML content to parse
- `selectors` (required): Object mapping field names to CSS selectors

**Example:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "extract_data",
    "arguments": {
      "html": "<html>...</html>",
      "selectors": {
        "title": "h1",
        "links": "a[href]",
        "prices": ".price"
      }
    }
  }
}
```

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
brainwires-scraper = { path = "path/to/brainwires-scraper" }
```

```rust
use brainwires_scraper::WebScraper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut scraper = WebScraper::new();
    
    let result = scraper.scrape(
        "https://example.com",
        true,  // wait_for_js
        Some(".content"), // selector
        true,  // extract_links
        false  // extract_images
    ).await?;
    
    println!("Title: {:?}", result.title);
    println!("Content: {}", result.content);
    println!("Links found: {}", result.links.len());
    
    Ok(())
}
```

## Data Structures

### ScrapedData
```rust
pub struct ScrapedData {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub links: Vec<Link>,
    pub images: Vec<Image>,
    pub metadata: HashMap<String, String>,
    pub extracted_data: Option<Value>,
}
```

### Link
```rust
pub struct Link {
    pub url: String,
    pub text: String,
    pub title: Option<String>,
}
```

### Image
```rust
pub struct Image {
    pub src: String,
    pub alt: Option<String>,
    pub title: Option<String>,
}
```

## JavaScript Safety

The JavaScript execution engine includes several safety measures:

- **Sandboxed Environment**: Limited global objects and APIs
- **Pattern Detection**: Blocks potentially dangerous JavaScript patterns
- **Timeout Protection**: 5-second execution limit
- **Size Limits**: Code length restrictions
- **Safe DOM APIs**: Simplified, secure DOM manipulation

### Blocked Patterns
- `eval()`, `Function()` constructors
- `XMLHttpRequest`, `fetch()` calls
- Module imports/requires
- Process and global object access
- Cookie and storage access
- Navigation manipulation
- Alert/prompt dialogs

## Building

### Prerequisites
- Rust 1.70+ (2021 edition)
- Cargo

### Development Build
```bash
cargo build
```

### Release Build
```bash
cargo build --release
```

### Running Tests
```bash
cargo test
```

## Dependencies

- **tokio**: Async runtime
- **reqwest**: HTTP client with rustls-tls
- **scraper**: HTML parsing and CSS selectors
- **boa_engine**: Pure Rust JavaScript engine
- **lightningcss**: CSS parsing and processing
- **taffy**: CSS layout calculations
- **serde/serde_json**: Serialization
- **anyhow/thiserror**: Error handling
- **tracing**: Logging
- **regex**: Pattern matching

## MCP Integration

This tool implements the Model Context Protocol (MCP) for seamless integration with AI models. The MCP server:

- Listens on STDIO for JSON-RPC messages
- Provides tool discovery via `tools/list`
- Executes tools via `tools/call`
- Returns structured responses

### MCP Capabilities

```json
{
  "protocol_version": "2024-11-05",
  "capabilities": {
    "tools": {}
  },
  "server_info": {
    "name": "brainwires-scraper",
    "version": "0.1.0"
  }
}
```

## Examples

### Basic Web Scraping
```rust
let mut scraper = WebScraper::new();
let result = scraper.scrape("https://news.ycombinator.com", true, None, true, false).await?;
println!("Found {} links", result.links.len());
```

### Data Extraction
```rust
let html = "<div class='product'><h2>Widget</h2><span class='price'>$19.99</span></div>";
let selectors = serde_json::json!({
    "name": ".product h2",
    "price": ".price"
});

let result = scraper.extract_data(html, selectors.as_object().unwrap()).await?;
```

### JavaScript-Heavy Sites
```rust
let result = scraper.scrape("https://spa-example.com", true, "#dynamic-content", false, false).await?;
// Content will include JavaScript-rendered elements
```

## Performance

- **Parallel Processing**: Concurrent request handling
- **Memory Efficient**: Streaming HTML parsing
- **Fast CSS**: Optimized CSS parsing with LightningCSS
- **Minimal Allocations**: Efficient string handling

## Security Considerations

- JavaScript execution is sandboxed but not fully isolated
- Network requests respect robots.txt (implementation dependent)
- No automatic cookie/session handling
- User-Agent identification as "Brainwires-Scraper/1.0"

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 🔧 **Development Roadmap**

### **Phase 1: Enhanced JavaScript Foundation** ⚠️ *In Progress*
- [x] Design enhanced JavaScript engine with real timers
- [x] Implement Promise support and async execution  
- [x] Add fetch API with network backend
- [x] Create module system with ES6 support
- [ ] Fix Boa engine API compatibility issues
- [ ] Complete SWC integration for JS transforms

### **Phase 2: Advanced DOM & React Support** 📋 *Planned*
- [x] Design real DOM implementation with events
- [x] Create React/Next.js streaming parser
- [x] Implement component hydration logic
- [ ] Complete DOM event system
- [ ] Integrate React processor with main engine
- [ ] Add mutation observer functionality

### **Phase 3: Production Integration** 🎯 *Future*
- [ ] Enable enhanced modules in main engine
- [ ] Performance optimization and memory management  
- [ ] Comprehensive testing of React applications
- [ ] Documentation and examples for enhanced features

## 📊 **Test Results**

The scraper has been tested on production applications including:

- **✅ Static Sites**: Perfect extraction of content and metadata
- **✅ Server-Rendered Apps**: Full content capture with SEO data
- **⚠️ React/Next.js Apps**: Basic content extracted, enhanced modules designed for full support
- **✅ Complex Forms**: Data extraction and link following
- **✅ E-commerce Sites**: Product data and pricing extraction

### **Example: Brainwires Studio Test**
When tested on `https://brainwires.studio` (a complex Next.js app), the scraper:
- ✅ Extracted title: "Brainwires Studio"
- ✅ Captured navigation: "Home About Chat Blogs Sign In"
- ✅ Retrieved main content: "Welcome to Brainwires Studio"
- ⚠️ Next.js streaming data captured but not fully processed (enhanced modules address this)

## 🛠️ **Technical Implementation Status**

| Component | Status | Description |
|-----------|--------|-------------|
| **Core Scraper** | ✅ Production | Basic HTML/CSS scraping with safety |
| **MCP Integration** | ✅ Production | Full JSON/STDIO protocol support |
| **JavaScript Safety** | ✅ Production | Sandboxed execution with pattern blocking |
| **Enhanced JS Engine** | 🚧 Development | Advanced timing, promises, modules |
| **Advanced DOM** | 🚧 Development | Event handling, storage, mutations |
| **React Processor** | 🚧 Development | Next.js streaming, hydration support |

## 🔐 **Security Model**

Our multi-layered security approach:

1. **JavaScript Sandboxing**: Dangerous patterns blocked at parse time
2. **Execution Limits**: 5-second timeouts prevent infinite loops
3. **Memory Boundaries**: Code size limits prevent resource exhaustion  
4. **Network Isolation**: No external requests from sandboxed JS
5. **API Restriction**: Limited global objects and DOM access

## Roadmap