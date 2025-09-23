use serde_json::Value;
use crate::protocols::mcp::McpResponse;
use crate::protocols::mcp_server::core::McpServer;
use std::sync::Arc;
use vfs::{VfsInstance, set_current_vfs};
use std::env;

/*
    Define the available tools for the MCP server, including AI memory management,
    Chrome DevTools Protocol interactions, web scraping, browser automation, and session management.
    Each tool includes a name, description, and input schema for validation.

    Tools are grouped into categories:
        - AI Memory Tools: Store and retrieve research data, credentials, bookmarks, and notes.
        - Chrome DevTools Protocol (CDP) Tools: Execute JavaScript, inspect DOM, manage cookies, capture screenshots, and retrieve console messages.
        - Web Scraping Tools: Scrape web pages and extract content.
        - Web Search Tools: Perform web searches using various search engines.
        - Browser Automation Tools: Interact with web pages by clicking elements and filling forms.
        - Session Management Tools: Create, manage, and clean up browser sessions for persistent AI interactions.

    Each category needs a corresponding environment variable to be enabled, e.g.:
        - THALORA_ENABLE_AI_MEMORY
        - THALORA_ENABLE_CDP (also enables SESSIONS automatically)
        - THALORA_ENABLE_SCRAPING (enabled by default - minimal toolset)
        - THALORA_ENABLE_SEARCH
        - THALORA_ENABLE_SESSIONS (browser automation + session management)

*/

impl McpServer {
    /// Check if sessions are enabled (either directly or through CDP dependency)
    fn is_sessions_enabled() -> bool {
        env::var("THALORA_ENABLE_SESSIONS").unwrap_or_else(|_| "false".to_string()) == "true" ||
        env::var("THALORA_ENABLE_CDP").unwrap_or_else(|_| "false".to_string()) == "true"
    }

