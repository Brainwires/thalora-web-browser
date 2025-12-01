use anyhow::Result;
use serde_json::Value;

use super::super::{CdpDomain, CdpEvent};

/// Performance domain - Performance monitoring
#[derive(Debug)]
pub struct PerformanceDomain {
    enabled: bool,
}

impl PerformanceDomain {
    pub fn new() -> Self {
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
