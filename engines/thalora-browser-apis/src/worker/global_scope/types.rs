//! Types for WorkerGlobalScope

use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, Receiver};
use crate::misc::structured_clone::StructuredCloneValue;

/// Types of worker global scopes
#[derive(Debug, Clone, PartialEq, Eq, Trace, Finalize)]
pub enum WorkerGlobalScopeType {
    Dedicated,
    Shared,
    Service,
}

/// Worker location information
#[derive(Debug, Clone, Trace, Finalize)]
pub struct WorkerLocation {
    pub href: String,
    pub origin: String,
    pub protocol: String,
    pub host: String,
    pub hostname: String,
    pub port: String,
    pub pathname: String,
    pub search: String,
    pub hash: String,
}

impl WorkerLocation {
    /// Create WorkerLocation from URL string
    pub fn from_url(url_str: &str) -> Result<Self, String> {
        use url::Url;

        let url = Url::parse(url_str)
            .map_err(|e| format!("Invalid URL: {}: {}", url_str, e))?;

        Ok(Self {
            href: url.as_str().to_string(),
            origin: format!("{}://{}", url.scheme(), url.host_str().unwrap_or("")),
            protocol: format!("{}:", url.scheme()),
            host: url.host_str().unwrap_or("").to_string(),
            hostname: url.host_str().unwrap_or("").to_string(),
            port: url.port().map_or_else(|| "".to_string(), |p| p.to_string()),
            pathname: url.path().to_string(),
            search: url.query().map_or_else(|| "".to_string(), |q| format!("?{}", q)),
            hash: url.fragment().map_or_else(|| "".to_string(), |f| format!("#{}", f)),
        })
    }
}

/// Message between worker and main thread
#[derive(Debug, Clone)]
pub struct WorkerMessage {
    pub data: StructuredCloneValue,
    pub ports: Vec<String>, // Serialized MessagePort objects for transferable
    pub source: MessageSource,
}

/// Source of a worker message
#[derive(Debug, Clone)]
pub enum MessageSource {
    MainThread,
    Worker,
    SharedWorkerPort(String), // port name/id
}
