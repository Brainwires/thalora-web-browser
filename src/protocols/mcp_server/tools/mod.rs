use serde_json::Value;
use crate::protocols::mcp::McpResponse;
use crate::protocols::mcp_server::core::McpServer;
use std::sync::Arc;
use vfs::{VfsInstance, set_current_vfs};
use std::env;

// Tool definition modules
mod definitions;

// Feature flag and routing modules
mod features;
mod routing;

// Re-export for internal use
use definitions::*;
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
    pub(super) fn get_tool_definitions(&self) -> Vec<Value> {
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

        tools
    }

    pub(super) async fn call_tool(&mut self, name: String, arguments: Value) -> McpResponse {
        eprintln!("🔍 DEBUG: call_tool - Tool: {}, Arguments: {}", name, arguments);

        // If a `session_id` argument is present, reuse or create a session-scoped VFS that persists across calls.
        let vfs_instance: Arc<VfsInstance>;
        let prev_vfs: Option<Arc<VfsInstance>>;
        if let Some(session_id) = arguments.get("session_id").and_then(|v| v.as_str()) {
            eprintln!("🔍 DEBUG: call_tool - Using session_id: {}", session_id);
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
        let resp = self.route_tool_call(&name, args_for_call).await;

        // Lifecycle:
        // - If ephemeral (no session_id): persist if `persistent=true`, otherwise delete backing file.
        // - If session-scoped: if `persistent=true` persist the session backing file; otherwise keep it in-memory for the session.
        let is_session = arguments.get("session_id").and_then(|v| v.as_str()).is_some();
        let should_persist = arguments.get("persistent").and_then(|v| v.as_bool()).unwrap_or(false);

        if is_session {
            if should_persist {
                if let Err(e) = vfs_instance.persist() {
                    drop(set_current_vfs(prev_vfs));
                    return McpResponse::error(-32001, format!("Failed to persist session VFS: {}", e));
                }
            }
            // for session VFS we keep the backing instance in `self.session_vfs` until explicit removal
        } else {
            if should_persist {
                if let Err(e) = vfs_instance.persist() {
                    drop(set_current_vfs(prev_vfs));
                    return McpResponse::error(-32002, format!("Failed to persist ephemeral VFS: {}", e));
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
