//! Navigation Bridge
//!
//! Provides a communication bridge between DOM events and browser navigation.
//! When JavaScript triggers navigation (e.g., clicking a link), requests are
//! queued here for the browser engine to process.

use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;

/// Global navigation queue for pending navigation requests
pub static NAVIGATION_QUEUE: OnceLock<Arc<Mutex<Vec<NavigationRequest>>>> = OnceLock::new();

/// A navigation request from JavaScript
#[derive(Debug, Clone)]
pub struct NavigationRequest {
    /// The URL to navigate to
    pub url: String,
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Form data for POST requests
    pub form_data: Option<HashMap<String, String>>,
    /// Timestamp when request was created
    pub timestamp: u64,
}

impl NavigationRequest {
    /// Create a new GET navigation request
    pub fn new_get(url: String) -> Self {
        Self {
            url,
            method: "GET".to_string(),
            form_data: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// Create a new POST navigation request with form data
    pub fn new_post(url: String, form_data: HashMap<String, String>) -> Self {
        Self {
            url,
            method: "POST".to_string(),
            form_data: Some(form_data),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }
}

/// Queue a navigation request (from link clicks)
pub fn queue_navigation(url: &str) {
    let queue = NAVIGATION_QUEUE.get_or_init(|| Arc::new(Mutex::new(Vec::new())));
    if let Ok(mut q) = queue.lock() {
        q.push(NavigationRequest::new_get(url.to_string()));
    }
}

/// Queue a form submission
pub fn queue_form_submission(action: &str, method: &str) {
    let queue = NAVIGATION_QUEUE.get_or_init(|| Arc::new(Mutex::new(Vec::new())));
    if let Ok(mut q) = queue.lock() {
        q.push(NavigationRequest {
            url: action.to_string(),
            method: method.to_uppercase(),
            form_data: None, // Form data collection would happen at the browser engine level
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        });
    }
}

/// Drain and return all pending navigation requests
/// Called by the browser engine after JS execution
pub fn drain_navigation_requests() -> Vec<NavigationRequest> {
    let queue = NAVIGATION_QUEUE.get_or_init(|| Arc::new(Mutex::new(Vec::new())));
    if let Ok(mut q) = queue.lock() {
        q.drain(..).collect()
    } else {
        Vec::new()
    }
}

/// Check if there are any pending navigation requests
pub fn has_pending_navigations() -> bool {
    let queue = NAVIGATION_QUEUE.get_or_init(|| Arc::new(Mutex::new(Vec::new())));
    if let Ok(q) = queue.lock() {
        !q.is_empty()
    } else {
        false
    }
}

/// Clear all pending navigation requests
pub fn clear_navigation_queue() {
    let queue = NAVIGATION_QUEUE.get_or_init(|| Arc::new(Mutex::new(Vec::new())));
    if let Ok(mut q) = queue.lock() {
        q.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_navigation() {
        clear_navigation_queue();
        queue_navigation("https://example.com");
        assert!(has_pending_navigations());

        let requests = drain_navigation_requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].url, "https://example.com");
        assert_eq!(requests[0].method, "GET");
    }

    #[test]
    fn test_queue_form_submission() {
        clear_navigation_queue();
        queue_form_submission("/submit", "POST");

        let requests = drain_navigation_requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].url, "/submit");
        assert_eq!(requests[0].method, "POST");
    }
}
