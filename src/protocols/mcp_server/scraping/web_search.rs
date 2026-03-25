use serde_json::Value;
use std::error::Error;

use crate::protocols::mcp::McpResponse;
use crate::protocols::mcp_server::core::McpServer;
use crate::protocols::security::{MAX_QUERY_LENGTH, limit_input_length};

use super::search;

impl McpServer {
    pub(in crate::protocols::mcp_server) async fn web_search(
        &mut self,
        arguments: Value,
    ) -> McpResponse {
        eprintln!("🔍 DEBUG: Starting web_search function");
        let query = arguments["query"].as_str().unwrap_or("");
        let num_results = arguments["num_results"].as_u64().unwrap_or(10) as usize;
        let search_engine = arguments["search_engine"].as_str().unwrap_or("duckduckgo");
        eprintln!(
            "🔍 DEBUG: Parameters - query: {}, num_results: {}, engine: {}",
            query, num_results, search_engine
        );

        if query.is_empty() {
            return McpResponse::error(-1, "Query parameter is required".to_string());
        }

        // SECURITY: Validate query length to prevent DoS attacks
        if let Err(e) = limit_input_length(query, MAX_QUERY_LENGTH, "Search query") {
            return McpResponse::error(-32602, format!("Input validation failed: {}", e));
        }

        let num_results = num_results.min(20); // Cap at 20 results
        eprintln!("🔍 DEBUG: About to call perform_web_search");

        // Execute search with comprehensive error handling
        // Note: We can't use tokio::spawn here because HeadlessWebBrowser is not Send/Sync
        // The panic hook installed in main.rs will catch any panics
        let search_result = search::perform_search(query, num_results, search_engine).await;

        match search_result {
            Ok(results) => {
                eprintln!(
                    "🔍 DEBUG: perform_web_search succeeded with {} results",
                    results.results.len()
                );
                // Wrap result in MCP text content format
                let results_json = serde_json::to_value(&results).unwrap_or_default();
                let mcp_content = serde_json::json!({
                    "type": "text",
                    "text": serde_json::to_string_pretty(&results_json).unwrap_or_else(|_| "[]".to_string())
                });
                McpResponse::success(mcp_content)
            }
            Err(e) => {
                // Provide detailed error information
                let error_chain = format_error_chain(&e);
                eprintln!("❌ ERROR: perform_web_search failed:\n{}", error_chain);

                // Return a detailed error message
                McpResponse::error(-1, format!("Web search failed: {}", error_chain))
            }
        }
    }
}

/// Format an error with its full chain of causes for better debugging
fn format_error_chain(error: &anyhow::Error) -> String {
    let mut message = error.to_string();
    let mut current: Option<&(dyn Error + 'static)> = error.source();
    let mut depth = 0;

    while let Some(cause) = current {
        depth += 1;
        message.push_str(&format!("\n  └─ Cause {}: {}", depth, cause));
        current = cause.source();
    }

    // Add backtrace hint if available
    let backtrace = error.backtrace();
    let backtrace_str = backtrace.to_string();
    if !backtrace_str.is_empty() && backtrace_str != "disabled backtrace" {
        message.push_str("\n\nBacktrace available - set RUST_BACKTRACE=1 for details");
    }

    message
}
