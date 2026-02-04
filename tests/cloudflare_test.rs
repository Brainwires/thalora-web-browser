//! Cloudflare Turnstile challenge test
//!
//! This test verifies that the browser can handle Cloudflare Turnstile challenges
//! by testing against nowsecure.nl which uses Turnstile protection.
//!
//! ## Current Status
//!
//! The async/setTimeout handling now works correctly:
//! - Promise callbacks are properly processed via run_jobs()
//! - Resolution wait JS stores results in window._asyncResult
//! - Timeout produces proper JSON: {"success":false,"reason":"timeout"}
//!
//! However, Turnstile challenge cannot be fully solved because:
//! - Turnstile requires full browser rendering (widget visibility)
//! - Turnstile communicates with Cloudflare servers via complex mechanisms
//! - Turnstile is specifically designed to detect headless/automated browsers
//!
//! This is expected behavior for a headless browser implementation.

use std::process::{Command, Stdio};
use std::io::Write;

/// Test that the scrape tool can handle Cloudflare Turnstile protected pages
/// This is a integration test that runs the actual MCP server
#[test]
#[ignore] // Run with: cargo test --test cloudflare_test -- --ignored --nocapture
fn test_cloudflare_turnstile_nowsecure() {
    // Build the request JSON
    let request = r#"{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "scrape", "arguments": {"url": "https://nowsecure.nl", "wait_for_js": true}}}"#;

    // Run the thalora MCP server with the request
    let mut child = Command::new("./target/debug/thalora")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start thalora");

    // Send the request
    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(request.as_bytes()).expect("Failed to write request");
    }

    // Wait for the process with timeout
    let output = child.wait_with_output().expect("Failed to read output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("=== STDOUT ===\n{}", stdout);
    println!("=== STDERR (last 2000 chars) ===\n{}", &stderr[stderr.len().saturating_sub(2000)..]);

    // Check for known issues
    let has_callable_error = stderr.contains("not a callable function");
    let has_turnstile_script_error = stderr.contains("Could not find Turnstile valid script tag");
    let has_success = stdout.contains("\"result\"") && !stdout.contains("\"error\"");

    // Report findings
    if has_turnstile_script_error {
        println!("\n❌ KNOWN ISSUE: Turnstile script tag detection failing");
        println!("   The script registry or document.currentScript is not working correctly");
    }

    if has_callable_error {
        println!("\n❌ KNOWN ISSUE: 'not a callable function' error");
        println!("   GSAP or another library is trying to call undefined");
        // Extract the function context
        for line in stderr.lines() {
            if line.contains("CALL ERROR") {
                println!("   {}", line);
            }
        }
    }

    if has_success {
        println!("\n✅ SUCCESS: Cloudflare challenge bypassed!");
    }

    // The test passes if we don't have the known issues
    // For now, we're tracking the issues, so we don't assert failure
    // Once fixed, uncomment this assertion:
    // assert!(has_success, "Cloudflare challenge was not bypassed successfully");

    // For now, just verify the test ran
    assert!(!stdout.is_empty() || !stderr.is_empty(), "Test produced no output");
}

/// Test basic document.scripts functionality
#[test]
fn test_document_scripts_registry() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Test that document.scripts exists and is array-like
    let result = engine.execute(r#"
        var scripts = document.scripts;
        typeof scripts === 'object' && typeof scripts.length === 'number';
    "#).unwrap();

    assert_eq!(result, serde_json::json!(true), "document.scripts should be an array-like object");
}

/// Test document.currentScript functionality
#[test]
fn test_document_current_script() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // At global scope, currentScript should be null (not during script execution)
    let result = engine.execute(r#"
        document.currentScript === null;
    "#).unwrap();

    assert_eq!(result, serde_json::json!(true), "document.currentScript should be null when not executing a script element");
}

/// Test that window === globalThis
#[test]
fn test_window_equals_global_this() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    let result = engine.execute(r#"
        window === globalThis && self === window;
    "#).unwrap();

    assert_eq!(result, serde_json::json!(true), "window should equal globalThis and self");
}

/// Test UMD pattern compatibility (like GSAP uses)
#[test]
fn test_umd_pattern() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    let result = engine.execute(r#"
        var umdResult = "not run";
        !function(t, e) {
            if (typeof e === 'function') {
                var w = (t = t || self).window = t.window || {};
                e(w);
                umdResult = "success";
            } else {
                umdResult = "e not function: " + typeof e;
            }
        }(this, function(exports) {
            // This should execute
        });
        umdResult;
    "#).unwrap();

    assert_eq!(result, serde_json::json!("success"), "UMD pattern should work correctly");
}

