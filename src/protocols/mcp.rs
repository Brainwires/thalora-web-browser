/// Internal tool response type used by all MCP handler code.
///
/// `McpResponse` is our own struct so that handler files compile without any
/// knowledge of rmcp types and without naming collisions.  When the
/// `http-transport` feature is enabled, `service.rs` converts `McpResponse`
/// into rmcp's `CallToolResult` via the `From` impl at the bottom.
#[derive(Debug)]
pub struct McpResponse {
    pub content: Vec<serde_json::Value>,
    pub is_error: bool,
}

impl McpResponse {
    /// Create a successful response wrapping `value`.
    /// If `value` is a JSON array each element becomes a content item;
    /// otherwise the value is wrapped in a single-element list.
    pub fn success(value: serde_json::Value) -> Self {
        let content = if let Some(arr) = value.as_array() {
            arr.clone()
        } else {
            vec![value]
        };
        Self { content, is_error: false }
    }

    /// Create an error response with a human-readable `message`.
    /// The `_code` parameter is accepted for API compatibility but unused —
    /// the MCP spec communicates errors via `is_error: true` in the result.
    pub fn error(_code: i32, message: String) -> Self {
        Self {
            content: vec![serde_json::json!({"type": "text", "text": message})],
            is_error: true,
        }
    }
}

/// Convert our internal `McpResponse` to rmcp's `CallToolResult` at the
/// service boundary.  This is the single point where internal types are
/// bridged to the rmcp wire format.
#[cfg(feature = "http-transport")]
impl From<McpResponse> for rmcp::model::CallToolResult {
    fn from(r: McpResponse) -> Self {
        use rmcp::model::Content;
        let content: Vec<Content> = r
            .content
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        if r.is_error {
            Self::error(content)
        } else {
            Self::success(content)
        }
    }
}
