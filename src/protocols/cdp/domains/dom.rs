use anyhow::Result;
use serde_json::Value;

use super::super::{CdpDomain, CdpEvent};

/// DOM domain - Document inspection and manipulation
#[derive(Debug)]
#[allow(dead_code)]
pub struct DomDomain {
    enabled: bool,
    next_node_id: i32,
}

impl DomDomain {
    pub fn new() -> Self {
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
