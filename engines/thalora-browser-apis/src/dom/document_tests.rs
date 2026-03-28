//! Comprehensive test suite for Document API
//! Tests Document constructor, properties, and methods

use boa_engine::string::JsString;
use boa_engine::{Context, JsValue, Source};

// Helper to initialize context with browser APIs
fn create_test_context() -> Context {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
    context
}

// ============================================================================
// Document Constructor Tests
// ============================================================================

#[test]
fn test_document_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Document")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_document_construction() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        doc instanceof Document;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Document Properties Tests
// ============================================================================

#[test]
fn test_document_ready_state_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        typeof doc.readyState === 'string';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_url_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        typeof doc.URL === 'string';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_title_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        typeof doc.title === 'string';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_title_setter() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        doc.title = 'Test Title';
        doc.title === 'Test Title';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_body_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        doc.body === null || doc.body === undefined || typeof doc.body === 'object';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_head_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        doc.head === null || doc.head === undefined || typeof doc.head === 'object';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Document.createElement Tests
// ============================================================================

#[test]
fn test_document_create_element_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        typeof doc.createElement === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_create_element_basic() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el !== null && el !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_create_element_tag_names() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let div = doc.createElement('div');
        let span = doc.createElement('span');
        let p = doc.createElement('p');
        div && span && p;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_create_element_uppercase() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('DIV');
        el !== null;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_create_element_mixedcase() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('DiV');
        el !== null;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_create_element_custom_tag() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('my-custom-element');
        el !== null;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Document.getElementById Tests
// ============================================================================

#[test]
fn test_document_get_element_by_id_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        typeof doc.getElementById === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_get_element_by_id_returns_null() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.getElementById('nonexistent');
        el === null;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_get_element_by_id_with_empty_string() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.getElementById('');
        el === null;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Document.querySelector Tests
// ============================================================================

#[test]
fn test_document_query_selector_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        typeof doc.querySelector === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_query_selector_returns_null() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.querySelector('.nonexistent');
        el === null;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_query_selector_tag_selector() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.querySelector('div');
        el === null;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_query_selector_class_selector() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.querySelector('.myclass');
        el === null;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_query_selector_id_selector() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.querySelector('#myid');
        el === null;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Document.querySelectorAll Tests
// ============================================================================

#[test]
fn test_document_query_selector_all_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        typeof doc.querySelectorAll === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_query_selector_all_returns_nodelist() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let list = doc.querySelectorAll('div');
        list !== null && list !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_query_selector_all_empty_result() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let list = doc.querySelectorAll('.nonexistent');
        typeof list === 'object';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Document Method Edge Cases
// ============================================================================

#[test]
fn test_document_create_element_without_argument() {
    let mut context = create_test_context();
    // Implementation doesn't throw error, returns element for "undefined"
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement();
        el !== null && el !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_create_element_with_null() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement(null);
        el !== null;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_create_element_with_number() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement(123);
        el !== null;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_get_element_by_id_without_argument() {
    let mut context = create_test_context();
    // Implementation doesn't throw error, returns null for undefined
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.getElementById();
        el === null;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_query_selector_without_argument() {
    let mut context = create_test_context();
    // Implementation doesn't throw error, returns null for undefined
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.querySelector();
        el === null;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_query_selector_all_without_argument() {
    let mut context = create_test_context();
    // Implementation doesn't throw error, returns empty result
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let list = doc.querySelectorAll();
        list !== null && list !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Document Inheritance Tests
// ============================================================================

#[test]
fn test_document_inherits_from_node() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        doc instanceof Node;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_inherits_from_event_target() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        doc instanceof EventTarget;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Document Property Descriptors
// ============================================================================

#[test]
fn test_document_ready_state_is_readonly() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let original = doc.readyState;
        doc.readyState = 'modified';
        doc.readyState === original;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_url_is_readonly() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let original = doc.URL;
        doc.URL = 'modified';
        doc.URL === original;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_body_is_readonly() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let original = doc.body;
        doc.body = 'modified';
        doc.body === original;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_head_is_readonly() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let original = doc.head;
        doc.head = 'modified';
        doc.head === original;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Multiple Document Tests
// ============================================================================

#[test]
fn test_multiple_documents() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc1 = new Document();
        let doc2 = new Document();
        doc1 !== doc2;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_create_multiple_elements() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el1 = doc.createElement('div');
        let el2 = doc.createElement('div');
        let el3 = doc.createElement('div');
        el1 && el2 && el3;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}
