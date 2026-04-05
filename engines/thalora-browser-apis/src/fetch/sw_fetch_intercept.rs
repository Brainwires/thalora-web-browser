//! Service Worker Fetch Interception
//!
//! Provides the ability to intercept fetch requests through an active service worker's
//! fetch event handler. When a service worker controls a page and has a `fetch` event
//! listener or `onfetch` handler, fetch requests are dispatched as FetchEvents to the
//! service worker before going to the network.

use boa_engine::{Context, JsResult, JsValue, js_string, object::JsObject};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Global registry of active service worker fetch handlers.
/// Maps scope URLs to their fetch handler info.
static SW_FETCH_REGISTRY: OnceLock<Mutex<HashMap<String, ServiceWorkerFetchHandler>>> =
    OnceLock::new();

fn get_registry() -> &'static Mutex<HashMap<String, ServiceWorkerFetchHandler>> {
    SW_FETCH_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Information about a service worker's fetch handler
#[derive(Clone)]
pub struct ServiceWorkerFetchHandler {
    /// The scope this service worker controls
    pub scope: String,
    /// Whether a fetch event handler is registered
    pub has_fetch_handler: bool,
    /// Callback to dispatch a FetchEvent to the service worker context
    /// The Arc<Mutex<...>> wraps a boxed closure that takes request info and returns
    /// an optional Response value
    #[allow(clippy::type_complexity)]
    pub dispatch_fn: Arc<
        Mutex<
            Box<
                dyn Fn(
                        &str,
                        &str,
                        &HashMap<String, String>,
                        &mut Context,
                    ) -> JsResult<Option<JsValue>>
                    + Send,
            >,
        >,
    >,
}

/// Register a service worker's fetch handler for a given scope
pub fn register_sw_fetch_handler(scope: String, handler: ServiceWorkerFetchHandler) {
    if let Ok(mut registry) = get_registry().lock() {
        registry.insert(scope, handler);
    }
}

/// Unregister a service worker's fetch handler for a given scope
pub fn unregister_sw_fetch_handler(scope: &str) {
    if let Ok(mut registry) = get_registry().lock() {
        registry.remove(scope);
    }
}

/// Check if a URL is controlled by an active service worker with a fetch handler.
/// Returns the matching scope if found.
pub fn find_controlling_sw(url: &str) -> Option<String> {
    let registry = get_registry().lock().ok()?;
    // Find the longest matching scope (most specific)
    let mut best_match: Option<(&str, usize)> = None;
    for (scope, handler) in registry.iter() {
        if handler.has_fetch_handler && url.starts_with(scope.as_str()) {
            let scope_len = scope.len();
            if best_match.map_or(true, |(_, len)| scope_len > len) {
                best_match = Some((scope.as_str(), scope_len));
            }
        }
    }
    best_match.map(|(scope, _)| scope.to_string())
}

/// Attempt to intercept a fetch request through a service worker.
///
/// Returns `Some(JsValue)` if the service worker provided a response via `respondWith()`,
/// or `None` if the request should fall through to the network.
pub fn try_sw_fetch_intercept(
    url: &str,
    method: &str,
    headers: &HashMap<String, String>,
    context: &mut Context,
) -> JsResult<Option<JsValue>> {
    let scope = match find_controlling_sw(url) {
        Some(s) => s,
        None => return Ok(None),
    };

    let dispatch_fn = {
        let registry = get_registry().lock().map_err(|_| {
            boa_engine::JsNativeError::error().with_message("Failed to lock SW fetch registry")
        })?;
        match registry.get(&scope) {
            Some(handler) => handler.dispatch_fn.clone(),
            None => return Ok(None),
        }
    };

    let dispatch = dispatch_fn.lock().map_err(|_| {
        boa_engine::JsNativeError::error().with_message("Failed to lock SW fetch dispatch function")
    })?;

    dispatch(url, method, headers, context)
}

/// Dispatch a FetchEvent to the service worker global scope and check if respondWith was called.
///
/// This is typically called from within the service worker's context.
pub fn dispatch_fetch_event_to_sw(
    url: &str,
    method: &str,
    headers: &HashMap<String, String>,
    client_id: &str,
    context: &mut Context,
) -> JsResult<Option<JsValue>> {
    // Create the FetchEvent
    let fetch_event =
        crate::events::fetch_event::create_fetch_event(url, method, headers, client_id, context)?;

    let global = context.global_object();

    // Check for onfetch handler first
    let onfetch = global.get(js_string!("onfetch"), context)?;
    let mut handler_called = false;

    if !onfetch.is_null() && !onfetch.is_undefined() {
        if let Some(handler) = onfetch.as_callable() {
            handler.call(
                &global.clone().into(),
                &[fetch_event.clone().into()],
                context,
            )?;
            handler_called = true;
        }
    }

    // Also try addEventListener-based dispatch
    if !handler_called {
        // Try dispatching via the dispatchEvent mechanism if available
        let dispatch_event_val = global.get(js_string!("dispatchEvent"), context)?;
        if let Some(dispatcher) = dispatch_event_val.as_callable() {
            let _ = dispatcher.call(
                &global.clone().into(),
                &[fetch_event.clone().into()],
                context,
            );
        }
    }

    // Check if respondWith was called on the event
    Ok(crate::events::fetch_event::get_fetch_event_response(
        &fetch_event,
    ))
}
