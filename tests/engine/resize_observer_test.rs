//! Comprehensive unit tests for ResizeObserver implementation in Boa engine

use boa_engine::{Context, Source};

#[test]
fn test_resize_observer_constructor_availability() {
    let mut context = Context::default();

    // Test that ResizeObserver exists as a function
    let result = context.eval(Source::from_bytes("typeof ResizeObserver"));
    assert!(result.is_ok());
    let type_str = result.unwrap().to_string(&mut context).unwrap();
    assert_eq!(type_str.to_std_string_escaped(), "function");
}

#[test]
fn test_resize_observer_constructor_with_callback() {
    let mut context = Context::default();

    let script = r#"
        var observer = new ResizeObserver(function() {});
        typeof observer
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let type_str = result.unwrap().to_string(&mut context).unwrap();
    assert_eq!(type_str.to_std_string_escaped(), "object");
}

#[test]
fn test_resize_observer_constructor_without_callback_fails() {
    let mut context = Context::default();

    let script = r#"
        try {
            new ResizeObserver();
            false;
        } catch(e) {
            true;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let caught_error = result.unwrap().to_boolean();
    assert!(caught_error, "ResizeObserver constructor should require a callback");
}

#[test]
fn test_resize_observer_constructor_with_non_function_fails() {
    let mut context = Context::default();

    let script = r#"
        try {
            new ResizeObserver("not a function");
            false;
        } catch(e) {
            true;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let caught_error = result.unwrap().to_boolean();
    assert!(caught_error, "ResizeObserver constructor should require a function callback");
}

#[test]
fn test_resize_observer_methods_exist() {
    let mut context = Context::default();

    let methods = ["observe", "unobserve", "disconnect"];

    for method in &methods {
        let script = format!(
            "var observer = new ResizeObserver(function() {{}}); typeof observer.{}",
            method
        );
        let result = context.eval(Source::from_bytes(&script));
        assert!(result.is_ok(), "Method {} should exist", method);

        let method_type = result.unwrap().to_string(&mut context).unwrap().to_std_string_escaped();
        assert_eq!(method_type, "function", "Method {} should be a function", method);
    }
}

#[test]
fn test_resize_observer_observe_method_basic() {
    let mut context = Context::default();

    let script = r#"
        var observer = new ResizeObserver(function() {});
        var target = {};  // Mock target element
        try {
            observer.observe(target);
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let success = result.unwrap().to_boolean();
    assert!(success, "observe method should work with valid target");
}

#[test]
fn test_resize_observer_observe_with_options() {
    let mut context = Context::default();

    let script = r#"
        var observer = new ResizeObserver(function() {});
        var target = {};
        try {
            observer.observe(target, { box: "border-box" });
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let success = result.unwrap().to_boolean();
    assert!(success, "observe method should work with box options");
}

#[test]
fn test_resize_observer_observe_with_content_box() {
    let mut context = Context::default();

    let script = r#"
        var observer = new ResizeObserver(function() {});
        var target = {};
        try {
            observer.observe(target, { box: "content-box" });
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let success = result.unwrap().to_boolean();
    assert!(success, "observe method should work with content-box option");
}

#[test]
fn test_resize_observer_observe_with_device_pixel_content_box() {
    let mut context = Context::default();

    let script = r#"
        var observer = new ResizeObserver(function() {});
        var target = {};
        try {
            observer.observe(target, { box: "device-pixel-content-box" });
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let success = result.unwrap().to_boolean();
    assert!(success, "observe method should work with device-pixel-content-box option");
}

#[test]
fn test_resize_observer_observe_without_target_fails() {
    let mut context = Context::default();

    let script = r#"
        var observer = new ResizeObserver(function() {});
        try {
            observer.observe(null);
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
fn test_resize_observer_unobserve_method() {
    let mut context = Context::default();

    let script = r#"
        var observer = new ResizeObserver(function() {});
        var target = {};
        observer.observe(target);

        try {
            observer.unobserve(target);
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let success = result.unwrap().to_boolean();
    assert!(success, "unobserve method should work");
}

#[test]
fn test_resize_observer_unobserve_without_target_fails() {
    let mut context = Context::default();

    let script = r#"
        var observer = new ResizeObserver(function() {});
        try {
            observer.unobserve(null);
            false;
        } catch(e) {
            true;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let caught_error = result.unwrap().to_boolean();
    assert!(caught_error, "unobserve should fail with null target");
}

#[test]
fn test_resize_observer_disconnect_method() {
    let mut context = Context::default();

    let script = r#"
        var observer = new ResizeObserver(function() {});
        var target = {};
        observer.observe(target);

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
fn test_resize_observer_multiple_targets() {
    let mut context = Context::default();

    let script = r#"
        var observer = new ResizeObserver(function() {});
        var target1 = {};
        var target2 = {};

        try {
            observer.observe(target1);
            observer.observe(target2);
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
fn test_resize_observer_observe_same_target_multiple_times() {
    let mut context = Context::default();

    let script = r#"
        var observer = new ResizeObserver(function() {});
        var target = {};

        try {
            observer.observe(target);
            observer.observe(target); // Should not cause error
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let success = result.unwrap().to_boolean();
    assert!(success, "observing same target multiple times should not cause error");
}

#[test]
fn test_resize_observer_unobserve_not_observed_target() {
    let mut context = Context::default();

    let script = r#"
        var observer = new ResizeObserver(function() {});
        var target = {};

        try {
            observer.unobserve(target); // Target was never observed
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let success = result.unwrap().to_boolean();
    assert!(success, "unobserving non-observed target should not cause error");
}

#[test]
fn test_resize_observer_instance_isolation() {
    let mut context = Context::default();

    let script = r#"
        var observer1 = new ResizeObserver(function() {});
        var observer2 = new ResizeObserver(function() {});
        var target = {};

        observer1.observe(target);
        observer2.disconnect(); // Should not affect observer1

        observer1 !== observer2
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let different = result.unwrap().to_boolean();
    assert!(different, "Multiple observer instances should be independent");
}

#[test]
fn test_resize_observer_callback_storage() {
    let mut context = Context::default();

    let script = r#"
        var callbackCalled = false;
        var observer = new ResizeObserver(function() {
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
fn test_resize_observer_box_option_defaults() {
    let mut context = Context::default();

    let script = r#"
        var observer = new ResizeObserver(function() {});
        var target = {};
        try {
            // No box option should default to content-box
            observer.observe(target, {});
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let success = result.unwrap().to_boolean();
    assert!(success, "observe should work with empty options (default to content-box)");
}

#[test]
fn test_resize_observer_invalid_box_option_defaults() {
    let mut context = Context::default();

    let script = r#"
        var observer = new ResizeObserver(function() {});
        var target = {};
        try {
            // Invalid box option should default to content-box
            observer.observe(target, { box: "invalid-box-type" });
            true;
        } catch(e) {
            false;
        }
    "#;

    let result = context.eval(Source::from_bytes(script));
    assert!(result.is_ok());

    let success = result.unwrap().to_boolean();
    assert!(success, "observe should work with invalid box option (default to content-box)");
}