// Handler modules organized by functionality
mod navigation;
mod interaction;
mod content;
mod session_management;
mod form_management;

// Re-export all handler methods are implemented directly on BrowserTools
// via impl blocks in each module, so no need for re-exports here.
// The handlers are accessible through BrowserTools instances.
