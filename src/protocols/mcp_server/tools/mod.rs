use crate::protocols::mcp::McpResponse;
use crate::protocols::mcp_server::core::McpServer;
use crate::protocols::rate_limiter::RateLimiter;
use serde_json::Value;
use std::env;
use std::sync::Arc;
use vfs::{VfsInstance, set_current_vfs};

// Tool definition modules
mod definitions;

// Feature flag and routing modules
pub(crate) mod features;
mod routing;

// Re-export for internal use
use definitions::*;
pub(crate) use features::is_brainclaw_preset;
use features::*;

/*
    Define the available tools for the MCP server, including AI memory management,
    Chrome DevTools Protocol interactions, web scraping, browser automation, and session management.
    Each tool includes a name, description, and input schema for validation.

    Tools are grouped into categories:
        - AI Memory Tools: Store and retrieve research data, credentials, bookmarks, and notes.
        - Chrome DevTools Protocol (CDP) Tools: Execute JavaScript, inspect DOM, manage cookies, capture screenshots, and retrieve console messages.
        - Web Scraping Tools: Scrape web pages and extract content.
        - Web Search Tools: Perform web searches using various search engines.
        - Browser Automation Tools: Interact with web pages by clicking elements and filling forms.
        - Session Management Tools: Create, manage, and clean up browser sessions for persistent AI interactions.

    Each category needs a corresponding environment variable to be enabled, e.g.:
        - THALORA_ENABLE_AI_MEMORY
        - THALORA_ENABLE_CDP (also enables SESSIONS automatically)
        - THALORA_ENABLE_SCRAPING (enabled by default - minimal toolset)
        - THALORA_ENABLE_SEARCH
        - THALORA_ENABLE_SESSIONS (browser automation + session management)

*/

impl McpServer {
    pub(crate) fn get_tool_definitions(&self) -> Vec<Value> {
        let mut tools = Vec::new();

        // Check MCP mode
        let mcp_mode = get_mcp_mode();
        eprintln!("🔧 Thalora MCP Mode: {}", mcp_mode);

        // In minimal mode, only expose basic scraping tools
        if is_minimal_mode() {
            return self.get_minimal_tool_definitions();
        }

        // Full mode continues below with all tools...

        // AI Memory Tools - Store and retrieve research data, credentials, bookmarks, and notes
        if is_ai_memory_enabled() {
            tools.extend(get_memory_tool_definitions());
        }

        // Chrome DevTools Protocol (CDP) Tools - Execute JavaScript, inspect DOM, manage cookies, capture screenshots, and retrieve console messages
        if is_cdp_enabled() {
            tools.extend(get_cdp_tool_definitions());
        }

        // Web Scraping Tools - Unified scraping tool that combines all capabilities (enabled by default)
        if is_scraping_enabled() {
            tools.extend(get_scraping_tool_definitions());
        }

        // Web Search Tools - Perform web searches using various search engines
        if is_search_enabled() {
            tools.extend(get_search_tool_definitions());
        }

        // Browser Automation Tools - Interact with web pages by clicking elements and filling forms
        if is_sessions_enabled() {
            tools.extend(get_browser_automation_tool_definitions());
        }

        // Session Management Tools - Create, manage, and clean up browser sessions for persistent AI interactions
        if is_sessions_enabled() {
            tools.extend(get_session_tool_definitions());
        }

        // Advanced Tools - PDF extraction, downloads, network interception
        // Available when scraping or sessions are enabled
        if is_scraping_enabled() || is_sessions_enabled() {
            tools.extend(get_advanced_tool_definitions());
        }

        // WASM Debug Tools - Load, inspect, disassemble, execute, and profile WASM modules
        #[cfg(feature = "wasm-debug")]
        if is_wasm_debug_enabled() {
            tools.extend(get_wasm_debug_tool_definitions());
        }

        // Accessibility Tools - Always enabled for AI semantic understanding
        tools.extend(get_accessibility_tool_definitions());

        // BrainClaw preset — add agent-friendly alias tools on top of the full toolset
        if is_brainclaw_preset() {
            tools.extend(get_brainclaw_alias_tool_definitions());
        }

        tools
    }

