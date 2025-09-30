use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum McpRequest {
    #[serde(rename = "initialize")]
    Initialize {
        #[serde(default)]
        params: Value,
    },
    #[serde(rename = "tools/list")]
    ListTools,
    #[serde(rename = "tools/call")]
    CallTool {
        #[serde(rename = "params")]
        params: ToolCall,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum McpNotification {
    #[serde(rename = "notifications/cancelled")]
    Cancelled {
        #[serde(default)]
        params: Value,
    },
    #[serde(rename = "notifications/initialized")]
    Initialized {
        #[serde(default)]
        params: Value,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: Value,
    #[serde(rename = "serverInfo")]
    pub server_info: Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpResponse {
    Initialize {
        result: InitializeResult,
    },
    ListTools {
        result: ListToolsResult,
    },
    ToolResult {
        content: Vec<Value>,
        #[serde(rename = "isError")]
        is_error: bool,
    },
    Error {
        error: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListToolsResult {
    pub tools: Vec<Value>,
}

impl McpResponse {
    /// Construct a success response wrapping arbitrary JSON value(s).
    /// Many call sites expect a `success` helper taking a `serde_json::Value`.
    pub fn success(value: Value) -> Self {
        // Try to normalize into ToolResult when possible (array or single value)
        if value.is_array() {
            // from_value consumes, so clone to preserve original for fallback.
            if let Ok(vec) = serde_json::from_value::<Vec<Value>>(value.clone()) {
                return McpResponse::ToolResult { content: vec, is_error: false };
            }
        }

        // Otherwise wrap the single value into a ToolResult.content
        McpResponse::ToolResult { content: vec![value], is_error: false }
    }

    /// Construct an error response. The code is currently ignored by the enum shape
    /// but callers pass an int and message; we'll include message in `Error`.
    pub fn error(_code: i32, message: String) -> Self {
        McpResponse::Error { error: message }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpMessage {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(flatten)]
    pub content: McpMessageContent,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpMessageContent {
    Request(McpRequest),
    Response(McpResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<Value>,
    #[serde(rename = "isError")]
    pub is_error: bool,
}