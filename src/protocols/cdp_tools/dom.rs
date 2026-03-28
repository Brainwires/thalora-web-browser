use crate::protocols::cdp::{CdpCommand, CdpMessage, CdpServer};
use crate::protocols::mcp::McpResponse;
use serde_json::Value;

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
                    McpResponse::error(
                        -1,
                        format!("CDP DOM domain enable failed: {:?}", response.error),
                    )
                } else {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": "CDP DOM domain enabled successfully"
                    }))
                }
            }
            Ok(_) => McpResponse::success(serde_json::json!({
                "type": "text",
                "text": "CDP DOM domain enabled"
            })),
            Err(e) => McpResponse::error(-1, format!("CDP DOM domain enable error: {}", e)),
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
                    McpResponse::error(
                        -1,
                        format!("CDP DOM document retrieval failed: {}", error.message),
                    )
                } else if let Some(result) = response.result {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": format!("CDP DOM document retrieved: {}", result)
                    }))
                } else {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": "CDP DOM document retrieved"
                    }))
                }
            }
            Ok(_) => McpResponse::success(serde_json::json!({
                "type": "text",
                "text": "CDP DOM document retrieved"
            })),
            Err(e) => McpResponse::error(-1, format!("CDP DOM document retrieval error: {}", e)),
        }
    }

    pub async fn query_selector(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let selector = match args.get("selector").and_then(|v| v.as_str()) {
            Some(sel) => sel,
            None => {
                return McpResponse::error(-1, "Missing required parameter: selector".to_string());
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
                    McpResponse::error(-1, format!("DOM query selector failed: {}", error.message))
                } else if let Some(result) = response.result {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": format!("Element found with selector '{}': {}", selector, result)
                    }))
                } else {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": format!("No element found with selector: {}", selector)
                    }))
                }
            }
            Ok(_) => McpResponse::error(-1, "No response from CDP server".to_string()),
            Err(err) => McpResponse::error(-1, format!("CDP query selector error: {}", err)),
        }
    }

    pub async fn get_attributes(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let node_id = match args.get("node_id").and_then(|v| v.as_i64()) {
            Some(id) => id,
            None => {
                return McpResponse::error(-1, "Missing required parameter: node_id".to_string());
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
                    McpResponse::error(-1, format!("Get attributes failed: {}", error.message))
                } else if let Some(result) = response.result {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": format!("Attributes for node {}: {}", node_id, result)
                    }))
                } else {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": format!("No attributes found for node: {}", node_id)
                    }))
                }
            }
            Ok(_) => McpResponse::error(-1, "No response from CDP server".to_string()),
            Err(err) => McpResponse::error(-1, format!("CDP get attributes error: {}", err)),
        }
    }

    pub async fn get_computed_style(
        &mut self,
        args: Value,
        cdp_server: &mut CdpServer,
    ) -> McpResponse {
        let node_id = match args.get("node_id").and_then(|v| v.as_i64()) {
            Some(id) => id,
            None => {
                return McpResponse::error(-1, "Missing required parameter: node_id".to_string());
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
                    McpResponse::error(-1, format!("Get computed style failed: {}", error.message))
                } else if let Some(result) = response.result {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": format!("Computed styles for node {}: {}", node_id, result)
                    }))
                } else {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": format!("No computed styles found for node: {}", node_id)
                    }))
                }
            }
            Ok(_) => McpResponse::error(-1, "No response from CDP server".to_string()),
            Err(err) => McpResponse::error(-1, format!("CDP get computed style error: {}", err)),
        }
    }
}
