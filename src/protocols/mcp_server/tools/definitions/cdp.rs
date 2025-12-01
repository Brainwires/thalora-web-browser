use serde_json::Value;

/// Chrome DevTools Protocol (CDP) tool definitions for debugging and inspection
pub(crate) fn get_cdp_tool_definitions() -> Vec<Value> {
    vec![
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
    ]
}
