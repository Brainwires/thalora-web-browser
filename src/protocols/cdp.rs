use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use uuid::Uuid;

/// Chrome DevTools Protocol implementation for Thalora
/// Provides debugging and inspection APIs for AI coding agents
#[derive(Clone)]
pub struct CdpServer {
    domains: Arc<Mutex<HashMap<String, Box<dyn CdpDomain + Send + Sync>>>>,
    event_sender: broadcast::Sender<CdpEvent>,
    session_id: String,
}

/// CDP message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CdpMessage {
    Command(CdpCommand),
    Response(CdpResponse),
    Event(CdpEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpCommand {
    pub id: i32,
    pub method: String,
    pub params: Option<Value>,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpResponse {
    pub id: i32,
    pub result: Option<Value>,
    pub error: Option<CdpError>,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpEvent {
    pub method: String,
    pub params: Option<Value>,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

/// Trait for CDP domain implementations
pub trait CdpDomain {
    fn name(&self) -> &str;
    fn handle_command(&mut self, method: &str, params: Option<Value>) -> Result<Value>;
    fn get_events(&mut self) -> Vec<CdpEvent>;
}

impl CdpServer {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        let session_id = Uuid::new_v4().to_string();

        let mut server = Self {
            domains: Arc::new(Mutex::new(HashMap::new())),
            event_sender,
            session_id,
        };

        // Register core domains
        server.register_core_domains();
        server
    }


    fn register_core_domains(&mut self) {
        let mut domains = self.domains.lock().unwrap();

        domains.insert("Runtime".to_string(), Box::new(RuntimeDomain::new()));
        domains.insert("Debugger".to_string(), Box::new(DebuggerDomain::new()));
        domains.insert("DOM".to_string(), Box::new(DomDomain::new()));
        domains.insert("Network".to_string(), Box::new(NetworkDomain::new()));
        domains.insert("Console".to_string(), Box::new(ConsoleDomain::new()));
        domains.insert("Page".to_string(), Box::new(PageDomain::new()));
        domains.insert("Performance".to_string(), Box::new(PerformanceDomain::new()));
        domains.insert("Storage".to_string(), Box::new(StorageDomain::new()));
    }


    pub fn handle_message(&mut self, message: CdpMessage) -> Result<Option<CdpMessage>> {
        match message {
            CdpMessage::Command(cmd) => {
                let response = self.handle_command(cmd)?;
                Ok(Some(CdpMessage::Response(response)))
            }
            _ => Ok(None), // Events and responses are not handled by server
        }
    }

    fn handle_command(&mut self, cmd: CdpCommand) -> Result<CdpResponse> {
        let parts: Vec<&str> = cmd.method.split('.').collect();
        if parts.len() != 2 {
            return Ok(CdpResponse {
                id: cmd.id,
                result: None,
                error: Some(CdpError {
                    code: -32602,
                    message: "Invalid method name".to_string(),
                    data: None,
                }),
                session_id: cmd.session_id,
            });
        }

        let domain_name = parts[0];
        let method_name = parts[1];

        let mut domains = self.domains.lock().unwrap();
        if let Some(domain) = domains.get_mut(domain_name) {
            match domain.handle_command(method_name, cmd.params) {
                Ok(result) => Ok(CdpResponse {
                    id: cmd.id,
                    result: Some(result),
                    error: None,
                    session_id: cmd.session_id,
                }),
                Err(e) => Ok(CdpResponse {
                    id: cmd.id,
                    result: None,
                    error: Some(CdpError {
                        code: -32000,
                        message: e.to_string(),
                        data: None,
                    }),
                    session_id: cmd.session_id,
                }),
            }
        } else {
            Ok(CdpResponse {
                id: cmd.id,
                result: None,
                error: Some(CdpError {
                    code: -32601,
                    message: format!("Domain '{}' not found", domain_name),
                    data: None,
                }),
                session_id: cmd.session_id,
            })
        }
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<CdpEvent> {
        self.event_sender.subscribe()
    }

    pub fn emit_event(&self, event: CdpEvent) {
        let _ = self.event_sender.send(event);
    }
}

/// Runtime domain - JavaScript execution and inspection
#[derive(Debug)]
pub struct RuntimeDomain {
    execution_contexts: Vec<ExecutionContext>,
    next_context_id: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionContext {
    pub id: i32,
    pub name: String,
    pub origin: String,
    #[serde(rename = "auxData")]
    pub aux_data: Option<Value>,
}

impl RuntimeDomain {
    fn new() -> Self {
        Self {
            execution_contexts: vec![ExecutionContext {
                id: 1,
                name: "main".to_string(),
                origin: "thalora://main".to_string(),
                aux_data: None,
            }],
            next_context_id: 2,
        }
    }
}

impl CdpDomain for RuntimeDomain {
    fn name(&self) -> &str {
        "Runtime"
    }

    fn handle_command(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        match method {
            "enable" => {
                // Enable runtime notifications
                Ok(serde_json::json!({}))
            }
            "disable" => {
                Ok(serde_json::json!({}))
            }
            "evaluate" => {
                let params = params.unwrap_or_default();
                let expression = params.get("expression")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                // TODO: Need to implement JavaScript execution via message passing to avoid thread safety issues
                // For now, return a clear indication that evaluation is not yet implemented
                Ok(serde_json::json!({
                    "result": {
                        "type": "string",
                        "value": format!("JavaScript evaluation not yet implemented: {}", expression)
                    }
                }))
            }
            "getProperties" => {
                let params = params.unwrap_or_default();
                let _object_id = params.get("objectId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                Ok(serde_json::json!({
                    "result": []
                }))
            }
            "compileScript" => {
                Ok(serde_json::json!({
                    "scriptId": "script_1"
                }))
            }
            _ => Err(anyhow::anyhow!("Unknown Runtime method: {}", method))
        }
    }

    fn get_events(&mut self) -> Vec<CdpEvent> {
        vec![CdpEvent {
            method: "Runtime.executionContextCreated".to_string(),
            params: Some(serde_json::json!({
                "context": self.execution_contexts[0]
            })),
            session_id: None,
        }]
    }
}

/// Debugger domain - JavaScript debugging
#[derive(Debug)]
pub struct DebuggerDomain {
    enabled: bool,
    breakpoints: HashMap<String, BreakpointInfo>,
    next_breakpoint_id: i32,
}

#[derive(Debug, Clone)]
pub struct BreakpointInfo {
    pub id: String,
    pub line_number: i32,
    pub column_number: Option<i32>,
    pub condition: Option<String>,
}

impl DebuggerDomain {
    fn new() -> Self {
        Self {
            enabled: false,
            breakpoints: HashMap::new(),
            next_breakpoint_id: 1,
        }
    }
}

impl CdpDomain for DebuggerDomain {
    fn name(&self) -> &str {
        "Debugger"
    }

    fn handle_command(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        match method {
            "enable" => {
                self.enabled = true;
                Ok(serde_json::json!({
                    "debuggerId": "thalora-debugger-1"
                }))
            }
            "disable" => {
                self.enabled = false;
                self.breakpoints.clear();
                Ok(serde_json::json!({}))
            }
            "setBreakpointByUrl" => {
                let params = params.unwrap_or_default();
                let line_number = params.get("lineNumber")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;
                let _url = params.get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                let breakpoint_id = format!("bp_{}", self.next_breakpoint_id);
                self.next_breakpoint_id += 1;
                
                self.breakpoints.insert(breakpoint_id.clone(), BreakpointInfo {
                    id: breakpoint_id.clone(),
                    line_number,
                    column_number: params.get("columnNumber").and_then(|v| v.as_i64()).map(|v| v as i32),
                    condition: params.get("condition").and_then(|v| v.as_str()).map(|s| s.to_string()),
                });
                
                Ok(serde_json::json!({
                    "breakpointId": breakpoint_id,
                    "locations": [{
                        "scriptId": "script_1",
                        "lineNumber": line_number,
                        "columnNumber": 0
                    }]
                }))
            }
            "removeBreakpoint" => {
                let params = params.unwrap_or_default();
                let breakpoint_id = params.get("breakpointId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                self.breakpoints.remove(breakpoint_id);
                Ok(serde_json::json!({}))
            }
            "resume" => {
                Ok(serde_json::json!({}))
            }
            "stepOver" => {
                Ok(serde_json::json!({}))
            }
            "stepInto" => {
                Ok(serde_json::json!({}))
            }
            "stepOut" => {
                Ok(serde_json::json!({}))
            }
            _ => Err(anyhow::anyhow!("Unknown Debugger method: {}", method))
        }
    }

    fn get_events(&mut self) -> Vec<CdpEvent> {
        if self.enabled {
            vec![CdpEvent {
                method: "Debugger.scriptParsed".to_string(),
                params: Some(serde_json::json!({
                    "scriptId": "script_1",
                    "url": "thalora://main",
                    "startLine": 0,
                    "startColumn": 0,
                    "endLine": 100,
                    "endColumn": 0,
                    "executionContextId": 1,
                    "hash": "thalora_hash"
                })),
                session_id: None,
            }]
        } else {
            vec![]
        }
    }
}

/// DOM domain - Document inspection and manipulation
#[derive(Debug)]
pub struct DomDomain {
    enabled: bool,
    next_node_id: i32,
}

impl DomDomain {
    fn new() -> Self {
        Self {
            enabled: false,
            next_node_id: 1,
        }
    }
}

impl CdpDomain for DomDomain {
    fn name(&self) -> &str {
        "DOM"
    }

    fn handle_command(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        match method {
            "enable" => {
                self.enabled = true;
                Ok(serde_json::json!({}))
            }
            "disable" => {
                self.enabled = false;
                Ok(serde_json::json!({}))
            }
            "getDocument" => {
                Ok(serde_json::json!({
                    "root": {
                        "nodeId": 1,
                        "backendNodeId": 1,
                        "nodeType": 9,
                        "nodeName": "#document",
                        "localName": "",
                        "nodeValue": "",
                        "childNodeCount": 1,
                        "children": [{
                            "nodeId": 2,
                            "backendNodeId": 2,
                            "nodeType": 1,
                            "nodeName": "HTML",
                            "localName": "html",
                            "nodeValue": "",
                            "childNodeCount": 2,
                            "attributes": []
                        }]
                    }
                }))
            }
            "querySelector" => {
                let params = params.unwrap_or_default();
                let _selector = params.get("selector")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                Ok(serde_json::json!({
                    "nodeId": 3
                }))
            }
            "querySelectorAll" => {
                let params = params.unwrap_or_default();
                let _selector = params.get("selector")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                Ok(serde_json::json!({
                    "nodeIds": [3, 4, 5]
                }))
            }
            "getAttributes" => {
                let params = params.unwrap_or_default();
                let _node_id = params.get("nodeId")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1);

                Ok(serde_json::json!({
                    "attributes": ["id", "test-element", "class", "example", "data-value", "123"]
                }))
            }
            _ => Err(anyhow::anyhow!("Unknown DOM method: {}", method))
        }
    }

    fn get_events(&mut self) -> Vec<CdpEvent> {
        vec![]
    }
}

/// Network domain - Network monitoring
#[derive(Debug)]
pub struct NetworkDomain {
    enabled: bool,
}

impl NetworkDomain {
    fn new() -> Self {
        Self { enabled: false }
    }
}

impl CdpDomain for NetworkDomain {
    fn name(&self) -> &str {
        "Network"
    }

    fn handle_command(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        match method {
            "enable" => {
                self.enabled = true;
                Ok(serde_json::json!({}))
            }
            "disable" => {
                self.enabled = false;
                Ok(serde_json::json!({}))
            }
            "getAllCookies" => {
                Ok(serde_json::json!({
                    "cookies": []
                }))
            }
            "getCookies" => {
                Ok(serde_json::json!({
                    "cookies": []
                }))
            }
            "setCookie" => {
                let params = params.unwrap_or_default();
                let _name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let _value = params.get("value").and_then(|v| v.as_str()).unwrap_or("");

                Ok(serde_json::json!({
                    "success": true
                }))
            }
            _ => Err(anyhow::anyhow!("Unknown Network method: {}", method))
        }
    }

    fn get_events(&mut self) -> Vec<CdpEvent> {
        vec![]
    }
}

/// Console domain - Console interaction
#[derive(Debug)]
pub struct ConsoleDomain {
    enabled: bool,
}

impl ConsoleDomain {
    fn new() -> Self {
        Self { enabled: false }
    }
}

impl CdpDomain for ConsoleDomain {
    fn name(&self) -> &str {
        "Console"
    }

    fn handle_command(&mut self, method: &str, _params: Option<Value>) -> Result<Value> {
        match method {
            "enable" => {
                self.enabled = true;
                Ok(serde_json::json!({}))
            }
            "disable" => {
                self.enabled = false;
                Ok(serde_json::json!({}))
            }
            "clearMessages" => {
                Ok(serde_json::json!({}))
            }
            _ => Err(anyhow::anyhow!("Unknown Console method: {}", method))
        }
    }

    fn get_events(&mut self) -> Vec<CdpEvent> {
        vec![]
    }
}

/// Page domain - Page lifecycle and navigation
#[derive(Debug)]
pub struct PageDomain {
    enabled: bool,
}

impl PageDomain {
    fn new() -> Self {
        Self { enabled: false }
    }
}

impl CdpDomain for PageDomain {
    fn name(&self) -> &str {
        "Page"
    }

    fn handle_command(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        match method {
            "enable" => {
                self.enabled = true;
                Ok(serde_json::json!({}))
            }
            "disable" => {
                self.enabled = false;
                Ok(serde_json::json!({}))
            }
            "navigate" => {
                let params = params.unwrap_or_default();
                let _url = params.get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                Ok(serde_json::json!({
                    "frameId": "main_frame",
                    "loaderId": "loader_1"
                }))
            }
            "reload" => {
                Ok(serde_json::json!({}))
            }
            "captureScreenshot" => {
                // Return a minimal base64 encoded 1x1 PNG
                Ok(serde_json::json!({
                    "data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg=="
                }))
            }
            _ => Err(anyhow::anyhow!("Unknown Page method: {}", method))
        }
    }

    fn get_events(&mut self) -> Vec<CdpEvent> {
        vec![]
    }
}

/// Performance domain - Performance monitoring
#[derive(Debug)]
pub struct PerformanceDomain {
    enabled: bool,
}

impl PerformanceDomain {
    fn new() -> Self {
        Self { enabled: false }
    }
}

impl CdpDomain for PerformanceDomain {
    fn name(&self) -> &str {
        "Performance"
    }

    fn handle_command(&mut self, method: &str, _params: Option<Value>) -> Result<Value> {
        match method {
            "enable" => {
                self.enabled = true;
                Ok(serde_json::json!({}))
            }
            "disable" => {
                self.enabled = false;
                Ok(serde_json::json!({}))
            }
            "getMetrics" => {
                Ok(serde_json::json!({
                    "metrics": [
                        {
                            "name": "Timestamp",
                            "value": chrono::Utc::now().timestamp_millis() as f64 / 1000.0
                        },
                        {
                            "name": "Documents",
                            "value": 1
                        },
                        {
                            "name": "Frames",
                            "value": 1
                        }
                    ]
                }))
            }
            _ => Err(anyhow::anyhow!("Unknown Performance method: {}", method))
        }
    }

    fn get_events(&mut self) -> Vec<CdpEvent> {
        vec![]
    }
}

/// Storage domain - Web storage inspection
#[derive(Debug)]
pub struct StorageDomain {
    enabled: bool,
}

impl StorageDomain {
    fn new() -> Self {
        Self { enabled: false }
    }
}

impl CdpDomain for StorageDomain {
    fn name(&self) -> &str {
        "Storage"
    }

    fn handle_command(&mut self, method: &str, _params: Option<Value>) -> Result<Value> {
        match method {
            "clearDataForOrigin" => {
                Ok(serde_json::json!({}))
            }
            "getStorageKeyForFrame" => {
                Ok(serde_json::json!({
                    "storageKey": "thalora://main"
                }))
            }
            "getUsageAndQuota" => {
                Ok(serde_json::json!({
                    "usage": 0,
                    "quota": 1073741824,
                    "overrideActive": false,
                    "usageBreakdown": []
                }))
            }
            _ => Err(anyhow::anyhow!("Unknown Storage method: {}", method))
        }
    }

    fn get_events(&mut self) -> Vec<CdpEvent> {
        vec![]
    }
}