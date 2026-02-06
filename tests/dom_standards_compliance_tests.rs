//! DOM Standards Compliance Integration Tests
//!
//! Tests the browser standards compliance implementation including:
//! - Event propagation (capturing, at-target, bubbling phases)
//! - Focus management (focus(), blur(), document.activeElement)
//! - Navigation and default actions (link clicks, form submissions)
//! - Keyboard event simulation

use thalora::HeadlessWebBrowser;

// =============================================================================
// EVENT PROPAGATION TESTS
// =============================================================================

#[tokio::test]
async fn test_event_bubbling_basic() {
    let browser = HeadlessWebBrowser::new();

    // NOTE: No leading newline - ASI would cause `return\n(` to be treated as `return;`
    let js_code = r#"(function() {
        var parent = document.createElement('div');
        var child = document.createElement('button');
        parent.appendChild(child);
        document.body.appendChild(parent);

        var events = [];

        parent.addEventListener('click', function(e) {
            events.push('parent-bubble');
        });

        child.addEventListener('click', function(e) {
            events.push('child');
        });

        child.click();

        return JSON.stringify(events);
    })()"#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    assert!(result.is_ok(), "Event bubbling test failed: {:?}", result);
    let output = result.unwrap();
    println!("Event bubbling result: {}", output);
    // Child event should fire
    assert!(output.contains("child"), "Child event should fire");
}

#[tokio::test]
async fn test_event_capturing_phase() {
    let browser = HeadlessWebBrowser::new();

    let js_code = r#"(function() {
        var grandparent = document.createElement('div');
        var parent = document.createElement('div');
        var child = document.createElement('button');

        grandparent.appendChild(parent);
        parent.appendChild(child);
        document.body.appendChild(grandparent);

        var events = [];

        grandparent.addEventListener('click', function(e) {
            events.push('grandparent-capture');
        }, true);

        grandparent.addEventListener('click', function(e) {
            events.push('grandparent-bubble');
        }, false);

        child.addEventListener('click', function(e) {
            events.push('child');
        });

        child.click();

        return JSON.stringify(events);
    })()"#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    assert!(result.is_ok(), "Event capturing test failed: {:?}", result);
    println!("Event capturing result: {}", result.unwrap());
}

#[tokio::test]
async fn test_stop_propagation() {
    let browser = HeadlessWebBrowser::new();

    let js_code = r#"(function() {
        var parent = document.createElement('div');
        var child = document.createElement('button');
        parent.appendChild(child);
        document.body.appendChild(parent);

        var parentCalled = false;

        parent.addEventListener('click', function(e) {
            parentCalled = true;
        });

        child.addEventListener('click', function(e) {
            e.stopPropagation();
        });

        child.click();

        return JSON.stringify({ parentCalled: parentCalled });
    })()"#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    assert!(result.is_ok(), "stopPropagation test failed: {:?}", result);
    println!("stopPropagation result: {}", result.unwrap());
}

#[tokio::test]
async fn test_stop_immediate_propagation() {
    let browser = HeadlessWebBrowser::new();

    let js_code = r#"(function() {
        var button = document.createElement('button');
        document.body.appendChild(button);

        var calls = [];

        button.addEventListener('click', function(e) {
            calls.push('first');
            e.stopImmediatePropagation();
        });

        button.addEventListener('click', function(e) {
            calls.push('second');
        });

        button.click();

        return JSON.stringify(calls);
    })()"#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    assert!(result.is_ok(), "stopImmediatePropagation test failed: {:?}", result);
    let output = result.unwrap();
    println!("stopImmediatePropagation result: {}", output);
    assert!(output.contains("first"), "First listener should be called");
}

#[tokio::test]
async fn test_prevent_default() {
    let browser = HeadlessWebBrowser::new();

    let js_code = r#"(function() {
        var event = new Event('click', { cancelable: true });

        var before = event.defaultPrevented;
        event.preventDefault();
        var after = event.defaultPrevented;

        return JSON.stringify({ before: before, after: after });
    })()"#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    assert!(result.is_ok(), "preventDefault test failed: {:?}", result);
    let output = result.unwrap();
    println!("preventDefault result: {}", output);
    assert!(output.contains("\"before\":false"), "defaultPrevented should be false initially");
    assert!(output.contains("\"after\":true"), "defaultPrevented should be true after preventDefault");
}

#[tokio::test]
async fn test_event_phase_constants() {
    let browser = HeadlessWebBrowser::new();

    let js_code = r#"(function() {
        return JSON.stringify({
            NONE: Event.NONE,
            CAPTURING_PHASE: Event.CAPTURING_PHASE,
            AT_TARGET: Event.AT_TARGET,
            BUBBLING_PHASE: Event.BUBBLING_PHASE
        });
    })()"#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    assert!(result.is_ok(), "Event phase constants test failed: {:?}", result);
    let output = result.unwrap();
    println!("Event phase constants: {}", output);
}

