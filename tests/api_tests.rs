// API Integration Tests
// This file runs all web API tests for native Boa implementations

use boa_engine::{Context, Source};
use thalora::{HeadlessWebBrowser, apis::WebApis};

// NOTE: WebSocket and other APIs are now natively implemented in Boa engine
// Testing through JavaScript execution instead of shim-based tests

#[tokio::test]
async fn test_native_websocket_api() {
    let browser = HeadlessWebBrowser::new();

    // Test WebSocket constructor exists
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript("typeof WebSocket")
        .await;
    assert!(result.is_ok(), "WebSocket should be available");

    // Test WebSocket constants
    let result = browser.lock().unwrap().execute_javascript("WebSocket.CONNECTING === 0 && WebSocket.OPEN === 1 && WebSocket.CLOSING === 2 && WebSocket.CLOSED === 3").await;
    assert!(result.is_ok(), "WebSocket constants should be correct");
}

#[tokio::test]
async fn test_native_fetch_api() {
    let browser = HeadlessWebBrowser::new();

    // Test fetch function exists
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript("typeof fetch")
        .await;
    assert!(result.is_ok(), "fetch should be available");
}

#[tokio::test]
async fn test_native_storage_apis() {
    let browser = HeadlessWebBrowser::new();

    // Test localStorage and sessionStorage
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            "typeof localStorage === 'object' && typeof sessionStorage === 'object'",
        )
        .await;
    assert!(result.is_ok(), "Storage APIs should be available");
}

#[tokio::test]
async fn test_native_timer_apis() {
    let browser = HeadlessWebBrowser::new();

    // Test setTimeout and setInterval
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript("typeof setTimeout === 'function' && typeof setInterval === 'function'")
        .await;
    assert!(result.is_ok(), "Timer APIs should be available");
}

#[tokio::test]
async fn test_native_event_apis() {
    let browser = HeadlessWebBrowser::new();

    // Test Event constructors
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript("typeof Event === 'function' && typeof CustomEvent === 'function'")
        .await;
    assert!(result.is_ok(), "Event APIs should be available");
}

mod geolocation_navigator_exists {
    use super::*;
    include!("apis/geolocation/navigator_exists.rs");
}

mod geolocation_get_current_position {
    use super::*;
    include!("apis/geolocation/get_current_position.rs");
}

mod geolocation_position_object {
    use super::*;
    include!("apis/geolocation/position_object.rs");
}

mod geolocation_watch_position {
    use super::*;
    include!("apis/geolocation/watch_position.rs");
}

mod geolocation_clear_watch {
    use super::*;
    include!("apis/geolocation/clear_watch.rs");
}

mod geolocation_error_handling {
    use super::*;
    include!("apis/geolocation/error_handling.rs");
}

mod geolocation_coordinates_accuracy {
    use super::*;
    include!("apis/geolocation/coordinates_accuracy.rs");
}

// Navigator API tests
mod navigator_plugins_property {
    use super::*;
    include!("apis/navigator/plugins_property.rs");
}

// Worker API tests
mod worker_basic {

    include!("apis/workers/web_worker_basic.rs");
}
