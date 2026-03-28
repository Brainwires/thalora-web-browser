//! Tests for the Console API implementation

use crate::{Context, JsString, JsValue, Source};

#[test]
fn test_console_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test that console exists as an object
    let result = context.eval(Source::from_bytes("typeof console")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("object")));
}

#[test]
fn test_console_log_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test that console.log exists as a function
    let result = context
        .eval(Source::from_bytes("typeof console.log"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_log_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test that console.log can be called without error
    let result = context.eval(Source::from_bytes("console.log('Hello, World!')"));
    assert!(result.is_ok());
}

#[test]
fn test_console_log_multiple_args() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test console.log with multiple arguments
    let result = context.eval(Source::from_bytes(
        "console.log('test', 123, true, null, undefined)",
    ));
    assert!(result.is_ok());
}

#[test]
fn test_console_error_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.error"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_error_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes("console.error('Error message')"));
    assert!(result.is_ok());
}

#[test]
fn test_console_warn_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.warn"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_warn_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes("console.warn('Warning message')"));
    assert!(result.is_ok());
}

#[test]
fn test_console_info_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.info"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_info_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes("console.info('Info message')"));
    assert!(result.is_ok());
}

#[test]
fn test_console_debug_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.debug"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_debug_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes("console.debug('Debug message')"));
    assert!(result.is_ok());
}

#[test]
fn test_console_trace_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.trace"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_trace_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes("console.trace('Trace message')"));
    assert!(result.is_ok());
}

#[test]
fn test_console_assert_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.assert"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_assert_true() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Assert with true condition should not error
    let result = context.eval(Source::from_bytes(
        "console.assert(true, 'Should not see this')",
    ));
    assert!(result.is_ok());
}

#[test]
fn test_console_assert_false() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Assert with false condition should still not throw (just logs)
    let result = context.eval(Source::from_bytes(
        "console.assert(false, 'Assertion failed')",
    ));
    assert!(result.is_ok());
}

#[test]
fn test_console_clear_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.clear"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_clear_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes("console.clear()"));
    assert!(result.is_ok());
}

#[test]
fn test_console_count_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.count"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_count_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes("console.count('test')"));
    assert!(result.is_ok());
}

#[test]
fn test_console_count_reset_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.countReset"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_count_reset_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes("console.countReset('test')"));
    assert!(result.is_ok());
}

#[test]
fn test_console_group_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.group"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_group_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes("console.group('Group Name')"));
    assert!(result.is_ok());
}

#[test]
fn test_console_group_collapsed_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.groupCollapsed"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_group_collapsed_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes(
        "console.groupCollapsed('Collapsed Group')",
    ));
    assert!(result.is_ok());
}

#[test]
fn test_console_group_end_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.groupEnd"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_group_end_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes("console.groupEnd()"));
    assert!(result.is_ok());
}

#[test]
fn test_console_time_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.time"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_time_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes("console.time('timer')"));
    assert!(result.is_ok());
}

#[test]
fn test_console_time_log_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.timeLog"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_time_log_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes(
        r#"
        console.time('timer');
        console.timeLog('timer');
    "#,
    ));
    assert!(result.is_ok());
}

#[test]
fn test_console_time_end_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.timeEnd"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_time_end_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes(
        r#"
        console.time('timer');
        console.timeEnd('timer');
    "#,
    ));
    assert!(result.is_ok());
}

#[test]
fn test_console_table_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.table"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_table_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes("console.table([1, 2, 3])"));
    assert!(result.is_ok());
}

#[test]
fn test_console_dir_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.dir"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_dir_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes("console.dir({a: 1, b: 2})"));
    assert!(result.is_ok());
}

#[test]
fn test_console_dir_xml_exists() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context
        .eval(Source::from_bytes("typeof console.dirxml"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_console_dir_xml_basic() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes("console.dirxml({a: 1})"));
    assert!(result.is_ok());
}

#[test]
fn test_console_chaining() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test multiple console methods can be called in sequence
    let result = context.eval(Source::from_bytes(
        r#"
        console.log('First');
        console.warn('Second');
        console.error('Third');
        console.info('Fourth');
    "#,
    ));
    assert!(result.is_ok());
}

#[test]
fn test_console_with_objects() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes(
        r#"
        const obj = { name: 'test', value: 123 };
        console.log('Object:', obj);
    "#,
    ));
    assert!(result.is_ok());
}

#[test]
fn test_console_with_arrays() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes(
        r#"
        const arr = [1, 2, 3, 4, 5];
        console.log('Array:', arr);
    "#,
    ));
    assert!(result.is_ok());
}

#[test]
fn test_console_formatting() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test string formatting with %s, %d, %i, %f
    let result = context.eval(Source::from_bytes(
        r#"
        console.log('String: %s, Number: %d', 'test', 123);
    "#,
    ));
    assert!(result.is_ok());
}

#[test]
fn test_console_nested_groups() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes(
        r#"
        console.group('Outer');
        console.log('Message 1');
        console.group('Inner');
        console.log('Message 2');
        console.groupEnd();
        console.groupEnd();
    "#,
    ));
    assert!(result.is_ok());
}

#[test]
fn test_console_multiple_timers() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes(
        r#"
        console.time('timer1');
        console.time('timer2');
        console.timeEnd('timer1');
        console.timeEnd('timer2');
    "#,
    ));
    assert!(result.is_ok());
}

#[test]
fn test_console_count_increments() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    let result = context.eval(Source::from_bytes(
        r#"
        console.count('counter');
        console.count('counter');
        console.count('counter');
        console.countReset('counter');
        console.count('counter');
    "#,
    ));
    assert!(result.is_ok());
}
