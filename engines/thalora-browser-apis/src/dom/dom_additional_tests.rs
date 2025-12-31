//! Additional DOM API tests for CharacterData, DocumentFragment, Range, and Text

use crate::boa_engine::{Context, Source, JsValue};
use crate::boa_engine::string::JsString;

// Helper to initialize context with browser APIs
fn create_test_context() -> Context {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context)
        .expect("Failed to initialize browser APIs");
    context
}

// ============================================================================
// CharacterData Tests
// ============================================================================

#[test]
fn test_character_data_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof CharacterData")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_character_data_data_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let text = document.createTextNode('test');
        text.data === 'test';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_character_data_length_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let text = document.createTextNode('hello');
        text.length === 5;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_character_data_substring_data_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let text = document.createTextNode('hello world');
        typeof text.substringData === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_character_data_append_data_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let text = document.createTextNode('hello');
        typeof text.appendData === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// DocumentFragment Tests
// ============================================================================

#[test]
fn test_document_fragment_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof DocumentFragment")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_document_create_document_fragment() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let frag = document.createDocumentFragment();
        frag !== null && frag !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_fragment_constructor_new() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let frag = new DocumentFragment();
        frag !== null && frag !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_fragment_is_node() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let frag = new DocumentFragment();
        frag instanceof Node;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_fragment_append_child() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let frag = new DocumentFragment();
        let div = document.createElement('div');
        typeof frag.appendChild === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Range Tests
// ============================================================================

#[test]
fn test_range_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Range")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_document_create_range() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let range = document.createRange();
        range !== null && range !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_range_constructor_new() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let range = new Range();
        range !== null && range !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_range_collapsed_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let range = new Range();
        typeof range.collapsed === 'boolean';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_range_set_start_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let range = new Range();
        typeof range.setStart === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_range_set_end_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let range = new Range();
        typeof range.setEnd === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Text Tests
// ============================================================================

#[test]
fn test_text_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Text")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_document_create_text_node() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let text = document.createTextNode('hello');
        text !== null && text !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_text_constructor_new() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let text = new Text('hello');
        text !== null && text !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_text_is_character_data() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let text = new Text('test');
        text instanceof CharacterData;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_text_whole_text_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let text = new Text('hello');
        typeof text.wholeText === 'string';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_text_split_text_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let text = new Text('hello world');
        typeof text.splitText === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_all_dom_additional_apis_available() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof CharacterData === 'function' &&
        typeof DocumentFragment === 'function' &&
        typeof Range === 'function' &&
        typeof Text === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_text_node_inheritance() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let text = new Text('test');
        text instanceof Text &&
        text instanceof CharacterData &&
        text instanceof Node;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_fragment_children() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let frag = new DocumentFragment();
        typeof frag.children !== 'undefined';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Property Descriptor Tests
// ============================================================================

#[test]
fn test_character_data_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'CharacterData');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_document_fragment_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'DocumentFragment');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_range_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'Range');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_text_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'Text');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Text Node Slot Assignment Tests
// ============================================================================

#[test]
fn test_text_assigned_slot_property_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let text = new Text('hello');
        'assignedSlot' in text;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_text_assigned_slot_initial_value() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let text = new Text('hello');
        // Initially, text node is not assigned to any slot
        text.assignedSlot === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_text_assigned_slot_accessor_is_getter() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(Text.prototype, 'assignedSlot');
        desc !== undefined && typeof desc.get === 'function' && desc.set === undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Range.createContextualFragment Tests
// ============================================================================

#[test]
fn test_range_create_contextual_fragment_method_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let range = new Range();
        typeof range.createContextualFragment === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_range_create_contextual_fragment_returns_document_fragment() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let range = new Range();
        let frag = range.createContextualFragment('<div>test</div>');
        frag instanceof DocumentFragment;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_range_create_contextual_fragment_returns_node() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let range = new Range();
        let frag = range.createContextualFragment('<span>content</span>');
        frag instanceof Node;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_range_create_contextual_fragment_with_empty_string() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let range = new Range();
        let frag = range.createContextualFragment('');
        frag !== null && frag instanceof DocumentFragment;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_range_create_contextual_fragment_with_text() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let range = new Range();
        let frag = range.createContextualFragment('plain text');
        frag instanceof DocumentFragment;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}
