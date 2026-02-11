use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tracing::{error, trace};

use crate::protocols::mcp::{McpRequest, McpNotification, McpResponse, McpMessage, McpMessageContent};
use super::super::core::McpServer;

/// Run the MCP server using stdio transport (JSON-RPC over stdin/stdout).
///
/// This is the original transport — reads newline-delimited JSON-RPC from stdin
/// and writes responses to stdout. Backward-compatible with all existing MCP clients.
pub async fn run_stdio(server: &mut McpServer) -> Result<()> {
    let stdin = tokio::io::stdin();
    let mut reader = AsyncBufReader::new(stdin);
    let mut stdout = tokio::io::stdout();

    // Configure idle timeout - if no input received for this duration, exit gracefully
    // Default: 5 minutes, can be overridden with THALORA_IDLE_TIMEOUT_SECS env var
    let idle_timeout_secs = std::env::var("THALORA_IDLE_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(300); // 5 minutes default

    let idle_timeout = std::time::Duration::from_secs(idle_timeout_secs);

    trace!("MCP Server starting stdio loop (idle timeout: {}s)", idle_timeout_secs);

    loop {
        trace!("Waiting for input...");
        let mut line = String::new();

        // Use timeout on the read operation to prevent hanging forever
        let read_result = tokio::time::timeout(idle_timeout, reader.read_line(&mut line)).await;

        match read_result {
            Ok(Ok(0)) => {
                trace!("EOF received, shutting down");
                break;
            }
            Ok(Err(e)) => {
                error!("Failed to read from stdin: {}", e);
                break;
            }
            Err(_) => {
                // Timeout occurred - no input received for idle_timeout duration
                trace!("Idle timeout reached ({}s with no input), shutting down", idle_timeout_secs);
                eprintln!("\u{23f1}\u{fe0f} MCP Server idle timeout reached ({}s), shutting down gracefully", idle_timeout_secs);
                break;
            }
            Ok(Ok(n)) => {
                trace!("Read {} bytes from stdin", n);
                let line = line.trim();
                if line.is_empty() {
                    trace!("Empty line, continuing");
                    continue;
                }

                trace!("Parsing JSON: {}", line);

                // First, check if this is a notification (no 'id' field) or a request (has 'id' field)
                let parsed: serde_json::Value = match serde_json::from_str(line) {
                    Ok(v) => {
                        trace!("JSON parsed successfully");
                        v
                    }
                    Err(e) => {
                        error!("Failed to parse JSON: {}", e);
                        continue;
                    }
                };

                // Check if this is a request (has non-null id) or notification (no id or null id)
                let request_id = parsed.get("id").filter(|id| !id.is_null());

                if let Some(request_id) = request_id {
                    trace!("Handling request with id: {}", request_id);
                    // This is a request - parse as McpRequest and send response
                    match serde_json::from_value::<McpRequest>(parsed.clone()) {
                        Ok(request) => {
                            trace!("Request parsed, calling handler");
                            let response = server.handle_request(request).await;
                            trace!("Handler returned, preparing response");

                            // Wrap response in proper JSON-RPC 2.0 format
                            let message = McpMessage {
                                jsonrpc: "2.0".to_string(),
                                id: Some(request_id.clone()),
                                content: McpMessageContent::Response(response),
                            };

                            trace!("Serializing response");
                            let response_json = serde_json::to_string(&message)?;
                            trace!("Writing response to stdout: {} bytes", response_json.len());
                            stdout.write_all(response_json.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            trace!("Flushing stdout");
                            stdout.flush().await?;
                            trace!("Response sent successfully");
                        }
                        Err(e) => {
                            error!("Failed to parse request: {}", e);

                            // Send a JSON-RPC error response to stdout for invalid methods
                            let error_response = McpResponse::Error {
                                error: format!("Invalid method or malformed request: {}", e),
                            };

                            // Wrap error in proper JSON-RPC 2.0 format
                            let message = McpMessage {
                                jsonrpc: "2.0".to_string(),
                                id: Some(request_id.clone()),
                                content: McpMessageContent::Response(error_response),
                            };

                            let response_json = serde_json::to_string(&message)?;
                            stdout.write_all(response_json.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                    }
                } else {
                    // This is a notification - parse as McpNotification and handle without response
                    match serde_json::from_value::<McpNotification>(parsed) {
                        Ok(notification) => {
                            server.handle_notification(notification).await;
                            // Notifications don't require responses
                        }
                        Err(e) => {
                            error!("Failed to parse notification: {}", e);
                            // For notifications, we don't send error responses
                        }
                    }
                }
            }
        }
    }

    // Cleanup all sessions before shutting down
    server.cleanup().await;
    Ok(())
}