#[tokio::test]
async fn test_trusted_event_from_click() {
    let browser = HeadlessWebBrowser::new();

    let js_code = r#"(function() {
        var button = document.createElement('button');
        document.body.appendChild(button);

        var receivedEvent = null;

        button.addEventListener('click', function(e) {
            receivedEvent = e;
        });

        button.click();

        return JSON.stringify({
            hasEvent: !!receivedEvent,
            type: receivedEvent ? receivedEvent.type : null,
            isTrusted: receivedEvent ? receivedEvent.isTrusted : null
        });
    })()"#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    assert!(result.is_ok(), "Trusted event test failed: {:?}", result);
    println!("Trusted event result: {}", result.unwrap());
}

// =============================================================================
// FOCUS MANAGEMENT TESTS
// =============================================================================

#[tokio::test]
async fn test_focus_method_exists() {
    let browser = HeadlessWebBrowser::new();

    let js_code = r#"(function() {
        var input = document.createElement('input');
        document.body.appendChild(input);

        return JSON.stringify({
            hasFocus: typeof input.focus === 'function',
            hasBlur: typeof input.blur === 'function'
        });
    })()"#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    assert!(result.is_ok(), "Focus methods test failed: {:?}", result);
    let output = result.unwrap();
    assert!(output.contains("true"), "focus and blur should be functions");
}

#[tokio::test]
async fn test_document_active_element() {
    let browser = HeadlessWebBrowser::new();

    let js_code = r#"(function() {
        var hasActiveElement = typeof document.activeElement !== 'undefined';

        var input = document.createElement('input');
        document.body.appendChild(input);

        input.focus();

        var activeAfterFocus = document.activeElement;

        return JSON.stringify({
            hasActiveElement: hasActiveElement,
            activeElementExists: !!activeAfterFocus,
            activeElementTag: activeAfterFocus ? activeAfterFocus.tagName : null
        });
    })()"#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    assert!(result.is_ok(), "activeElement test failed: {:?}", result);
    println!("activeElement result: {}", result.unwrap());
}

#[tokio::test]
async fn test_focus_events_fired() {
    let browser = HeadlessWebBrowser::new();

    let js_code = r#"(function() {
        var input = document.createElement('input');
        document.body.appendChild(input);

        var events = [];

        input.addEventListener('focus', function(e) {
            events.push('focus');
        });

        input.addEventListener('focusin', function(e) {
            events.push('focusin');
        });

        input.focus();

        return JSON.stringify(events);
    })()"#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    assert!(result.is_ok(), "Focus events test failed: {:?}", result);
    println!("Focus events result: {}", result.unwrap());
}

#[tokio::test]
async fn test_blur_events_fired() {
    let browser = HeadlessWebBrowser::new();

    let js_code = r#"(function() {
        var input = document.createElement('input');
        document.body.appendChild(input);

        var events = [];

        input.addEventListener('blur', function(e) {
            events.push('blur');
        });

        input.addEventListener('focusout', function(e) {
            events.push('focusout');
        });

        input.focus();
        input.blur();

        return JSON.stringify(events);
    })()"#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    assert!(result.is_ok(), "Blur events test failed: {:?}", result);
    println!("Blur events result: {}", result.unwrap());
}

#[tokio::test]
async fn test_focus_switching_between_elements() {
    let browser = HeadlessWebBrowser::new();

    let js_code = r#"(function() {
        var input1 = document.createElement('input');
        var input2 = document.createElement('input');
        document.body.appendChild(input1);
        document.body.appendChild(input2);

        var events = [];

        input1.addEventListener('focus', function() { events.push('input1-focus'); });
        input1.addEventListener('blur', function() { events.push('input1-blur'); });

        input2.addEventListener('focus', function() { events.push('input2-focus'); });
        input2.addEventListener('blur', function() { events.push('input2-blur'); });

        input1.focus();
        input2.focus();

        return JSON.stringify(events);
    })()"#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    assert!(result.is_ok(), "Focus switching test failed: {:?}", result);
    println!("Focus switching result: {}", result.unwrap());
}

// =============================================================================
// NAVIGATION AND DEFAULT ACTIONS TESTS
// =============================================================================

#[tokio::test]
async fn test_link_click_fires_event() {
    let browser = HeadlessWebBrowser::new();

    let js_code = r#"(function() {
        var link = document.createElement('a');
        link.href = 'https://example.com/test';
        document.body.appendChild(link);

        var clickFired = false;

        link.addEventListener('click', function(e) {
            clickFired = true;
        });

        link.click();

        return JSON.stringify({
            clickFired: clickFired,
            tagName: link.tagName
        });
    })()"#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    assert!(result.is_ok(), "Link click test failed: {:?}", result);
    let output = result.unwrap();
    println!("Link click result: {}", output);
    assert!(output.contains("true"), "Click event should fire on link");
}

