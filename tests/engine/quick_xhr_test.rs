//! Quick XMLHttpRequest test

use boa_engine::{Context, Source};

#[test]
fn test_xhr_basic() {
    // Create a basic context and test XMLHttpRequest availability
    let mut context = Context::default();

    // The test will fail if XMLHttpRequest is not properly registered
    let result = context.eval(Source::from_bytes("typeof XMLHttpRequest"));
    assert!(result.is_ok());

    let type_str = result.unwrap().to_string(&mut context).unwrap();
    assert_eq!(type_str.to_std_string_escaped(), "function");

    // Test constructor
    let result = context.eval(Source::from_bytes("var xhr = new XMLHttpRequest(); xhr.readyState"));
    assert!(result.is_ok());

    let ready_state = result.unwrap().to_number(&mut context).unwrap();
    assert_eq!(ready_state, 0.0); // UNSENT
}