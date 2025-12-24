use serde_json::Value;
use std::error::Error;

use crate::protocols::mcp::McpResponse;
use crate::protocols::mcp_server::core::McpServer;

use super::search;

impl McpServer {
    /// Handle image_search MCP tool
    pub(in crate::protocols::mcp_server) async fn image_search(&mut self, arguments: Value) -> McpResponse {
        eprintln!("🖼️ DEBUG: Starting image_search function");
        let query = arguments["query"].as_str().unwrap_or("");
        let num_results = arguments["num_results"].as_u64().unwrap_or(10) as usize;
        let search_engine = arguments["search_engine"].as_str().unwrap_or("duckduckgo");
        eprintln!("🖼️ DEBUG: Parameters - query: {}, num_results: {}, engine: {}", query, num_results, search_engine);

        if query.is_empty() {
            return McpResponse::error(-1, "Query parameter is required".to_string());
        }

        let num_results = num_results.min(20); // Cap at 20 results

        // Execute image search
        let search_result = search::perform_image_search(query, num_results, search_engine).await;

        match search_result {
            Ok(results) => {
                eprintln!("🖼️ DEBUG: image_search succeeded with {} results", results.results.len());
                let results_json = serde_json::to_value(&results).unwrap_or_default();
                let mcp_content = serde_json::json!({
                    "type": "text",
                    "text": serde_json::to_string_pretty(&results_json).unwrap_or_else(|_| "[]".to_string())
                });
                McpResponse::success(mcp_content)
            },
            Err(e) => {
                let error_chain = format_error_chain(&e);
                eprintln!("❌ ERROR: image_search failed:\n{}", error_chain);
                McpResponse::error(-1, format!("Image search failed: {}", error_chain))
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

    message
}
