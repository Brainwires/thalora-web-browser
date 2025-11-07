// Navigation module - provides browser navigation capabilities
//
// This module is split into focused sub-modules for better organization:
// - core: Basic URL navigation and HTTP requests
// - javascript: JS execution integration with navigation
// - forms: Form interaction and submission
// - cookies: Cookie management (placeholder)
// - state: Session and page state management

mod core;
mod javascript;
mod forms;
mod cookies;
mod state;

// Re-export all public navigation methods
// This allows users to call these methods directly on HeadlessWebBrowser
// without needing to know about the internal module structure
