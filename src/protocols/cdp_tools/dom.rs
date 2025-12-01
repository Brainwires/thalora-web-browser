use serde_json::Value;
use crate::protocols::mcp::McpResponse;
use crate::protocols::cdp::{CdpServer, CdpCommand, CdpMessage};

/// DOM domain - DOM queries, manipulation, and CSS inspection
pub struct DomTools;

impl DomTools {
    pub fn new() -> Self {
        Self
    }

    pub async fn enable_dom(&mut self, _args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let command = CdpCommand {
            id: 5,
            method: "DOM.enable".to_string(),
            params: None,
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if response.error.is_some() {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP DOM domain enable failed: {:?}", response.error)
                        })],
                        is_error: true,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "CDP DOM domain enabled successfully"
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "CDP DOM domain enabled"
                    })],
                    is_error: false,
                }
            }
            Err(e) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": format!("CDP DOM domain enable error: {}", e)
                    })],
                    is_error: true,
                }
            }
        }
    }

    pub async fn get_document(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let depth = args.get("depth").and_then(|v| v.as_i64()).unwrap_or(1);

        let command = CdpCommand {
            id: 6,
            method: "DOM.getDocument".to_string(),
            params: Some(serde_json::json!({
                "depth": depth
            })),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP DOM document retrieval failed: {}", error.message)
                        })],
                        is_error: true,
                    }
                } else if let Some(result) = response.result {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("CDP DOM document retrieved: {}", result)
                        })],
                        is_error: false,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": "CDP DOM document retrieved"
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "CDP DOM document retrieved"
                    })],
                    is_error: false,
                }
            }
            Err(e) => {
                McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": format!("CDP DOM document retrieval error: {}", e)
                    })],
                    is_error: true,
                }
            }
        }
    }

    pub async fn query_selector(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let selector = match args.get("selector").and_then(|v| v.as_str()) {
            Some(sel) => sel,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: selector"
                    })],
                    is_error: true,
                };
            }
        };

        let node_id = args.get("node_id").and_then(|v| v.as_i64()).unwrap_or(1);

        let command = CdpCommand {
            id: 10,
            method: "DOM.querySelector".to_string(),
            params: Some(serde_json::json!({
                "nodeId": node_id,
                "selector": selector
            })),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("DOM query selector failed: {}", error.message)
                        })],
                        is_error: true,
                    }
                } else if let Some(result) = response.result {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Element found with selector '{}': {}", selector, result)
                        })],
                        is_error: false,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("No element found with selector: {}", selector)
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "No response from CDP server"
                })],
                is_error: true,
            },
            Err(err) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("CDP query selector error: {}", err)
                })],
                is_error: true,
            }
        }
    }

    pub async fn get_attributes(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let node_id = match args.get("node_id").and_then(|v| v.as_i64()) {
            Some(id) => id,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: node_id"
                    })],
                    is_error: true,
                };
            }
        };

        let command = CdpCommand {
            id: 11,
            method: "DOM.getAttributes".to_string(),
            params: Some(serde_json::json!({
                "nodeId": node_id
            })),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Get attributes failed: {}", error.message)
                        })],
                        is_error: true,
                    }
                } else if let Some(result) = response.result {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Attributes for node {}: {}", node_id, result)
                        })],
                        is_error: false,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("No attributes found for node: {}", node_id)
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "No response from CDP server"
                })],
                is_error: true,
            },
            Err(err) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("CDP get attributes error: {}", err)
                })],
                is_error: true,
            }
        }
    }

    pub async fn get_computed_style(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let node_id = match args.get("node_id").and_then(|v| v.as_i64()) {
            Some(id) => id,
            None => {
                return McpResponse::ToolResult {
                    content: vec![serde_json::json!({
                        "type": "text",
                        "text": "Missing required parameter: node_id"
                    })],
                    is_error: true,
                };
            }
        };

        let command = CdpCommand {
            id: 12,
            method: "CSS.getComputedStyleForNode".to_string(),
            params: Some(serde_json::json!({
                "nodeId": node_id
            })),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Get computed style failed: {}", error.message)
                        })],
                        is_error: true,
                    }
                } else if let Some(result) = response.result {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("Computed styles for node {}: {}", node_id, result)
                        })],
                        is_error: false,
                    }
                } else {
                    McpResponse::ToolResult {
                        content: vec![serde_json::json!({
                            "type": "text",
                            "text": format!("No computed styles found for node: {}", node_id)
                        })],
                        is_error: false,
                    }
                }
            }
            Ok(_) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": "No response from CDP server"
                })],
                is_error: true,
            },
            Err(err) => McpResponse::ToolResult {
                content: vec![serde_json::json!({
                    "type": "text",
                    "text": format!("CDP get computed style error: {}", err)
                })],
                is_error: true,
            }
        }
    }
}
