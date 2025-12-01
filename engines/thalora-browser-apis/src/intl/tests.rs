//! Tests for Intl API (provided by Boa engine)
//!
//! The Intl API is enabled via the `intl_bundled` feature in Boa engine.
//!
//! Note: Some Intl features are not yet fully implemented in Boa:
//! - Intl.RelativeTimeFormat - not available
//! - Intl.DisplayNames - not available
//! - Intl.DateTimeFormat.format() - returns undefined
//! - Intl.NumberFormat with options - throws "unimplemented"

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
// Intl Object Tests
// ============================================================================

#[test]
fn test_intl_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Intl")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));
}

#[test]
fn test_intl_number_format_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Intl.NumberFormat")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_intl_date_time_format_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Intl.DateTimeFormat")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_intl_collator_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Intl.Collator")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_intl_list_format_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Intl.ListFormat")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_intl_plural_rules_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Intl.PluralRules")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_intl_relative_time_format_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Intl.RelativeTimeFormat")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_intl_segmenter_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Intl.Segmenter")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_intl_locale_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Intl.Locale")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_intl_display_names_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Intl.DisplayNames")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

// ============================================================================
// Intl.NumberFormat Tests
// ============================================================================

#[test]
fn test_intl_number_format_constructor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let formatter = new Intl.NumberFormat('en-US');
        formatter !== null && formatter !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_intl_number_format_format() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let formatter = new Intl.NumberFormat('en-US');
        typeof formatter.format === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
#[ignore = "Boa: NumberFormat with currency style not yet implemented"]
fn test_intl_number_format_with_currency() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let formatter = new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' });
        formatter !== null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Intl.DateTimeFormat Tests
// ============================================================================

#[test]
fn test_intl_date_time_format_constructor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let formatter = new Intl.DateTimeFormat('en-US');
        formatter !== null && formatter !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
#[ignore = "Boa: DateTimeFormat.prototype.format not yet implemented"]
fn test_intl_date_time_format_format() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let formatter = new Intl.DateTimeFormat('en-US');
        typeof formatter.format === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Intl.Collator Tests
// ============================================================================

#[test]
fn test_intl_collator_constructor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let collator = new Intl.Collator('en-US');
        collator !== null && collator !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_intl_collator_compare() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let collator = new Intl.Collator('en-US');
        typeof collator.compare === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Intl.ListFormat Tests
// ============================================================================

#[test]
fn test_intl_list_format_constructor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let formatter = new Intl.ListFormat('en-US');
        formatter !== null && formatter !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_intl_list_format_format() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let formatter = new Intl.ListFormat('en-US');
        typeof formatter.format === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Intl.PluralRules Tests
// ============================================================================

#[test]
fn test_intl_plural_rules_constructor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let rules = new Intl.PluralRules('en-US');
        rules !== null && rules !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_intl_plural_rules_select() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let rules = new Intl.PluralRules('en-US');
        typeof rules.select === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Intl.RelativeTimeFormat Tests
// ============================================================================

#[test]
fn test_intl_relative_time_format_constructor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let formatter = new Intl.RelativeTimeFormat('en-US');
        formatter !== null && formatter !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_intl_relative_time_format_format() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let formatter = new Intl.RelativeTimeFormat('en-US');
        typeof formatter.format === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Intl.Segmenter Tests
// ============================================================================

#[test]
fn test_intl_segmenter_constructor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let segmenter = new Intl.Segmenter('en-US');
        segmenter !== null && segmenter !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_intl_segmenter_segment() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let segmenter = new Intl.Segmenter('en-US');
        typeof segmenter.segment === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Intl.Locale Tests
// ============================================================================

#[test]
fn test_intl_locale_constructor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let locale = new Intl.Locale('en-US');
        locale !== null && locale !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_intl_locale_language() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let locale = new Intl.Locale('en-US');
        locale.language === 'en';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Intl.DisplayNames Tests
// ============================================================================

#[test]
fn test_intl_display_names_constructor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let names = new Intl.DisplayNames('en-US', { type: 'language' });
        names !== null && names !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_intl_display_names_of() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let names = new Intl.DisplayNames('en-US', { type: 'language' });
        typeof names.of === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}