    pub(crate) async fn call_tool(&mut self, name: String, arguments: Value) -> McpResponse {
        eprintln!(
            "🔍 DEBUG: call_tool - Tool: {}, Arguments: {}",
            name, arguments
        );

        // SECURITY: Check rate limit before executing tool (DoS prevention)
        let category = RateLimiter::tool_to_category(&name);
        if let Err(wait_duration) = self.rate_limiter.check(category) {
            eprintln!(
                "⚠️ Rate limit exceeded for tool {} (category: {}), retry after {:.1}s",
                name,
                category,
                wait_duration.as_secs_f64()
            );
            return McpResponse::error(
                -32029,
                format!(
                    "Rate limit exceeded for {} operations. Retry after {:.1} seconds.",
                    category,
                    wait_duration.as_secs_f64()
                ),
            );
        }

        // If a `session_id` argument is present, reuse or create a session-scoped VFS that persists across calls.
        let vfs_instance: Arc<VfsInstance>;
        let prev_vfs: Option<Arc<VfsInstance>>;
        if let Some(session_id) = arguments.get("session_id").and_then(|v| v.as_str()) {
            eprintln!("🔍 DEBUG: call_tool - Using session_id: {}", session_id);
            // SECURITY: Reuse or create a session VFS with validated session_id
            let v = match self.get_or_create_session_vfs(session_id, None) {
                Ok(vfs) => vfs,
                Err(e) => return McpResponse::error(-32602, format!("Invalid session_id: {}", e)),
            };
            vfs_instance = v.clone();
            prev_vfs = set_current_vfs(Some(vfs_instance.clone()));
        } else {
            // ephemeral per-call VFS
            let tmp_dir = env::temp_dir();
            let v = match VfsInstance::new_temp_in_dir(&tmp_dir) {
                Ok(v) => Arc::new(v),
                Err(e) => {
                    return McpResponse::error(-32000, format!("Failed to create VFS: {}", e));
                }
            };
            prev_vfs = set_current_vfs(Some(v.clone()));
            vfs_instance = v;
        }

        // Run the tool while VFS is installed
        // Clone `arguments` for the call so we can still inspect the original after the call (lifecycle checks).
        let args_for_call = arguments.clone();

        // Execute the tool with proper error handling, logging, and timeout
        let start_time = std::time::Instant::now();
        eprintln!("🔧 Starting tool execution: {}", name);

        let resp = match tokio::time::timeout(
            std::time::Duration::from_secs(60),
            self.route_tool_call(&name, args_for_call),
        )
        .await
        {
            Ok(response) => response,
            Err(_) => {
                eprintln!("⚠️ Tool {} timed out after 60 seconds", name);
                McpResponse::error(
                    -32000,
                    format!("Tool '{}' timed out after 60 seconds", name),
                )
            }
        };

        let elapsed = start_time.elapsed();
        eprintln!("🔧 Tool execution completed: {} (took {:?})", name, elapsed);

        // Log if the response indicates an error
        if resp.is_error {
            eprintln!("⚠️ Tool {} returned error: {:?}", name, resp.content);
        }

        // Lifecycle:
        // - If ephemeral (no session_id): persist if `persistent=true`, otherwise delete backing file.
        // - If session-scoped: if `persistent=true` persist the session backing file with encryption; otherwise keep it in-memory for the session.
        let session_id_opt = arguments.get("session_id").and_then(|v| v.as_str());
        let should_persist = arguments
            .get("persistent")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if let Some(session_id) = session_id_opt {
            if should_persist {
                // SECURITY: Use encrypted persistence for session data at rest
                let key = vfs::derive_session_key(session_id);
                if let Err(e) = vfs_instance.persist_encrypted(&*key) {
                    drop(set_current_vfs(prev_vfs));
                    return McpResponse::error(
                        -32001,
                        format!("Failed to persist encrypted session VFS: {}", e),
                    );
                }
            }
            // for session VFS we keep the backing instance in `self.session_vfs` until explicit removal
        } else {
            if should_persist {
                // Ephemeral VFS uses unencrypted persistence (no session context)
                if let Err(e) = vfs_instance.persist() {
                    drop(set_current_vfs(prev_vfs));
                    return McpResponse::error(
                        -32002,
                        format!("Failed to persist ephemeral VFS: {}", e),
                    );
                }
            } else {
                drop(vfs_instance.delete_backing_file());
            }
        }

        // Restore previous VFS (if any)
        drop(set_current_vfs(prev_vfs));

        resp
    }

    /// Get minimal tool definitions for basic web scraping only
    /// This is the default MCP mode - stateless, simple, and reliable
    fn get_minimal_tool_definitions(&self) -> Vec<Value> {
        let mut tools = Vec::new();

        // Add minimal scraping tools
        tools.extend(get_minimal_scraping_tool_definitions());

        // Add minimal search tools
        tools.extend(get_minimal_search_tool_definitions());

        tools
    }
}
