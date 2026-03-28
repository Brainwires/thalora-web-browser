use crate::protocols::cdp::{CdpCommand, CdpMessage, CdpServer};
use crate::protocols::mcp::McpResponse;
use serde_json::Value;

/// Debugger domain - Breakpoints, stepping, and script debugging
pub struct DebuggerTools;

impl DebuggerTools {
    pub fn new() -> Self {
        Self
    }

    pub async fn enable_debugger(
        &mut self,
        _args: Value,
        cdp_server: &mut CdpServer,
    ) -> McpResponse {
        let command = CdpCommand {
            id: 3,
            method: "Debugger.enable".to_string(),
            params: None,
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if response.error.is_some() {
                    McpResponse::error(
                        -1,
                        format!("CDP Debugger domain enable failed: {:?}", response.error),
                    )
                } else {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": "CDP Debugger domain enabled successfully"
                    }))
                }
            }
            Ok(_) => McpResponse::success(serde_json::json!({
                "type": "text",
                "text": "CDP Debugger domain enabled"
            })),
            Err(e) => McpResponse::error(-1, format!("CDP Debugger domain enable error: {}", e)),
        }
    }

    pub async fn set_breakpoint(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let line_number = match args.get("line_number").and_then(|v| v.as_i64()) {
            Some(line) => line,
            None => {
                return McpResponse::error(
                    -1,
                    "Missing required parameter: line_number".to_string(),
                );
            }
        };

        let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");

        let command = CdpCommand {
            id: 4,
            method: "Debugger.setBreakpointByUrl".to_string(),
            params: Some(serde_json::json!({
                "lineNumber": line_number,
                "url": url
            })),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::error(-1, format!("CDP Breakpoint set failed: {}", error.message))
                } else if let Some(result) = response.result {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": format!("CDP Breakpoint set successfully at line {} in {}: {}", line_number, url, result)
                    }))
                } else {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": format!("CDP Breakpoint set at line {} in {}", line_number, url)
                    }))
                }
            }
            Ok(_) => McpResponse::success(serde_json::json!({
                "type": "text",
                "text": format!("CDP Breakpoint set at line {} in {}", line_number, url)
            })),
            Err(e) => McpResponse::error(-1, format!("CDP Breakpoint set error: {}", e)),
        }
    }
}
