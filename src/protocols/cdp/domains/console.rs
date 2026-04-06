use anyhow::Result;
use serde_json::Value;

use super::super::{CdpDomain, CdpEvent};

/// Console domain - Console interaction
#[derive(Debug)]
pub struct ConsoleDomain {
    enabled: bool,
}

impl Default for ConsoleDomain {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleDomain {
    pub fn new() -> Self {
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
            "clearMessages" => Ok(serde_json::json!({})),
            _ => Err(anyhow::anyhow!("Unknown Console method: {}", method)),
        }
    }

    fn get_events(&mut self) -> Vec<CdpEvent> {
        vec![]
    }
}
