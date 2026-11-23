// There is no JavaScript engine at the moment: Boa was removed and V8 (via
// the Brainwires deno fork) is not wired in yet. Operations that need one
// must fail with a clear error instead of appearing to succeed.

// The browser is only used from this test's own thread, so holding its
// MutexGuard across .await cannot contend or deadlock.
#![allow(clippy::await_holding_lock)]

use thalora::HeadlessWebBrowser;
use thalora::engine::JS_UNAVAILABLE;

#[tokio::test]
async fn execute_javascript_reports_that_js_is_unavailable() {
    let browser = HeadlessWebBrowser::new();
    let result = browser.lock().unwrap().execute_javascript("1 + 1").await;
    let error = result.expect_err("JavaScript must not appear to run");
    assert_eq!(error.to_string(), JS_UNAVAILABLE);
    assert_eq!(
        JS_UNAVAILABLE,
        "JavaScript execution is unavailable: the Boa engine was removed; V8 support is in progress"
    );
}