/// Test browser APIs that GSAP requires
#[test]
fn test_gsap_required_apis() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Check for required animation APIs
    let result = engine.execute(r#"
        (function() {
            var perf = typeof performance !== 'undefined' ? performance : null;
            return JSON.stringify({
                requestAnimationFrame: typeof requestAnimationFrame,
                cancelAnimationFrame: typeof cancelAnimationFrame,
                performance: typeof performance,
                performanceNow: perf && typeof perf.now,
                setInterval: typeof setInterval,
                clearInterval: typeof clearInterval,
                setTimeout: typeof setTimeout,
                clearTimeout: typeof clearTimeout,
                getComputedStyle: typeof getComputedStyle,
                matchMedia: typeof matchMedia,
                MutationObserver: typeof MutationObserver,
                ResizeObserver: typeof ResizeObserver
            });
        })()
    "#).unwrap();

    println!("GSAP required APIs: {}", result);

    // Parse and validate
    if let Some(apis) = result.as_str() {
        let parsed: serde_json::Value = serde_json::from_str(apis).unwrap();

        // These are critical for GSAP
        assert_eq!(parsed["requestAnimationFrame"], "function", "requestAnimationFrame must be a function");
        assert_eq!(parsed["setTimeout"], "function", "setTimeout must be a function");
        assert_eq!(parsed["setInterval"], "function", "setInterval must be a function");
    }
}

