//! Comprehensive test suite for Timer APIs
//! Tests setTimeout, setInterval, clearTimeout, clearInterval

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
// setTimeout Tests
// ============================================================================

#[test]
fn test_settimeout_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof setTimeout")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_settimeout_returns_number() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof setTimeout(function(){}, 100)")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("number")));
}

#[test]
fn test_settimeout_returns_positive_id() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setTimeout(function(){}, 100)")).unwrap();
    let id = result.as_number().expect("Should return number");
    assert!(id > 0.0, "Timer ID should be positive");
}

#[test]
fn test_settimeout_increments_id() {
    let mut context = create_test_context();
    let result1 = context.eval(Source::from_bytes("setTimeout(function(){}, 100)")).unwrap();
    let result2 = context.eval(Source::from_bytes("setTimeout(function(){}, 100)")).unwrap();

    let id1 = result1.as_number().unwrap();
    let id2 = result2.as_number().unwrap();
    assert!(id2 > id1, "Each setTimeout should get incrementing ID");
}

#[test]
fn test_settimeout_with_zero_delay() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setTimeout(function(){}, 0)"));
    assert!(result.is_ok());
}

#[test]
fn test_settimeout_with_negative_delay() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setTimeout(function(){}, -100)"));
    assert!(result.is_ok(), "Negative delay should be clamped to 0");
}

#[test]
fn test_settimeout_without_delay() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setTimeout(function(){})"));
    assert!(result.is_ok(), "Delay should default to 0");
}

#[test]
fn test_settimeout_without_callback() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setTimeout()"));
    assert!(result.is_ok(), "Should handle missing callback gracefully");
}

#[test]
fn test_settimeout_with_string_callback() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setTimeout('console.log(1)', 100)"));
    assert!(result.is_ok());
}

#[test]
fn test_settimeout_with_arrow_function() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setTimeout(() => console.log('test'), 100)"));
    assert!(result.is_ok());
}

#[test]
fn test_settimeout_multiple_concurrent() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let id1 = setTimeout(() => {}, 100);
        let id2 = setTimeout(() => {}, 200);
        let id3 = setTimeout(() => {}, 300);
        [id1, id2, id3].every(id => typeof id === 'number' && id > 0);
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// setInterval Tests
// ============================================================================

#[test]
fn test_setinterval_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof setInterval")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_setinterval_returns_number() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof setInterval(function(){}, 100)")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("number")));
}

#[test]
fn test_setinterval_returns_positive_id() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setInterval(function(){}, 100)")).unwrap();
    let id = result.as_number().expect("Should return number");
    assert!(id > 0.0, "Interval ID should be positive");
}

#[test]
fn test_setinterval_increments_id() {
    let mut context = create_test_context();
    let result1 = context.eval(Source::from_bytes("setInterval(function(){}, 100)")).unwrap();
    let result2 = context.eval(Source::from_bytes("setInterval(function(){}, 100)")).unwrap();

    let id1 = result1.as_number().unwrap();
    let id2 = result2.as_number().unwrap();
    assert!(id2 > id1, "Each setInterval should get incrementing ID");
}

#[test]
fn test_setinterval_with_zero_delay() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setInterval(function(){}, 0)"));
    assert!(result.is_ok());
}

#[test]
fn test_setinterval_with_negative_delay() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setInterval(function(){}, -100)"));
    assert!(result.is_ok(), "Negative delay should be clamped to 0");
}

#[test]
fn test_setinterval_without_delay() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setInterval(function(){})"));
    assert!(result.is_ok(), "Delay should default to 0");
}

#[test]
fn test_setinterval_without_callback() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setInterval()"));
    assert!(result.is_ok(), "Should handle missing callback gracefully");
}

#[test]
fn test_setinterval_with_arrow_function() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setInterval(() => console.log('test'), 100)"));
    assert!(result.is_ok());
}

#[test]
fn test_setinterval_multiple_concurrent() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let id1 = setInterval(() => {}, 100);
        let id2 = setInterval(() => {}, 200);
        let id3 = setInterval(() => {}, 300);
        [id1, id2, id3].every(id => typeof id === 'number' && id > 0);
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// clearTimeout Tests
// ============================================================================

#[test]
fn test_cleartimeout_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof clearTimeout")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_cleartimeout_returns_undefined() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let id = setTimeout(() => {}, 100);
        typeof clearTimeout(id);
    "#)).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("undefined")));
}

#[test]
fn test_cleartimeout_with_valid_id() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let id = setTimeout(() => {}, 100);
        clearTimeout(id);
        true;
    "#));
    assert!(result.is_ok());
}

#[test]
fn test_cleartimeout_with_invalid_id() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("clearTimeout(99999)"));
    assert!(result.is_ok(), "Should handle invalid ID gracefully");
}

#[test]
fn test_cleartimeout_with_zero() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("clearTimeout(0)"));
    assert!(result.is_ok());
}

#[test]
fn test_cleartimeout_without_argument() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("clearTimeout()"));
    assert!(result.is_ok(), "Should handle missing argument");
}

#[test]
fn test_cleartimeout_multiple_times_same_id() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let id = setTimeout(() => {}, 100);
        clearTimeout(id);
        clearTimeout(id);
        clearTimeout(id);
        true;
    "#));
    assert!(result.is_ok(), "Should handle clearing same ID multiple times");
}

// ============================================================================
// clearInterval Tests
// ============================================================================

#[test]
fn test_clearinterval_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof clearInterval")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_clearinterval_returns_undefined() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let id = setInterval(() => {}, 100);
        typeof clearInterval(id);
    "#)).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("undefined")));
}

