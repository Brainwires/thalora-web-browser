use anyhow::Result;
use serde_json::Value;

use super::super::{CdpDomain, CdpEvent};

/// Storage domain - Web storage inspection
#[derive(Debug)]
#[allow(dead_code)]
pub struct StorageDomain {
    enabled: bool,
}

impl Default for StorageDomain {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageDomain {
    pub fn new() -> Self {
        Self { enabled: false }
    }
}

impl CdpDomain for StorageDomain {
    fn name(&self) -> &str {
        "Storage"
    }

    fn handle_command(&mut self, method: &str, _params: Option<Value>) -> Result<Value> {
        match method {
            "clearDataForOrigin" => Ok(serde_json::json!({})),
            "getStorageKeyForFrame" => Ok(serde_json::json!({
                "storageKey": "thalora://main"
            })),
            "getUsageAndQuota" => Ok(serde_json::json!({
                "usage": 0,
                "quota": 1073741824,
                "overrideActive": false,
                "usageBreakdown": []
            })),
            _ => Err(anyhow::anyhow!("Unknown Storage method: {}", method)),
        }
    }

    fn get_events(&mut self) -> Vec<CdpEvent> {
        vec![]
    }
}
