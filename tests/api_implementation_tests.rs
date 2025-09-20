//! Integration tests for the implemented JavaScript APIs
//! Tests XMLHttpRequest, MutationObserver, IntersectionObserver, and ResizeObserver

use boa_engine::{Context, Source};

#[test]
fn test_all_apis_available() {
    let mut context = Context::default();

    // Test all APIs are available as functions
    let script = r#"
        [
            typeof XMLHttpRequest,
            typeof MutationObserver,
            typeof IntersectionObserver,
            typeof ResizeObserver
        ].join(',')
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());
    let type_str = result.unwrap().to_string(&mut context).unwrap();
    assert_eq!(type_str.to_std_string_escaped(), "function,function,function,function");
}

#[test]
fn test_xmlhttprequest_basic() {
    let mut context = Context::default();

    // Test XMLHttpRequest constructor and basic functionality
    let script = r#"
        try {
            var xhr = new XMLHttpRequest();
            xhr.open("GET", "https://example.com");
            xhr.readyState === 1; // OPENED state
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());
    let success = result.unwrap().to_boolean();
    assert!(success);
}

#[test]
fn test_mutation_observer_basic() {
    let mut context = Context::default();

    // Test MutationObserver constructor and methods
    let script = r#"
        try {
            var observer = new MutationObserver(function() {});
            var target = {};
            observer.observe(target, { childList: true });
            observer.disconnect();
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());
    let success = result.unwrap().to_boolean();
    assert!(success);
}

#[test]
fn test_intersection_observer_basic() {
    let mut context = Context::default();

    // Test IntersectionObserver constructor and methods
    let script = r#"
        try {
            var observer = new IntersectionObserver(function() {});
            var target = {};
            observer.observe(target);
            observer.unobserve(target);
            observer.disconnect();
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());
    let success = result.unwrap().to_boolean();
    assert!(success);
}

#[test]
fn test_resize_observer_basic() {
    let mut context = Context::default();

    // Test ResizeObserver constructor and methods
    let script = r#"
        try {
            var observer = new ResizeObserver(function() {});
            var target = {};
            observer.observe(target);
            observer.unobserve(target);
            observer.disconnect();
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());
    let success = result.unwrap().to_boolean();
    assert!(success);
}

#[test]
fn test_xmlhttprequest_methods() {
    let mut context = Context::default();

    // Test XMLHttpRequest has all required methods
    let script = r#"
        var xhr = new XMLHttpRequest();
        [
            typeof xhr.open,
            typeof xhr.send,
            typeof xhr.setRequestHeader,
            typeof xhr.getResponseHeader,
            typeof xhr.getAllResponseHeaders,
            typeof xhr.abort
        ].join(',')
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());
    let methods = result.unwrap().to_string(&mut context).unwrap();
    assert_eq!(methods.to_std_string_escaped(), "function,function,function,function,function,function");
}

#[test]
fn test_observer_apis_methods() {
    let mut context = Context::default();

    // Test all observer APIs have required methods
    let script = r#"
        var mo = new MutationObserver(function() {});
        var io = new IntersectionObserver(function() {});
        var ro = new ResizeObserver(function() {});

        [
            typeof mo.observe,
            typeof mo.disconnect,
            typeof mo.takeRecords,
            typeof io.observe,
            typeof io.unobserve,
            typeof io.disconnect,
            typeof io.takeRecords,
            typeof ro.observe,
            typeof ro.unobserve,
            typeof ro.disconnect
        ].join(',')
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());
    let methods = result.unwrap().to_string(&mut context).unwrap();
    assert_eq!(methods.to_std_string_escaped(), "function,function,function,function,function,function,function,function,function,function");
}

#[test]
fn test_xmlhttprequest_error_handling() {
    let mut context = Context::default();

    // Test XMLHttpRequest error handling
    let script = r#"
        try {
            new XMLHttpRequest(); // Should not fail without arguments
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());
    let success = result.unwrap().to_boolean();
    assert!(success);
}

#[test]
fn test_observer_error_handling() {
    let mut context = Context::default();

    // Test observer error handling for missing callbacks
    let script = r#"
        var errors = 0;

        try {
            new MutationObserver();
        } catch(e) {
            errors++;
        }

        try {
            new IntersectionObserver();
        } catch(e) {
            errors++;
        }

        try {
            new ResizeObserver();
        } catch(e) {
            errors++;
        }

        errors === 3; // All should throw errors
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());
    let all_errored = result.unwrap().to_boolean();
    assert!(all_errored);
}