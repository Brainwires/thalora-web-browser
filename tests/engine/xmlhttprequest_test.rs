//! Unit tests for XMLHttpRequest implementation in Boa engine

use boa_engine::{Context, Source};
use thalora_web_browser::engine::renderer::RustRenderer;

#[test]
fn test_xmlhttprequest_constructor() {
    let mut renderer = RustRenderer::new();

    // Test that XMLHttpRequest constructor exists and works with 'new'
    let result = renderer.js_context.eval(Source::from_bytes("typeof XMLHttpRequest"));
    assert!(result.is_ok());
    let type_result = renderer.js_value_to_string(result.unwrap());
    assert_eq!(type_result, "function");

    // Test constructor with 'new' operator
    let result = renderer.js_context.eval(Source::from_bytes("var xhr = new XMLHttpRequest(); xhr"));
    assert!(result.is_ok());

    // Test that constructor fails without 'new' (should work in modern JS but our impl requires 'new')
    let result = renderer.js_context.eval(Source::from_bytes("XMLHttpRequest()"));
    assert!(result.is_err());
}

#[test]
fn test_xmlhttprequest_initial_state() {
    let mut renderer = RustRenderer::new();

    // Create XMLHttpRequest and test initial state
    let result = renderer.js_context.eval(Source::from_bytes(
        "var xhr = new XMLHttpRequest(); xhr.readyState"
    ));
    assert!(result.is_ok());
    let ready_state = renderer.js_value_to_string(result.unwrap());
    assert_eq!(ready_state, "0"); // UNSENT

    // Test initial status
    let result = renderer.js_context.eval(Source::from_bytes("xhr.status"));
    assert!(result.is_ok());
    let status = renderer.js_value_to_string(result.unwrap());
    assert_eq!(status, "0");

    // Test initial statusText
    let result = renderer.js_context.eval(Source::from_bytes("xhr.statusText"));
    assert!(result.is_ok());
    let status_text = renderer.js_value_to_string(result.unwrap());
    assert_eq!(status_text, "");

    // Test initial responseText
    let result = renderer.js_context.eval(Source::from_bytes("xhr.responseText"));
    assert!(result.is_ok());
    let response_text = renderer.js_value_to_string(result.unwrap());
    assert_eq!(response_text, "");
}

#[test]
fn test_xmlhttprequest_open_method() {
    let mut renderer = RustRenderer::new();

    // Test open method with valid parameters
    let result = renderer.js_context.eval(Source::from_bytes(
        r#"
        var xhr = new XMLHttpRequest();
        xhr.open("GET", "https://example.com");
        xhr.readyState
        "#
    ));
    assert!(result.is_ok());
    let ready_state = renderer.js_value_to_string(result.unwrap());
    assert_eq!(ready_state, "1"); // OPENED

    // Test open method with invalid URL
    let result = renderer.js_context.eval(Source::from_bytes(
        r#"
        var xhr2 = new XMLHttpRequest();
        try {
            xhr2.open("GET", "invalid-url");
            false;
        } catch(e) {
            true;
        }
        "#
    ));
    assert!(result.is_ok());
    let caught_error = renderer.js_value_to_string(result.unwrap());
    assert_eq!(caught_error, "true");

    // Test open method with invalid HTTP method
    let result = renderer.js_context.eval(Source::from_bytes(
        r#"
        var xhr3 = new XMLHttpRequest();
        try {
            xhr3.open("INVALID", "https://example.com");
            false;
        } catch(e) {
            true;
        }
        "#
    ));
    assert!(result.is_ok());
    let caught_error = renderer.js_value_to_string(result.unwrap());
    assert_eq!(caught_error, "true");
}

#[test]
fn test_xmlhttprequest_methods_exist() {
    let mut renderer = RustRenderer::new();

    // Test that all required methods exist
    let methods = [
        "open",
        "send",
        "setRequestHeader",
        "getResponseHeader",
        "getAllResponseHeaders",
        "abort"
    ];

    for method in &methods {
        let script = format!(
            "var xhr = new XMLHttpRequest(); typeof xhr.{}",
            method
        );
        let result = renderer.js_context.eval(Source::from_bytes(&script));
        assert!(result.is_ok());
        let method_type = renderer.js_value_to_string(result.unwrap());
        assert_eq!(method_type, "function", "Method {} should be a function", method);
    }
}

#[test]
fn test_xmlhttprequest_constants() {
    let mut renderer = RustRenderer::new();

    // Test that XMLHttpRequest constants exist and have correct values
    let constants = [
        ("UNSENT", "0"),
        ("OPENED", "1"),
        ("HEADERS_RECEIVED", "2"),
        ("LOADING", "3"),
        ("DONE", "4")
    ];

    for (constant, expected_value) in &constants {
        let script = format!("XMLHttpRequest.{}", constant);
        let result = renderer.js_context.eval(Source::from_bytes(&script));
        assert!(result.is_ok());
        let value = renderer.js_value_to_string(result.unwrap());
        assert_eq!(value, *expected_value, "Constant {} should equal {}", constant, expected_value);
    }
}

