use serde_json::Value;

/// Browser automation tool definitions for interacting with web pages
pub(crate) fn get_browser_automation_tool_definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "browser_click_element",
            "description": "Click an element (by ref from browser_snapshot or CSS selector). Submit buttons submit their form and links navigate unless a handler calls preventDefault().",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "ref": {
                        "type": "string",
                        "description": "Element ref from browser_snapshot (e.g. \"e12\"); use instead of selector"
                    },
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
                "required": []
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
                    "ref": {
                        "type": "string",
                        "description": "Element ref from browser_snapshot (e.g. \"e12\"); use instead of selector"
                    },
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
                "required": ["text"]
            }
        }),
        serde_json::json!({
            "name": "browser_select_option",
            "description": "Choose an option of a <select> by its value or visible text (fires input/change). The choice is included when the form is submitted.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "value": {
                        "type": "string",
                        "description": "Option value or visible text"
                    },
                    "ref": {
                        "type": "string",
                        "description": "Element ref from browser_snapshot"
                    },
                    "selector": {
                        "type": "string",
                        "description": "CSS selector (alternative to ref)"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID (optional)"
                    }
                },
                "required": ["value"]
            }
        }),
        serde_json::json!({
            "name": "browser_check",
            "description": "Check or uncheck a checkbox or radio button (fires input/change). The state is used when the form is submitted.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "checked": {
                        "type": "boolean",
                        "description": "true to check (default), false to uncheck"
                    },
                    "ref": {
                        "type": "string",
                        "description": "Element ref from browser_snapshot"
                    },
                    "selector": {
                        "type": "string",
                        "description": "CSS selector (alternative to ref)"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID (optional)"
                    }
                },
                "required": []
            }
        }),
        serde_json::json!({
            "name": "browser_press_key",
            "description": "Press a key (keydown/keypress/keyup), e.g. \"Enter\", \"Escape\", \"ArrowDown\". Enter in a single-line form field submits the form unless prevented. Without ref/selector the key goes to the focused element.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "key": {
                        "type": "string",
                        "description": "Key name as in KeyboardEvent.key"
                    },
                    "ref": {
                        "type": "string",
                        "description": "Element ref from browser_snapshot"
                    },
                    "selector": {
                        "type": "string",
                        "description": "CSS selector (alternative to ref)"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID (optional)"
                    }
                },
                "required": ["key"]
            }
        }),
        serde_json::json!({
            "name": "browser_hover",
            "description": "Move the pointer over an element (mouseover/mouseenter/mousemove), e.g. to open hover menus.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "ref": {
                        "type": "string",
                        "description": "Element ref from browser_snapshot"
                    },
                    "selector": {
                        "type": "string",
                        "description": "CSS selector (alternative to ref)"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID (optional)"
                    }
                },
                "required": []
            }
        }),
        serde_json::json!({
            "name": "browser_scroll",
            "description": "Scroll an element into view (ref/selector) or scroll the page by delta_y pixels, firing scroll events (e.g. to trigger lazy loading).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "delta_y": {
                        "type": "integer",
                        "description": "Pixels to scroll the page when no element is given (default 600)"
                    },
                    "ref": {
                        "type": "string",
                        "description": "Element ref from browser_snapshot"
                    },
                    "selector": {
                        "type": "string",
                        "description": "CSS selector (alternative to ref)"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID (optional)"
                    }
                },
                "required": []
            }
        }),
        serde_json::json!({
            "name": "browser_console_messages",
            "description": "Console messages (log/info/warn/error/debug) logged by the current page, oldest first (up to 500).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "clear": {
                        "type": "boolean",
                        "description": "Clear the buffer after reading (default false)"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID (optional)"
                    }
                }
            }
        }),
        serde_json::json!({
            "name": "browser_wait",
            "description": "Wait (running page timers and network) until a condition holds: an element by ref or selector exists, the page text contains a string, the URL contains a string, or the network is idle. Give exactly one condition. Returns met=false on timeout.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "ref": {
                        "type": "string",
                        "description": "Element ref from browser_snapshot"
                    },
                    "selector": {
                        "type": "string",
                        "description": "CSS selector that must match"
                    },
                    "text": {
                        "type": "string",
                        "description": "Text that must appear on the page"
                    },
                    "url_contains": {
                        "type": "string",
                        "description": "Substring the current URL must contain"
                    },
                    "network_idle": {
                        "type": "boolean",
                        "description": "Wait until no requests for 500 ms"
                    },
                    "timeout_ms": {
                        "type": "integer",
                        "description": "Maximum wait (default 5000, max 60000)"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID (optional)"
                    }
                }
            }
        }),
        serde_json::json!({
            "name": "browser_wait_for_element",
            "description": "Wait for an element to appear on the page",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "ref": {
                        "type": "string",
                        "description": "Element ref from browser_snapshot (e.g. \"e12\"); use instead of selector"
                    },
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
                "required": []
            }
        }),
        serde_json::json!({
            "name": "browser_prepare_form_submission",
            "description": "Prepare for a form submission that will open a new window by creating a predictive session",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "form_selector": {
                        "type": "string",
                        "description": "CSS selector for the form element"
                    },
                    "submit_button_selector": {
                        "type": "string",
                        "description": "CSS selector for the submit button (optional)"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Current browser session ID (optional, defaults to 'default')"
                    }
                },
                "required": ["form_selector"]
            }
        }),
        serde_json::json!({
            "name": "browser_validate_session",
            "description": "Validate that a browser session exists and optionally check if it has loaded expected content",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID to validate"
                    },
                    "expected_url_pattern": {
                        "type": "string",
                        "description": "Optional regex pattern to match against current URL"
                    },
                    "expected_content": {
                        "type": "string",
                        "description": "Optional text that should be present in page content"
                    },
                    "timeout": {
                        "type": "number",
                        "description": "Timeout in milliseconds to wait for conditions (default: 5000)"
                    }
                },
                "required": ["session_id"]
            }
        }),
    ]
}
