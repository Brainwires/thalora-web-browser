use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

use super::super::{CdpDomain, CdpEvent};

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
    pub fn new() -> Self {
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
