#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(unused)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_must_use)]
#![allow(let_underscore_drop)]

// WASM bindings module (only for wasm builds)
#[cfg(feature = "wasm")]
pub mod wasm_bindings;

// Platform abstraction layer
pub mod platform;

// Core browser engine
pub mod engine;

// Web APIs and standards
pub mod apis;

// Advanced browser features
pub mod features;

// Standalone web search module (requires core networking)
#[cfg(feature = "core")]
pub mod web_search;

// FFI layer for C-compatible bindings (requires tokio runtime from core)
#[cfg(feature = "core")]
pub mod ffi;

// Communication protocols (requires tokio/networking from core)
#[cfg(feature = "core")]
pub mod protocols;

// Debug utilities
pub mod debug_utils;

// Re-export main components for clean public API
#[cfg(feature = "core")]
pub use engine::{HeadlessWebBrowser, ScrapedData, Link, Image, Form, FormField, InteractionResponse, BrowserStorage, AuthContext};
pub use engine::{RustRenderer, CssProcessor, LayoutEngine, LayoutResult};
#[cfg(any(feature = "core", feature = "wasm"))]
pub use engine::JavaScriptEngine;
pub use engine::{EngineType, EngineFactory, ThaloraBrowserEngine, EngineConfig};
// EventListener is now natively implemented in Boa engine

// websocket API is now natively implemented in Boa engine
// WebStorage is now natively implemented in Boa engine
// events API is now natively implemented in Boa engine

#[cfg(feature = "core")]
pub use features::{BrowserFingerprint, FingerprintManager, BrowserType};
#[cfg(feature = "core")]
pub use features::{AiMemoryHeap, MemoryData, ResearchEntry, CredentialEntry, SessionData, BookmarkEntry, NoteEntry, MemorySearchCriteria, MemorySortBy, SessionStatus, NotePriority, MemoryStatistics};

// Web search re-exports for convenient access
#[cfg(feature = "core")]
pub use web_search::{perform_search, SearchResult, SearchResults};

// Protocol exports (requires core networking)
#[cfg(feature = "core")]
pub use protocols::{McpRequest, McpResponse, ToolCall, McpMessage, McpMessageContent, ToolResult};
#[cfg(feature = "core")]
pub use protocols::{CdpServer, CdpMessage, CdpCommand, CdpResponse, CdpEvent, CdpError, CdpDomain};
#[cfg(feature = "core")]
pub use protocols::{McpServer, MemoryTools};
#[cfg(feature = "wasm-debug")]
pub use protocols::wasm_debug_tools::WasmDebugTools;
