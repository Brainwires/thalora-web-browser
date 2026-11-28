// Page JavaScript runs on V8, on browser threads (`BrowserThread`). A page
// that is static — JavaScript disabled, the page runtime not started, or a
// browser used outside a browser thread, as here — must fail operations
// that need script with a clear error instead of appearing to succeed.
//
// The tests that run JavaScript are the unit tests of
// src/engine/browser/navigation/scripts.rs (in process) and
// tests/protocols/e2e_fixture_tests.rs (through the MCP tools).

// The browser is only used from this test's own thread, so holding its
// MutexGuard across .await cannot contend or deadlock.
#![allow(clippy::await_holding_lock)]

use thalora::HeadlessWebBrowser;
use thalora::engine::JS_UNAVAILABLE;

#[tokio::test]
async fn evaluating_without_a_page_is_an_error() {
    let browser = HeadlessWebBrowser::new();
    let result = browser.lock().unwrap().execute_javascript("1 + 1").await;
    let error = result.expect_err("JavaScript must not appear to run");
    assert!(error.to_string().contains("No page loaded"), "{error}");
}

#[test]
fn the_static_page_error_says_what_is_wrong() {
    assert_eq!(
        JS_UNAVAILABLE,
        "JavaScript execution is unavailable: this page was loaded as static HTML"
    );
    assert_eq!(
        thalora::engine::js_unavailable().to_string(),
        JS_UNAVAILABLE
    );
}
