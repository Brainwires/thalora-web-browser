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
