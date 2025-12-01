use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::{CdpDomain, CdpEvent};

/// Runtime domain - JavaScript execution and inspection
#[derive(Debug)]
#[allow(dead_code)]
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
    pub fn new() -> Self {
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
