// Event Bubbling Integration Tests
// Tests DOM spec 3-phase event propagation: capture → target → bubble

use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_event_target_set_during_dispatch() {
    let browser = HeadlessWebBrowser::new();
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
            (function() {
                var target = new EventTarget();
                var receivedTarget = null;
                target.addEventListener('test', function(e) {
                    receivedTarget = e.target;
                });
                var event = new Event('test');
                target.dispatchEvent(event);
                return receivedTarget !== null ? 'target_set' : 'target_null';
            })()
            "#,
        )
        .await;
    assert!(result.is_ok(), "dispatchEvent should not crash");
    let val = result.unwrap();
    assert!(
        val.contains("target_set"),
        "event.target should be set during dispatch, got: {}",
        val
    );
}

#[tokio::test]
async fn test_event_phase_at_target() {
    let browser = HeadlessWebBrowser::new();
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
            (function() {
                var target = new EventTarget();
                var phase = -1;
                target.addEventListener('test', function(e) {
                    phase = e.eventPhase;
                });
                target.dispatchEvent(new Event('test'));
                return 'phase=' + phase;
            })()
            "#,
        )
        .await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(
        val.contains("phase=2"),
        "Should be AT_TARGET (2), got: {}",
        val
    );
}

#[tokio::test]
async fn test_stop_immediate_propagation() {
    let browser = HeadlessWebBrowser::new();
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
            (function() {
                var target = new EventTarget();
                var calls = 0;
                target.addEventListener('test', function(e) {
                    calls++;
                    e.stopImmediatePropagation();
                });
                target.addEventListener('test', function(e) {
                    calls++; // Should NOT be called
                });
                target.dispatchEvent(new Event('test'));
                return 'calls=' + calls;
            })()
            "#,
        )
        .await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(
        val.contains("calls=1"),
        "stopImmediatePropagation should prevent second listener, got: {}",
        val
    );
}

#[tokio::test]
async fn test_prevent_default_returns_false() {
    let browser = HeadlessWebBrowser::new();
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
            (function() {
                var target = new EventTarget();
                target.addEventListener('test', function(e) {
                    e.preventDefault();
                });
                var result = target.dispatchEvent(new Event('test', {cancelable: true}));
                return 'result=' + result;
            })()
            "#,
        )
        .await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(
        val.contains("result=false"),
        "preventDefault should make dispatchEvent return false, got: {}",
        val
    );
}

#[tokio::test]
async fn test_once_listener_removed_after_fire() {
    let browser = HeadlessWebBrowser::new();
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
            (function() {
                var target = new EventTarget();
                var calls = 0;
                target.addEventListener('test', function() { calls++; }, {once: true});
                target.dispatchEvent(new Event('test'));
                target.dispatchEvent(new Event('test'));
                return 'calls=' + calls;
            })()
            "#,
        )
        .await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(
        val.contains("calls=1"),
        "once listener should fire exactly once, got: {}",
        val
    );
}

#[tokio::test]
async fn test_capture_vs_bubble_listeners() {
    let browser = HeadlessWebBrowser::new();
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
            (function() {
                var target = new EventTarget();
                var order = [];
                // At target phase, both capture and non-capture fire
                target.addEventListener('test', function() { order.push('bubble'); }, false);
                target.addEventListener('test', function() { order.push('capture'); }, true);
                target.dispatchEvent(new Event('test'));
                return order.join(',');
            })()
            "#,
        )
        .await;
    assert!(result.is_ok());
    let val = result.unwrap();
    // At target phase, all listeners fire in registration order
    assert!(
        val.contains("bubble") && val.contains("capture"),
        "Both capture and bubble listeners should fire at target, got: {}",
        val
    );
}

#[tokio::test]
async fn test_non_bubbling_event() {
    let browser = HeadlessWebBrowser::new();
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
            (function() {
                var target = new EventTarget();
                var called = false;
                target.addEventListener('test', function() { called = true; });
                // Event with bubbles=false (default)
                target.dispatchEvent(new Event('test', {bubbles: false}));
                return 'called=' + called;
            })()
            "#,
        )
        .await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(
        val.contains("called=true"),
        "Non-bubbling event should still fire on target, got: {}",
        val
    );
}
