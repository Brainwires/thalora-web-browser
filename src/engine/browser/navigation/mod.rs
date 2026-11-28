// Navigation module - provides browser navigation capabilities
//
// This module is split into focused sub-modules for better organization:
// - core: Basic URL navigation and HTTP requests
// - javascript: page loading (fetch, security headers, stylesheets) and waits
// - scripts: running the page's scripts on V8 (the deno fork's page runtime),
//   pumping its event loop, navigations requested by scripts
// - forms: Form interaction and submission
// - cookies: Cookie management (placeholder)
// - state: Session and page state management

mod cookies;
mod core;
pub(crate) mod csp;
mod forms;
pub mod hsts;
mod javascript;
pub(crate) mod mixed_content;
mod scripts;
pub(crate) mod sri;
mod state;

// Re-export all public navigation methods
// This allows users to call these methods directly on HeadlessWebBrowser
// without needing to know about the internal module structure
