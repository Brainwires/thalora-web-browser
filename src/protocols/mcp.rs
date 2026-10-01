/// Internal tool response type used by all MCP handler code.
///
/// `McpResponse` is our own struct so that handler files compile without any
/// knowledge of rmcp types and without naming collisions.  When the
/// `http-transport` feature is enabled, `service.rs` converts `McpResponse`
/// into rmcp's `CallToolResult` via the `From` impl at the bottom.
///
/// Every item in `content` is guaranteed to be a valid MCP content block
/// (an object with a `"type"` such as `"text"` or `"image"`).  Handlers may
/// pass arbitrary JSON to [`McpResponse::success`]; anything that is not
/// already a content block is serialized into a `{"type":"text"}` item so it
/// is never silently dropped by a transport.
#[derive(Debug)]
pub struct McpResponse {
    pub content: Vec<serde_json::Value>,
    pub is_error: bool,
}

/// MCP content block types (2025-06-18 spec) that clients understand natively.
const CONTENT_TYPES: &[&str] = &["text", "image", "audio", "resource", "resource_link"];

/// Returns true if `value` already looks like an MCP content block.
fn is_content_block(value: &serde_json::Value) -> bool {
    value
        .get("type")
        .and_then(|t| t.as_str())
        .is_some_and(|t| CONTENT_TYPES.contains(&t))
}

/// Wrap arbitrary JSON as a text content block.  Strings are emitted as-is;
/// everything else is pretty-printed JSON.
fn text_block(value: &serde_json::Value) -> serde_json::Value {
    let text = match value {
        serde_json::Value::String(s) => s.clone(),
        other => serde_json::to_string_pretty(other).unwrap_or_else(|_| other.to_string()),
    };
    serde_json::json!({"type": "text", "text": text})
}

impl McpResponse {
    /// Create a successful response from `value`.
    ///
    /// - A content block (`{"type":"text",...}`, `{"type":"image",...}`, …)
    ///   is used as-is.
    /// - An array whose elements are all content blocks becomes multiple
    ///   content items.
    /// - Anything else (a plain object, array, string, number…) is serialized
    ///   into a single text content block.
    pub fn success(value: serde_json::Value) -> Self {
        let content = match value {
            serde_json::Value::Array(items)
                if !items.is_empty() && items.iter().all(is_content_block) =>
            {
                items
            }
            v if is_content_block(&v) => vec![v],
            v => vec![text_block(&v)],
        };
        Self {
            content,
            is_error: false,
        }
    }

    /// Create a successful response with a single plain-text content block.
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![serde_json::json!({"type": "text", "text": text.into()})],
            is_error: false,
        }
    }

    /// Create an error response with a human-readable `message`.
    ///
    /// The MCP spec communicates tool failures via `is_error: true`; the
    /// numeric `code` is prefixed to the message so agents can still tell
    /// e.g. rate-limit errors apart from invalid parameters.
    pub fn error(code: i32, message: String) -> Self {
        Self {
            content: vec![serde_json::json!({
                "type": "text",
                "text": format!("[{code}] {message}"),
            })],
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
        // Never drop an item: anything rmcp can't parse is downgraded to text.
        let content: Vec<Content> = r
            .content
            .into_iter()
            .map(|v| {
                serde_json::from_value::<Content>(v.clone()).unwrap_or_else(|_| {
                    serde_json::from_value(text_block(&v))
                        .expect("a text content block always deserializes")
                })
            })
            .collect();
        if r.is_error {
            Self::error(content)
        } else {
            Self::success(content)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn bare_object_is_wrapped_as_text() {
        let r = McpResponse::success(json!({"success": true, "url": "https://x"}));
        assert_eq!(r.content.len(), 1);
        assert_eq!(r.content[0]["type"], "text");
        let inner: serde_json::Value =
            serde_json::from_str(r.content[0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(inner["url"], "https://x");
    }

    #[test]
    fn content_blocks_pass_through() {
        let r = McpResponse::success(json!({"type": "text", "text": "hi"}));
        assert_eq!(r.content, vec![json!({"type": "text", "text": "hi"})]);

        let r = McpResponse::success(json!([
            {"type": "text", "text": "a"},
            {"type": "image", "data": "AA==", "mimeType": "image/png"}
        ]));
        assert_eq!(r.content.len(), 2);
        assert_eq!(r.content[1]["type"], "image");
    }

    #[test]
    fn plain_arrays_become_one_text_block() {
        let r = McpResponse::success(json!([{"id": "a"}, {"id": "b"}]));
        assert_eq!(r.content.len(), 1);
        assert_eq!(r.content[0]["type"], "text");
        let r = McpResponse::success(json!([]));
        assert_eq!(r.content.len(), 1);
    }

    #[test]
    fn unknown_type_field_is_not_mistaken_for_content() {
        let r = McpResponse::success(json!({"type": "form", "fields": []}));
        assert_eq!(r.content[0]["type"], "text");
    }

    #[test]
    fn error_keeps_code() {
        let r = McpResponse::error(-32029, "Rate limited".into());
        assert!(r.is_error);
        assert_eq!(r.content[0]["text"], "[-32029] Rate limited");
    }

    #[cfg(feature = "http-transport")]
    #[test]
    fn rmcp_conversion_never_drops_content() {
        let mut r = McpResponse::success(json!({"nodes": [1, 2, 3]}));
        // Simulate a handler that pushed a malformed item directly.
        r.content.push(json!({"bogus": true}));
        // Inspect the wire format rather than rmcp's struct fields.
        let result: rmcp::model::CallToolResult = r.into();
        let wire = serde_json::to_value(&result).unwrap();
        assert_eq!(wire["content"].as_array().unwrap().len(), 2);
        assert_eq!(wire["content"][1]["type"], "text");

        let err: rmcp::model::CallToolResult = McpResponse::error(-1, "boom".into()).into();
        let wire = serde_json::to_value(&err).unwrap();
        assert_eq!(wire["isError"], true);
        assert_eq!(wire["content"][0]["text"], "[-1] boom");
    }
}
