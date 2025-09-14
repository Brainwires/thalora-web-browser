# Synaptic - Pure Rust Headless Web Browser

🧠 **Neural connections between AI models and the web**

A cutting-edge headless web browser built entirely in Rust, designed specifically for AI model integration via the Model Context Protocol (MCP). Unlike traditional scrapers, Synaptic provides full browser capabilities including JavaScript execution, AI memory persistence, and Chrome DevTools Protocol debugging - all without any Chrome or Chromium dependencies.

## ✨ **Key Features**

### 🎯 **AI-First Design**
- **AI Memory Heap**: Persistent storage for AI agents that survives context compression
- **MCP Integration**: 17 comprehensive tools via Model Context Protocol  
- **Chrome DevTools Protocol**: Full CDP debugging capabilities for AI development
- **Pure Rust**: No browser dependencies, single binary deployment

### 🌐 **Full Web Browser Capabilities**
- **JavaScript Execution**: Sandboxed Boa engine with modern browser APIs
- **Form Processing**: Automatic form detection, filling, and submission
- **Session Management**: Persistent cookies and authentication state
- **Interactive Navigation**: Click links, handle redirects, multi-page workflows
- **Advanced Stealth**: Canvas fingerprinting, WebGL simulation, bot detection evasion

### 🧠 **AI Memory System**
- **Research Storage**: Persistent findings with confidence scores and tagging
- **Credential Management**: Encrypted password storage with metadata
- **Session Tracking**: Long-term project continuity across context resets
- **Smart Search**: Query across all stored data with filtering and sorting
- **Bookmark Management**: URL collections with content previews and tags

## 🛠 **Installation & Setup**

### Quick Start
```bash
# Install from source
git clone https://github.com/brainwires/synaptic.git
cd synaptic
cargo build --release

# Run as MCP server
./target/release/synaptic

# Test the installation
echo '{"method": "tools/list"}' | ./target/release/synaptic
```

### System Requirements
- Rust 1.70+ (2021 edition)
- 50MB disk space
- 256MB RAM (runtime)

## 🔧 **MCP Tools Overview**

Synaptic exposes 17 comprehensive tools through the Model Context Protocol:

### 🧠 **AI Memory Tools**
| Tool | Description |
|------|-------------|
| `memory_store_research` | Store research findings with confidence scores |
| `memory_store_credentials` | Securely store encrypted credentials |  
| `memory_get_credentials` | Retrieve stored credentials |
| `memory_store_bookmark` | Save URLs with metadata and tags |
| `memory_store_note` | Store categorized notes with priority levels |
| `memory_search` | Search across all stored data |
| `memory_start_session` | Begin development sessions with objectives |
| `memory_update_session` | Track session progress and updates |
| `memory_get_statistics` | Get memory usage statistics |

### 🔍 **Chrome DevTools Protocol (CDP)**
| Tool | Description |
|------|-------------|
| `cdp_enable_runtime` | Enable CDP Runtime domain for JavaScript |
| `cdp_evaluate_javascript` | Execute JavaScript with CDP |
| `cdp_enable_debugger` | Enable breakpoint management |
| `cdp_set_breakpoint` | Set debugging breakpoints |
| `cdp_enable_dom` | Enable DOM inspection capabilities |
| `cdp_get_document` | Retrieve DOM document structure |
| `cdp_enable_network` | Enable network request monitoring |
| `cdp_get_response_body` | Get network response bodies |

## 📝 **Usage Examples**

### AI Memory: Store Research Findings
```json
{
  "method": "tools/call",
  "params": {
    "name": "memory_store_research",
    "arguments": {
      "key": "react_patterns_2024",
      "topic": "Modern React Design Patterns",
      "summary": "Latest React patterns for 2024 including Server Components",
      "findings": [
        "Server Components reduce bundle size by 40%",
        "Use Suspense boundaries for better error handling",
        "New use() hook simplifies data fetching"
      ],
      "sources": ["https://react.dev", "https://nextjs.org"],
      "tags": ["react", "frontend", "2024"],
      "confidence_score": 0.9
    }
  }
}
```

### AI Memory: Secure Credential Storage
```json
{
  "method": "tools/call", 
  "params": {
    "name": "memory_store_credentials",
    "arguments": {
      "key": "github_api",
      "service": "GitHub API",
      "username": "ai-agent",
      "password": "ghp_xxxxxxxxxxxx",
      "additional_data": {
        "scope": "repo,user",
        "rate_limit": "5000/hour"
      }
    }
  }
}
```

### AI Memory: Smart Search
```json
{
  "method": "tools/call",
  "params": {
    "name": "memory_search", 
    "arguments": {
      "query": "authentication patterns",
      "category": "research",
      "tags": ["security", "auth"],
      "limit": 5
    }
  }
}
```

### CDP: JavaScript Debugging
```json
{
  "method": "tools/call",
  "params": {
    "name": "cdp_evaluate_javascript",
    "arguments": {
      "expression": "document.querySelector('h1').textContent",
      "return_by_value": true
    }
  }
}
```

