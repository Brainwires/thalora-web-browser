pub mod mcp;
pub mod renderer;
pub mod browser;
// pub mod enhanced_js;
pub mod enhanced_dom; 
pub mod react_processor;
pub mod websocket;
pub mod challenge_solver;
pub mod fingerprinting;

pub use browser::{HeadlessWebBrowser, ScrapedData, Link, Image, Form, FormField, InteractionResponse, BrowserStorage, AuthContext};
pub use renderer::{RustRenderer, CssProcessor, LayoutEngine, LayoutResult};
pub use mcp::{McpRequest, McpResponse, ToolCall, McpMessage, McpMessageContent, ToolResult};
// pub use enhanced_js::EnhancedJavaScriptEngine;
pub use enhanced_dom::{DomManager, DomElement, DomMutation, EventListener, WebStorage};
pub use react_processor::{ReactProcessor, ReactElement, ProcessedReactData};
pub use websocket::{WebSocketManager, WebSocketConnection, WebSocketMessage, WebSocketJsApi};
pub use challenge_solver::{ChallengeSolver, ChallengeType, ChallengeResult, SolverConfig, ChallengePatterns};
pub use fingerprinting::{BrowserFingerprint, FingerprintManager, BrowserType};