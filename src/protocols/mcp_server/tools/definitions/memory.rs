use serde_json::Value;

/// AI Memory tool definitions for storing and retrieving research data, credentials, bookmarks, and notes
pub(crate) fn get_memory_tool_definitions() -> Vec<Value> {
    vec![
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
        serde_json::json!({
            "name": "ai_memory_store_credentials",
            "description": "Securely store credentials (passwords, API keys, tokens) in encrypted AI memory",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "service": {
                        "type": "string",
                        "description": "Service name or identifier for the credentials"
                    },
                    "username": {
                        "type": "string",
                        "description": "Username or identifier"
                    },
                    "password": {
                        "type": "string",
                        "description": "Password or secret value"
                    },
                    "additional_data": {
                        "type": "object",
                        "description": "Additional credential data (API keys, tokens, etc.)"
                    },
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Optional tags for categorization"
                    }
                },
                "required": ["service", "username", "password"]
            }
        }),
        serde_json::json!({
            "name": "ai_memory_get_credentials",
            "description": "Retrieve stored credentials from encrypted AI memory",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "service": {
                        "type": "string",
                        "description": "Service name to retrieve credentials for"
                    },
                    "username": {
                        "type": "string",
                        "description": "Username to filter by (optional)"
                    }
                },
                "required": ["service"]
            }
        }),
        serde_json::json!({
            "name": "ai_memory_store_bookmark",
            "description": "Store bookmark with URL, title, and metadata in AI memory",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL of the bookmark"
                    },
                    "title": {
                        "type": "string",
                        "description": "Title or name of the bookmark"
                    },
                    "description": {
                        "type": "string",
                        "description": "Optional description or notes"
                    },
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Tags for categorization"
                    },
                    "folder": {
                        "type": "string",
                        "description": "Optional folder or category"
                    }
                },
                "required": ["url", "title"]
            }
        }),
        serde_json::json!({
            "name": "ai_memory_get_bookmarks",
            "description": "Retrieve stored bookmarks from AI memory",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Filter by tags"
                    },
                    "folder": {
                        "type": "string",
                        "description": "Filter by folder"
                    },
                    "query": {
                        "type": "string",
                        "description": "Search query for title or description"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of results (default: 20)"
                    }
                }
            }
        }),
        serde_json::json!({
            "name": "ai_memory_store_note",
            "description": "Store notes and documentation in AI memory",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Note title"
                    },
                    "content": {
                        "type": "string",
                        "description": "Note content (supports markdown)"
                    },
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Tags for categorization"
                    },
                    "category": {
                        "type": "string",
                        "description": "Note category (optional)"
                    }
                },
                "required": ["title", "content"]
            }
        }),
        serde_json::json!({
            "name": "ai_memory_get_notes",
            "description": "Retrieve stored notes from AI memory",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Filter by tags"
                    },
                    "category": {
                        "type": "string",
                        "description": "Filter by category"
                    },
                    "query": {
                        "type": "string",
                        "description": "Search query for title or content"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of results (default: 20)"
                    }
                }
            }
        }),
    ]
}
