use serde_json::Value;
use crate::protocols::mcp::McpResponse;
use crate::protocols::mcp_server::core::McpServer;
use std::sync::Arc;
use vfs::{VfsInstance, set_current_vfs};
use std::env;

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
            // Additional CDP debugging tools
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
            // Web search tools
            serde_json::json!({
                "name": "web_search",
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
            // Session management tools
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
                        }
                    }
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
        ]
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
            "ai_memory_get_research" => self.memory_tools.search(args_for_call.clone(), &mut self.ai_memory).await,
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

            // Web scraping and navigation tools
            "scrape_url" => self.scrape_url(args_for_call.clone()).await,
            "web_search" => self.google_search(args_for_call.clone()).await,

            // Browser automation tools
            "browser_click_element" => self.browser_tools.handle_click_element(args_for_call.clone()).await,
            "browser_fill_form" => self.browser_tools.handle_fill_form(args_for_call.clone()).await,
            "browser_get_page_content" => self.browser_tools.handle_get_page_content(args_for_call.clone()).await,
            "browser_navigate_back" => self.browser_tools.handle_navigate_back(args_for_call.clone()).await,
            "browser_navigate_forward" => self.browser_tools.handle_navigate_forward(args_for_call.clone()).await,
            "browser_session_management" => self.browser_tools.handle_session_management(args_for_call.clone()).await,

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
                    let _ = set_current_vfs(prev_vfs);
                    return McpResponse::error(-32001, format!("Failed to persist session VFS: {}", e));
                }
            }
            // for session VFS we keep the backing instance in `self.session_vfs` until explicit removal
        } else {
            if should_persist {
                if let Err(e) = vfs_instance.persist() {
                    let _ = set_current_vfs(prev_vfs);
                    return McpResponse::error(-32002, format!("Failed to persist ephemeral VFS: {}", e));
                }
            } else {
                let _ = vfs_instance.delete_backing_file();
            }
        }

        // Restore previous VFS (if any)
        let _ = set_current_vfs(prev_vfs);

        resp
    }
}