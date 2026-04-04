use crate::protocols::mcp::McpResponse;
use crate::protocols::mcp_server::core::McpServer;
use serde_json::Value;

impl McpServer {
    /// Route tool calls to appropriate handlers
    pub(super) async fn route_tool_call(&mut self, name: &str, arguments: Value) -> McpResponse {
        match name {
            // AI Memory tools
            "ai_memory_store_research" => {
                self.memory_tools
                    .store_research(arguments, &mut self.ai_memory)
                    .await
            }
            "ai_memory_get_research" => {
                // Validate required "key" parameter
                let key = match arguments.get("key").and_then(|v| v.as_str()) {
                    Some(key) => key,
                    None => {
                        return McpResponse::error(
                            -1,
                            "Missing required parameter: key".to_string(),
                        );
                    }
                };

                // Convert key to query for search
                let search_args = serde_json::json!({
                    "query": key,
                    "category": "research",
                    "limit": 1
                });
                self.memory_tools
                    .search(search_args, &mut self.ai_memory)
                    .await
            }
            "ai_memory_search_research" => {
                self.memory_tools
                    .search(arguments, &mut self.ai_memory)
                    .await
            }
            "ai_memory_store_credentials" => {
                self.memory_tools
                    .store_credentials(arguments, &mut self.ai_memory)
                    .await
            }
            "ai_memory_get_credentials" => {
                self.memory_tools
                    .get_credentials(arguments, &mut self.ai_memory)
                    .await
            }
            "ai_memory_store_bookmark" => {
                self.memory_tools
                    .store_bookmark(arguments, &mut self.ai_memory)
                    .await
            }
            "ai_memory_get_bookmarks" => {
                self.memory_tools
                    .search(arguments, &mut self.ai_memory)
                    .await
            }
            "ai_memory_store_note" => {
                self.memory_tools
                    .store_note(arguments, &mut self.ai_memory)
                    .await
            }
            "ai_memory_get_notes" => {
                self.memory_tools
                    .search(arguments, &mut self.ai_memory)
                    .await
            }

            // Chrome DevTools Protocol tools - comprehensive debugging toolkit
            "cdp_runtime_evaluate" => {
                self.cdp_tools
                    .evaluate_javascript(arguments, &mut self.cdp_server)
                    .await
            }
            "cdp_dom_get_document" => {
                self.cdp_tools
                    .get_document(arguments, &mut self.cdp_server)
                    .await
            }
            "cdp_dom_query_selector" => {
                self.cdp_tools
                    .query_selector(arguments, &mut self.cdp_server)
                    .await
            }
            "cdp_dom_get_attributes" => {
                self.cdp_tools
                    .get_attributes(arguments, &mut self.cdp_server)
                    .await
            }
            "cdp_dom_get_computed_style" => {
                self.cdp_tools
                    .get_computed_style(arguments, &mut self.cdp_server)
                    .await
            }
            "cdp_network_get_cookies" => {
                self.cdp_tools
                    .get_cookies(arguments, &mut self.cdp_server)
                    .await
            }
            "cdp_network_set_cookie" => {
                self.cdp_tools
                    .set_cookie(arguments, &mut self.cdp_server)
                    .await
            }
            "cdp_console_get_messages" => {
                self.cdp_tools
                    .get_console_messages(arguments, &mut self.cdp_server)
                    .await
            }
            "cdp_page_screenshot" => {
                self.cdp_tools
                    .take_screenshot(arguments, &mut self.cdp_server)
                    .await
            }
            "cdp_page_reload" => {
                self.cdp_tools
                    .reload_page(arguments, &mut self.cdp_server)
                    .await
            }

            // Unified snapshot tool (captures point-in-time page snapshot)
            "snapshot_url" => self.handle_snapshot_url(arguments).await,

            // Web search tools
            "web_search" => self.web_search(arguments).await,
            "image_search" => self.image_search(arguments).await,

            // Session management tools
            "browser_session_management" => {
                self.browser_tools
                    .handle_session_management(arguments)
                    .await
            }
            "browser_get_page_content" => {
                self.browser_tools.handle_get_page_content(arguments).await
            }
            "browse_readable_content" => self.browse_readable_content(arguments).await,

            // Browser automation tools
            "browser_fill_form" => self.browser_tools.handle_fill_form(arguments).await,
            "browser_click_element" => self.browser_tools.handle_click_element(arguments).await,
            "browser_type_text" => self.browser_tools.handle_type_text(arguments).await,
            "browser_wait_for_element" => {
                self.browser_tools.handle_wait_for_element(arguments).await
            }
            "browser_prepare_form_submission" => {
                self.browser_tools
                    .handle_prepare_form_submission(arguments)
                    .await
            }
            "browser_validate_session" => {
                self.browser_tools.handle_validate_session(arguments).await
            }

            // Navigation tools
            "browser_refresh_page" => self.browser_tools.handle_refresh_page(arguments).await,
            "browser_navigate_forward" => {
                self.browser_tools.handle_navigate_forward(arguments).await
            }
            "browser_navigate_back" => self.browser_tools.handle_navigate_back(arguments).await,
            "browser_navigate_to" => self.browser_tools.handle_navigate_to(arguments).await,

            // ── BrainClaw agent-friendly aliases ────────────────────────────────
            // One-shot read: navigate + extract markdown in a single call
            "browser_read_url" => self.handle_browser_read_url(arguments).await,
            // Navigation
            "browser_navigate" => self.browser_tools.handle_navigate_to(arguments).await,
            // Interaction
            "browser_click" => self.browser_tools.handle_click_element(arguments).await,
            "browser_fill" => self.browser_tools.handle_fill_form(arguments).await,
            // CDP
            "browser_eval" => {
                self.cdp_tools
                    .evaluate_javascript(arguments, &mut self.cdp_server)
                    .await
            }
            "browser_screenshot" => {
                self.cdp_tools
                    .take_screenshot(arguments, &mut self.cdp_server)
                    .await
            }
            // Extraction / search
            "browser_extract" => self.handle_snapshot_url(arguments).await,
            "browser_search" => self.web_search(arguments).await,

            // WASM Debug tools
            #[cfg(feature = "wasm-debug")]
            "wasm_debug_load_module"
            | "wasm_debug_unload_module"
            | "wasm_debug_list_modules"
            | "wasm_debug_validate"
            | "wasm_debug_inspect"
            | "wasm_debug_disassemble"
            | "wasm_debug_read_memory"
            | "wasm_debug_write_memory"
            | "wasm_debug_call_function"
            | "wasm_debug_profile_function" => self.route_wasm_debug_tool(name, arguments).await,

            // Accessibility tools
            "get_accessibility_tree" => {
                let session_id = arguments.get("session_id").and_then(|v| v.as_str()).unwrap_or("default");
                let max_depth = arguments.get("max_depth").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

                let browser = self.browser_tools.get_or_create_session(session_id, false);
                match browser.lock() {
                    Ok(guard) => {
                        let content = guard.get_current_content();
                        if content.is_empty() {
                            McpResponse::error(-1, "No page content available. Navigate to a page first.".to_string())
                        } else {
                            let tree = crate::engine::browser::accessibility::build_accessibility_tree(&content, max_depth);
                            McpResponse::success(tree)
                        }
                    }
                    Err(_) => McpResponse::error(-1, "Failed to acquire browser lock".to_string()),
                }
            }

            // Unknown/Unhandled tool
            _ => McpResponse::error(-32601, format!("Tool not found: {}", name)),
        }
    }

