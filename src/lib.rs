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
pub use engine::{DomElement, EnhancedDom, EventListener, DomMutation};

pub use apis::websocket::{WebSocketManager, WebSocketConnection, WebSocketMessage, WebSocketJsApi};
pub use apis::storage::WebStorage;
pub use apis::events::DomEvent;

pub use features::{BrowserFingerprint, FingerprintManager, BrowserType};
pub use features::{ReactProcessor, ReactElement, ProcessedReactData};
pub use features::{AiMemoryHeap, MemoryData, ResearchEntry, CredentialEntry, SessionData, BookmarkEntry, NoteEntry, MemorySearchCriteria, MemorySortBy, SessionStatus, NotePriority, MemoryStatistics};

pub use protocols::{McpRequest, McpResponse, ToolCall, McpMessage, McpMessageContent, ToolResult};
pub use protocols::{CdpServer, CdpMessage, CdpCommand, CdpResponse, CdpEvent, CdpError, CdpDomain};
pub use protocols::{McpServer, MemoryTools};