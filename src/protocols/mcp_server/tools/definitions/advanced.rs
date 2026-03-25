use serde_json::Value;

/// Advanced tool definitions for PDF extraction, downloads, and network interception
pub(crate) fn get_advanced_tool_definitions() -> Vec<Value> {
    vec![
        // PDF Extraction Tool
        serde_json::json!({
            "name": "extract_pdf",
            "description": "Extract text, links, and metadata from a PDF document. Supports both URL-based and base64-encoded PDFs.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL of the PDF to extract (optional if pdf_base64 provided)"
                    },
                    "pdf_base64": {
                        "type": "string",
                        "description": "Base64-encoded PDF content (optional if url provided)"
                    },
                    "extract_images": {
                        "type": "boolean",
                        "description": "Whether to extract embedded images (default: false)"
                    },
                    "extract_tables": {
                        "type": "boolean",
                        "description": "Whether to extract table structures (default: false)"
                    },
                    "page_range": {
                        "type": "string",
                        "description": "Page range to extract (e.g., '1-5', '1,3,5', default: all pages)"
                    },
                    "output_format": {
                        "type": "string",
                        "enum": ["text", "markdown", "json"],
                        "description": "Output format for extracted content (default: markdown)"
                    }
                }
            }
        }),
        // Download File Tool
        serde_json::json!({
            "name": "download_file",
            "description": "Download a file from a URL. Returns the file content, size, and metadata.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL of the file to download"
                    },
                    "save_path": {
                        "type": "string",
                        "description": "Optional path to save the file locally"
                    },
                    "return_base64": {
                        "type": "boolean",
                        "description": "Whether to return file content as base64 (default: true for small files)"
                    },
                    "max_size_mb": {
                        "type": "number",
                        "description": "Maximum file size in MB to download (default: 50)"
                    },
                    "timeout_seconds": {
                        "type": "number",
                        "description": "Download timeout in seconds (default: 60)"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID for authenticated downloads (optional)"
                    }
                },
                "required": ["url"]
            }
        }),
        // Network Request Interception Tool
        serde_json::json!({
            "name": "intercept_requests",
            "description": "Configure network request interception patterns for monitoring or modifying requests.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID"
                    },
                    "patterns": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "url_pattern": {
                                    "type": "string",
                                    "description": "URL pattern to intercept (supports glob)"
                                },
                                "resource_type": {
                                    "type": "string",
                                    "enum": ["document", "stylesheet", "script", "image", "font", "xhr", "fetch", "websocket", "other"],
                                    "description": "Type of resource to intercept"
                                },
                                "action": {
                                    "type": "string",
                                    "enum": ["block", "modify", "log"],
                                    "description": "Action to take on matched requests"
                                }
                            },
                            "required": ["url_pattern"]
                        },
                        "description": "Array of interception patterns"
                    }
                },
                "required": ["session_id", "patterns"]
            }
        }),
        // Get Intercepted Requests Tool
        serde_json::json!({
            "name": "get_intercepted_requests",
            "description": "Get all intercepted network requests from a browser session.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID"
                    },
                    "filter_type": {
                        "type": "string",
                        "enum": ["all", "xhr", "fetch", "document", "script", "stylesheet", "image"],
                        "description": "Filter by request type (default: all)"
                    },
                    "include_response_body": {
                        "type": "boolean",
                        "description": "Whether to include response bodies (default: false)"
                    },
                    "max_results": {
                        "type": "number",
                        "description": "Maximum number of requests to return (default: 100)"
                    }
                },
                "required": ["session_id"]
            }
        }),
        // Screenshot PDF Tool
        serde_json::json!({
            "name": "page_to_pdf",
            "description": "Convert the current page to a PDF document.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to convert to PDF (optional if session_id provided)"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "Browser session ID (optional)"
                    },
                    "paper_format": {
                        "type": "string",
                        "enum": ["letter", "legal", "a4", "a3"],
                        "description": "Paper format (default: a4)"
                    },
                    "landscape": {
                        "type": "boolean",
                        "description": "Landscape orientation (default: false)"
                    },
                    "print_background": {
                        "type": "boolean",
                        "description": "Include background graphics (default: true)"
                    },
                    "margin": {
                        "type": "object",
                        "properties": {
                            "top": { "type": "string" },
                            "right": { "type": "string" },
                            "bottom": { "type": "string" },
                            "left": { "type": "string" }
                        },
                        "description": "Page margins (default: 0.5in on all sides)"
                    },
                    "save_path": {
                        "type": "string",
                        "description": "Optional path to save the PDF locally"
                    }
                }
            }
        }),
    ]
}