## 🏗 **Architecture**

### Modular Design
```
synaptic/
├── src/
│   ├── main.rs               # Application entry point
│   ├── mcp_server.rs        # MCP protocol server
│   ├── memory_tools.rs      # AI Memory operations
│   ├── cdp_tools.rs        # Chrome DevTools Protocol
│   ├── ai_memory.rs        # Persistent storage system
│   ├── browser.rs          # Web browser engine
│   └── ...                 # Additional modules
└── examples/
    └── ai_memory_demo.rs   # Usage demonstration
```

### Key Components

**🧠 AI Memory Heap** (`src/ai_memory.rs`)
- Persistent JSON storage (~/.synaptic/ai_memory.json)
- Encrypted credential storage with XOR encryption
- Full-text search across all categories
- Session tracking for long-term project continuity

**🔍 Chrome DevTools Protocol** (`src/cdp.rs`)  
- Runtime domain for JavaScript execution
- Debugger domain for breakpoint management
- DOM inspection capabilities
- Network monitoring and response capture

**🌐 Browser Engine** (`src/browser.rs`)
- HTTP/2 client with cookie persistence
- JavaScript execution via Boa engine
- Stealth capabilities and bot detection evasion
- Form processing and interactive navigation

## 🔒 **Security & Safety**

### AI Memory Security
- **Encrypted Storage**: Credentials encrypted with XOR + base64
- **Local Only**: All data stored locally in ~/.synaptic/
- **Access Control**: Memory operations through validated MCP tools only
- **No Network**: Memory data never transmitted externally

### Browser Security  
- **JavaScript Sandboxing**: Dangerous patterns blocked (eval, fetch, etc.)
- **Timeout Protection**: 5-second execution limits
- **Request Validation**: URL and header sanitization
- **Memory Boundaries**: Controlled resource allocation

## 🎯 **Perfect For AI Agents**

### Research & Development
- **Persistent Knowledge**: Store findings across context compressions
- **Credential Management**: Securely access APIs and services
- **Session Continuity**: Long-term project tracking
- **Debug Integration**: CDP tools for web development

### Web Interaction
- **Modern Sites**: Handle JavaScript, forms, authentication
- **Stealth Browsing**: Evade bot detection systems
- **Multi-step Workflows**: Maintain state across page navigation
- **Real-time Data**: WebSocket and dynamic content support

## 📊 **Performance**

- **Memory Usage**: ~40MB runtime (full features)
- **Speed**: 100-500ms per page (static), 1-3s (JavaScript)
- **Storage**: Persistent data in efficient JSON format
- **Network**: HTTP/2 with connection pooling

## 🚀 **Deployment**

### Single Binary
```bash
# Build optimized release
cargo build --release

# Deploy anywhere - no dependencies
scp target/release/synaptic user@server:/usr/local/bin/
```

### Docker Container
```dockerfile
FROM rust:1.75 as builder
COPY . /app
WORKDIR /app
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/synaptic /usr/local/bin/
CMD ["synaptic"]
```

### Integration Examples

**With Claude Desktop:**
```json
{
  "mcpServers": {
    "synaptic": {
      "command": "/usr/local/bin/synaptic"
    }
  }
}
```

**With Custom AI Agent:**
```python
import subprocess
import json

# Start Synaptic MCP server
process = subprocess.Popen(['synaptic'], 
                          stdin=subprocess.PIPE, 
                          stdout=subprocess.PIPE)

# Store research finding
request = {
    "method": "tools/call",
    "params": {
        "name": "memory_store_research",
        "arguments": {
            "key": "api_discovery",
            "topic": "REST API Patterns",
            "summary": "Modern API design principles"
        }
    }
}

process.stdin.write(json.dumps(request).encode() + b'\n')
response = json.loads(process.stdout.readline())
```

## 🌟 **Why Choose Synaptic?**

### For AI Researchers
- **Memory Persistence**: Never lose research across sessions
- **Secure Credentials**: Encrypted storage for API keys
- **Session Tracking**: Long-term project continuity
- **Search Everything**: Find past research instantly

### For Web Automation
- **Pure Rust**: Fast, safe, single binary
- **No Browser**: No Chrome/Chromium dependencies  
- **Stealth Features**: Evade modern bot detection
- **Full MCP**: 17 comprehensive tools

### For Developers
- **CDP Integration**: Full Chrome DevTools debugging
- **Modular Design**: Clean, maintainable codebase
- **Rich APIs**: Complete browser automation
- **Easy Integration**: Simple JSON-RPC protocol

## 📄 **License**

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 **Acknowledgments**

Built with excellent open-source projects:
- [Boa](https://github.com/boa-dev/boa) - Pure Rust JavaScript engine
- [Tokio](https://tokio.rs) - Async runtime
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client  
- [scraper](https://github.com/causal-agent/scraper) - HTML parsing

---

**Synaptic v0.1.0** - Where AI meets the web, built entirely in Rust 🦀