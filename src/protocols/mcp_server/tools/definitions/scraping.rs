use serde_json::Value;

/// Web scraping tool definitions for content extraction
pub(crate) fn get_scraping_tool_definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "snapshot_url",
            "description": "Capture a point-in-time snapshot of a web page. Combines all extraction capabilities: basic content (links, images, metadata), CSS selectors, readability algorithms, and structured content (tables, lists, code blocks). The returned data is a non-interactive snapshot. Enable specific extraction types as needed with extract_* parameters.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to scrape (optional if session_id provided)"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID (optional)"
                    },
                    "wait_for_js": {
                        "type": "boolean",
                        "description": "Whether to wait for JavaScript execution (default: false)"
                    },
                    "wait_timeout": {
                        "type": "number",
                        "description": "Timeout for JavaScript execution in milliseconds (default: 5000)"
                    },
                    "extract_basic": {
                        "type": "boolean",
                        "description": "Extract basic content: links, images, and metadata (default: true)"
                    },
                    "extract_readable": {
                        "type": "boolean",
                        "description": "Extract clean, readable content using readability algorithms (default: false)"
                    },
                    "extract_structured": {
                        "type": "boolean",
                        "description": "Extract structured content: tables, lists, code blocks (default: false)"
                    },
                    "selectors": {
                        "type": "object",
                        "description": "Custom CSS selectors mapped to content names (e.g., {\"title\": \"h1\", \"price\": \".price\"})",
                        "additionalProperties": {"type": "string"}
                    },
                    "format": {
                        "type": "string",
                        "enum": ["markdown", "text", "structured"],
                        "description": "Output format for readable content (default: markdown, only used if extract_readable=true)"
                    },
                    "include_images": {
                        "type": "boolean",
                        "description": "Whether to include images in readable content (default: true)"
                    },
                    "include_metadata": {
                        "type": "boolean",
                        "description": "Whether to include article metadata like author, date, etc. (default: true)"
                    },
                    "min_content_score": {
                        "type": "number",
                        "description": "Minimum content quality score threshold for readability (0.0-1.0, default: 0.3)"
                    },
                    "content_types": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["tables", "lists", "code_blocks", "metadata"]
                        },
                        "description": "Types of structured content to extract (default: all types, only used if extract_structured=true)"
                    },
                    "max_output_size": {
                        "type": "number",
                        "description": "Maximum output size in characters (default: 50000, 0 = unlimited). When exceeded, content is progressively truncated: links/images first, then tables, then readable content."
                    }
                }
            }
        }),
    ]
}

/// Web search tool definitions
pub(crate) fn get_search_tool_definitions() -> Vec<Value> {
    vec![
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
                        "description": "Search engine to use: 'duckduckgo', 'bing', 'google', 'startpage' (default: 'duckduckgo')",
                        "enum": ["duckduckgo", "bing", "google", "startpage"]
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
    ]
}

/// Minimal scraping tool definitions for basic mode
pub(crate) fn get_minimal_scraping_tool_definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "snapshot_url",
            "description": "Capture a point-in-time snapshot of a web page with multiple extraction methods (basic content, custom selectors, readable content, structured data). Stateless - browser closes after each use.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to scrape"
                    },
                    "wait_for_js": {
                        "type": "boolean",
                        "description": "Wait for JavaScript to execute before scraping (default: false)"
                    },
                    "extract_basic": {
                        "type": "boolean",
                        "description": "Extract links, images, and metadata (default: true)"
                    },
                    "extract_readable": {
                        "type": "boolean",
                        "description": "Extract clean readable content using readability algorithms (default: false)"
                    },
                    "extract_structured": {
                        "type": "boolean",
                        "description": "Extract structured content like tables, lists, code blocks (default: false)"
                    },
                    "selectors": {
                        "type": "object",
                        "description": "Custom CSS selectors mapped to names (e.g., {\"title\": \"h1\", \"price\": \".price\"})"
                    },
                    "max_output_size": {
                        "type": "number",
                        "description": "Maximum output size in characters (default: 50000, 0 = unlimited)"
                    }
                },
                "required": ["url"]
            }
        }),
    ]
}

/// Minimal search tool definitions for basic mode
pub(crate) fn get_minimal_search_tool_definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "web_search",
            "description": "Search the web using various search engines. Returns organic search results. Stateless - browser closes after each use.",
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
                        "enum": ["duckduckgo", "bing", "google", "startpage"],
                        "description": "Search engine to use (default: duckduckgo)"
                    }
                },
                "required": ["query"]
            }
        })
    ]
}
