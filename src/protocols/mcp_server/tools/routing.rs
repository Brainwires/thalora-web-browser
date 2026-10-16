use crate::protocols::mcp::McpResponse;
use crate::protocols::mcp_server::core::McpServer;
use serde_json::Value;

impl McpServer {
    /// Route tool calls to appropriate handlers
    pub(super) async fn route_tool_call(&mut self, name: &str, arguments: Value) -> McpResponse {
        match name {
            // Credential vault tools
            "vault_store_credential" => {
                crate::protocols::credential_tools::handle_store_credential(
                    &arguments,
                    &mut self.vault,
                )
            }
            "vault_list_credentials" => {
                crate::protocols::credential_tools::handle_list_credentials(&arguments, &self.vault)
            }
            "vault_delete_credential" => {
                crate::protocols::credential_tools::handle_delete_credential(
                    &arguments,
                    &mut self.vault,
                )
            }
            "browser_fill_credential" => self.handle_fill_credential(arguments).await,

            // JavaScript evaluation (the tool name predates the removal of the
            // mock CDP layer; a real CDP endpoint is planned)
            "cdp_runtime_evaluate" => {
                self.browser_tools
                    .handle_evaluate_javascript(arguments)
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
            "browser_snapshot" => self.browser_tools.handle_snapshot(arguments).await,
            "browser_wait" => self.browser_tools.handle_wait(arguments).await,
            "browser_select_option"
            | "browser_check"
            | "browser_press_key"
            | "browser_hover"
            | "browser_scroll" => {
                self.browser_tools
                    .handle_element_action(name, arguments)
                    .await
            }
            "browser_console_messages" => {
                self.browser_tools.handle_console_messages(arguments).await
            }

            // ── BrainClaw agent-friendly aliases ────────────────────────────────
            // One-shot read: navigate + extract markdown in a single call
            "browser_read_url" => self.handle_browser_read_url(arguments).await,
            // Navigation
            "browser_navigate" => self.browser_tools.handle_navigate_to(arguments).await,
            // Interaction
            "browser_click" => self.browser_tools.handle_click_element(arguments).await,
            "browser_fill" => self.browser_tools.handle_fill_field(arguments).await,
            "browser_eval" => {
                self.browser_tools
                    .handle_evaluate_javascript(arguments)
                    .await
            }
            "browser_screenshot" => self.browser_tools.handle_screenshot(arguments).await,
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
                let session_id = arguments
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                let max_depth = arguments
                    .get("max_depth")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as usize;

                let browser = match self.browser_tools.get_session(session_id) {
                    Ok(browser) => browser,
                    Err(e) => return McpResponse::error(-32602, e),
                };
                match crate::protocols::browser_tools::core::page_state(&browser).await {
                    Ok((page_url, content)) => {
                        if content.is_empty() {
                            McpResponse::error(
                                -1,
                                "No page content available. Navigate to a page first.".to_string(),
                            )
                        } else {
                            let tree =
                                crate::engine::browser::accessibility::build_accessibility_tree(
                                    &content, max_depth,
                                );
                            McpResponse::page_content(page_url.as_deref(), tree)
                        }
                    }
                    Err(e) => McpResponse::error(-1, e),
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