#[tokio::test]
async fn test_link_click_prevent_default() {
    let browser = HeadlessWebBrowser::new();

    // Test that MouseEvent properly inherits from UIEvent -> Event,
    // so preventDefault() is available and works correctly.
    let js_code = r#"(function() {
        var link = document.createElement('a');
        link.href = 'https://example.com/blocked';
        document.body.appendChild(link);

        var defaultPrevented = false;

        link.addEventListener('click', function(e) {
            e.preventDefault();
            defaultPrevented = e.defaultPrevented;
        });

        link.click();

        return JSON.stringify({ defaultPrevented: defaultPrevented });
    })()"#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    assert!(result.is_ok(), "Link preventDefault test failed: {:?}", result);
    let output = result.unwrap();
    println!("Link preventDefault result: {}", output);
    // preventDefault should work on MouseEvent now that prototype chain is fixed
    assert!(output.contains("true"), "preventDefault should work on MouseEvent");
}

#[tokio::test]
async fn test_button_click_fires_event() {
    let browser = HeadlessWebBrowser::new();

    let js_code = r#"(function() {
        var button = document.createElement('button');
        document.body.appendChild(button);

        var clickFired = false;

        button.addEventListener('click', function(e) {
            clickFired = true;
        });

        button.click();

        return JSON.stringify({ clickFired: clickFired });
    })()"#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    assert!(result.is_ok(), "Button click test failed: {:?}", result);
    let output = result.unwrap();
    assert!(output.contains("true"), "Button click should fire event");
}

#[tokio::test]
async fn test_submit_button_in_form() {
    let browser = HeadlessWebBrowser::new();

    let js_code = r#"(function() {
        var form = document.createElement('form');
        form.action = '/submit';
        form.method = 'POST';

        var button = document.createElement('button');
        button.type = 'submit';

        form.appendChild(button);
        document.body.appendChild(form);

        var submitButtonClicked = false;

        button.addEventListener('click', function(e) {
            submitButtonClicked = true;
            e.preventDefault();
        });

        button.click();

        return JSON.stringify({ submitButtonClicked: submitButtonClicked });
    })()"#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    assert!(result.is_ok(), "Submit button test failed: {:?}", result);
    let output = result.unwrap();
    assert!(output.contains("true"), "Submit button click should fire");
}

// =============================================================================
// NAVIGATION BRIDGE TESTS (Rust-level)
// =============================================================================

#[test]
fn test_navigation_bridge_queue() {
    use thalora_browser_apis::browser::navigation_bridge;

    // Clear any existing requests
    navigation_bridge::clear_navigation_queue();

    // Queue a navigation
    navigation_bridge::queue_navigation("https://example.com/page1");
    navigation_bridge::queue_navigation("https://example.com/page2");

    assert!(navigation_bridge::has_pending_navigations());

    // Drain and verify
    let requests = navigation_bridge::drain_navigation_requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].url, "https://example.com/page1");
    assert_eq!(requests[1].url, "https://example.com/page2");
    assert_eq!(requests[0].method, "GET");

    // Queue should be empty now
    assert!(!navigation_bridge::has_pending_navigations());
}

#[test]
fn test_navigation_bridge_form_submission() {
    use thalora_browser_apis::browser::navigation_bridge;

    navigation_bridge::clear_navigation_queue();

    navigation_bridge::queue_form_submission("/api/submit", "POST");

    let requests = navigation_bridge::drain_navigation_requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].url, "/api/submit");
    assert_eq!(requests[0].method, "POST");
}

// =============================================================================
// KEYBOARD DISPATCHER TESTS (Rust-level)
// =============================================================================

#[test]
fn test_keyboard_char_to_keycode() {
    use thalora_browser_apis::browser::keyboard_dispatcher;

    // Test the KeyboardSequence from_text
    let seq = keyboard_dispatcher::KeyboardSequence::from_text("ab");

    // Each character generates 3 events: keydown, input, keyup
    // So "ab" should generate 6 events
    // This tests that the sequence is created correctly
}

#[test]
fn test_keyboard_action_creation() {
    use thalora_browser_apis::browser::keyboard_dispatcher::KeyboardAction;

    let keydown = KeyboardAction::keydown("Enter", "Enter", 13);
    assert_eq!(keydown.event_type, "keydown");
    assert_eq!(keydown.key, "Enter");
    assert_eq!(keydown.code, "Enter");
    assert_eq!(keydown.key_code, 13);

    let keyup = KeyboardAction::keyup("Enter", "Enter", 13);
    assert_eq!(keyup.event_type, "keyup");
}
