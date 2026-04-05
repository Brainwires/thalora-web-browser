// Web Animations API tests
// Tests Element.animate() state machine correctness

use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_animate_returns_animation_object() {
    let browser = HeadlessWebBrowser::new();
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
        (function() {
            var div = document.createElement('div');
            var anim = div.animate([{opacity: 0}, {opacity: 1}], 100);
            return typeof anim === 'object' && anim !== null ? 'ok' : 'fail';
        })()
        "#,
        )
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("ok"));
}

#[tokio::test]
async fn test_animate_play_state_running() {
    let browser = HeadlessWebBrowser::new();
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
        (function() {
            var div = document.createElement('div');
            var anim = div.animate([], 1000);
            return anim.playState;
        })()
        "#,
        )
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("running"));
}

#[tokio::test]
async fn test_animate_pause_state() {
    let browser = HeadlessWebBrowser::new();
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
        (function() {
            var div = document.createElement('div');
            var anim = div.animate([], 1000);
            anim.pause();
            return anim.playState;
        })()
        "#,
        )
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("paused"));
}

#[tokio::test]
async fn test_animate_cancel_state() {
    let browser = HeadlessWebBrowser::new();
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
        (function() {
            var div = document.createElement('div');
            var anim = div.animate([], 1000);
            anim.cancel();
            return anim.playState;
        })()
        "#,
        )
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("idle"));
}

#[tokio::test]
async fn test_animate_finish_resolves() {
    let browser = HeadlessWebBrowser::new();
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
        (function() {
            var div = document.createElement('div');
            var anim = div.animate([], 1000);
            anim.finish();
            return anim.playState;
        })()
        "#,
        )
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("finished"));
}

#[tokio::test]
async fn test_animate_has_finished_promise() {
    let browser = HeadlessWebBrowser::new();
    let result = browser.lock().unwrap().execute_javascript(
        r#"
        (function() {
            var div = document.createElement('div');
            var anim = div.animate([], 100);
            return typeof anim.finished === 'object' && anim.finished instanceof Promise ? 'has_promise' : 'no_promise';
        })()
        "#
    ).await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(
        val.contains("has_promise"),
        "Animation should have finished promise, got: {}",
        val
    );
}

#[tokio::test]
async fn test_animate_finish_event_listener() {
    let browser = HeadlessWebBrowser::new();
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
        (function() {
            var div = document.createElement('div');
            var anim = div.animate([], 100);
            var fired = false;
            anim.addEventListener('finish', function() { fired = true; });
            anim.finish();
            return 'fired=' + fired;
        })()
        "#,
        )
        .await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(
        val.contains("fired=true"),
        "Finish event should fire, got: {}",
        val
    );
}

#[tokio::test]
async fn test_animate_reverse() {
    let browser = HeadlessWebBrowser::new();
    let result = browser
        .lock()
        .unwrap()
        .execute_javascript(
            r#"
        (function() {
            var div = document.createElement('div');
            var anim = div.animate([], 1000);
            var before = anim.playbackRate;
            anim.reverse();
            return 'before=' + before + ',after=' + anim.playbackRate;
        })()
        "#,
        )
        .await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(
        val.contains("before=1") && val.contains("after=-1"),
        "reverse should negate playbackRate, got: {}",
        val
    );
}
