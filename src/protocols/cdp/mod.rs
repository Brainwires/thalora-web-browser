use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use uuid::Uuid;

pub mod domains;

// Re-export domain types
pub use domains::{
    BreakpointInfo, ConsoleDomain, DebuggerDomain, DomDomain, NetworkDomain, PageDomain,
    PerformanceDomain, RuntimeDomain, StorageDomain,
};

/// Chrome DevTools Protocol implementation for Thalora
/// Provides debugging and inspection APIs for AI coding agents
#[derive(Clone)]
#[allow(dead_code)]
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
        domains.insert(
            "Performance".to_string(),
            Box::new(PerformanceDomain::new()),
        );
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
        drop(self.event_sender.send(event));
    }
}