#[test]
fn test_xmlhttprequest_set_request_header() {
    let mut renderer = RustRenderer::new();

    // Test setRequestHeader after open
    let result = renderer.js_context.eval(Source::from_bytes(
        r#"
        var xhr = new XMLHttpRequest();
        xhr.open("POST", "https://example.com");
        xhr.setRequestHeader("Content-Type", "application/json");
        xhr.readyState
        "#
    ));
    assert!(result.is_ok());
    let ready_state = renderer.js_value_to_string(result.unwrap());
    assert_eq!(ready_state, "1"); // Should still be OPENED

    // Test setRequestHeader before open (should fail)
    let result = renderer.js_context.eval(Source::from_bytes(
        r#"
        var xhr2 = new XMLHttpRequest();
        try {
            xhr2.setRequestHeader("Content-Type", "application/json");
            false;
        } catch(e) {
            true;
        }
        "#
    ));
    assert!(result.is_ok());
    let caught_error = renderer.js_value_to_string(result.unwrap());
    assert_eq!(caught_error, "true");
}

#[test]
fn test_xmlhttprequest_abort() {
    let mut renderer = RustRenderer::new();

    // Test abort method
    let result = renderer.js_context.eval(Source::from_bytes(
        r#"
        var xhr = new XMLHttpRequest();
        xhr.open("GET", "https://example.com");
        xhr.abort();
        xhr.readyState
        "#
    ));
    assert!(result.is_ok());
    let ready_state = renderer.js_value_to_string(result.unwrap());
    assert_eq!(ready_state, "4"); // DONE after abort

    // Test status after abort
    let result = renderer.js_context.eval(Source::from_bytes("xhr.status"));
    assert!(result.is_ok());
    let status = renderer.js_value_to_string(result.unwrap());
    assert_eq!(status, "0"); // Status should be 0 after abort
}

#[test]
fn test_xmlhttprequest_properties_exist() {
    let mut renderer = RustRenderer::new();

    // Test that all required properties exist
    let properties = [
        "readyState",
        "status",
        "statusText",
        "responseText",
        "responseXML",
        "timeout",
        "withCredentials",
        "upload",
        "onreadystatechange",
        "onload",
        "onerror",
        "onabort",
        "ontimeout",
        "onloadstart",
        "onloadend",
        "onprogress"
    ];

    for property in &properties {
        let script = format!(
            "var xhr = new XMLHttpRequest(); '{}' in xhr",
            property
        );
        let result = renderer.js_context.eval(Source::from_bytes(&script));
        assert!(result.is_ok());
        let has_property = renderer.js_value_to_string(result.unwrap());
        assert_eq!(has_property, "true", "Property {} should exist", property);
    }
}

#[test]
fn test_xmlhttprequest_event_handlers() {
    let mut renderer = RustRenderer::new();

    // Test that event handlers can be set
    let result = renderer.js_context.eval(Source::from_bytes(
        r#"
        var xhr = new XMLHttpRequest();
        var called = false;
        xhr.onreadystatechange = function() { called = true; };
        xhr.open("GET", "https://example.com");
        called
        "#
    ));
    assert!(result.is_ok());
    let was_called = renderer.js_value_to_string(result.unwrap());
    assert_eq!(was_called, "true"); // Event handler should have been called during open()
}

#[test]
fn test_xmlhttprequest_get_response_header() {
    let mut renderer = RustRenderer::new();

    // Test getResponseHeader before any response (should return null)
    let result = renderer.js_context.eval(Source::from_bytes(
        r#"
        var xhr = new XMLHttpRequest();
        xhr.open("GET", "https://example.com");
        xhr.getResponseHeader("Content-Type")
        "#
    ));
    assert!(result.is_ok());
    let header_value = renderer.js_value_to_string(result.unwrap());
    assert_eq!(header_value, "null"); // Should return null before response
}

#[test]
fn test_xmlhttprequest_get_all_response_headers() {
    let mut renderer = RustRenderer::new();

    // Test getAllResponseHeaders before any response (should return empty string)
    let result = renderer.js_context.eval(Source::from_bytes(
        r#"
        var xhr = new XMLHttpRequest();
        xhr.open("GET", "https://example.com");
        xhr.getAllResponseHeaders()
        "#
    ));
    assert!(result.is_ok());
    let all_headers = renderer.js_value_to_string(result.unwrap());
    assert_eq!(all_headers, ""); // Should return empty string before response
}