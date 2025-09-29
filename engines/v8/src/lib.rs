// V8 Engine Integration for Thalora
//
// This module provides a V8 JavaScript engine implementation that is compatible
// with the existing Thalora engine interface, allowing for direct comparison
// and testing between Boa and V8 engines.

pub mod engine;
pub mod runtime;
pub mod context;
pub mod polyfills;

// Re-exports for clean API
pub use engine::V8JavaScriptEngine;
pub use runtime::V8Runtime;
pub use context::V8Context;