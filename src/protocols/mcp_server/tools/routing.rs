use serde_json::Value;
use crate::protocols::mcp::McpResponse;
use crate::protocols::mcp_server::core::McpServer;

impl McpServer {
    /// Route tool calls to appropriate handlers
    pub(super) async fn route_tool_call(&mut self, name: &str, arguments: Value) -> McpResponse {
        match name {
            // AI Memory tools
            "ai_memory_store_research" => self.memory_tools.store_research(arguments, &mut self.ai_memory).await,
            "ai_memory_get_research" => {
                // Validate required "key" parameter
                let key = match arguments.get("key").and_then(|v| v.as_str()) {
                    Some(key) => key,
                    None => {
                        return McpResponse::ToolResult {
                            content: vec![serde_json::json!({
                                "type": "text",
                                "text": "Missing required parameter: key"
                            })],
                            is_error: true,
                        };
                    }
                };

                // Convert key to query for search
                let search_args = serde_json::json!({
                    "query": key,
                    "category": "research",
                    "limit": 1
                });
                self.memory_tools.search(search_args, &mut self.ai_memory).await
            },
            "ai_memory_search_research" => self.memory_tools.search(arguments, &mut self.ai_memory).await,
            "ai_memory_store_credentials" => self.memory_tools.store_credentials(arguments, &mut self.ai_memory).await,
            "ai_memory_get_credentials" => self.memory_tools.get_credentials(arguments, &mut self.ai_memory).await,
            "ai_memory_store_bookmark" => self.memory_tools.store_bookmark(arguments, &mut self.ai_memory).await,
            "ai_memory_get_bookmarks" => self.memory_tools.search(arguments, &mut self.ai_memory).await,
            "ai_memory_store_note" => self.memory_tools.store_note(arguments, &mut self.ai_memory).await,
            "ai_memory_get_notes" => self.memory_tools.search(arguments, &mut self.ai_memory).await,

            // Chrome DevTools Protocol tools - comprehensive debugging toolkit
            "cdp_runtime_evaluate" => self.cdp_tools.evaluate_javascript(arguments, &mut self.cdp_server).await,
            "cdp_dom_get_document" => self.cdp_tools.get_document(arguments, &mut self.cdp_server).await,
            "cdp_dom_query_selector" => self.cdp_tools.query_selector(arguments, &mut self.cdp_server).await,
            "cdp_dom_get_attributes" => self.cdp_tools.get_attributes(arguments, &mut self.cdp_server).await,
            "cdp_dom_get_computed_style" => self.cdp_tools.get_computed_style(arguments, &mut self.cdp_server).await,
            "cdp_network_get_cookies" => self.cdp_tools.get_cookies(arguments, &mut self.cdp_server).await,
            "cdp_network_set_cookie" => self.cdp_tools.set_cookie(arguments, &mut self.cdp_server).await,
            "cdp_console_get_messages" => self.cdp_tools.get_console_messages(arguments, &mut self.cdp_server).await,
            "cdp_page_screenshot" => self.cdp_tools.take_screenshot(arguments, &mut self.cdp_server).await,
            "cdp_page_reload" => self.cdp_tools.reload_page(arguments, &mut self.cdp_server).await,

            // Unified scraping tool
            "scrape" => self.scrape_unified(arguments).await,

            // Web search tools
            "web_search" => self.web_search(arguments).await,
            "image_search" => self.image_search(arguments).await,

            // Session management tools
            "browser_session_management" => self.browser_tools.handle_session_management(arguments).await,
            "browser_get_page_content" => self.browser_tools.handle_get_page_content(arguments).await,
            "browse_readable_content" => self.browse_readable_content(arguments).await,

            // Browser automation tools
            "browser_fill_form" => self.browser_tools.handle_fill_form(arguments).await,
            "browser_click_element" => self.browser_tools.handle_click_element(arguments).await,
            "browser_type_text" => self.browser_tools.handle_type_text(arguments).await,
            "browser_wait_for_element" => self.browser_tools.handle_wait_for_element(arguments).await,
            "browser_prepare_form_submission" => self.browser_tools.handle_prepare_form_submission(arguments).await,
            "browser_validate_session" => self.browser_tools.handle_validate_session(arguments).await,

            // Navigation tools
            "browser_refresh_page" => self.browser_tools.handle_refresh_page(arguments).await,
            "browser_navigate_forward" => self.browser_tools.handle_navigate_forward(arguments).await,
            "browser_navigate_back" => self.browser_tools.handle_navigate_back(arguments).await,
            "browser_navigate_to" => self.browser_tools.handle_navigate_to(arguments).await,

            // Unknown/Unhandled tool
            _ => McpResponse::error(-32601, format!("Tool not found: {}", name))
        }
    }
}
