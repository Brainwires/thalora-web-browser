// Handler modules organized by functionality
mod content;
mod form_management;
mod interaction;
mod navigation;
mod session_management;

// Re-export all handler methods are implemented directly on BrowserTools
// via impl blocks in each module, so no need for re-exports here.
// The handlers are accessible through BrowserTools instances.
