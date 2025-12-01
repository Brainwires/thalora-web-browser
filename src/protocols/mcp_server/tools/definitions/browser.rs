use serde_json::Value;

/// Browser automation tool definitions for interacting with web pages
pub(crate) fn get_browser_automation_tool_definitions() -> Vec<Value> {
    vec![
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
