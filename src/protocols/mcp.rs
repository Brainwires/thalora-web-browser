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
        tools: Vec<Value>,
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