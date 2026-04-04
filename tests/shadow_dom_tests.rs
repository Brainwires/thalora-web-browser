// Shadow DOM Integration Tests
// Tests for attachShadow, ShadowRoot properties, and borrow safety

use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_attach_shadow_open_mode() {
    let browser = HeadlessWebBrowser::new();

    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
            (function() {
                var div = document.createElement('div');
                var shadow = div.attachShadow({mode: 'open'});
                return shadow !== null && shadow !== undefined ? 'ok' : 'fail';
            })()
            "#,
        )
        .await;
    assert!(result.is_ok(), "attachShadow should not crash");
    let val = result.unwrap();
    assert!(val.contains("ok"), "Shadow root should be created, got: {}", val);
}

#[tokio::test]
async fn test_attach_shadow_closed_mode() {
    let browser = HeadlessWebBrowser::new();

    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
            (function() {
                var div = document.createElement('div');
                var shadow = div.attachShadow({mode: 'closed'});
                var shadowRootIsNull = div.shadowRoot === null;
                return 'shadow=' + (shadow !== null) + ', shadowRoot_null=' + shadowRootIsNull;
            })()
            "#,
        )
        .await;
    assert!(result.is_ok(), "Closed shadow should not crash");
    let val = result.unwrap();
    assert!(
        val.contains("shadow=true"),
        "Closed shadow root should be returned to caller, got: {}",
        val
    );
}

#[tokio::test]
async fn test_shadow_root_mode_property() {
    let browser = HeadlessWebBrowser::new();

    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
            (function() {
                var div = document.createElement('div');
                var shadow = div.attachShadow({mode: 'open'});
                return shadow.mode;
            })()
            "#,
        )
        .await;
    assert!(result.is_ok(), "Getting shadow mode should not crash");
    let val = result.unwrap();
    assert!(
        val.contains("open"),
        "Shadow mode should be 'open', got: {}",
        val
    );
}

#[tokio::test]
async fn test_shadow_root_host_property() {
    let browser = HeadlessWebBrowser::new();

    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
            (function() {
                var div = document.createElement('div');
                var shadow = div.attachShadow({mode: 'open'});
                return shadow.host !== null && shadow.host !== undefined ? 'has_host' : 'no_host';
            })()
            "#,
        )
        .await;
    assert!(result.is_ok(), "Getting shadow host should not crash (no BorrowMutError)");
}

#[tokio::test]
async fn test_shadow_root_inner_html() {
    let browser = HeadlessWebBrowser::new();

    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
            (function() {
                var div = document.createElement('div');
                var shadow = div.attachShadow({mode: 'open'});
                shadow.innerHTML = '<p>Hello Shadow</p>';
                return typeof shadow.innerHTML;
            })()
            "#,
        )
        .await;
    assert!(result.is_ok(), "Setting shadow innerHTML should not crash");
}

#[tokio::test]
async fn test_attach_shadow_duplicate_throws() {
    let browser = HeadlessWebBrowser::new();

    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
            (function() {
                try {
                    var div = document.createElement('div');
                    div.attachShadow({mode: 'open'});
                    div.attachShadow({mode: 'open'});
                    return 'should_have_thrown';
                } catch(e) {
                    return 'correctly_threw: ' + e.message;
                }
            })()
            "#,
        )
        .await;
    assert!(result.is_ok(), "Duplicate attachShadow should throw, not crash");
    let val = result.unwrap();
    assert!(
        val.contains("correctly_threw") || val.contains("already"),
        "Should throw on duplicate, got: {}",
        val
    );
}

#[tokio::test]
async fn test_shadow_dom_no_borrow_panic() {
    let browser = HeadlessWebBrowser::new();

    // This is the scenario most likely to trigger BorrowMutError:
    // Create shadow, immediately access host, set innerHTML, access again
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
            (function() {
                try {
                    var div = document.createElement('div');
                    var shadow = div.attachShadow({mode: 'open'});

                    // Rapid access pattern that could trigger re-entrant borrows
                    var host = shadow.host;
                    var mode = shadow.mode;
                    shadow.innerHTML = '<slot></slot>';
                    var host2 = shadow.host;
                    var mode2 = shadow.mode;

                    return 'ok';
                } catch(e) {
                    return 'error: ' + e.message;
                }
            })()
            "#,
        )
        .await;
    assert!(result.is_ok(), "Rapid shadow DOM access should not panic with BorrowMutError");
}
