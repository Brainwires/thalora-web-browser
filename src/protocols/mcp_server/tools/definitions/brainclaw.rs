use serde_json::Value;

/// Agent-friendly tool definitions for BrainClaw preset.
///
/// These are shorter, cleaner aliases for the verbose default tool names plus
/// `browser_read_url` (navigate + extract in one call — the most common agent pattern).
/// They are registered in addition to the full toolset when `THALORA_PRESET=brainclaw`.
pub(crate) fn get_brainclaw_alias_tool_definitions() -> Vec<Value> {
    vec![
        // ── One-shot read ────────────────────────────────────────────────────────
        serde_json::json!({
            "name": "browser_read_url",
            "description": "Navigate to a URL and return its content as clean markdown in one call. \
                            This is the fastest way to read a web page — no session needed. \
                            Use `browser_navigate` + `browser_get_page_content` for session-based workflows instead.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to navigate to and extract"
                    },
                    "wait_for_js": {
                        "type": "boolean",
                        "description": "Wait for JavaScript execution before extracting (default: false)"
                    },
                    "include_images": {
                        "type": "boolean",
                        "description": "Include image alt text in markdown output (default: true)"
                    },
                    "max_output_size": {
                        "type": "number",
                        "description": "Maximum output size in characters (default: 50000, 0 = unlimited)"
                    }
                },
                "required": ["url"]
            }
        }),
        // ── Navigation aliases ───────────────────────────────────────────────────
        serde_json::json!({
            "name": "browser_navigate",
            "description": "Navigate a browser session to a URL. Alias for `browser_navigate_to`.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to navigate to"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID"
                    },
                    "wait_for_js": {
                        "type": "boolean",
                        "description": "Wait for JavaScript to settle after navigation (default: false)"
                    }
                },
                "required": ["url", "session_id"]
            }
        }),
        // ── Interaction aliases ──────────────────────────────────────────────────
        serde_json::json!({
            "name": "browser_click",
            "description": "Click an element in a browser session by CSS selector. Alias for `browser_click_element`.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "selector": {
                        "type": "string",
                        "description": "CSS selector for the element to click"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID"
                    }
                },
                "required": ["selector", "session_id"]
            }
        }),
        serde_json::json!({
            "name": "browser_fill",
            "description": "Fill an input field in a browser session. Alias for `browser_fill_form`.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "selector": {
                        "type": "string",
                        "description": "CSS selector for the input field"
                    },
                    "value": {
                        "type": "string",
                        "description": "Value to fill into the field"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID"
                    }
                },
                "required": ["selector", "value", "session_id"]
            }
        }),
        // ── CDP aliases ──────────────────────────────────────────────────────────
        serde_json::json!({
            "name": "browser_eval",
            "description": "Execute JavaScript in the browser and return the result. Alias for `cdp_runtime_evaluate`.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "JavaScript expression to evaluate"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID (optional)"
                    },
                    "return_by_value": {
                        "type": "boolean",
                        "description": "Return result as JSON value instead of object reference (default: true)"
                    }
                },
                "required": ["expression"]
            }
        }),
        serde_json::json!({
            "name": "browser_screenshot",
            "description": "Capture a screenshot of the current browser page. Returns base64-encoded PNG. Alias for `cdp_page_screenshot`.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID (optional)"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["png", "jpeg"],
                        "description": "Image format (default: png)"
                    },
                    "quality": {
                        "type": "number",
                        "description": "JPEG quality 0-100 (only used when format=jpeg, default: 80)"
                    }
                }
            }
        }),
        // ── Extraction / search aliases ──────────────────────────────────────────
        serde_json::json!({
            "name": "browser_extract",
            "description": "Capture a snapshot of a web page with configurable extraction. Alias for `snapshot_url`.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to extract (optional if session_id provided)"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID (optional)"
                    },
                    "extract_readable": {
                        "type": "boolean",
                        "description": "Extract clean readable content using readability algorithms (default: false)"
                    },
                    "extract_structured": {
                        "type": "boolean",
                        "description": "Extract structured content: tables, lists, code blocks (default: false)"
                    },
                    "selectors": {
                        "type": "object",
                        "description": "Custom CSS selectors mapped to content names",
                        "additionalProperties": {"type": "string"}
                    },
                    "format": {
                        "type": "string",
                        "enum": ["markdown", "text", "structured"],
                        "description": "Output format (default: markdown)"
                    }
                }
            }
        }),
        serde_json::json!({
            "name": "browser_search",
            "description": "Search the web and return top results as titles, URLs, and snippets. Alias for `web_search` with sensible defaults.",
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
                    },
                    "time_range": {
                        "type": "string",
                        "enum": ["day", "week", "month", "year"],
                        "description": "Limit results by recency"
                    }
                },
                "required": ["query"]
            }
        }),
    ]
}
