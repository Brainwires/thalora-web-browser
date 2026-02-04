//! Window Registry - Global frame hierarchy tracking for iframe communication
//!
//! This module provides a registry that tracks the parent-child relationships
//! between windows (main window and iframes), enabling proper implementation of:
//! - `window.parent` - returns parent window or self if top-level
//! - `window.top` - returns topmost window in hierarchy
//! - `window.frameElement` - returns iframe element or null
//! - `postMessage` - cross-window message routing
//!
//! ## Architecture
//!
//! The registry stores WindowEntry objects indexed by WindowId, containing only
//! the relationships between windows (parent_id, children) and metadata (origin).
//!
//! **IMPORTANT**: We do NOT store JsObject references in the registry because:
//! 1. JsObjects are garbage-collected by Boa's GC
//! 2. Storing them in external collections can cause use-after-free
//! 3. The GC may not see our references and could collect live objects
//!
//! Instead, Window objects store their window_id in WindowData, and we can
//! reconstruct the hierarchy by walking up the parent chain.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique identifier for a window in the registry
pub type WindowId = u64;

/// Counter for generating unique window IDs
static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(1);

/// Entry representing a window (main or iframe) in the registry
/// Does NOT store JsObject references to avoid GC issues
#[derive(Debug, Clone)]
pub struct WindowEntry {
    /// Unique ID of this window
    pub id: WindowId,
    /// Parent window ID (None if this is the top-level window)
    pub parent_id: Option<WindowId>,
    /// Child window IDs
    pub children: Vec<WindowId>,
    /// Origin of this window (for same-origin policy)
    pub origin: String,
}

/// Registry for tracking window hierarchy
/// Uses thread_local storage since it's used from single-threaded JS execution
pub struct WindowRegistry {
    /// Map of window ID to window entry
    windows: HashMap<WindowId, WindowEntry>,
    /// The ID of the top-level window
    top_level_id: Option<WindowId>,
}

impl WindowRegistry {
    /// Create a new window registry
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            top_level_id: None,
        }
    }

    /// Generate a new unique window ID
    fn next_id() -> WindowId {
        NEXT_WINDOW_ID.fetch_add(1, Ordering::SeqCst)
    }

    /// Register the top-level window and return its ID
    pub fn register_top_window(&mut self, origin: String) -> WindowId {
        let id = Self::next_id();

        let entry = WindowEntry {
            id,
            parent_id: None,
            children: Vec::new(),
            origin,
        };

        self.windows.insert(id, entry);
        self.top_level_id = Some(id);

        eprintln!("📋 WINDOW_REGISTRY: Registered top-level window with ID {}", id);

        id
    }

    /// Register an iframe window and return its ID
    pub fn register_iframe_window(&mut self, parent_id: WindowId, origin: String) -> WindowId {
        let id = Self::next_id();

        let entry = WindowEntry {
            id,
            parent_id: Some(parent_id),
            children: Vec::new(),
            origin,
        };

        self.windows.insert(id, entry);

        // Add this window as a child of the parent
        if let Some(parent) = self.windows.get_mut(&parent_id) {
            parent.children.push(id);
        }

        eprintln!(
            "📋 WINDOW_REGISTRY: Registered iframe window with ID {} (parent: {})",
            id, parent_id
        );

        id
    }

    /// Get the top-level window ID
    pub fn get_top_level_id(&self) -> Option<WindowId> {
        self.top_level_id
    }

    /// Get a window entry by ID
    pub fn get_entry(&self, id: WindowId) -> Option<&WindowEntry> {
        self.windows.get(&id)
    }

    /// Get the parent window ID
    pub fn get_parent_id(&self, id: WindowId) -> Option<WindowId> {
        self.windows.get(&id).and_then(|e| e.parent_id)
    }

    /// Get the top-level window ID by walking up the hierarchy
    pub fn get_top_id(&self, id: WindowId) -> Option<WindowId> {
        let mut current_id = id;

        while let Some(entry) = self.windows.get(&current_id) {
            if let Some(parent_id) = entry.parent_id {
                current_id = parent_id;
            } else {
                // This is the top-level window
                return Some(current_id);
            }
        }

        None
    }

    /// Check if a window is the top-level window
    pub fn is_top_level(&self, id: WindowId) -> bool {
        self.windows.get(&id).map(|e| e.parent_id.is_none()).unwrap_or(false)
    }

    /// Get all child window IDs
    pub fn get_children(&self, id: WindowId) -> Vec<WindowId> {
        self.windows
            .get(&id)
            .map(|e| e.children.clone())
            .unwrap_or_default()
    }

    /// Get the origin of a window
    pub fn get_origin(&self, id: WindowId) -> Option<String> {
        self.windows.get(&id).map(|e| e.origin.clone())
    }

    /// Update the origin of a window
    pub fn set_origin(&mut self, id: WindowId, origin: String) {
        if let Some(entry) = self.windows.get_mut(&id) {
            entry.origin = origin;
        }
    }

    /// Check if two windows have the same origin
    pub fn same_origin(&self, id1: WindowId, id2: WindowId) -> bool {
        match (self.windows.get(&id1), self.windows.get(&id2)) {
            (Some(e1), Some(e2)) => e1.origin == e2.origin,
            _ => false,
        }
    }

    /// Unregister a window and its children
    pub fn unregister(&mut self, id: WindowId) {
        if let Some(entry) = self.windows.remove(&id) {
            // Remove this window from parent's children list
            if let Some(parent_id) = entry.parent_id {
                if let Some(parent) = self.windows.get_mut(&parent_id) {
                    parent.children.retain(|&child_id| child_id != id);
                }
            }

            // Recursively unregister children
            for child_id in entry.children {
                self.unregister(child_id);
            }
        }
    }

    /// Clear the registry (for testing)
    pub fn clear(&mut self) {
        self.windows.clear();
        self.top_level_id = None;
    }

    /// Get statistics about the registry
    pub fn stats(&self) -> (usize, usize) {
        let top_level = self.windows.values().filter(|e| e.parent_id.is_none()).count();
        let iframes = self.windows.values().filter(|e| e.parent_id.is_some()).count();
        (top_level, iframes)
    }
}

