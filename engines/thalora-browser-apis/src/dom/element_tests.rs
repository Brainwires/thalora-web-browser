//! Comprehensive test suite for Element API
//! Tests Element constructor, properties, and methods

use boa_engine::string::JsString;
use boa_engine::{Context, JsValue, Source};

// Helper to initialize context with browser APIs
fn create_test_context() -> Context {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
    context
}

// ============================================================================
// Element Constructor Tests
// ============================================================================

#[test]
fn test_element_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Element")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_element_construction() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let el = new Element();
        el instanceof Element;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Element Properties Tests
// ============================================================================

#[test]
fn test_element_tag_name_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.tagName === 'string';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_id_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.id === 'string';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_id_setter() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.id = 'test-id';
        el.id === 'test-id';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_class_name_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.className === 'string';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_class_name_setter() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.className = 'test-class';
        el.className === 'test-class';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_inner_html_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.innerHTML === 'string';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_inner_html_setter() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.innerHTML = '<span>test</span>';
        el.innerHTML.includes('span');
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_text_content_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.textContent === 'string';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_text_content_setter() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.textContent = 'test text';
        el.textContent === 'test text';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_children_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.children !== null && el.children !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_parent_node_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.parentNode === null;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_style_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.style !== null && el.style !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Element.setAttribute Tests
// ============================================================================

#[test]
fn test_element_set_attribute_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.setAttribute === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_set_attribute_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(
        r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.setAttribute('data-test', 'value');
        true;
    "#,
    ));
    assert!(result.is_ok());
}

#[test]
fn test_element_set_attribute_multiple() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(
        r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.setAttribute('attr1', 'value1');
        el.setAttribute('attr2', 'value2');
        el.setAttribute('attr3', 'value3');
        true;
    "#,
    ));
    assert!(result.is_ok());
}

// ============================================================================
// Element.getAttribute Tests
// ============================================================================

#[test]
fn test_element_get_attribute_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.getAttribute === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_get_attribute_returns_null() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        let val = el.getAttribute('nonexistent');
        val === null;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_get_attribute_after_set() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.setAttribute('test', 'value');
        el.getAttribute('test') === 'value';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Element.hasAttribute Tests
// ============================================================================

#[test]
fn test_element_has_attribute_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.hasAttribute === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_has_attribute_returns_false() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.hasAttribute('nonexistent') === false;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_has_attribute_returns_true() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.setAttribute('test', 'value');
        el.hasAttribute('test') === true;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Element.removeAttribute Tests
// ============================================================================

#[test]
fn test_element_remove_attribute_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.removeAttribute === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_remove_attribute_basic() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.setAttribute('test', 'value');
        el.removeAttribute('test');
        el.hasAttribute('test') === false;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_remove_nonexistent_attribute() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(
        r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.removeAttribute('nonexistent');
        true;
    "#,
    ));
    assert!(result.is_ok());
}

// ============================================================================
// Element.appendChild Tests
// ============================================================================

#[test]
fn test_element_append_child_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.appendChild === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_append_child_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(
        r#"
        let doc = new Document();
        let parent = doc.createElement('div');
        let child = doc.createElement('span');
        parent.appendChild(child);
        true;
    "#,
    ));
    assert!(result.is_ok());
}

#[test]
fn test_element_append_multiple_children() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(
        r#"
        let doc = new Document();
        let parent = doc.createElement('div');
        let child1 = doc.createElement('span');
        let child2 = doc.createElement('p');
        parent.appendChild(child1);
        parent.appendChild(child2);
        true;
    "#,
    ));
    assert!(result.is_ok());
}

// ============================================================================
// Element.removeChild Tests
// ============================================================================

#[test]
fn test_element_remove_child_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.removeChild === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Element Event Methods Tests
// ============================================================================

#[test]
fn test_element_add_event_listener_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.addEventListener === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_remove_event_listener_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.removeEventListener === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_dispatch_event_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.dispatchEvent === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_add_event_listener_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(
        r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.addEventListener('click', function() {});
        true;
    "#,
    ));
    assert!(result.is_ok());
}

#[test]
fn test_element_dispatch_event_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(
        r#"
        let doc = new Document();
        let el = doc.createElement('div');
        let event = new Event('click');
        el.dispatchEvent(event);
        true;
    "#,
    ));
    assert!(result.is_ok());
}

#[test]
fn test_element_event_listener_called() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        let called = false;
        el.addEventListener('test', function() { called = true; });
        el.dispatchEvent(new Event('test'));
        called;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Element HTML Methods Tests
// ============================================================================

#[test]
fn test_element_set_html_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.setHTML === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_set_html_unsafe_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.setHTMLUnsafe === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Element Shadow DOM Tests
// ============================================================================

#[test]
fn test_element_attach_shadow_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        typeof el.attachShadow === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Element Edge Cases
// ============================================================================

#[test]
fn test_element_set_attribute_with_empty_value() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.setAttribute('test', '');
        el.getAttribute('test') === '';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_set_attribute_override() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.setAttribute('test', 'value1');
        el.setAttribute('test', 'value2');
        el.getAttribute('test') === 'value2';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_set_attribute_case_sensitive() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.setAttribute('Test', 'value1');
        el.setAttribute('test', 'value2');
        el.getAttribute('Test') === 'value1' && el.getAttribute('test') === 'value2';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_inner_html_with_nested_elements() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.innerHTML = '<div><span>nested</span></div>';
        el.innerHTML.includes('nested');
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_text_content_strips_html() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.innerHTML = '<span>test</span>';
        el.textContent.includes('test');
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_id_empty_string() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.id = '';
        el.id === '';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_class_name_multiple_classes() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('div');
        el.className = 'class1 class2 class3';
        el.className === 'class1 class2 class3';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Multiple Elements Tests
// ============================================================================

#[test]
fn test_multiple_elements_independent() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el1 = doc.createElement('div');
        let el2 = doc.createElement('div');
        el1.id = 'first';
        el2.id = 'second';
        el1.id === 'first' && el2.id === 'second';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_multiple_elements_attributes() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el1 = doc.createElement('div');
        let el2 = doc.createElement('div');
        el1.setAttribute('test', 'value1');
        el2.setAttribute('test', 'value2');
        el1.getAttribute('test') === 'value1' && el2.getAttribute('test') === 'value2';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_tag_name_case() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let doc = new Document();
        let el = doc.createElement('DiV');
        typeof el.tagName === 'string';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}
