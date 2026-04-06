use serde_json::Value;

use crate::features::ai_memory::AiMemoryHeap;
use crate::protocols::mcp::McpResponse;
use crate::protocols::security::{MAX_QUERY_LENGTH, limit_input_length};

// Submodules
mod bookmarks;
mod credentials;
mod notes;
mod research;
mod sessions;

// Re-export for backward compatibility
pub use bookmarks::*;
pub use credentials::*;
pub use notes::*;
pub use research::*;
pub use sessions::*;

pub struct MemoryTools {}

impl Default for MemoryTools {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryTools {
    pub fn new() -> Self {
        Self {}
    }

    // Research operations
    pub async fn store_research(
        &mut self,
        args: Value,
        ai_memory: &mut AiMemoryHeap,
    ) -> McpResponse {
        research::handle_store_research(args, ai_memory).await
    }

    // Credentials operations
    pub async fn store_credentials(
        &mut self,
        args: Value,
        ai_memory: &mut AiMemoryHeap,
    ) -> McpResponse {
        credentials::handle_store_credentials(args, ai_memory).await
    }

    pub async fn get_credentials(
        &mut self,
        args: Value,
        ai_memory: &mut AiMemoryHeap,
    ) -> McpResponse {
        credentials::handle_retrieve_credentials(args, ai_memory).await
    }

    // Bookmark operations
    pub async fn store_bookmark(
        &mut self,
        args: Value,
        ai_memory: &mut AiMemoryHeap,
    ) -> McpResponse {
        bookmarks::handle_store_bookmark(args, ai_memory).await
    }

    // Note operations
    pub async fn store_note(&mut self, args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
        notes::handle_store_note(args, ai_memory).await
    }

    // Search operations (cross-category)
    pub async fn search(&mut self, args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
        // Search across all categories
        use crate::features::ai_memory::{MemorySearchCriteria, MemorySortBy};

        let query = args.get("query").and_then(|v| v.as_str());
        let category = args.get("category").and_then(|v| v.as_str());
        let tags = args.get("tags").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        });
        let limit = args
            .get("limit")
            .and_then(|v| v.as_i64())
            .map(|l| l as usize);

        // SECURITY: Validate query length if provided
        if let Some(q) = query
            && let Err(e) = limit_input_length(q, MAX_QUERY_LENGTH, "Search query")
        {
            return McpResponse::error(-1, format!("Input validation failed: {}", e));
        }

        let criteria = MemorySearchCriteria {
            query: query.map(|s| s.to_string()),
            tags,
            date_range: None,
            category: category.map(|s| s.to_string()),
            limit,
            sort_by: MemorySortBy::UpdatedAt,
        };

        let mut results = serde_json::json!({
            "query": query,
            "category": category,
            "results": {}
        });

        match category {
            Some("research") => {
                let research_results = ai_memory.search_research(&criteria);
                if !research_results.is_empty() {
                    results["results"]["research"] = serde_json::json!(research_results);
                }
            }
            Some("bookmarks") => {
                let bookmark_results = ai_memory.search_bookmarks(&criteria);
                if !bookmark_results.is_empty() {
                    results["results"]["bookmarks"] = serde_json::json!(bookmark_results);
                }
            }
            Some("notes") => {
                let note_results = ai_memory.search_notes(&criteria);
                if !note_results.is_empty() {
                    results["results"]["notes"] = serde_json::json!(note_results);
                }
            }
            None => {
                // Search all categories when no specific category is requested
                let research_results = ai_memory.search_research(&criteria);
                if !research_results.is_empty() {
                    results["results"]["research"] = serde_json::json!(research_results);
                }

                let bookmark_results = ai_memory.search_bookmarks(&criteria);
                if !bookmark_results.is_empty() {
                    results["results"]["bookmarks"] = serde_json::json!(bookmark_results);
                }

                let note_results = ai_memory.search_notes(&criteria);
                if !note_results.is_empty() {
                    results["results"]["notes"] = serde_json::json!(note_results);
                }
            }
            _ => {}
        }

        McpResponse::success(serde_json::json!({
            "type": "text",
            "text": serde_json::to_string_pretty(&results).unwrap_or_default()
        }))
    }

    // Session operations
    pub async fn start_session(
        &mut self,
        args: Value,
        ai_memory: &mut AiMemoryHeap,
    ) -> McpResponse {
        sessions::handle_start_session(args, ai_memory).await
    }

    pub async fn update_session(
        &mut self,
        args: Value,
        ai_memory: &mut AiMemoryHeap,
    ) -> McpResponse {
        sessions::handle_update_session(args, ai_memory).await
    }

    // Statistics
    pub async fn get_statistics(
        &mut self,
        _args: Value,
        ai_memory: &mut AiMemoryHeap,
    ) -> McpResponse {
        let stats = ai_memory.get_statistics();

        let stats_json = serde_json::json!({
            "research_entries": stats.research_count,
            "credential_entries": stats.credential_count,
            "session_entries": stats.session_count,
            "bookmark_entries": stats.bookmark_count,
            "note_entries": stats.note_count,
            "total_entries": stats.total_entries,
            "file_size_bytes": stats.file_size_bytes,
            "last_updated": stats.last_updated,
            "version": stats.version
        });

        McpResponse::success(serde_json::json!({
            "type": "text",
            "text": serde_json::to_string_pretty(&stats_json).unwrap_or_default()
        }))
    }
}
