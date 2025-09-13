# Brainwires Scraper

A pure Rust web scraper that works as both a standalone library and an MCP (Model Context Protocol) tool. Features a complete Rust-based HTML/CSS/JavaScript rendering engine without any Chrome or Chromium dependencies.

## Features

- **Pure Rust Implementation**: No external browser dependencies
- **MCP Protocol Support**: JSON/STDIO communication for AI model integration
- **JavaScript Execution**: Safe JavaScript execution using Boa engine
- **CSS Processing**: Full CSS parsing and layout calculations
- **Flexible Data Extraction**: CSS selector-based data extraction
- **Dual Purpose**: Works as both library and MCP tool

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

## Roadmap

- [ ] Enhanced JavaScript API support
- [ ] CSS media query processing
- [ ] WebAssembly module support
- [ ] Custom user agent configuration
- [ ] Proxy support
- [ ] Rate limiting
- [ ] Caching mechanisms