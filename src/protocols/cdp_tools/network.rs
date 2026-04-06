use crate::protocols::cdp::{CdpCommand, CdpMessage, CdpServer};
use crate::protocols::mcp::McpResponse;
use crate::protocols::security::validate_cookie;
use serde_json::Value;

/// Network domain - Network monitoring, cookies, and request/response inspection
pub struct NetworkTools;

impl NetworkTools {
    pub fn new() -> Self {
        Self
    }

    pub async fn enable_network(
        &mut self,
        _args: Value,
        cdp_server: &mut CdpServer,
    ) -> McpResponse {
        let command = CdpCommand {
            id: 7,
            method: "Network.enable".to_string(),
            params: None,
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if response.error.is_some() {
                    McpResponse::error(
                        -1,
                        format!("CDP Network domain enable failed: {:?}", response.error),
                    )
                } else {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": "CDP Network domain enabled successfully"
                    }))
                }
            }
            Ok(_) => McpResponse::success(serde_json::json!({
                "type": "text",
                "text": "CDP Network domain enabled"
            })),
            Err(e) => McpResponse::error(-1, format!("CDP Network domain enable error: {}", e)),
        }
    }

    pub async fn get_response_body(
        &mut self,
        args: Value,
        cdp_server: &mut CdpServer,
    ) -> McpResponse {
        let request_id = match args.get("request_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => {
                return McpResponse::error(
                    -1,
                    "Missing required parameter: request_id".to_string(),
                );
            }
        };

        let command = CdpCommand {
            id: 8,
            method: "Network.getResponseBody".to_string(),
            params: Some(serde_json::json!({
                "requestId": request_id
            })),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::error(
                        -1,
                        format!("CDP Response body retrieval failed: {}", error.message),
                    )
                } else if let Some(result) = response.result {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": format!("CDP Response body for request {}: {}", request_id, result)
                    }))
                } else {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": format!("CDP Response body for request {} retrieved", request_id)
                    }))
                }
            }
            Ok(_) => McpResponse::success(serde_json::json!({
                "type": "text",
                "text": format!("CDP Response body for request {} retrieved", request_id)
            })),
            Err(e) => McpResponse::error(-1, format!("CDP Response body retrieval error: {}", e)),
        }
    }

    pub async fn get_cookies(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let urls = args
            .get("urls")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>());

        let command = CdpCommand {
            id: 13,
            method: "Network.getCookies".to_string(),
            params: urls.map(|urls| serde_json::json!({ "urls": urls })),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::error(-1, format!("Get cookies failed: {}", error.message))
                } else if let Some(result) = response.result {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": format!("Cookies: {}", result)
                    }))
                } else {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": "No cookies found"
                    }))
                }
            }
            Ok(_) => McpResponse::error(-1, "No response from CDP server".to_string()),
            Err(err) => McpResponse::error(-1, format!("CDP get cookies error: {}", err)),
        }
    }

    pub async fn set_cookie(&mut self, args: Value, cdp_server: &mut CdpServer) -> McpResponse {
        let name = match args.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => {
                return McpResponse::error(-1, "Missing required parameter: name".to_string());
            }
        };

        let value = match args.get("value").and_then(|v| v.as_str()) {
            Some(v) => v,
            None => {
                return McpResponse::error(-1, "Missing required parameter: value".to_string());
            }
        };

        // SECURITY: Validate cookie name and value to prevent injection attacks (CWE-113)
        if let Err(e) = validate_cookie(name, value) {
            return McpResponse::error(-1, format!("Invalid cookie: {}", e));
        }

        let domain = args.get("domain").and_then(|v| v.as_str());
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("/");
        let secure = args
            .get("secure")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let http_only = args
            .get("http_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut cookie_params = serde_json::json!({
            "name": name,
            "value": value,
            "path": path,
            "secure": secure,
            "httpOnly": http_only
        });

        if let Some(domain) = domain {
            cookie_params["domain"] = serde_json::Value::String(domain.to_string());
        }

        let command = CdpCommand {
            id: 14,
            method: "Network.setCookie".to_string(),
            params: Some(cookie_params),
            session_id: None,
        };

        match cdp_server.handle_message(CdpMessage::Command(command)) {
            Ok(Some(CdpMessage::Response(response))) => {
                if let Some(error) = response.error {
                    McpResponse::error(-1, format!("Set cookie failed: {}", error.message))
                } else {
                    McpResponse::success(serde_json::json!({
                        "type": "text",
                        "text": format!("Cookie '{}' set successfully", name)
                    }))
                }
            }
            Ok(_) => McpResponse::error(-1, "No response from CDP server".to_string()),
            Err(err) => McpResponse::error(-1, format!("CDP set cookie error: {}", err)),
        }
    }
}
