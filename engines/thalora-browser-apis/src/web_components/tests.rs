//! Tests for Web Components API
//! Tests CustomElementRegistry and HTMLTemplateElement

use crate::boa_engine::string::JsString;
use crate::boa_engine::{Context, JsValue, Source};

// Helper to initialize context with browser APIs
fn create_test_context() -> Context {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
    context
}

// ============================================================================
// CustomElementRegistry Tests
// ============================================================================

#[test]
fn test_custom_elements_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes("typeof customElements"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));
}

#[test]
fn test_custom_elements_define_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes("typeof customElements.define"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_custom_elements_get_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes("typeof customElements.get"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_custom_elements_when_defined_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes("typeof customElements.whenDefined"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_custom_elements_upgrade_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes("typeof customElements.upgrade"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_custom_elements_define_basic() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        class MyElement {
            constructor() {}
        }
        customElements.define('my-element', MyElement);
        true;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_custom_elements_get_after_define() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        class TestElement {
            constructor() {}
        }
        customElements.define('test-element', TestElement);
        customElements.get('test-element') !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_custom_elements_get_undefined() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        customElements.get('nonexistent-element') === undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_custom_elements_invalid_name_no_hyphen() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        try {
            customElements.define('myelement', class {});
            false;
        } catch(e) {
            true;
        }
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_custom_elements_invalid_name_starts_uppercase() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        try {
            customElements.define('My-element', class {});
            false;
        } catch(e) {
            true;
        }
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_custom_elements_define_not_function() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        try {
            customElements.define('my-element2', {});
            false;
        } catch(e) {
            true;
        }
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_custom_elements_when_defined_returns_promise() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let result = customElements.whenDefined('future-element');
        result instanceof Promise;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_custom_elements_with_extends_option() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        class FancyButton {
            constructor() {}
        }
        customElements.define('fancy-button', FancyButton, { extends: 'button' });
        true;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// HTMLTemplateElement Tests
// ============================================================================

#[test]
fn test_html_template_element_exists() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes("typeof HTMLTemplateElement"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_html_template_element_constructor() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let template = new HTMLTemplateElement();
        template !== null && template !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_html_template_element_content_property() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let template = new HTMLTemplateElement();
        template.content !== null && template.content !== undefined;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_html_template_element_content_is_fragment() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let template = new HTMLTemplateElement();
        template.content.nodeType === 11;
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_html_template_element_tag_name() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let template = new HTMLTemplateElement();
        template.tagName === 'TEMPLATE';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_html_template_element_content_has_child_nodes() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let template = new HTMLTemplateElement();
        Array.isArray(template.content.childNodes);
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_html_template_element_content_append_child() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let template = new HTMLTemplateElement();
        typeof template.content.appendChild === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_html_template_element_content_clone_node() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let template = new HTMLTemplateElement();
        typeof template.content.cloneNode === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_html_template_element_content_query_selector() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let template = new HTMLTemplateElement();
        typeof template.content.querySelector === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_html_template_element_content_query_selector_all() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        let template = new HTMLTemplateElement();
        typeof template.content.querySelectorAll === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_web_components_available() {
    let mut context = create_test_context();
    let result = context
        .eval(Source::from_bytes(
            r#"
        typeof customElements === 'object' &&
        typeof HTMLTemplateElement === 'function';
    "#,
        ))
        .unwrap();
    assert_eq!(result.to_boolean(), true);
}
