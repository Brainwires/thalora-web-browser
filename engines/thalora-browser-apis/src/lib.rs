//! Thalora Browser APIs
//!
//! Web standard APIs extracted from Boa engine fork for use in Thalora browser.

// Re-export Boa engine types needed for API bindings
pub use boa_engine;

// DOM APIs
pub mod dom;

// Fetch & Networking APIs
pub mod fetch;

// Storage APIs
pub mod storage;

// Web Workers
pub mod worker;

// File APIs
pub mod file;

// Event APIs
pub mod events;

// Browser objects
pub mod browser;

// Crypto APIs
pub mod crypto;

// Console API
pub mod console;

// Timer APIs
pub mod timers;

// WebRTC APIs
pub mod webrtc;

// Streams APIs
pub mod streams;

// Observer APIs
pub mod observers;

// Messaging APIs
pub mod messaging;

// Miscellaneous APIs
pub mod misc;

/// Initialize all browser APIs in a Boa context
pub fn initialize_browser_apis(context: &mut boa_engine::Context) -> anyhow::Result<()> {
    // APIs are initialized within Boa engine for now
    // This function serves as a placeholder for future initialization
    Ok(())
}
