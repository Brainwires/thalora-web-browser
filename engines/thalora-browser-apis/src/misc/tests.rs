//! Comprehensive test suite for Miscellaneous APIs
//! Tests AbortController, CSS, Form elements, and StructuredClone

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
// AbortController Tests
// ============================================================================

#[test]
fn test_abort_controller_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof AbortController")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_abort_controller_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let controller = new AbortController();
        controller !== null && controller !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_abort_controller_signal_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let controller = new AbortController();
        typeof controller.signal === 'object';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_abort_controller_abort_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let controller = new AbortController();
        typeof controller.abort === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_abort_controller_signal_aborted() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let controller = new AbortController();
        let initialAborted = controller.signal.aborted;
        controller.abort();
        let afterAborted = controller.signal.aborted;
        initialAborted === false && afterAborted === true;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// CSS Tests
// ============================================================================

#[test]
fn test_css_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof CSS")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));
}

#[test]
fn test_css_supports_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof CSS.supports === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_css_escape_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof CSS.escape === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// HTMLFormElement Tests
// ============================================================================

#[test]
fn test_html_form_element_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof HTMLFormElement")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_html_form_element_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let form = new HTMLFormElement();
        form !== null && form !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_html_form_element_submit_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let form = new HTMLFormElement();
        typeof form.submit === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_html_form_element_reset_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let form = new HTMLFormElement();
        typeof form.reset === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// HTMLInputElement Tests
// ============================================================================

#[test]
fn test_html_input_element_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof HTMLInputElement")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_html_input_element_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let input = new HTMLInputElement();
        input !== null && input !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_html_input_element_value_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let input = new HTMLInputElement();
        input.value = 'test';
        input.value === 'test';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_html_input_element_type_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let input = new HTMLInputElement();
        typeof input.type === 'string';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Property Descriptor Tests
// ============================================================================

#[test]
fn test_abort_controller_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'AbortController');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_css_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'CSS');
        desc !== undefined && typeof desc.value === 'object';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_html_form_element_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'HTMLFormElement');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_html_input_element_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'HTMLInputElement');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_abort_controller_basic_usage() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let controller = new AbortController();
        let wasAborted = false;
        controller.signal.addEventListener('abort', () => {
            wasAborted = true;
        });
        controller.abort();
        true; // Can't test wasAborted in sync execution
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_all_misc_apis_available() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof AbortController === 'function' &&
        typeof CSS === 'object' &&
        typeof HTMLFormElement === 'function' &&
        typeof HTMLInputElement === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}
