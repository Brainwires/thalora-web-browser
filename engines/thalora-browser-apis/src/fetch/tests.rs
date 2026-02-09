//! Comprehensive test suite for Fetch API
//! Tests fetch(), Request, Response, Headers, and related APIs

use boa_engine::{Context, Source, JsValue};
use boa_engine::string::JsString;

// Helper to initialize context with browser APIs
fn create_test_context() -> Context {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context)
        .expect("Failed to initialize browser APIs");
    context
}

// ============================================================================
// fetch() Global Function Tests
// ============================================================================

#[test]
fn test_fetch_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof fetch")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_fetch_returns_promise() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let result = fetch('https://httpbin.org/get');
        result instanceof Promise;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_fetch_no_arguments() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        try {
            fetch();
            false;
        } catch(e) {
            true;
        }
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_fetch_invalid_url() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        try {
            fetch('not a valid url');
            false;
        } catch(e) {
            true;
        }
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_fetch_with_string_url() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let promise = fetch('https://httpbin.org/get');
        typeof promise === 'object' && promise !== null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Request Constructor Tests
// ============================================================================

#[test]
fn test_request_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Request")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_request_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let req = new Request('https://example.com');
        req !== null && req !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_request_constructor_no_arguments() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        try {
            new Request();
            false;
        } catch(e) {
            true;
        }
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_request_url_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let req = new Request('https://example.com/test');
        req.url === 'https://example.com/test';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_request_method_property_default() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let req = new Request('https://example.com');
        req.method === 'GET';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_request_method_property_custom() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let req = new Request('https://example.com', { method: 'POST' });
        req.method === 'POST';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_request_headers_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let req = new Request('https://example.com');
        req.headers !== null && req.headers !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_request_headers_identity() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let req = new Request('https://example.com');
        req.headers === req.headers;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_request_body_null_default() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let req = new Request('https://example.com');
        req.body === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_request_body_with_post() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let req = new Request('https://example.com', { method: 'POST', body: 'hello' });
        req.body === 'hello';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_request_clone() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let req = new Request('https://example.com', { method: 'POST' });
        let cloned = req.clone();
        cloned.url === req.url && cloned.method === req.method && cloned !== req;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_request_with_headers_init() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let req = new Request('https://example.com', {
            headers: { 'Content-Type': 'application/json' }
        });
        req !== null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_request_with_body() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let req = new Request('https://example.com', {
            method: 'POST',
            body: 'test data'
        });
        req !== null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Response Constructor Tests
// ============================================================================

#[test]
fn test_response_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Response")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_response_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response();
        res !== null && res !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_constructor_with_body() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response('test body');
        res !== null && res !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_constructor_with_init() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response('test', {
            status: 404,
            statusText: 'Not Found'
        });
        res !== null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_status_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response('test', { status: 404 });
        res.status === 404;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_status_default() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response();
        res.status === 200;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_status_text_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response('test', { statusText: 'Custom Status' });
        res.statusText === 'Custom Status';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_ok_property_true() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response('test', { status: 200 });
        res.ok === true;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_ok_property_false() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response('test', { status: 404 });
        res.ok === false;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_headers_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response();
        res.headers !== null && res.headers !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_headers_identity() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response();
        res.headers === res.headers;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_headers_get() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response('test', {
            headers: { 'Content-Type': 'text/plain' }
        });
        res.headers.get('content-type') === 'text/plain';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_url_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response();
        typeof res.url === 'string';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_type_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response();
        res.type === 'basic';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_redirected_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response();
        res.redirected === false;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_body_used_default() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response('test');
        res.bodyUsed === false;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Response Methods Tests
// ============================================================================

#[test]
fn test_response_text_method_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response('test');
        typeof res.text === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_json_method_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response('{}');
        typeof res.json === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_clone_method_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response('test');
        typeof res.clone === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_clone_copies_data() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = new Response('test body', { status: 201 });
        let cloned = res.clone();
        cloned.status === 201 && cloned !== res;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Response Static Methods
// ============================================================================

