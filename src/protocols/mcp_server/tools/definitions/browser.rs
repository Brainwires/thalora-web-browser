use serde_json::Value;

/// Browser automation tool definitions for interacting with web pages
pub(crate) fn get_browser_automation_tool_definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "browser_click_element",
            "description": "Click an element (by ref from browser_snapshot or CSS selector). Submit buttons submit their form, links navigate and checkboxes/radio buttons change state. The click dispatches pointer, mouse and click events, so page handlers run and can prevent the default action; a navigation a handler starts is followed. On a static page (JavaScript disabled) only default actions work.",
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
            "description": "Type text into a named input or textarea; the value is sent when its form is submitted",
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
            "description": "Press a key on an element (ref/selector) or on the focused element: keydown, keypress and keyup are dispatched, then the key's default action unless a page handler prevents it (a printable key or Backspace edits a text field, Enter in a single-line field submits its form, Enter/Space activates a button or link). On a static page (JavaScript disabled) only Enter in a form field works.",
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
            "description": "Move the pointer over an element (pointerover/pointerenter/mouseover/mouseenter/mousemove are dispatched), e.g. to open script-driven hover menus. CSS :hover is not applied. Fails on a static page (JavaScript disabled).",
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
            "description": "Dispatch a scroll event, as scrolling an element into view (ref/selector) or the page by delta_y pixels would, e.g. to trigger scroll-driven lazy loading. The page has no layout, so scroll positions do not change and loaders that measure positions may not react. Fails on a static page (JavaScript disabled).",
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
            "description": "Console output of the current page, oldest first (the newest 1000): console.* calls (level debug/log/warning/error), uncaught errors, and the browser's notes about the page's scripts (blocked by CSP, failed to load, timed out). The `javascript` field says whether the page runs JavaScript.",
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
            "description": "Wait for a condition on the current page: an element by ref or selector exists, the page contains a string, the URL contains a string, or the page is idle. Give exactly one condition. The page's timers and requests run while waiting. Returns met=true as soon as the condition holds, and met=false at the timeout or as soon as the page has nothing pending that could change it (always at once on a static page).",
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
                        "description": "Wait until the page has nothing pending: no request, timer or task (a page with a repeating timer never becomes idle)"
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
            "description": "Wait until an element is present on the current page; the page's timers and requests run while waiting. Answers found=false at the timeout, or as soon as the page has nothing pending that could add the element (at once on a static page).",
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
                        "description": "Timeout in milliseconds (default: 10000, max 60000)"
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
