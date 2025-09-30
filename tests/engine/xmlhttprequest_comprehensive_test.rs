//! Comprehensive unit tests for XMLHttpRequest implementation in Boa engine

use boa_engine::{Context, Source};

#[test]
fn test_xmlhttprequest_constructor_availability() {
    let mut context = Context::default();

    // Test that XMLHttpRequest exists as a function
    let result = context.eval(Source::from_bytes("typeof XMLHttpRequest"));
    assert!(result.is_ok());
    let type_str = result.unwrap().to_string(&mut context).unwrap();
    assert_eq!(type_str.to_std_string_escaped(), "function");
}

#[test]
fn test_xmlhttprequest_constructor_with_new() {
    let mut context = Context::default();

    // Test constructor with 'new' operator
    let result = context.eval(Source::from_bytes("var xhr = new XMLHttpRequest(); xhr"));
    assert!(result.is_ok());

    // Test that the created object has the right type
    let result = context.eval(Source::from_bytes("typeof xhr"));
    assert!(result.is_ok());
    let type_str = result.unwrap().to_string(&mut context).unwrap();
    assert_eq!(type_str.to_std_string_escaped(), "object");
}

#[test]
fn test_xmlhttprequest_constructor_without_new_fails() {
    let mut context = Context::default();

    // Test that constructor fails without 'new' operator
    let result = context.eval(Source::from_bytes("XMLHttpRequest()"));
    assert!(result.is_err());
}

#[test]
fn test_xmlhttprequest_initial_state() {
    let mut context = Context::default();

    let script = r#"
        var xhr = new XMLHttpRequest();
        JSON.stringify({
            readyState: xhr.readyState,
            status: xhr.status,
            statusText: xhr.statusText,
            responseText: xhr.responseText
        })
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let json_str = result.unwrap().to_string(&mut context).unwrap().to_std_string_escaped();
    assert!(json_str.contains("\"readyState\":0"));
    assert!(json_str.contains("\"status\":0"));
    assert!(json_str.contains("\"statusText\":\"\""));
    assert!(json_str.contains("\"responseText\":\"\""));
}

#[test]
fn test_xmlhttprequest_constants_exist() {
    let mut context = Context::default();

    let constants = [
        ("UNSENT", "0"),
        ("OPENED", "1"),
        ("HEADERS_RECEIVED", "2"),
        ("LOADING", "3"),
        ("DONE", "4")
    ];

    for (constant, expected_value) in &constants {
        let script = format!("XMLHttpRequest.{}", constant);
        let result = context.eval(Source::from_bytes(&script));
        assert!(result.is_ok(), "Constant {} should exist", constant);

        let value = result.unwrap().to_number(&mut context).unwrap();
        assert_eq!(value, expected_value.parse::<f64>().unwrap(),
                   "Constant {} should equal {}", constant, expected_value);
    }
}

#[test]
fn test_xmlhttprequest_methods_exist() {
    let mut context = Context::default();

    let methods = [
        "open",
        "send",
        "setRequestHeader",
        "getResponseHeader",
        "getAllResponseHeaders",
        "abort"
    ];

    for method in &methods {
        let script = format!("var xhr = new XMLHttpRequest(); typeof xhr.{}", method);
        let result = context.eval(Source::from_bytes(&script));
        assert!(result.is_ok(), "Method {} should exist", method);

        let method_type = result.unwrap().to_string(&mut context).unwrap().to_std_string_escaped();
        assert_eq!(method_type, "function", "Method {} should be a function", method);
    }
}

#[test]
fn test_xmlhttprequest_open_method_basic() {
    let mut context = Context::default();

    let script = r#"
        var xhr = new XMLHttpRequest();
        xhr.open("GET", "https://example.com");
        xhr.readyState
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let ready_state = result.unwrap().to_number(&mut context).unwrap();
    assert_eq!(ready_state, 1.0); // OPENED state
}

