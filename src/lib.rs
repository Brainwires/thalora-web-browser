#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(unused)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_must_use)]
#![allow(let_underscore_drop)]
// Core browser engine
pub mod engine;

// Web APIs and standards
pub mod apis;

// Advanced browser features
pub mod features;

// Communication protocols
pub mod protocols;

// Re-export main components for clean public API
pub use engine::{HeadlessWebBrowser, ScrapedData, Link, Image, Form, FormField, InteractionResponse, BrowserStorage, AuthContext};
pub use engine::{RustRenderer, CssProcessor, LayoutEngine, LayoutResult, JavaScriptEngine};
// EventListener is now natively implemented in Boa engine

// websocket API is now natively implemented in Boa engine
// WebStorage is now natively implemented in Boa engine
// events API is now natively implemented in Boa engine

pub use features::{BrowserFingerprint, FingerprintManager, BrowserType};
pub use features::{AiMemoryHeap, MemoryData, ResearchEntry, CredentialEntry, SessionData, BookmarkEntry, NoteEntry, MemorySearchCriteria, MemorySortBy, SessionStatus, NotePriority, MemoryStatistics};

pub use protocols::{McpRequest, McpResponse, ToolCall, McpMessage, McpMessageContent, ToolResult};
pub use protocols::{CdpServer, CdpMessage, CdpCommand, CdpResponse, CdpEvent, CdpError, CdpDomain};
pub use protocols::{McpServer, MemoryTools};