//! Fetch & Networking APIs

// Native implementations using reqwest/tokio
#[cfg(feature = "native")]
pub mod event_source;
#[cfg(feature = "native")]
pub mod fetch;
#[cfg(feature = "native")]
pub mod xmlhttprequest;

// WebSocket implementations (tokio-tungstenite)
#[cfg(feature = "native")]
pub mod websocket;
#[cfg(feature = "native")]
pub mod websocket_stream;

/// Service Worker fetch event interception
pub mod sw_fetch_intercept;

#[cfg(test)]
mod tests;