#[test]
fn test_xmlhttprequest_open_method_invalid_url() {
    let mut context = Context::default();

    let script = r#"
        var xhr = new XMLHttpRequest();
        try {
            xhr.open("GET", "invalid-url");
            false;
        } catch(e) {
            true;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let caught_error = result.unwrap().to_boolean();
    assert!(caught_error, "Invalid URL should throw an error");
}

#[test]
fn test_xmlhttprequest_open_method_invalid_method() {
    let mut context = Context::default();

    let script = r#"
        var xhr = new XMLHttpRequest();
        try {
            xhr.open("INVALID", "https://example.com");
            false;
        } catch(e) {
            true;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let caught_error = result.unwrap().to_boolean();
    assert!(caught_error, "Invalid HTTP method should throw an error");
}

#[test]
fn test_xmlhttprequest_set_request_header_after_open() {
    let mut context = Context::default();

    let script = r#"
        var xhr = new XMLHttpRequest();
        xhr.open("POST", "https://example.com");
        xhr.setRequestHeader("Content-Type", "application/json");
        xhr.readyState
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let ready_state = result.unwrap().to_number(&mut context).unwrap();
    assert_eq!(ready_state, 1.0); // Should still be OPENED
}

#[test]
fn test_xmlhttprequest_set_request_header_before_open_fails() {
    let mut context = Context::default();

    let script = r#"
        var xhr = new XMLHttpRequest();
        try {
            xhr.setRequestHeader("Content-Type", "application/json");
            false;
        } catch(e) {
            true;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let caught_error = result.unwrap().to_boolean();
    assert!(caught_error, "setRequestHeader before open should throw an error");
}

#[test]
fn test_xmlhttprequest_abort_method() {
    let mut context = Context::default();

    let script = r#"
        var xhr = new XMLHttpRequest();
        xhr.open("GET", "https://example.com");
        xhr.abort();
        JSON.stringify({
            readyState: xhr.readyState,
            status: xhr.status
        })
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let json_str = result.unwrap().to_string(&mut context).unwrap().to_std_string_escaped();
    assert!(json_str.contains("\"readyState\":4")); // DONE after abort
    assert!(json_str.contains("\"status\":0"));     // Status should be 0 after abort
}

#[test]
fn test_xmlhttprequest_properties_exist() {
    let mut context = Context::default();

    let properties = [
        "readyState",
        "status",
        "statusText",
        "responseText",
        "responseXML",
        "timeout",
        "withCredentials",
        "upload"
    ];

    for property in &properties {
        let script = format!("var xhr = new XMLHttpRequest(); '{}' in xhr", property);
        let result = context.eval(Source::from_bytes(&script));
        assert!(result.is_ok(), "Property {} should exist", property);

        let has_property = result.unwrap().to_boolean();
        assert!(has_property, "Property {} should exist on XMLHttpRequest instance", property);
    }
}

#[test]
fn test_xmlhttprequest_event_handlers_exist() {
    let mut context = Context::default();

    let event_handlers = [
        "onreadystatechange",
        "onload",
        "onerror",
        "onabort",
        "ontimeout",
        "onloadstart",
        "onloadend",
        "onprogress"
    ];

    for handler in &event_handlers {
        let script = format!("var xhr = new XMLHttpRequest(); '{}' in xhr", handler);
        let result = context.eval(Source::from_bytes(&script));
        assert!(result.is_ok(), "Event handler {} should exist", handler);

        let has_handler = result.unwrap().to_boolean();
        assert!(has_handler, "Event handler {} should exist on XMLHttpRequest instance", handler);
    }
}

#[test]
fn test_xmlhttprequest_event_handler_assignment() {
    let mut context = Context::default();

    let script = r#"
        var xhr = new XMLHttpRequest();
        var called = false;
        xhr.onreadystatechange = function() { called = true; };
        xhr.open("GET", "https://example.com");
        called
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let was_called = result.unwrap().to_boolean();
    assert!(was_called, "Event handler should be called during open()");
}

#[test]
fn test_xmlhttprequest_get_response_header_before_response() {
    let mut context = Context::default();

    let script = r#"
        var xhr = new XMLHttpRequest();
        xhr.open("GET", "https://example.com");
        xhr.getResponseHeader("Content-Type")
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let header_value = result.unwrap();
    assert!(header_value.is_null(), "getResponseHeader should return null before response");
}

#[test]
fn test_xmlhttprequest_get_all_response_headers_before_response() {
    let mut context = Context::default();

    let script = r#"
        var xhr = new XMLHttpRequest();
        xhr.open("GET", "https://example.com");
        xhr.getAllResponseHeaders()
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let all_headers = result.unwrap().to_string(&mut context).unwrap().to_std_string_escaped();
    assert_eq!(all_headers, "", "getAllResponseHeaders should return empty string before response");
}

#[test]
fn test_xmlhttprequest_upload_property() {
    let mut context = Context::default();

    let script = r#"
        var xhr = new XMLHttpRequest();
        typeof xhr.upload
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let upload_type = result.unwrap().to_string(&mut context).unwrap().to_std_string_escaped();
    assert_eq!(upload_type, "object", "upload property should be an object");
}

#[test]
fn test_xmlhttprequest_multiple_instances() {
    let mut context = Context::default();

    let script = r#"
        var xhr1 = new XMLHttpRequest();
        var xhr2 = new XMLHttpRequest();
        xhr1.open("GET", "https://example1.com");
        xhr2.open("POST", "https://example2.com");
        JSON.stringify({
            xhr1_state: xhr1.readyState,
            xhr2_state: xhr2.readyState,
            different_instances: xhr1 !== xhr2
        })
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let json_str = result.unwrap().to_string(&mut context).unwrap().to_std_string_escaped();
    assert!(json_str.contains("\"xhr1_state\":1"));
    assert!(json_str.contains("\"xhr2_state\":1"));
    assert!(json_str.contains("\"different_instances\":true"));
}