use serde_json::Value;
use chrono::Utc;
use std::collections::HashMap;

use crate::protocols::mcp::McpResponse;
use crate::features::ai_memory::{AiMemoryHeap, ResearchEntry, MemorySearchCriteria, MemorySortBy, NotePriority};

pub struct MemoryTools {
}

impl MemoryTools {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn store_research(&mut self, args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
        let key = match args.get("key").and_then(|v| v.as_str()) {
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

        let topic = match args.get("topic").and_then(|v| v.as_str()) {
            Some(topic) => topic,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: topic"
                    })],
                    is_error: true,
                };
            }
        };

        let summary = match args.get("summary").and_then(|v| v.as_str()) {
            Some(summary) => summary,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: summary"
                    })],
                    is_error: true,
                };
            }
        };

        let findings = args.get("findings")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(Vec::new);

        let sources = args.get("sources")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(Vec::new);

        let tags = args.get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(Vec::new);

        let confidence_score = args.get("confidence_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);

        let related_topics = args.get("related_topics")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(Vec::new);

        let research_entry = ResearchEntry {
            topic: topic.to_string(),
            summary: summary.to_string(),
            findings,
            sources,
            tags,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            confidence_score,
            related_topics,
        };

        match ai_memory.store_research(key, research_entry) {
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("Research entry '{}' stored successfully in AI memory heap", key)
                })],
                is_error: false,
            },
            Err(e) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("Failed to store research entry: {}", e)
                })],
                is_error: true,
            }
        }
    }

    pub async fn store_credentials(&mut self, args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
        let key = match args.get("key").and_then(|v| v.as_str()) {
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

        let service = match args.get("service").and_then(|v| v.as_str()) {
            Some(service) => service,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: service"
                    })],
                    is_error: true,
                };
            }
        };

        let username = match args.get("username").and_then(|v| v.as_str()) {
            Some(username) => username,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: username"
                    })],
                    is_error: true,
                };
            }
        };

        let password = match args.get("password").and_then(|v| v.as_str()) {
            Some(password) => password,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: password"
                    })],
                    is_error: true,
                };
            }
        };

        let additional_data: HashMap<String, String> = args.get("additional_data")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_else(HashMap::new);

        match ai_memory.store_credentials(key, service, username, password, additional_data) {
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("Credentials for '{}' stored securely in AI memory heap", service)
                })],
                is_error: false,
            },
            Err(e) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("Failed to store credentials: {}", e)
                })],
                is_error: true,
            }
        }
    }

    pub async fn get_credentials(&mut self, args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
        let key = match args.get("key").and_then(|v| v.as_str()) {
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

        match ai_memory.get_credentials(key) {
            Ok(Some((service, username, password, additional_data))) => {
                let response_json = serde_json::json!({
                    "service": service,
                    "username": username,
                    "password": password,
                    "additional_data": additional_data,
                    "retrieved_from": "ai_memory_heap"
                });

                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": serde_json::to_string_pretty(&response_json).unwrap_or_default()
                    })],
                    is_error: false,
                }
            },
            Ok(None) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("No credentials found for key: {}", key)
                })],
                is_error: false,
            },
            Err(e) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("Failed to retrieve credentials: {}", e)
                })],
                is_error: true,
            }
        }
    }

    pub async fn store_bookmark(&mut self, args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
        let key = match args.get("key").and_then(|v| v.as_str()) {
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

        let url = match args.get("url").and_then(|v| v.as_str()) {
            Some(url) => url,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: url"
                    })],
                    is_error: true,
                };
            }
        };

        let title = match args.get("title").and_then(|v| v.as_str()) {
            Some(title) => title,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: title"
                    })],
                    is_error: true,
                };
            }
        };

        let description = args.get("description").and_then(|v| v.as_str()).unwrap_or("");
        let content_preview = args.get("content_preview").and_then(|v| v.as_str()).unwrap_or("");
        
        let tags = args.get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(Vec::new);

        match ai_memory.store_bookmark(key, url, title, description, content_preview, tags) {
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("Bookmark '{}' stored successfully in AI memory heap", title)
                })],
                is_error: false,
            },
            Err(e) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("Failed to store bookmark: {}", e)
                })],
                is_error: true,
            }
        }
    }

    pub async fn store_note(&mut self, args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
        let key = match args.get("key").and_then(|v| v.as_str()) {
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

        let title = match args.get("title").and_then(|v| v.as_str()) {
            Some(title) => title,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: title"
                    })],
                    is_error: true,
                };
            }
        };

        let content = match args.get("content").and_then(|v| v.as_str()) {
            Some(content) => content,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: content"
                    })],
                    is_error: true,
                };
            }
        };

        let category = args.get("category").and_then(|v| v.as_str()).unwrap_or("general");
        
        let tags = args.get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(Vec::new);

        let priority = args.get("priority")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "Low" => Some(NotePriority::Low),
                "Medium" => Some(NotePriority::Medium),
                "High" => Some(NotePriority::High),
                "Critical" => Some(NotePriority::Critical),
                _ => None,
            })
            .unwrap_or(NotePriority::Medium);

        match ai_memory.store_note(key, title, content, category, tags, priority) {
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("Note '{}' stored successfully in AI memory heap", title)
                })],
                is_error: false,
            },
            Err(e) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("Failed to store note: {}", e)
                })],
                is_error: true,
            }
        }
    }

    pub async fn search(&mut self, args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
        let query = args.get("query").and_then(|v| v.as_str());
        let category = args.get("category").and_then(|v| v.as_str());
        let tags = args.get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
        let limit = args.get("limit").and_then(|v| v.as_i64()).map(|l| l as usize);

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
            },
            Some("bookmarks") => {
                let bookmark_results = ai_memory.search_bookmarks(&criteria);
                if !bookmark_results.is_empty() {
                    results["results"]["bookmarks"] = serde_json::json!(bookmark_results);
                }
            },
            Some("notes") => {
                let note_results = ai_memory.search_notes(&criteria);
                if !note_results.is_empty() {
                    results["results"]["notes"] = serde_json::json!(note_results);
                }
            },
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
            },
            _ => {}
        }

        McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": serde_json::to_string_pretty(&results).unwrap_or_default()
            })],
            is_error: false,
        }
    }

    pub async fn start_session(&mut self, args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
        let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
            Some(session_id) => session_id,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: session_id"
                    })],
                    is_error: true,
                };
            }
        };

        let description = match args.get("description").and_then(|v| v.as_str()) {
            Some(description) => description,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: description"
                    })],
                    is_error: true,
                };
            }
        };

        let objectives = args.get("objectives")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(Vec::new);

        match ai_memory.start_session(session_id, description, objectives) {
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("Session '{}' started successfully in AI memory heap", session_id)
                })],
                is_error: false,
            },
            Err(e) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("Failed to start session: {}", e)
                })],
                is_error: true,
            }
        }
    }

    pub async fn update_session(&mut self, args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
        let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
            Some(session_id) => session_id,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: session_id"
                    })],
                    is_error: true,
                };
            }
        };

        if let Some(progress_key) = args.get("progress_key").and_then(|v| v.as_str()) {
            if let Some(progress_value) = args.get("progress_value") {
                match ai_memory.update_session_progress(session_id, progress_key, progress_value.clone()) {
                    Ok(_) => McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Session '{}' progress updated: {} = {:?}", session_id, progress_key, progress_value)
                        })],
                        is_error: false,
                    },
                    Err(e) => McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Failed to update session progress: {}", e)
                        })],
                        is_error: true,
                    }
                }
            } else {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: progress_value"
                    })],
                    is_error: true,
                }
            }
        } else {
            McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "No progress_key provided - no updates made"
                })],
                is_error: false,
            }
        }
    }

    pub async fn get_statistics(&mut self, _args: Value, ai_memory: &mut AiMemoryHeap) -> McpResponse {
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

        McpResponse::ToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": serde_json::to_string_pretty(&stats_json).unwrap_or_default()
            })],
            is_error: false,
        }
    }
}