    pub(super) fn get_tool_definitions(&self) -> Vec<Value> {
        let mut tools = Vec::new();

        // AI Memory Tools - Store and retrieve research data, credentials, bookmarks, and notes
        if env::var("THALORA_ENABLE_AI_MEMORY").unwrap_or_else(|_| "false".to_string()) == "true" {
            tools.extend_from_slice(&[
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
                serde_json::json!({
                    "name": "ai_memory_store_credentials",
                    "description": "Securely store credentials (passwords, API keys, tokens) in encrypted AI memory",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "service": {
                                "type": "string",
                                "description": "Service name or identifier for the credentials"
                            },
                            "username": {
                                "type": "string",
                                "description": "Username or identifier"
                            },
                            "password": {
                                "type": "string",
                                "description": "Password or secret value"
                            },
                            "additional_data": {
                                "type": "object",
                                "description": "Additional credential data (API keys, tokens, etc.)"
                            },
                            "tags": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Optional tags for categorization"
                            }
                        },
                        "required": ["service", "username", "password"]
                    }
                }),
                serde_json::json!({
                    "name": "ai_memory_get_credentials",
                    "description": "Retrieve stored credentials from encrypted AI memory",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "service": {
                                "type": "string",
                                "description": "Service name to retrieve credentials for"
                            },
                            "username": {
                                "type": "string",
                                "description": "Username to filter by (optional)"
                            }
                        },
                        "required": ["service"]
                    }
                }),
                serde_json::json!({
                    "name": "ai_memory_store_bookmark",
                    "description": "Store bookmark with URL, title, and metadata in AI memory",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "URL of the bookmark"
                            },
                            "title": {
                                "type": "string",
                                "description": "Title or name of the bookmark"
                            },
                            "description": {
                                "type": "string",
                                "description": "Optional description or notes"
                            },
                            "tags": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Tags for categorization"
                            },
                            "folder": {
                                "type": "string",
                                "description": "Optional folder or category"
                            }
                        },
                        "required": ["url", "title"]
                    }
                }),
                serde_json::json!({
                    "name": "ai_memory_get_bookmarks",
                    "description": "Retrieve stored bookmarks from AI memory",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "tags": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Filter by tags"
                            },
                            "folder": {
                                "type": "string",
                                "description": "Filter by folder"
                            },
                            "query": {
                                "type": "string",
                                "description": "Search query for title or description"
                            },
                            "limit": {
                                "type": "number",
                                "description": "Maximum number of results (default: 20)"
                            }
                        }
                    }
                }),
                serde_json::json!({
                    "name": "ai_memory_store_note",
                    "description": "Store notes and documentation in AI memory",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "title": {
                                "type": "string",
                                "description": "Note title"
                            },
                            "content": {
                                "type": "string",
                                "description": "Note content (supports markdown)"
                            },
                            "tags": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Tags for categorization"
                            },
                            "category": {
                                "type": "string",
                                "description": "Note category (optional)"
                            }
                        },
                        "required": ["title", "content"]
                    }
                }),
                serde_json::json!({
                    "name": "ai_memory_get_notes",
                    "description": "Retrieve stored notes from AI memory",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "tags": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Filter by tags"
                            },
                            "category": {
                                "type": "string",
                                "description": "Filter by category"
                            },
                            "query": {
                                "type": "string",
                                "description": "Search query for title or content"
                            },
                            "limit": {
                                "type": "number",
                                "description": "Maximum number of results (default: 20)"
                            }
                        }
                    }
                }),
            ]);
        }

        // Chrome DevTools Protocol (CDP) Tools - Execute JavaScript, inspect DOM, manage cookies, capture screenshots, and retrieve console messages
        if env::var("THALORA_ENABLE_CDP").unwrap_or_else(|_| "false".to_string()) == "true" {
            tools.extend_from_slice(&[
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
                                "description": "Whether to await promise results (default: false)"
                            },
                            "return_by_value": {
                                "type": "boolean",
                                "description": "Whether to return result by value (default: true)"
                            },
                            "timeout": {
                                "type": "number",
                                "description": "Execution timeout in milliseconds (default: 5000)"
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
                                "description": "Depth of DOM tree to retrieve (default: 2, max: 10)"
                            },
                            "pierce_shadow": {
                                "type": "boolean",
                                "description": "Whether to pierce shadow DOM (default: false)"
                            }
                        }
                    }
                }),
                serde_json::json!({
                    "name": "cdp_dom_query_selector",
                    "description": "Find elements using CSS selectors via Chrome DevTools Protocol",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "selector": {
                                "type": "string",
                                "description": "CSS selector to find elements"
                            },
                            "node_id": {
                                "type": "number",
                                "description": "Optional node ID to search within (default: document)"
                            }
                        },
                        "required": ["selector"]
                    }
                }),
                serde_json::json!({
                    "name": "cdp_dom_get_attributes",
                    "description": "Get all attributes of an element via Chrome DevTools Protocol",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "node_id": {
                                "type": "number",
                                "description": "Node ID of the element"
                            }
                        },
                        "required": ["node_id"]
                    }
                }),
                serde_json::json!({
                    "name": "cdp_dom_get_computed_style",
                    "description": "Get computed CSS styles of an element",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "node_id": {
                                "type": "number",
                                "description": "Node ID of the element"
                            }
                        },
                        "required": ["node_id"]
                    }
                }),
                serde_json::json!({
                    "name": "cdp_network_get_cookies",
                    "description": "Get all cookies from the current page",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "urls": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Optional URLs to filter cookies (default: current page)"
                            }
                        }
                    }
                }),
                serde_json::json!({
                    "name": "cdp_network_set_cookie",
                    "description": "Set a cookie via Chrome DevTools Protocol",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Cookie name"
                            },
                            "value": {
                                "type": "string",
                                "description": "Cookie value"
                            },
                            "domain": {
                                "type": "string",
                                "description": "Cookie domain (optional)"
                            },
                            "path": {
                                "type": "string",
                                "description": "Cookie path (default: /)"
                            },
                            "secure": {
                                "type": "boolean",
                                "description": "Whether cookie is secure (default: false)"
                            },
                            "http_only": {
                                "type": "boolean",
                                "description": "Whether cookie is HTTP only (default: false)"
                            },
                            "expires": {
                                "type": "number",
                                "description": "Cookie expiration timestamp (optional)"
                            }
                        },
                        "required": ["name", "value"]
                    }
                }),
                serde_json::json!({
                    "name": "cdp_console_get_messages",
                    "description": "Get console messages (logs, errors, warnings) from the page",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "level": {
                                "type": "string",
                                "description": "Filter by message level: 'log', 'info', 'warn', 'error', 'debug' (optional)",
                                "enum": ["log", "info", "warn", "error", "debug"]
                            },
                            "limit": {
                                "type": "number",
                                "description": "Maximum number of messages to return (default: 100)"
                            }
                        }
                    }
                }),
                serde_json::json!({
                    "name": "cdp_page_screenshot",
                    "description": "Take a screenshot of the current page",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "format": {
                                "type": "string",
                                "description": "Image format: 'png' or 'jpeg' (default: png)",
                                "enum": ["png", "jpeg"]
                            },
                            "quality": {
                                "type": "number",
                                "description": "Image quality 0-100 for JPEG (default: 80)"
                            },
                            "full_page": {
                                "type": "boolean",
                                "description": "Capture full page height (default: false)"
                            },
                            "clip": {
                                "type": "object",
                                "properties": {
                                    "x": {"type": "number"},
                                    "y": {"type": "number"},
                                    "width": {"type": "number"},
                                    "height": {"type": "number"}
                                },
                                "description": "Optional clip rectangle for partial screenshots"
                            }
                        }
                    }
                }),
                serde_json::json!({
                    "name": "cdp_page_reload",
                    "description": "Reload the current page",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "ignore_cache": {
                                "type": "boolean",
                                "description": "Whether to ignore cache and reload from server (default: false)"
                            }
                        }
                    }
                }),
            ]);
        }

        // Web Scraping Tools - Scrape web pages and extract content (enabled by default)
        if env::var("THALORA_ENABLE_SCRAPING").unwrap_or_else(|_| "true".to_string()) == "true" {
            tools.extend_from_slice(&[
                serde_json::json!({
                    "name": "scrape",
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
                                "description": "Whether to wait for JavaScript execution (default: false)"
                            },
                            "wait_timeout": {
                                "type": "number",
                                "description": "Timeout for JavaScript execution in milliseconds (default: 5000)"
                            },
                            "extract_links": {
                                "type": "boolean",
                                "description": "Whether to extract all links (default: true)"
                            },
                            "extract_images": {
                                "type": "boolean",
                                "description": "Whether to extract image URLs (default: true)"
                            },
                            "extract_metadata": {
                                "type": "boolean",
                                "description": "Whether to extract page metadata (default: true)"
                            },
                            "session_id": {
                                "type": "string",
                                "description": "Browser session ID (optional)"
                            }
                        },
                        "required": ["url"]
                    }
                }),
                serde_json::json!({
                    "name": "scrape_content_by_selector",
                    "description": "Extract specific content from a page using CSS selectors",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "URL to scrape (optional if session_id provided)"
                            },
                            "selectors": {
                                "type": "object",
                                "description": "CSS selectors mapped to content names",
                                "additionalProperties": {"type": "string"}
                            },
                            "session_id": {
                                "type": "string",
                                "description": "Browser session ID (optional)"
                            }
                        },
                        "required": ["selectors"]
                    }
                }),
                serde_json::json!({
                    "name": "scrape_readable_content",
                    "description": "Extract clean, readable content from a webpage using advanced readability algorithms",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "URL to extract readable content from"
                            },
                            "format": {
                                "type": "string",
                                "enum": ["markdown", "text", "structured"],
                                "description": "Output format for the content (default: markdown)"
                            },
                            "include_images": {
                                "type": "boolean",
                                "description": "Whether to include images in the output (default: true)"
                            },
                            "include_metadata": {
                                "type": "boolean",
                                "description": "Whether to include article metadata like author, date, etc. (default: true)"
                            },
                            "min_content_score": {
                                "type": "number",
                                "description": "Minimum content quality score threshold (0.0-1.0, default: 0.3)"
                            }
                        },
                        "required": ["url"]
                    }
                }),
                serde_json::json!({
                    "name": "browse_readable_content",
                    "description": "Extract content from multi-page articles with session support and automatic pagination handling",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "Starting URL for content extraction"
                            },
                            "format": {
                                "type": "string",
                                "enum": ["markdown", "text", "structured"],
                                "description": "Output format for the content (default: markdown)"
                            },
                            "follow_pagination": {
                                "type": "boolean",
                                "description": "Whether to automatically follow pagination links (default: true)"
                            },
                            "max_pages": {
                                "type": "number",
                                "description": "Maximum number of pages to process (default: 10)"
                            },
                            "wait_for_js": {
                                "type": "boolean",
                                "description": "Whether to wait for JavaScript execution (default: false)"
                            },
                            "include_images": {
                                "type": "boolean",
                                "description": "Whether to include images in the output (default: true)"
                            },
                            "session_id": {
                                "type": "string",
                                "description": "Browser session ID to maintain state across pages (optional)"
                            }
                        },
                        "required": ["url"]
                    }
                }),
                serde_json::json!({
                    "name": "scrape_structured_content",
                    "description": "Extract structured content (tables, lists, code blocks, metadata) from a webpage",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "URL to extract structured content from"
                            },
                            "content_types": {
                                "type": "array",
                                "items": {
                                    "type": "string",
                                    "enum": ["tables", "lists", "code_blocks", "metadata"]
                                },
                                "description": "Types of content to extract (default: all types)",
                                "default": ["tables", "lists", "code_blocks", "metadata"]
                            }
                        },
                        "required": ["url"]
                    }
                }),
            ]);
        }

        // Web Search Tools - Perform web searches using various search engines
        if env::var("THALORA_ENABLE_SEARCH").unwrap_or_else(|_| "false".to_string()) == "true" {
            tools.extend_from_slice(&[
                serde_json::json!({
                    "name": "web_search",
                    "description": "Search the web using various search engines and return organic results with title, URL, and snippet",
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
                            },
                            "search_engine": {
                                "type": "string",
                                "description": "Search engine to use: 'duckduckgo', 'bing', 'startpage', 'searx' (default: 'duckduckgo')",
                                "enum": ["duckduckgo", "bing", "startpage", "searx"]
                            },
                            "region": {
                                "type": "string",
                                "description": "Search region/country code (e.g., 'us', 'uk', 'de')"
                            },
                            "time_range": {
                                "type": "string",
                                "description": "Time range filter: 'day', 'week', 'month', 'year'",
                                "enum": ["day", "week", "month", "year"]
                            }
                        },
                        "required": ["query"]
                    }
                }),
                serde_json::json!({
                    "name": "image_search",
                    "description": "Search for images using web search engines",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Image search query"
                            },
                            "num_results": {
                                "type": "number",
                                "description": "Number of image results to return (default: 10, max: 20)"
                            },
                            "search_engine": {
                                "type": "string",
                                "description": "Search engine to use: 'duckduckgo', 'bing' (default: 'duckduckgo')",
                                "enum": ["duckduckgo", "bing"]
                            },
                            "size": {
                                "type": "string",
                                "description": "Image size filter: 'small', 'medium', 'large'",
                                "enum": ["small", "medium", "large"]
                            }
                        },
                        "required": ["query"]
                    }
                }),
            ]);
        }

        // Browser Automation Tools - Interact with web pages by clicking elements and filling forms
        if Self::is_sessions_enabled() {
            tools.extend_from_slice(&[
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
                            "wait_for_navigation": {
                                "type": "boolean",
                                "description": "Whether to wait for page navigation after click (default: false)"
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
                            "submit": {
                                "type": "boolean",
                                "description": "Whether to submit the form after filling (default: true)"
                            },
                            "session_id": {
                                "type": "string",
                                "description": "Browser session ID (optional)"
                            }
                        },
                        "required": ["form_data"]
                    }
                }),
                serde_json::json!({
                    "name": "browser_type_text",
                    "description": "Type text into an input field or element",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "selector": {
                                "type": "string",
                                "description": "CSS selector for the input element"
                            },
                            "text": {
                                "type": "string",
                                "description": "Text to type"
                            },
                            "clear_first": {
                                "type": "boolean",
                                "description": "Whether to clear the field before typing (default: true)"
                            },
                            "session_id": {
                                "type": "string",
                                "description": "Browser session ID (optional)"
                            }
                        },
                        "required": ["selector", "text"]
                    }
                }),
                serde_json::json!({
                    "name": "browser_wait_for_element",
                    "description": "Wait for an element to appear on the page",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "selector": {
                                "type": "string",
                                "description": "CSS selector for the element to wait for"
                            },
                            "timeout": {
                                "type": "number",
                                "description": "Timeout in milliseconds (default: 10000)"
                            },
                            "visible": {
                                "type": "boolean",
                                "description": "Whether to wait for element to be visible (default: true)"
                            },
                            "session_id": {
                                "type": "string",
                                "description": "Browser session ID (optional)"
                            }
                        },
                        "required": ["selector"]
                    }
                }),
            ]);
        }

        // Session Management Tools - Create, manage, and clean up browser sessions for persistent AI interactions
        if Self::is_sessions_enabled() {
            tools.extend_from_slice(&[
                serde_json::json!({
                    "name": "browser_session_management",
                    "description": "Manage browser sessions for persistent AI interactions",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "action": {
                                "type": "string",
                                "description": "Action to perform: 'create', 'info', 'list', 'close', 'cleanup'",
                                "enum": ["create", "info", "list", "close", "cleanup"]
                            },
                            "session_id": {
                                "type": "string",
                                "description": "Session ID (required for info/close actions)"
                            },
                            "persistent": {
                                "type": "boolean",
                                "description": "Whether to make session persistent (for create action)"
                            },
                            "max_age_seconds": {
                                "type": "number",
                                "description": "Maximum age for cleanup action (default: 3600)"
                            }
                        },
                        "required": ["action"]
                    }
                }),
                serde_json::json!({
                    "name": "browser_get_page_content",
                    "description": "Get the current page content and URL from a browser session",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "session_id": {
                                "type": "string",
                                "description": "Browser session ID (optional, defaults to 'default')"
                            },
                            "include_html": {
                                "type": "boolean",
                                "description": "Whether to include raw HTML (default: false)"
                            },
                            "include_text": {
                                "type": "boolean",
                                "description": "Whether to include extracted text (default: true)"
                            }
                        }
                    }
                }),
                serde_json::json!({
                    "name": "browser_navigate_to",
                    "description": "Navigate to a specific URL in a browser session",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "URL to navigate to"
                            },
                            "wait_for_load": {
                                "type": "boolean",
                                "description": "Whether to wait for page to fully load (default: true)"
                            },
                            "session_id": {
                                "type": "string",
                                "description": "Browser session ID (optional, defaults to 'default')"
                            }
                        },
                        "required": ["url"]
                    }
                }),
                serde_json::json!({
                    "name": "browser_navigate_back",
                    "description": "Navigate back in browser history",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "session_id": {
                                "type": "string",
                                "description": "Browser session ID (optional, defaults to 'default')"
                            }
                        }
                    }
                }),
                serde_json::json!({
                    "name": "browser_navigate_forward",
                    "description": "Navigate forward in browser history",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "session_id": {
                                "type": "string",
                                "description": "Browser session ID (optional, defaults to 'default')"
                            }
                        }
                    }
                }),
                serde_json::json!({
                    "name": "browser_refresh_page",
                    "description": "Refresh/reload the current page in the browser",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "session_id": {
                                "type": "string",
                                "description": "Browser session ID (optional, defaults to 'default')"
                            }
                        }
                    }
                }),
            ]);
        }

        tools
    }

    pub(super) async fn call_tool(&mut self, name: String, arguments: Value) -> McpResponse {
        // If a `session_id` argument is present, reuse or create a session-scoped VFS that persists across calls.
        let vfs_instance: Arc<VfsInstance>;
        let prev_vfs: Option<Arc<VfsInstance>>;
        if let Some(session_id) = arguments.get("session_id").and_then(|v| v.as_str()) {
            // Reuse or create a session VFS. Backing path is in temp dir by default.
            let v = self.get_or_create_session_vfs(session_id, None);
            vfs_instance = v.clone();
            prev_vfs = set_current_vfs(Some(vfs_instance.clone()));
        } else {
            // ephemeral per-call VFS
            let tmp_dir = env::temp_dir();
            let v = match VfsInstance::new_temp_in_dir(&tmp_dir) {
                Ok(v) => Arc::new(v),
                Err(e) => return McpResponse::error(-32000, format!("Failed to create VFS: {}", e)),
            };
            prev_vfs = set_current_vfs(Some(v.clone()));
            vfs_instance = v;
        }

        // Run the tool while VFS is installed
        // Clone `arguments` for the call so we can still inspect the original after the call (lifecycle checks).
        let args_for_call = arguments.clone();
          let resp = match name.as_str() {
            // AI Memory tools
            "ai_memory_store_research" => self.memory_tools.store_research(args_for_call.clone(), &mut self.ai_memory).await,
            // There is no direct `get_research` async tool; use `search` with a key filter
            "ai_memory_get_research" => {
                // Validate required "key" parameter
                let key = match args_for_call.get("key").and_then(|v| v.as_str()) {
                    Some(key) => key,
                    None => {
                        return McpResponse::ToolResult {
                            content: vec![serde_json::json!({
                                "type": "text",
                                "text": "Missing required parameter: key"
                            })],
                            is_error: true,
                        };
                    }
                };

                // Convert key to query for search
                let search_args = serde_json::json!({
                    "query": key,
                    "category": "research",
                    "limit": 1
                });
                self.memory_tools.search(search_args, &mut self.ai_memory).await
            },
            "ai_memory_search_research" => self.memory_tools.search(args_for_call.clone(), &mut self.ai_memory).await,
            "ai_memory_store_credentials" => self.memory_tools.store_credentials(args_for_call.clone(), &mut self.ai_memory).await,
            "ai_memory_get_credentials" => self.memory_tools.get_credentials(args_for_call.clone(), &mut self.ai_memory).await,
            "ai_memory_store_bookmark" => self.memory_tools.store_bookmark(args_for_call.clone(), &mut self.ai_memory).await,
            // no direct get_bookmarks; map to search with category=bookmarks
            "ai_memory_get_bookmarks" => self.memory_tools.search(args_for_call.clone(), &mut self.ai_memory).await,
            "ai_memory_store_note" => self.memory_tools.store_note(args_for_call.clone(), &mut self.ai_memory).await,
            // no direct get_notes; map to search with category=notes
            "ai_memory_get_notes" => self.memory_tools.search(args_for_call.clone(), &mut self.ai_memory).await,

            // Chrome DevTools Protocol tools - comprehensive debugging toolkit
            "cdp_runtime_evaluate" => self.cdp_tools.evaluate_javascript(args_for_call.clone(), &mut self.cdp_server).await,
            "cdp_dom_get_document" => self.cdp_tools.get_document(args_for_call.clone(), &mut self.cdp_server).await,

            // DOM debugging tools
            "cdp_dom_query_selector" => self.cdp_tools.query_selector(args_for_call.clone(), &mut self.cdp_server).await,
            "cdp_dom_get_attributes" => self.cdp_tools.get_attributes(args_for_call.clone(), &mut self.cdp_server).await,
            "cdp_dom_get_computed_style" => self.cdp_tools.get_computed_style(args_for_call.clone(), &mut self.cdp_server).await,

            // Network debugging tools
            "cdp_network_get_cookies" => self.cdp_tools.get_cookies(args_for_call.clone(), &mut self.cdp_server).await,
            "cdp_network_set_cookie" => self.cdp_tools.set_cookie(args_for_call.clone(), &mut self.cdp_server).await,

            // Console debugging tools
            "cdp_console_get_messages" => self.cdp_tools.get_console_messages(args_for_call.clone(), &mut self.cdp_server).await,

            // Page control tools
            "cdp_page_screenshot" => self.cdp_tools.take_screenshot(args_for_call.clone(), &mut self.cdp_server).await,
            "cdp_page_reload" => self.cdp_tools.reload_page(args_for_call.clone(), &mut self.cdp_server).await,

            // Scraping and Search tools (stateless)
            "scrape" => self.scrape_url(args_for_call.clone()).await,
            "scrape_readable_content" => self.scrape_readable_content(args_for_call.clone()).await,
            "scrape_structured_content" => self.extract_structured_content(args_for_call.clone()).await,

            // Maybe make the searches stateful in the future with session_id
            "web_search" => self.web_search(args_for_call.clone()).await,
            // Note: image_search not implemented yet - placeholder for future
            "image_search" => McpResponse::error(-32601, "Tool not implemented yet: image_search".to_string()),

            // Session management tools
            "browser_session_management" => self.browser_tools.handle_session_management(args_for_call.clone()).await,
            "browser_get_page_content" => self.browser_tools.handle_get_page_content(args_for_call.clone()).await,
            "browse_readable_content" => self.browse_readable_content(args_for_call.clone()).await,

            // Browser automation tools
            "browser_fill_form" => self.browser_tools.handle_fill_form(args_for_call.clone()).await,
            
            // User Events Simulation
            "browser_click_element" => self.browser_tools.handle_click_element(args_for_call.clone()).await,
            // Note: Additional browser automation tools not implemented yet - placeholders for future
            "browser_output_text" => McpResponse::error(-32601, "Tool not implemented yet: browser_type_text".to_string()),
            "browser_wait_for_element" => McpResponse::error(-32601, "Tool not implemented yet: browser_wait_for_element".to_string()),

            // Navigation tools
            "browser_refresh_page" => self.browser_tools.handle_refresh_page(args_for_call.clone()).await,
            "browser_navigate_forward" => self.browser_tools.handle_navigate_forward(args_for_call.clone()).await,
            "browser_navigate_back" => self.browser_tools.handle_navigate_back(args_for_call.clone()).await,
            "browser_navigate_to" => self.browser_tools.handle_navigate_to(args_for_call.clone()).await,

            // Unknown/Unhandled tool
            _ => McpResponse::error(-32601, format!("Tool not found: {}", name))
          };

        // Lifecycle:
        // - If ephemeral (no session_id): persist if `persistent=true`, otherwise delete backing file.
        // - If session-scoped: if `persistent=true` persist the session backing file; otherwise keep it in-memory for the session.
        let is_session = arguments.get("session_id").and_then(|v| v.as_str()).is_some();
        let should_persist = arguments.get("persistent").and_then(|v| v.as_bool()).unwrap_or(false);

        if is_session {
            if should_persist {
                if let Err(e) = vfs_instance.persist() {
                    drop(set_current_vfs(prev_vfs));
                    return McpResponse::error(-32001, format!("Failed to persist session VFS: {}", e));
                }
            }
            // for session VFS we keep the backing instance in `self.session_vfs` until explicit removal
        } else {
            if should_persist {
                if let Err(e) = vfs_instance.persist() {
                    drop(set_current_vfs(prev_vfs));
                    return McpResponse::error(-32002, format!("Failed to persist ephemeral VFS: {}", e));
                }
            } else {
                drop(vfs_instance.delete_backing_file());
            }
        }

        // Restore previous VFS (if any)
    drop(set_current_vfs(prev_vfs));

        resp
    }
}