    /// `browser_read_url` — navigate to a URL and return clean markdown in one shot.
    ///
    /// Forwards to `handle_snapshot_url` with `extract_readable=true` and `format="markdown"`
    /// preset, while preserving any extra arguments the caller supplied.
    async fn handle_browser_read_url(&mut self, mut arguments: serde_json::Value) -> McpResponse {
        // Default to readable markdown extraction
        if arguments.get("extract_readable").is_none() {
            arguments["extract_readable"] = serde_json::json!(true);
        }
        if arguments.get("format").is_none() {
            arguments["format"] = serde_json::json!("markdown");
        }
        // extract_basic true by default in snapshot_url already; keep that behaviour
        self.handle_snapshot_url(arguments).await
    }

    /// Route WASM debug tool calls to the WasmDebugTools handler
    #[cfg(feature = "wasm-debug")]
    async fn route_wasm_debug_tool(&mut self, name: &str, arguments: Value) -> McpResponse {
        let tools = match self.wasm_debug_tools.as_mut() {
            Some(t) => t,
            None => {
                return McpResponse::error(
                    -32603,
                    "WASM debug tools are not initialized. Set THALORA_ENABLE_WASM_DEBUG=true"
                        .to_string(),
                );
            }
        };

        let result = match name {
            "wasm_debug_load_module" => tools.load_module(arguments),
            "wasm_debug_unload_module" => tools.unload_module(arguments),
            "wasm_debug_list_modules" => Ok(tools.list_modules()),
            "wasm_debug_validate" => tools.validate(arguments),
            "wasm_debug_inspect" => tools.inspect(arguments),
            "wasm_debug_disassemble" => tools.disassemble(arguments),
            "wasm_debug_read_memory" => tools.read_memory(arguments),
            "wasm_debug_write_memory" => tools.write_memory(arguments),
            "wasm_debug_call_function" => tools.call_function(arguments).await,
            "wasm_debug_profile_function" => tools.profile_function(arguments).await,
            _ => return McpResponse::error(-32601, format!("Unknown wasm debug tool: {}", name)),
        };

        match result {
            Ok(value) => McpResponse::success(serde_json::json!({
                "type": "text",
                "text": serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{}", value))
            })),
            Err(e) => McpResponse::error(-1, format!("Error: {e}")),
        }
    }
}
