//! Platform abstraction layer for Thalora Web Browser
//!
//! This module provides platform-specific implementations for:
//! - HTTP client (reqwest on native, web-sys fetch on WASM)
//! - WebSocket (tokio-tungstenite on native, web-sys WebSocket on WASM)
//! - Storage (sled/file on native, IndexedDB on WASM)
//! - Timers (tokio on native, web-sys setTimeout on WASM)
//!
//! The abstraction allows the same high-level code to work on both platforms.

// Native platform implementation
#[cfg(feature = "core")]
pub mod native;

// WASM platform implementation
#[cfg(feature = "wasm")]
pub mod wasm;

// Re-export the appropriate platform module
#[cfg(feature = "core")]
pub use native as platform;

#[cfg(feature = "wasm")]
pub use wasm as platform;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// HTTP request method
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Patch => "PATCH",
        }
    }
}

/// HTTP request builder
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub url: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub timeout_ms: Option<u64>,
}

impl HttpRequest {
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Get,
            headers: HashMap::new(),
            body: None,
            timeout_ms: Some(30000),
        }
    }

    pub fn post(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Post,
            headers: HashMap::new(),
            body: None,
            timeout_ms: Some(30000),
        }
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn json_body(mut self, json: &str) -> Self {
        self.body = Some(json.as_bytes().to_vec());
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self
    }

    pub fn timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
}

/// HTTP response
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub url: String,
}

impl HttpResponse {
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    pub fn is_redirect(&self) -> bool {
        self.status >= 300 && self.status < 400
    }
}

/// HTTP client error
#[derive(Debug, Clone)]
pub struct HttpError {
    pub message: String,
    pub kind: HttpErrorKind,
}

#[derive(Debug, Clone)]
pub enum HttpErrorKind {
    Network,
    Timeout,
    InvalidUrl,
    InvalidResponse,
    Other,
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for HttpError {}

/// Platform HTTP client trait
///
/// On native platforms, this requires Send + Sync for thread safety.
/// On WASM, these bounds are relaxed since WASM is single-threaded.
#[cfg(feature = "core")]
pub trait HttpClient: Send + Sync {
    fn request(
        &self,
        request: HttpRequest,
    ) -> Pin<Box<dyn Future<Output = Result<HttpResponse, HttpError>> + Send + '_>>;
}

#[cfg(feature = "wasm")]
pub trait HttpClient {
    fn request(
        &self,
        request: HttpRequest,
    ) -> Pin<Box<dyn Future<Output = Result<HttpResponse, HttpError>> + '_>>;
}

/// WebSocket message types
#[derive(Debug, Clone)]
pub enum WsMessage {
    Text(String),
    Binary(Vec<u8>),
    Close,
    Ping(Vec<u8>),
    Pong(Vec<u8>),
}

/// WebSocket event callback type
#[cfg(feature = "core")]
pub type WsCallback = Box<dyn Fn(WsMessage) + Send + Sync>;
#[cfg(feature = "wasm")]
pub type WsCallback = Box<dyn Fn(WsMessage)>;

/// Platform WebSocket trait
#[cfg(feature = "core")]
pub trait WebSocketClient: Send + Sync {
    fn connect(
        &mut self,
        url: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>>;
    fn send(
        &mut self,
        message: WsMessage,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>>;
    fn close(&mut self) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>>;
    fn on_message(&mut self, callback: WsCallback);
}

#[cfg(feature = "wasm")]
pub trait WebSocketClient {
    fn connect(&mut self, url: &str) -> Pin<Box<dyn Future<Output = Result<(), String>> + '_>>;
    fn send(
        &mut self,
        message: WsMessage,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + '_>>;
    fn close(&mut self) -> Pin<Box<dyn Future<Output = Result<(), String>> + '_>>;
    fn on_message(&mut self, callback: WsCallback);
}

/// Storage key-value operations
#[cfg(feature = "core")]
pub trait Storage: Send + Sync {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    fn set(&mut self, key: &str, value: &[u8]) -> Result<(), String>;
    fn delete(&mut self, key: &str) -> Result<(), String>;
    fn clear(&mut self) -> Result<(), String>;
    fn keys(&self) -> Vec<String>;
}

#[cfg(feature = "wasm")]
pub trait Storage {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    fn set(&mut self, key: &str, value: &[u8]) -> Result<(), String>;
    fn delete(&mut self, key: &str) -> Result<(), String>;
    fn clear(&mut self) -> Result<(), String>;
    fn keys(&self) -> Vec<String>;
}

/// Timer operations
#[cfg(feature = "core")]
pub trait Timer {
    fn set_timeout<F>(callback: F, delay_ms: u32) -> u32
    where
        F: FnOnce() + Send + 'static;

    fn clear_timeout(id: u32);

    fn set_interval<F>(callback: F, interval_ms: u32) -> u32
    where
        F: Fn() + Send + Sync + 'static;

    fn clear_interval(id: u32);
}

#[cfg(feature = "wasm")]
pub trait Timer {
    fn set_timeout<F>(callback: F, delay_ms: u32) -> u32
    where
        F: FnOnce() + 'static;

    fn clear_timeout(id: u32);

    fn set_interval<F>(callback: F, interval_ms: u32) -> u32
    where
        F: Fn() + 'static;

    fn clear_interval(id: u32);
}

/// Platform info
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub name: &'static str,
    pub is_native: bool,
    pub is_wasm: bool,
    pub supports_threads: bool,
    pub supports_filesystem: bool,
}

/// Get current platform info
pub fn get_platform_info() -> PlatformInfo {
    #[cfg(feature = "core")]
    {
        PlatformInfo {
            name: "native",
            is_native: true,
            is_wasm: false,
            supports_threads: true,
            supports_filesystem: true,
        }
    }

    #[cfg(feature = "wasm")]
    {
        PlatformInfo {
            name: "wasm",
            is_native: false,
            is_wasm: true,
            supports_threads: false,
            supports_filesystem: false,
        }
    }

    #[cfg(not(any(feature = "core", feature = "wasm")))]
    {
        PlatformInfo {
            name: "unknown",
            is_native: false,
            is_wasm: false,
            supports_threads: false,
            supports_filesystem: false,
        }
    }
}

/// Create the default HTTP client for the current platform
#[cfg(any(feature = "core", feature = "wasm"))]
pub fn create_http_client() -> Box<dyn HttpClient> {
    #[cfg(feature = "core")]
    {
        Box::new(native::NativeHttpClient::new())
    }

    #[cfg(feature = "wasm")]
    {
        Box::new(wasm::WasmHttpClient::new())
    }
}
