use serde_json::Value;

/// Get accessibility tool definitions for MCP
pub(crate) fn get_accessibility_tool_definitions() -> Vec<Value> {
    vec![serde_json::json!({
        "name": "get_accessibility_tree",
        "description": "Get the accessibility tree for the current page with semantic ARIA roles, accessible names, and states. Returns a structured tree computed from HTML elements and ARIA attributes per the WAI-ARIA and HTML-AAM specifications. Useful for understanding page semantics, finding interactive elements, and navigating by landmarks.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "Browser session ID to get the accessibility tree for"
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum depth of the tree (default: 10)",
                    "default": 10
                }
            }
        }
    })]
}