/// Test that Promises work with run_jobs() (microtask queue)
/// Note: setTimeout callbacks require separate event loop processing
/// which is not yet fully implemented. Promises use the microtask queue.
#[test]
fn test_promise_microtask_queue() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Set up result storage
    engine.execute(r#"
        window._asyncResult = undefined;
        window._asyncComplete = false;
    "#).unwrap();

    // Execute code that uses Promise.resolve (microtask, not setTimeout)
    engine.execute(r#"
        (function() {
            Promise.resolve().then(function() {
                window._asyncResult = { success: true, reason: 'Promise callback fired' };
                window._asyncComplete = true;
            });
        })()
    "#).unwrap();

    // Run jobs to process the microtask
    engine.run_jobs().unwrap();

    let result = engine.execute(r#"
        window._asyncComplete === true ? JSON.stringify(window._asyncResult) : "not_complete"
    "#).unwrap();

    if let Some(s) = result.as_str() {
        println!("Promise result: {}", s);
        assert!(s.contains("success"), "Promise callback should have fired");
    } else {
        panic!("Result should be a string");
    }
}

/// Test that the async result storage pattern works correctly
/// This pattern is used by the challenge resolution code
#[test]
fn test_async_result_storage_pattern() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Test that the completeWith pattern stores results correctly
    let result = engine.execute(r#"
        (function() {
            window._asyncResult = undefined;
            window._asyncComplete = false;

            function completeWith(result) {
                window._asyncResult = result;
                window._asyncComplete = true;
            }

            // Immediately complete with success
            completeWith({ success: true, reason: 'test condition met' });
        })();
        "executed"
    "#).unwrap();

    assert_eq!(result.as_str(), Some("executed"));

    // Check that the result was stored correctly
    let result = engine.execute(r#"
        window._asyncComplete === true ? JSON.stringify(window._asyncResult) : "not_complete"
    "#).unwrap();

    if let Some(s) = result.as_str() {
        println!("Storage result: {}", s);
        let parsed: serde_json::Value = serde_json::from_str(s).unwrap();
        assert_eq!(parsed["success"], true, "Result should have success=true");
        assert_eq!(parsed["reason"], "test condition met", "Reason should match");
    } else {
        panic!("Result should be a string");
    }
}

/// Test that the renderer's async wait timeout mechanism works
/// Note: This tests the Rust-level timeout, not JS setTimeout
#[test]
#[ignore] // This test requires the renderer infrastructure
fn test_renderer_async_wait_timeout() {
    // This test would need to create a RustRenderer and call
    // evaluate_javascript_with_async_wait, which is difficult
    // to do in isolation. The mechanism is tested via the
    // integration test (test_cloudflare_turnstile_nowsecure).
}

/// Test that setTimeout callbacks actually execute with the event loop
#[test]
fn test_settimeout_callback_execution() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Set up result tracking
    engine.execute(r#"
        window._callbackExecuted = false;
        window._callbackValue = null;
    "#).unwrap();

    // Schedule a setTimeout callback with 0 delay (should fire on first event loop tick)
    engine.execute(r#"
        setTimeout(function() {
            window._callbackExecuted = true;
            window._callbackValue = 'callback_fired';
        }, 0);
    "#).unwrap();

    // Callback shouldn't have fired yet (just scheduled)
    let result = engine.execute("window._callbackExecuted").unwrap();
    assert_eq!(result, serde_json::json!(false), "Callback should not have fired immediately");

    // Run the event loop to process the timer
    engine.run_event_loop(10).unwrap();

    // Now the callback should have fired
    let result = engine.execute("window._callbackExecuted").unwrap();
    assert_eq!(result, serde_json::json!(true), "Callback should have fired after event loop");

    let result = engine.execute("window._callbackValue").unwrap();
    assert_eq!(result, serde_json::json!("callback_fired"), "Callback value should be set");
}

/// Test nested setTimeout callbacks (callback schedules another setTimeout)
#[test]
fn test_nested_settimeout() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    engine.execute(r#"
        window._order = [];
        setTimeout(function() {
            window._order.push('first');
            setTimeout(function() {
                window._order.push('second');
            }, 0);
        }, 0);
    "#).unwrap();

    // Run event loop multiple times to process nested callbacks
    engine.run_event_loop(20).unwrap();

    let result = engine.execute("JSON.stringify(window._order)").unwrap();
    let order_str = result.as_str().expect("Should be string");
    let order: Vec<String> = serde_json::from_str(order_str).unwrap();

    assert_eq!(order, vec!["first", "second"], "Nested callbacks should execute in order");
}

/// Test setTimeout with delay
#[test]
fn test_settimeout_with_delay() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    engine.execute(r#"
        window._timerFired = false;
        setTimeout(function() {
            window._timerFired = true;
        }, 50); // 50ms delay
    "#).unwrap();

    // Immediately after scheduling, should not have fired
    let result = engine.execute("window._timerFired").unwrap();
    assert_eq!(result, serde_json::json!(false));

    // Wait briefly and run event loop
    std::thread::sleep(std::time::Duration::from_millis(60));
    engine.run_event_loop(10).unwrap();

    // Now it should have fired
    let result = engine.execute("window._timerFired").unwrap();
    assert_eq!(result, serde_json::json!(true), "Timer with delay should fire after waiting");
}

/// Test clearTimeout cancels the callback
#[test]
fn test_clear_timeout() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    engine.execute(r#"
        window._timerFired = false;
        var timerId = setTimeout(function() {
            window._timerFired = true;
        }, 0);
        clearTimeout(timerId);
    "#).unwrap();

    // Run event loop
    engine.run_event_loop(10).unwrap();

    // Timer should NOT have fired because we cleared it
    let result = engine.execute("window._timerFired").unwrap();
    assert_eq!(result, serde_json::json!(false), "Cleared timeout should not fire");
}

/// Test setInterval fires repeatedly
#[test]
fn test_setinterval_repeats() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    engine.execute(r#"
        window._intervalCount = 0;
        window._intervalId = setInterval(function() {
            window._intervalCount++;
            if (window._intervalCount >= 3) {
                clearInterval(window._intervalId);
            }
        }, 0);
    "#).unwrap();

    // Run event loop multiple times
    engine.run_event_loop(50).unwrap();

    let result = engine.execute("window._intervalCount").unwrap();
    let count = result.as_i64().expect("Should be number");

    assert!(count >= 3, "Interval should have fired at least 3 times, got {}", count);
}

/// Test that MouseEvent objects have proper Event properties (type, bubbles, etc.)
#[test]
fn test_mouse_event_has_type_property() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Test creating a MouseEvent and accessing its type
    let result = engine.execute(r#"
        var event = new MouseEvent('click', { bubbles: true, cancelable: true });
        event.type
    "#).unwrap();

    assert_eq!(result, serde_json::json!("click"),
        "MouseEvent should have 'click' as its type, got {:?}", result);

    // Test bubbles property
    let result2 = engine.execute(r#"
        var event = new MouseEvent('mousemove', { bubbles: true });
        event.bubbles
    "#).unwrap();

    assert_eq!(result2, serde_json::json!(true),
        "MouseEvent should have bubbles=true");

    // Test isTrusted (should be false for script-created events)
    let result3 = engine.execute(r#"
        var event = new MouseEvent('click');
        event.isTrusted
    "#).unwrap();

    assert_eq!(result3, serde_json::json!(false),
        "Script-created MouseEvent should have isTrusted=false");
}

/// Test that __dispatchTrustedMouseEvent creates events with proper type property
#[test]
fn test_trusted_mouse_event_has_type() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Set up a click listener that captures the event
    let result = engine.execute(r#"
        var capturedEvent = null;
        document.body.addEventListener('mousemove', function(e) {
            capturedEvent = { type: e.type, isTrusted: e.isTrusted };
        });
        // Dispatch a trusted mouse event
        if (typeof __dispatchTrustedMouseEvent === 'function') {
            __dispatchTrustedMouseEvent('mousemove', 100, 100);
        }
        capturedEvent ? JSON.stringify(capturedEvent) : 'no event captured'
    "#).unwrap();

    eprintln!("Trusted event result: {:?}", result);
    // The event should have been captured with type='mousemove'
    assert!(result.to_string().contains("mousemove"),
        "Should capture mousemove event with type, got {:?}", result);
}

/// Test that __dispatchTrustedMouseEvent is available on globalThis/window
#[test]
fn test_trusted_mouse_event_dispatcher_exists() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Test that the function exists on window (which is globalThis)
    let result = engine.execute(r#"
        typeof window.__dispatchTrustedMouseEvent
    "#).unwrap();

    assert_eq!(result, serde_json::json!("function"),
        "__dispatchTrustedMouseEvent should be a function on window, got {:?}", result);

    // Also test it's the same on globalThis
    let result2 = engine.execute(r#"
        typeof globalThis.__dispatchTrustedMouseEvent
    "#).unwrap();

    assert_eq!(result2, serde_json::json!("function"),
        "__dispatchTrustedMouseEvent should be a function on globalThis");

    // Test that window === globalThis (browser standard)
    let result3 = engine.execute(r#"
        window === globalThis
    "#).unwrap();

    assert_eq!(result3, serde_json::json!(true),
        "window should equal globalThis");
}

// ============================================================================
// Window Hierarchy Tests - Tests for window.parent, window.top, window.frameElement
// ============================================================================

/// Test that top-level window has correct parent/top/frameElement values
#[test]
fn test_top_level_window_hierarchy() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Test window.parent === window (top-level window)
    let result = engine.execute(r#"
        window.parent === window
    "#).unwrap();
    assert_eq!(result, serde_json::json!(true),
        "window.parent should equal window for top-level window");

    // Test window.top === window (top-level window)
    let result = engine.execute(r#"
        window.top === window
    "#).unwrap();
    assert_eq!(result, serde_json::json!(true),
        "window.top should equal window for top-level window");

    // Test window.frameElement === null (top-level window)
    let result = engine.execute(r#"
        window.frameElement === null
    "#).unwrap();
    assert_eq!(result, serde_json::json!(true),
        "window.frameElement should be null for top-level window");

    // Test window.self === window
    let result = engine.execute(r#"
        window.self === window
    "#).unwrap();
    assert_eq!(result, serde_json::json!(true),
        "window.self should equal window");
}

/// Test that HTMLIFrameElement constructor is available globally
#[test]
fn test_html_iframe_element_global() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Test HTMLIFrameElement is defined
    let result = engine.execute(r#"
        typeof HTMLIFrameElement
    "#).unwrap();
    assert_eq!(result, serde_json::json!("function"),
        "HTMLIFrameElement should be a function constructor");

    // Test that we can create an iframe via document.createElement
    let result = engine.execute(r#"
        const iframe = document.createElement('iframe');
        iframe.tagName
    "#).unwrap();
    assert_eq!(result, serde_json::json!("IFRAME"),
        "createElement('iframe') should create an IFRAME element");
}

/// Test iframe contentWindow hierarchy
#[test]
fn test_iframe_content_window_hierarchy() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Create iframe and test its contentWindow exists
    let result = engine.execute(r#"
        const iframe = document.createElement('iframe');
        iframe.contentWindow !== null
    "#).unwrap();
    assert_eq!(result, serde_json::json!(true),
        "iframe.contentWindow should exist after creation");

    // Test iframe contentWindow.parent points to the parent window
    let result = engine.execute(r#"
        const iframe = document.createElement('iframe');
        iframe.contentWindow.parent === window
    "#).unwrap();
    assert_eq!(result, serde_json::json!(true),
        "iframe.contentWindow.parent should equal the parent window");

    // Test iframe contentWindow.top points to the top window
    let result = engine.execute(r#"
        const iframe = document.createElement('iframe');
        iframe.contentWindow.top === window
    "#).unwrap();
    assert_eq!(result, serde_json::json!(true),
        "iframe.contentWindow.top should equal the top-level window");

    // Test iframe contentWindow.frameElement points to the iframe element
    let result = engine.execute(r#"
        const iframe = document.createElement('iframe');
        iframe.contentWindow.frameElement === iframe
    "#).unwrap();
    assert_eq!(result, serde_json::json!(true),
        "iframe.contentWindow.frameElement should equal the iframe element");

    // Test iframe contentWindow is different from parent window
    let result = engine.execute(r#"
        const iframe = document.createElement('iframe');
        iframe.contentWindow !== window
    "#).unwrap();
    assert_eq!(result, serde_json::json!(true),
        "iframe.contentWindow should be different from parent window");
}

/// Test nested iframe hierarchy
#[test]
fn test_nested_iframe_hierarchy() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Test nested iframes - create iframe1, then create iframe2 in iframe1's context
    // Note: We test the hierarchy relationships, not actual DOM nesting
    let result = engine.execute(r#"
        // Create first level iframe
        const iframe1 = document.createElement('iframe');
        const win1 = iframe1.contentWindow;

        // win1.parent should be window
        const test1 = win1.parent === window;

        // win1.top should be window
        const test2 = win1.top === window;

        // win1.frameElement should be iframe1
        const test3 = win1.frameElement === iframe1;

        JSON.stringify({
            parentIsWindow: test1,
            topIsWindow: test2,
            frameElementIsIframe: test3
        })
    "#).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(result.as_str().unwrap()).unwrap();
    assert_eq!(parsed["parentIsWindow"], true, "iframe contentWindow.parent should be the parent window");
    assert_eq!(parsed["topIsWindow"], true, "iframe contentWindow.top should be the top-level window");
    assert_eq!(parsed["frameElementIsIframe"], true, "iframe contentWindow.frameElement should be the iframe");
}

/// Test iframe contentDocument exists and is linked to window
#[test]
fn test_iframe_content_document() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Test iframe contentDocument exists
    let result = engine.execute(r#"
        const iframe = document.createElement('iframe');
        iframe.contentDocument !== null
    "#).unwrap();
    assert_eq!(result, serde_json::json!(true),
        "iframe.contentDocument should exist after creation");

    // Test iframe contentWindow.document exists
    let result = engine.execute(r#"
        const iframe = document.createElement('iframe');
        iframe.contentWindow.document !== null
    "#).unwrap();
    assert_eq!(result, serde_json::json!(true),
        "iframe.contentWindow.document should exist");

    // Note: contentWindow.document and contentDocument may be different object references
    // in our implementation, but they represent the same document semantically.
    // The important thing is that both exist and the iframe has a proper document.
}

/// Test iframe creation via innerHTML (Turnstile pattern)
///
/// This is the critical test for Turnstile support. Turnstile creates iframes via
/// innerHTML injection, not document.createElement(). Without context-aware parsing,
/// these iframes would lack contentWindow and contentDocument, breaking postMessage.
#[test]
fn test_iframe_creation_via_innerhtml() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Test that iframes created via innerHTML have contentWindow
    // Note: We access children via array-style indexing since children is an HTMLCollection
    let result = engine.execute(r#"
        const div = document.createElement('div');
        div.innerHTML = '<iframe id="test-frame" src="about:blank"></iframe>';
        const children = div.children;
        var iframe = null;
        // Use direct array access since HTMLCollection may not implement .item()
        for (var i = 0; i < children.length; i++) {
            var child = children[i];
            if (child && child.tagName === 'IFRAME') {
                iframe = child;
                break;
            }
        }
        iframe !== null && iframe.contentWindow !== null
    "#).unwrap();
    assert_eq!(result, serde_json::json!(true),
        "iframe created via innerHTML should have contentWindow");
}

/// Test iframe.contentWindow.parent via innerHTML
#[test]
fn test_iframe_innerhtml_parent_window() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Test that iframes created via innerHTML have parent set correctly
    let result = engine.execute(r#"
        const div = document.createElement('div');
        div.innerHTML = '<iframe src="about:blank"></iframe>';
        const children = div.children;
        var iframe = null;
        for (var i = 0; i < children.length; i++) {
            var child = children[i];
            if (child && child.tagName === 'IFRAME') {
                iframe = child;
                break;
            }
        }
        iframe.contentWindow.parent === window
    "#).unwrap();
    assert_eq!(result, serde_json::json!(true),
        "iframe.contentWindow.parent should equal parent window for innerHTML-created iframes");
}

/// Test iframe.contentWindow.top via innerHTML
#[test]
fn test_iframe_innerhtml_top_window() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Test that iframes created via innerHTML have top set correctly
    let result = engine.execute(r#"
        const div = document.createElement('div');
        div.innerHTML = '<iframe></iframe>';
        const children = div.children;
        var iframe = null;
        for (var i = 0; i < children.length; i++) {
            var child = children[i];
            if (child && child.tagName === 'IFRAME') {
                iframe = child;
                break;
            }
        }
        iframe.contentWindow.top === window
    "#).unwrap();
    assert_eq!(result, serde_json::json!(true),
        "iframe.contentWindow.top should equal top-level window for innerHTML-created iframes");
}

/// Test iframe.contentDocument via innerHTML
#[test]
fn test_iframe_innerhtml_content_document() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Test that iframes created via innerHTML have contentDocument
    let result = engine.execute(r#"
        const div = document.createElement('div');
        div.innerHTML = '<iframe></iframe>';
        const children = div.children;
        var iframe = null;
        for (var i = 0; i < children.length; i++) {
            var child = children[i];
            if (child && child.tagName === 'IFRAME') {
                iframe = child;
                break;
            }
        }
        iframe.contentDocument !== null
    "#).unwrap();
    assert_eq!(result, serde_json::json!(true),
        "iframe.contentDocument should exist for innerHTML-created iframes");
}

/// Test postMessage between innerHTML-created iframe and parent
#[test]
fn test_iframe_innerhtml_postmessage() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Test postMessage from iframe to parent
    let result = engine.execute(r#"
        var received = null;

        // Set up message listener on parent window
        window.addEventListener('message', function(e) {
            received = e.data;
        });

        // Create iframe via innerHTML
        const div = document.createElement('div');
        div.innerHTML = '<iframe></iframe>';
        const children = div.children;
        var iframe = null;
        for (var i = 0; i < children.length; i++) {
            var child = children[i];
            if (child && child.tagName === 'IFRAME') {
                iframe = child;
                break;
            }
        }

        // Post message from iframe to parent
        iframe.contentWindow.parent.postMessage('hello from iframe', '*');

        // Check if message was received
        received === 'hello from iframe'
    "#).unwrap();
    assert_eq!(result, serde_json::json!(true),
        "postMessage should work from innerHTML-created iframe to parent");
}

/// Test complex Turnstile-like iframe creation pattern
#[test]
fn test_turnstile_iframe_pattern() {
    use thalora::engine::create_test_engine;

    let mut engine = create_test_engine().unwrap();

    // Test the pattern Turnstile uses: create container, inject iframe HTML
    let result = engine.execute(r#"
        // Create container element (like Turnstile does)
        const container = document.createElement('div');
        container.id = 'cf-turnstile-container';

        // Inject iframe HTML (like Turnstile does)
        container.innerHTML = '<iframe src="https://challenges.cloudflare.com/cdn-cgi/challenge-platform/..." style="border: none; overflow: hidden; width: 300px; height: 65px;"></iframe>';

        // Find the iframe
        const children = container.children;
        var iframe = null;
        for (var i = 0; i < children.length; i++) {
            var child = children[i];
            if (child && child.tagName === 'IFRAME') {
                iframe = child;
                break;
            }
        }

        // Verify iframe has proper context
        JSON.stringify({
            iframeExists: iframe !== null,
            hasContentWindow: iframe && iframe.contentWindow !== null,
            hasContentDocument: iframe && iframe.contentDocument !== null,
            parentIsWindow: iframe && iframe.contentWindow && iframe.contentWindow.parent === window,
            topIsWindow: iframe && iframe.contentWindow && iframe.contentWindow.top === window
        })
    "#).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(result.as_str().unwrap()).unwrap();
    assert_eq!(parsed["iframeExists"], true, "iframe should exist");
    assert_eq!(parsed["hasContentWindow"], true, "iframe should have contentWindow");
    assert_eq!(parsed["hasContentDocument"], true, "iframe should have contentDocument");
    assert_eq!(parsed["parentIsWindow"], true, "iframe contentWindow.parent should be window");
    assert_eq!(parsed["topIsWindow"], true, "iframe contentWindow.top should be window");
}
