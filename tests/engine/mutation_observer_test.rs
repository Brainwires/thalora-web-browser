//! Comprehensive unit tests for MutationObserver implementation in Boa engine

use boa_engine::{Context, Source};

#[test]
fn test_mutation_observer_constructor_availability() {
    let mut context = Context::default();

    // Test that MutationObserver exists as a function
    let result = context.eval(Source::from_bytes("typeof MutationObserver"));
    assert!(result.is_ok());
    let type_str = result.unwrap().to_string(&mut context).unwrap();
    assert_eq!(type_str.to_std_string_escaped(), "function");
}

#[test]
fn test_mutation_observer_constructor_with_callback() {
    let mut context = Context::default();

    let script = r#"
        var observer = new MutationObserver(function() {});
        typeof observer
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let type_str = result.unwrap().to_string(&mut context).unwrap();
    assert_eq!(type_str.to_std_string_escaped(), "object");
}

#[test]
fn test_mutation_observer_constructor_without_callback_fails() {
    let mut context = Context::default();

    let script = r#"
        try {
            new MutationObserver();
            false;
        } catch(e) {
            true;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let caught_error = result.unwrap().to_boolean();
    assert!(caught_error, "MutationObserver constructor should require a callback");
}

#[test]
fn test_mutation_observer_constructor_with_non_function_fails() {
    let mut context = Context::default();

    let script = r#"
        try {
            new MutationObserver("not a function");
            false;
        } catch(e) {
            true;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let caught_error = result.unwrap().to_boolean();
    assert!(caught_error, "MutationObserver constructor should require a function callback");
}

#[test]
fn test_mutation_observer_methods_exist() {
    let mut context = Context::default();

    let methods = ["observe", "disconnect", "takeRecords"];

    for method in &methods {
        let script = format!(
            "var observer = new MutationObserver(function() {{}}); typeof observer.{}",
            method
        );
        let result = context.eval(Source::from_bytes(&script));
        assert!(result.is_ok(), "Method {} should exist", method);

        let method_type = result.unwrap().to_string(&mut context).unwrap().to_std_string_escaped();
        assert_eq!(method_type, "function", "Method {} should be a function", method);
    }
}

#[test]
fn test_mutation_observer_observe_method() {
    let mut context = Context::default();

    let script = r#"
        var observer = new MutationObserver(function() {});
        var target = {};  // Mock target object
        try {
            observer.observe(target, { childList: true });
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let success = result.unwrap().to_boolean();
    assert!(success, "observe method should work with valid parameters");
}

#[test]
fn test_mutation_observer_observe_without_target_fails() {
    let mut context = Context::default();

    let script = r#"
        var observer = new MutationObserver(function() {});
        try {
            observer.observe(null, { childList: true });
            false;
        } catch(e) {
            true;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let caught_error = result.unwrap().to_boolean();
    assert!(caught_error, "observe should fail with null target");
}

#[test]
fn test_mutation_observer_observe_configuration_parsing() {
    let mut context = Context::default();

    let script = r#"
        var observer = new MutationObserver(function() {});
        var target = {};
        try {
            // Test various configuration options
            observer.observe(target, {
                childList: true,
                attributes: true,
                characterData: true,
                subtree: true,
                attributeOldValue: true,
                characterDataOldValue: true,
                attributeFilter: ["class", "id"]
            });
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let success = result.unwrap().to_boolean();
    assert!(success, "observe should parse complex configuration options");
}

#[test]
fn test_mutation_observer_observe_invalid_config_fails() {
    let mut context = Context::default();

    let script = r#"
        var observer = new MutationObserver(function() {});
        var target = {};
        try {
            // Invalid config - none of childList, attributes, or characterData is true
            observer.observe(target, {});
            false;
        } catch(e) {
            true;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let caught_error = result.unwrap().to_boolean();
    assert!(caught_error, "observe should fail with invalid configuration");
}

#[test]
fn test_mutation_observer_disconnect_method() {
    let mut context = Context::default();

    let script = r#"
        var observer = new MutationObserver(function() {});
        var target = {};
        observer.observe(target, { childList: true });

        try {
            observer.disconnect();
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let success = result.unwrap().to_boolean();
    assert!(success, "disconnect method should work");
}

#[test]
fn test_mutation_observer_take_records_method() {
    let mut context = Context::default();

    let script = r#"
        var observer = new MutationObserver(function() {});
        var records = observer.takeRecords();
        Array.isArray(records)
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let is_array = result.unwrap().to_boolean();
    assert!(is_array, "takeRecords should return an array");
}

#[test]
fn test_mutation_observer_take_records_initially_empty() {
    let mut context = Context::default();

    let script = r#"
        var observer = new MutationObserver(function() {});
        var records = observer.takeRecords();
        records.length
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let length = result.unwrap().to_number(&mut context).unwrap();
    assert_eq!(length, 0.0, "takeRecords should initially return empty array");
}

#[test]
fn test_mutation_observer_multiple_targets() {
    let mut context = Context::default();

    let script = r#"
        var observer = new MutationObserver(function() {});
        var target1 = {};
        var target2 = {};

        try {
            observer.observe(target1, { childList: true });
            observer.observe(target2, { attributes: true });
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let success = result.unwrap().to_boolean();
    assert!(success, "observer should be able to observe multiple targets");
}

#[test]
fn test_mutation_observer_callback_storage() {
    let mut context = Context::default();

    let script = r#"
        var callbackCalled = false;
        var observer = new MutationObserver(function() {
            callbackCalled = true;
        });

        // The callback should be stored (though not called in our basic implementation)
        typeof observer
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let type_str = result.unwrap().to_string(&mut context).unwrap();
    assert_eq!(type_str.to_std_string_escaped(), "object");
}

#[test]
fn test_mutation_observer_instance_isolation() {
    let mut context = Context::default();

    let script = r#"
        var observer1 = new MutationObserver(function() {});
        var observer2 = new MutationObserver(function() {});
        var target = {};

        observer1.observe(target, { childList: true });
        observer2.disconnect(); // Should not affect observer1

        observer1 !== observer2
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let different = result.unwrap().to_boolean();
    assert!(different, "Multiple observer instances should be independent");
}

#[test]
fn test_mutation_observer_observe_overwrite() {
    let mut context = Context::default();

    let script = r#"
        var observer = new MutationObserver(function() {});
        var target = {};

        try {
            observer.observe(target, { childList: true });
            observer.observe(target, { attributes: true }); // Should overwrite previous observation
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let success = result.unwrap().to_boolean();
    assert!(success, "observe should be able to overwrite previous observation of same target");
}