#[test]
fn test_response_error_static() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = Response.error();
        res.status === 0 && res.type === 'basic';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_redirect_static() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = Response.redirect('https://example.com', 301);
        res.status === 301 && res.headers.get('location') === 'https://example.com';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_redirect_default_status() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = Response.redirect('https://example.com');
        res.status === 302;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_redirect_invalid_status() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        try {
            Response.redirect('https://example.com', 200);
            false;
        } catch(e) {
            true;
        }
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_json_static() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = Response.json({ hello: 'world' });
        res.status === 200 && res.headers.get('content-type') === 'application/json';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_json_static_custom_status() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let res = Response.json({ error: 'not found' }, { status: 404 });
        res.status === 404;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Headers Constructor Tests
// ============================================================================

#[test]
fn test_headers_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Headers")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_headers_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers();
        headers !== null && headers !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_constructor_with_object() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers({
            'Content-Type': 'application/json',
            'X-Custom': 'value'
        });
        headers !== null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Headers Methods Tests
// ============================================================================

#[test]
fn test_headers_get_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers();
        typeof headers.get === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_set_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers();
        typeof headers.set === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_has_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers();
        typeof headers.has === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_delete_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers();
        typeof headers.delete === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_append_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers();
        typeof headers.append === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_get_returns_value() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers({ 'Content-Type': 'text/html' });
        headers.get('content-type') === 'text/html';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_get_returns_null_for_missing() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers();
        headers.get('x-missing') === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_get_case_insensitive() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers({ 'Content-Type': 'text/html' });
        headers.get('Content-Type') === 'text/html' &&
        headers.get('CONTENT-TYPE') === 'text/html' &&
        headers.get('content-type') === 'text/html';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_set_and_get() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers();
        headers.set('X-Custom', 'hello');
        headers.get('x-custom') === 'hello';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_set_overwrites() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers({ 'X-Custom': 'old' });
        headers.set('X-Custom', 'new');
        headers.get('x-custom') === 'new';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_has_true() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers({ 'Content-Type': 'text/html' });
        headers.has('Content-Type') === true;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_has_false() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers();
        headers.has('X-Missing') === false;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_delete_removes() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers({ 'X-Custom': 'value' });
        headers.delete('X-Custom');
        headers.has('x-custom') === false;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_append_new_header() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers();
        headers.append('Accept', 'text/html');
        headers.get('accept') === 'text/html';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_append_comma_separation() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers();
        headers.append('Accept', 'text/html');
        headers.append('Accept', 'application/json');
        headers.get('accept') === 'text/html, application/json';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_entries() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers({ 'b-header': '2', 'a-header': '1' });
        let entries = headers.entries();
        entries.length === 2 && entries[0][0] === 'a-header' && entries[1][0] === 'b-header';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_keys() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers({ 'b-header': '2', 'a-header': '1' });
        let keys = headers.keys();
        keys.length === 2 && keys[0] === 'a-header' && keys[1] === 'b-header';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_values() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers({ 'b-header': '2', 'a-header': '1' });
        let vals = headers.values();
        vals.length === 2 && vals[0] === '1' && vals[1] === '2';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_foreach() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let headers = new Headers({ 'content-type': 'text/html' });
        let collected = [];
        headers.forEach(function(value, name) {
            collected.push(name + ':' + value);
        });
        collected.length === 1 && collected[0] === 'content-type:text/html';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Fetch API Integration Tests
// ============================================================================

#[test]
fn test_fetch_with_request_object() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let req = new Request('https://httpbin.org/get');
        let promise = fetch(req);
        promise instanceof Promise;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_fetch_with_init_object() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let promise = fetch('https://httpbin.org/post', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: '{"test": true}'
        });
        promise instanceof Promise;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Property Descriptor Tests
// ============================================================================

#[test]
fn test_fetch_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'fetch');
        desc !== undefined && desc.value === fetch;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_request_constructor_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'Request');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_response_constructor_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'Response');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_headers_constructor_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'Headers');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}