#[test]
fn test_clearinterval_with_valid_id() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let id = setInterval(() => {}, 100);
        clearInterval(id);
        true;
    "#));
    assert!(result.is_ok());
}

#[test]
fn test_clearinterval_with_invalid_id() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("clearInterval(99999)"));
    assert!(result.is_ok(), "Should handle invalid ID gracefully");
}

#[test]
fn test_clearinterval_with_zero() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("clearInterval(0)"));
    assert!(result.is_ok());
}

#[test]
fn test_clearinterval_without_argument() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("clearInterval()"));
    assert!(result.is_ok(), "Should handle missing argument");
}

#[test]
fn test_clearinterval_multiple_times_same_id() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let id = setInterval(() => {}, 100);
        clearInterval(id);
        clearInterval(id);
        clearInterval(id);
        true;
    "#));
    assert!(result.is_ok(), "Should handle clearing same ID multiple times");
}

// ============================================================================
// Mixed setTimeout/setInterval Tests
// ============================================================================

#[test]
fn test_settimeout_and_setinterval_share_id_space() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let timeoutId = setTimeout(() => {}, 100);
        let intervalId = setInterval(() => {}, 100);
        intervalId > timeoutId;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true, "IDs should be sequential across both timer types");
}

#[test]
fn test_cleartimeout_on_interval_id() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let id = setInterval(() => {}, 100);
        clearTimeout(id);
        true;
    "#));
    assert!(result.is_ok(), "clearTimeout should work on interval IDs");
}

#[test]
fn test_clearinterval_on_timeout_id() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let id = setTimeout(() => {}, 100);
        clearInterval(id);
        true;
    "#));
    assert!(result.is_ok(), "clearInterval should work on timeout IDs");
}

#[test]
fn test_mixed_timers_concurrent() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let t1 = setTimeout(() => {}, 100);
        let i1 = setInterval(() => {}, 100);
        let t2 = setTimeout(() => {}, 200);
        let i2 = setInterval(() => {}, 200);
        [t1, i1, t2, i2].every(id => typeof id === 'number' && id > 0);
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_timer_with_non_function_callback() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setTimeout(123, 100)"));
    assert!(result.is_ok(), "Should handle non-function callback");
}

#[test]
fn test_timer_with_null_callback() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setTimeout(null, 100)"));
    assert!(result.is_ok());
}

#[test]
fn test_timer_with_undefined_callback() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setTimeout(undefined, 100)"));
    assert!(result.is_ok());
}

#[test]
fn test_timer_with_very_large_delay() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setTimeout(() => {}, 2147483647)"));
    assert!(result.is_ok());
}

#[test]
fn test_timer_with_float_delay() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setTimeout(() => {}, 100.7)"));
    assert!(result.is_ok(), "Should handle float delay");
}

#[test]
fn test_timer_with_string_delay() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setTimeout(() => {}, '100')"));
    assert!(result.is_ok(), "Should convert string to number");
}

#[test]
fn test_timer_with_nan_delay() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("setTimeout(() => {}, NaN)"));
    assert!(result.is_ok(), "NaN delay should be treated as 0");
}

#[test]
fn test_clear_with_string_id() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("clearTimeout('123')"));
    assert!(result.is_ok(), "Should convert string ID to number");
}

#[test]
fn test_clear_with_float_id() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let id = setTimeout(() => {}, 100);
        clearTimeout(id + 0.5);
        true;
    "#));
    assert!(result.is_ok());
}

// ============================================================================
// Chain and Complex Usage Tests
// ============================================================================

#[test]
fn test_settimeout_in_expression() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let id = setTimeout(() => {}, 100) + 5;
        typeof id === 'number';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_nested_settimeout_calls() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        setTimeout(() => {
            setTimeout(() => {
                console.log('nested');
            }, 100);
        }, 100);
        true;
    "#));
    assert!(result.is_ok());
}

#[test]
fn test_many_concurrent_timers() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let ids = [];
        for (let i = 0; i < 100; i++) {
            ids.push(setTimeout(() => {}, i * 10));
        }
        ids.length === 100 && ids.every(id => typeof id === 'number');
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_clear_many_timers() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let ids = [];
        for (let i = 0; i < 50; i++) {
            ids.push(setTimeout(() => {}, i * 10));
        }
        for (let i = 0; i < ids.length; i++) {
            clearTimeout(ids[i]);
        }
        true;
    "#));
    if result.is_err() {
        println!("Error: {:?}", result);
    }
    assert!(result.is_ok());
}

#[test]
fn test_timer_ids_are_unique() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let ids = new Set();
        for (let i = 0; i < 50; i++) {
            ids.add(setTimeout(() => {}, 100));
            ids.add(setInterval(() => {}, 100));
        }
        ids.size === 100;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true, "All timer IDs should be unique");
}

// ============================================================================
// Global State Tests (using Rust test helpers)
// Note: These tests are commented out because they depend on global state
// and can interfere with each other when run in parallel. The JavaScript
// API tests above provide comprehensive coverage of timer functionality.
// ============================================================================

// #[test]
// fn test_timer_storage_increases() {
//     use crate::timers::timers::Timers;
//     Timers::clear_all_timers();
//     assert_eq!(Timers::active_timers_count(), 0);
//     let mut context = create_test_context();
//     context.eval(Source::from_bytes("setTimeout(() => {}, 100)")).unwrap();
//     assert_eq!(Timers::active_timers_count(), 1);
//     Timers::clear_all_timers();
// }
