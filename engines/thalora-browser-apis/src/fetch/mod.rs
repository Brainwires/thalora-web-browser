//! Fetch & Networking APIs

// Native implementations using rquest/tokio
#[cfg(feature = "_native-core")]
pub mod event_source;
#[cfg(feature = "_native-core")]
pub mod fetch;
#[cfg(feature = "_native-core")]
pub mod xmlhttprequest;

// WASM stubs - browser's native APIs are used directly
#[cfg(feature = "wasm")]
pub mod event_source_wasm;
#[cfg(feature = "wasm")]
pub mod fetch_wasm;
#[cfg(feature = "wasm")]
pub mod xmlhttprequest_wasm;

// Re-export for uniform API
#[cfg(feature = "wasm")]
pub use event_source_wasm as event_source;
#[cfg(feature = "wasm")]
pub use fetch_wasm as fetch;
#[cfg(feature = "wasm")]
pub use xmlhttprequest_wasm as xmlhttprequest;

// WebSocket implementations - native uses tokio-tungstenite, WASM uses web-sys
#[cfg(feature = "_native-core")]
pub mod websocket;
#[cfg(feature = "_native-core")]
pub mod websocket_stream;

// WASM WebSocket stubs (web-sys WebSocket is used directly in JavaScript)
#[cfg(feature = "wasm")]
pub mod websocket_wasm;
#[cfg(feature = "wasm")]
pub use websocket_wasm as websocket;

#[cfg(test)]
mod tests;
