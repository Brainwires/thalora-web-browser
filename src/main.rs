use anyhow::Result;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tracing::{error, info};

mod mcp;
mod scraper;
mod renderer;

use mcp::{McpRequest, McpResponse};
use scraper::WebScraper;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let mut server = McpServer::new();
    server.run().await
}

struct McpServer {
    scraper: WebScraper,
}

impl McpServer {
    fn new() -> Self {
        Self {
            scraper: WebScraper::new(),
        }
    }

    async fn run(&mut self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut reader = AsyncBufReader::new(stdin);
        let mut stdout = tokio::io::stdout();
        let mut line = String::new();

        info!("MCP Web Scraper starting...");

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if let Ok(request) = serde_json::from_str::<McpRequest>(&line.trim()) {
                        let response = self.handle_request(request).await;
                        let response_json = serde_json::to_string(&response)?;
                        stdout.write_all(response_json.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                    }
                }
                Err(e) => {
                    error!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_request(&mut self, request: McpRequest) -> McpResponse {
        match request {
            McpRequest::ListTools => self.list_tools(),
            McpRequest::CallTool { params } => {
                self.call_tool(params.name, params.arguments).await
            }
            McpRequest::Initialize { .. } => McpResponse::Initialize {
                protocol_version: "2024-11-05".to_string(),
                capabilities: serde_json::json!({
                    "tools": {}
                }),
                server_info: serde_json::json!({
                    "name": "brainwires-scraper",
                    "version": "0.1.0"
                }),
            },
        }
    }

    fn list_tools(&self) -> McpResponse {
        McpResponse::ListTools {
            tools: vec![
                serde_json::json!({
                    "name": "scrape_url",
                    "description": "Scrape content from a URL using pure Rust HTML/CSS/JS rendering",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "The URL to scrape"
                            },
                            "wait_for_js": {
                                "type": "boolean",
                                "description": "Whether to execute JavaScript and wait for dynamic content",
                                "default": true
                            },
                            "selector": {
                                "type": "string",
                                "description": "Optional CSS selector to extract specific elements"
                            },
                            "extract_links": {
                                "type": "boolean", 
                                "description": "Whether to extract all links from the page",
                                "default": false
                            },
                            "extract_images": {
                                "type": "boolean",
                                "description": "Whether to extract all images from the page", 
                                "default": false
                            }
                        },
                        "required": ["url"]
                    }
                }),
                serde_json::json!({
                    "name": "extract_data",
                    "description": "Extract structured data from HTML using CSS selectors",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "html": {
                                "type": "string",
                                "description": "The HTML content to parse"
                            },
                            "selectors": {
                                "type": "object",
                                "description": "Object mapping field names to CSS selectors"
                            }
                        },
                        "required": ["html", "selectors"]
                    }
                })
            ]
        }
    }

    async fn call_tool(&mut self, name: String, arguments: Value) -> McpResponse {
        match name.as_str() {
            "scrape_url" => self.scrape_url(arguments).await,
            "extract_data" => self.extract_data(arguments).await,
            _ => McpResponse::Error {
                error: format!("Unknown tool: {}", name),
            },
        }
    }

    async fn scrape_url(&mut self, args: Value) -> McpResponse {
        let url = match args.get("url").and_then(|v| v.as_str()) {
            Some(url) => url,
            None => return McpResponse::Error {
                error: "Missing required parameter: url".to_string(),
            },
        };

        let wait_for_js = args.get("wait_for_js")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let selector = args.get("selector")
            .and_then(|v| v.as_str());

        let extract_links = args.get("extract_links")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let extract_images = args.get("extract_images")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        match self.scraper.scrape(url, wait_for_js, selector, extract_links, extract_images).await {
            Ok(result) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                })],
                is_error: false,
            },
            Err(e) => McpResponse::Error {
                error: format!("Scraping failed: {}", e),
            },
        }
    }

    async fn extract_data(&mut self, args: Value) -> McpResponse {
        let html = match args.get("html").and_then(|v| v.as_str()) {
            Some(html) => html,
            None => return McpResponse::Error {
                error: "Missing required parameter: html".to_string(),
            },
        };

        let selectors = match args.get("selectors").and_then(|v| v.as_object()) {
            Some(selectors) => selectors,
            None => return McpResponse::Error {
                error: "Missing required parameter: selectors".to_string(),
            },
        };

        match self.scraper.extract_data(html, selectors).await {
            Ok(result) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text", 
                    "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                })],
                is_error: false,
            },
            Err(e) => McpResponse::Error {
                error: format!("Data extraction failed: {}", e),
            },
        }
    }
}
