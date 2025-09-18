use serde_json::Value;
use crate::protocols::mcp::McpResponse;
use crate::protocols::mcp_server::core::McpServer;

impl McpServer {
    pub(super) fn get_tool_definitions(&self) -> Vec<Value> {
        vec![
            // AI Memory tools
            serde_json::json!({
                "name": "ai_memory_store_research",
                "description": "Store research data in AI memory for persistent access across sessions",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "key": {
                            "type": "string",
                            "description": "Unique identifier for the research data"
                        },
                        "data": {
                            "type": "object",
                            "description": "Research data to store (any JSON object)"
                        },
                        "tags": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Optional tags for categorization"
                        }
                    },
                    "required": ["key", "data"]
                }
            }),
            serde_json::json!({
                "name": "ai_memory_get_research",
                "description": "Retrieve research data from AI memory",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "key": {
                            "type": "string",
                            "description": "Unique identifier for the research data"
                        }
                    },
                    "required": ["key"]
                }
            }),
            serde_json::json!({
                "name": "ai_memory_search_research",
                "description": "Search research data in AI memory by tags or content",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "tags": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Tags to filter by"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of results (default: 10)"
                        }
                    }
                }
            }),
            // Chrome DevTools Protocol tools
            serde_json::json!({
                "name": "cdp_runtime_evaluate",
                "description": "Execute JavaScript in the browser context using Chrome DevTools Protocol",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "JavaScript expression to evaluate"
                        },
                        "await_promise": {
                            "type": "boolean",
                            "description": "Whether to await promise results"
                        }
                    },
                    "required": ["expression"]
                }
            }),
            serde_json::json!({
                "name": "cdp_dom_get_document",
                "description": "Get the DOM document structure using Chrome DevTools Protocol",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "depth": {
                            "type": "number",
                            "description": "Depth of DOM tree to retrieve (default: 2)"
                        }
                    }
                }
            }),
            // Web scraping tools
            serde_json::json!({
                "name": "scrape_url",
                "description": "Scrape a web page and extract content, links, images, and metadata",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "URL to scrape"
                        },
                        "wait_for_js": {
                            "type": "boolean",
                            "description": "Whether to wait for JavaScript execution",
                            "default": false
                        },
                        "session_id": {
                            "type": "string",
                            "description": "Browser session ID (optional)"
                        }
                    },
                    "required": ["url"]
                }
            }),
            // Google search tools
            serde_json::json!({
                "name": "google_search",
                "description": "Search Google and return organic results with title, URL, and snippet",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "num_results": {
                            "type": "number",
                            "description": "Number of results to return (default: 10, max: 20)"
                        }
                    },
                    "required": ["query"]
                }
            }),
            // Browser automation tools
            serde_json::json!({
                "name": "browser_click_element",
                "description": "Click on an element in the current page",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "selector": {
                            "type": "string",
                            "description": "CSS selector or link text to click"
                        },
                        "session_id": {
                            "type": "string",
                            "description": "Browser session ID (optional)"
                        }
                    },
                    "required": ["selector"]
                }
            }),
            serde_json::json!({
                "name": "browser_fill_form",
                "description": "Fill out and submit a form on the current page",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "form_data": {
                            "type": "object",
                            "description": "Key-value pairs of form field names and values"
                        },
                        "form_selector": {
                            "type": "string",
                            "description": "CSS selector for the form (default: 'form')"
                        },
                        "session_id": {
                            "type": "string",
                            "description": "Browser session ID (optional)"
                        }
                    },
                    "required": ["form_data"]
                }
            }),
        ]
    }

    pub(super) async fn call_tool(&mut self, name: String, arguments: Value) -> McpResponse {
        match name.as_str() {
            // AI Memory tools
            "ai_memory_store_research" => self.memory_tools.store_research(&mut self.ai_memory, arguments).await,
            "ai_memory_get_research" => self.memory_tools.get_research(&self.ai_memory, arguments).await,
            "ai_memory_search_research" => self.memory_tools.search_research(&self.ai_memory, arguments).await,
            "ai_memory_store_credentials" => self.memory_tools.store_credentials(&mut self.ai_memory, arguments).await,
            "ai_memory_get_credentials" => self.memory_tools.get_credentials(&self.ai_memory, arguments).await,
            "ai_memory_store_bookmark" => self.memory_tools.store_bookmark(&mut self.ai_memory, arguments).await,
            "ai_memory_get_bookmarks" => self.memory_tools.get_bookmarks(&self.ai_memory, arguments).await,
            "ai_memory_store_note" => self.memory_tools.store_note(&mut self.ai_memory, arguments).await,
            "ai_memory_get_notes" => self.memory_tools.get_notes(&self.ai_memory, arguments).await,

            // Chrome DevTools Protocol tools
            "cdp_runtime_evaluate" => self.cdp_tools.runtime_evaluate(arguments).await,
            "cdp_dom_get_document" => self.cdp_tools.dom_get_document(arguments).await,
            "cdp_dom_query_selector" => self.cdp_tools.dom_query_selector(arguments).await,
            "cdp_dom_get_attributes" => self.cdp_tools.dom_get_attributes(arguments).await,
            "cdp_network_get_cookies" => self.cdp_tools.network_get_cookies(arguments).await,
            "cdp_network_set_cookie" => self.cdp_tools.network_set_cookie(arguments).await,

            // Web scraping and navigation tools
            "scrape_url" => self.scrape_url(arguments).await,
            "google_search" => self.google_search(arguments).await,

            // Browser automation tools
            "browser_click_element" => self.browser_tools.handle_click_element(arguments).await,
            "browser_fill_form" => self.browser_tools.handle_fill_form(arguments).await,
            "browser_get_page_content" => self.browser_tools.handle_get_page_content(arguments).await,
            "browser_navigate_back" => self.browser_tools.handle_navigate_back(arguments).await,
            "browser_navigate_forward" => self.browser_tools.handle_navigate_forward(arguments).await,
            "browser_session_management" => self.browser_tools.handle_session_management(arguments).await,

            _ => McpResponse::error(-32601, format!("Tool not found: {}", name))
        }
    }
}