impl Default for WindowRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-local storage for the window registry
thread_local! {
    static WINDOW_REGISTRY: RefCell<WindowRegistry> = RefCell::new(WindowRegistry::new());
}

/// Get access to the window registry
pub fn with_registry<F, R>(f: F) -> R
where
    F: FnOnce(&WindowRegistry) -> R,
{
    WINDOW_REGISTRY.with(|registry| f(&registry.borrow()))
}

/// Get mutable access to the window registry
pub fn with_registry_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut WindowRegistry) -> R,
{
    WINDOW_REGISTRY.with(|registry| f(&mut registry.borrow_mut()))
}

/// Get the window registry handle
pub fn get_registry() -> RegistryHandle {
    RegistryHandle
}

/// Handle for accessing the window registry
#[derive(Clone, Copy)]
pub struct RegistryHandle;

impl RegistryHandle {
    /// Register the top-level window
    pub fn register_top_window(&self, origin: String) -> WindowId {
        with_registry_mut(|r| r.register_top_window(origin))
    }

    /// Register an iframe window
    pub fn register_iframe_window(&self, parent_id: WindowId, origin: String) -> WindowId {
        with_registry_mut(|r| r.register_iframe_window(parent_id, origin))
    }

    /// Get the top-level window ID
    pub fn get_top_level_id(&self) -> Option<WindowId> {
        with_registry(|r| r.get_top_level_id())
    }

    /// Get the parent window ID
    pub fn get_parent_id(&self, id: WindowId) -> Option<WindowId> {
        with_registry(|r| r.get_parent_id(id))
    }

    /// Get the top-level window ID from any window
    pub fn get_top_id(&self, id: WindowId) -> Option<WindowId> {
        with_registry(|r| r.get_top_id(id))
    }

    /// Check if window is top-level
    pub fn is_top_level(&self, id: WindowId) -> bool {
        with_registry(|r| r.is_top_level(id))
    }

    /// Get the origin
    pub fn get_origin(&self, id: WindowId) -> Option<String> {
        with_registry(|r| r.get_origin(id))
    }

    /// Set the origin
    pub fn set_origin(&self, id: WindowId, origin: String) {
        with_registry_mut(|r| r.set_origin(id, origin))
    }

    /// Check same origin
    pub fn same_origin(&self, id1: WindowId, id2: WindowId) -> bool {
        with_registry(|r| r.same_origin(id1, id2))
    }
}

/// Reset the window registry (for testing only)
#[cfg(test)]
pub fn reset_registry() {
    WINDOW_REGISTRY.with(|registry| registry.borrow_mut().clear());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_top_window() {
        reset_registry();

        let id = get_registry().register_top_window("https://example.com".to_string());

        assert!(id > 0);
        assert_eq!(get_registry().get_top_level_id(), Some(id));
        assert!(get_registry().is_top_level(id));
    }

    #[test]
    fn test_register_iframe() {
        reset_registry();

        let top_id = get_registry().register_top_window("https://example.com".to_string());
        let iframe_id = get_registry().register_iframe_window(top_id, "https://example.com".to_string());

        assert!(get_registry().get_parent_id(iframe_id).is_some());
        assert_eq!(get_registry().get_parent_id(iframe_id), Some(top_id));
        assert!(!get_registry().is_top_level(iframe_id));
    }

    #[test]
    fn test_hierarchy_traversal() {
        reset_registry();

        let top_id = get_registry().register_top_window("https://example.com".to_string());
        let iframe1_id = get_registry().register_iframe_window(top_id, "https://example.com".to_string());
        let iframe2_id = get_registry().register_iframe_window(iframe1_id, "https://example.com".to_string());

        // Test get_top_id from nested iframe
        assert_eq!(get_registry().get_top_id(iframe2_id), Some(top_id));

        // Test get_parent_id chain
        assert_eq!(get_registry().get_parent_id(iframe2_id), Some(iframe1_id));
        assert_eq!(get_registry().get_parent_id(iframe1_id), Some(top_id));

        // Test top window has no parent
        assert_eq!(get_registry().get_parent_id(top_id), None);
    }
}
