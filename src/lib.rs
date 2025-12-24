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

// Standalone web search module (available for web-search feature without full MCP)
#[cfg(any(feature = "native", feature = "web-search"))]
pub mod web_search;

// Communication protocols (only for native builds - requires tokio/networking)
#[cfg(feature = "native")]
pub mod protocols;

// GUI module (requires gui feature - includes windowing, egui, wgpu)
#[cfg(feature = "gui")]
pub mod gui;

// Debug utilities
pub mod debug_utils;

// Re-export main components for clean public API
#[cfg(any(feature = "native", feature = "web-search", feature = "wasm"))]
pub use engine::{HeadlessWebBrowser, ScrapedData, Link, Image, Form, FormField, InteractionResponse, BrowserStorage, AuthContext};
pub use engine::{RustRenderer, CssProcessor, LayoutEngine, LayoutResult};
#[cfg(any(feature = "native", feature = "web-search", feature = "wasm"))]
pub use engine::JavaScriptEngine;
pub use engine::{EngineType, EngineFactory, ThaloraBrowserEngine, EngineConfig};
// EventListener is now natively implemented in Boa engine

// websocket API is now natively implemented in Boa engine
// WebStorage is now natively implemented in Boa engine
// events API is now natively implemented in Boa engine

#[cfg(any(feature = "native", feature = "web-search"))]
pub use features::{BrowserFingerprint, FingerprintManager, BrowserType};
#[cfg(any(feature = "native", feature = "web-search"))]
pub use features::{AiMemoryHeap, MemoryData, ResearchEntry, CredentialEntry, SessionData, BookmarkEntry, NoteEntry, MemorySearchCriteria, MemorySortBy, SessionStatus, NotePriority, MemoryStatistics};

// Web search re-exports for convenient access
#[cfg(any(feature = "native", feature = "web-search"))]
pub use web_search::{perform_search, SearchResult, SearchResults};

// Protocol exports (only for native builds)
#[cfg(feature = "native")]
pub use protocols::{McpRequest, McpResponse, ToolCall, McpMessage, McpMessageContent, ToolResult};
#[cfg(feature = "native")]
pub use protocols::{CdpServer, CdpMessage, CdpCommand, CdpResponse, CdpEvent, CdpError, CdpDomain};
#[cfg(feature = "native")]
pub use protocols::{McpServer, MemoryTools};

// GUI exports (requires gui feature)
#[cfg(feature = "gui")]
pub use gui::{BrowserUI, NavigationState, BrowserAction, InputHandler, TabManager, Tab, TabId};