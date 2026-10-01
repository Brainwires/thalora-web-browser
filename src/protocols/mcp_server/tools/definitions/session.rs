use serde_json::Value;

/// Session management tool definitions for persistent browser sessions
pub(crate) fn get_session_tool_definitions() -> Vec<Value> {
    vec![
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
            "name": "browser_snapshot",
            "description": "Compact outline of the current page for deciding what to do next: interactive elements with refs (e.g. [ref=e12]) plus headings, landmarks and text, within a token budget. Pass a ref instead of a CSS selector to browser_click_element, browser_type_text, browser_fill and browser_wait_for_element. Refs stay valid until the page content changes; a stale ref returns an error asking for a new snapshot.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID (default: \"default\")"
                    },
                    "max_tokens": {
                        "type": "integer",
                        "description": "Approximate output budget in tokens (default 4000, 200-50000). Text is dropped first, interactive elements last."
                    },
                    "interactive_only": {
                        "type": "boolean",
                        "description": "Only list interactive elements (default false)"
                    },
                    "focus_ref": {
                        "type": "string",
                        "description": "Only snapshot the subtree of this ref"
                    }
                }
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
            "description": "Navigate to a specific URL in a browser session with optional JavaScript execution",
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
                    "wait_for_js": {
                        "type": "boolean",
                        "description": "Whether to execute page JavaScript and wait for DOM to stabilize (default: false). Enable for SPAs and dynamic sites."
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
    ]
}
