//! Tests for Worker JavaScript API (Phase 3)
//! Tests the Worker constructor and methods as they would be used from JavaScript

#[cfg(test)]
mod worker_api_tests {
    use crate::worker::worker::register_worker_api;
    use boa_engine::{Context, Source, JsValue, js_string};
    use std::time::Duration;
    use std::thread;

    /// Helper to set up a context with Worker API registered
    fn setup_context() -> Context {
        let mut context = Context::default();
        register_worker_api(&mut context).expect("Failed to register Worker API");
        context
    }

    /// Test that Worker constructor is available globally
    #[test]
    fn test_worker_constructor_exists() {
        let mut context = setup_context();

        let result = context.eval(Source::from_bytes("typeof Worker"));
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Some(s) = value.as_string() {
            assert_eq!(s.to_std_string_escaped(), "function");
        } else {
            panic!("Worker should be a function");
        }
    }

    /// Test Worker constructor requires a URL
    #[test]
    fn test_worker_constructor_requires_url() {
        let mut context = setup_context();

        let result = context.eval(Source::from_bytes("new Worker()"));
        assert!(result.is_err(), "Worker constructor should require a URL");
    }

    /// Test Worker constructor with simple script URL
    #[test]
    fn test_worker_constructor_with_url() {
        let mut context = setup_context();

        // Use a data URL for testing
        let script = r#"
            var worker = new Worker('data:text/javascript,console.log("worker")');
            typeof worker;
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok(), "Worker constructor should accept URL");

        let value = result.unwrap();
        if let Some(s) = value.as_string() {
            assert_eq!(s.to_std_string_escaped(), "object");
        }
    }

    /// Test Worker has postMessage method
    #[test]
    fn test_worker_has_post_message() {
        let mut context = setup_context();

        let script = r#"
            var worker = new Worker('data:text/javascript,self.onmessage=()=>{}');
            typeof worker.postMessage;
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Some(s) = value.as_string() {
            assert_eq!(s.to_std_string_escaped(), "function");
        }
    }

    /// Test Worker has terminate method
    #[test]
    fn test_worker_has_terminate() {
        let mut context = setup_context();

        let script = r#"
            var worker = new Worker('data:text/javascript,');
            typeof worker.terminate;
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Some(s) = value.as_string() {
            assert_eq!(s.to_std_string_escaped(), "function");
        }
    }

    /// Test Worker onmessage property
    #[test]
    fn test_worker_onmessage_property() {
        let mut context = setup_context();

        let script = r#"
            var worker = new Worker('data:text/javascript,');
            worker.onmessage === null;
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.as_boolean().unwrap_or(false), "onmessage should be null initially");
    }

    /// Test setting Worker onmessage handler
    #[test]
    fn test_worker_set_onmessage() {
        let mut context = setup_context();

        let script = r#"
            var worker = new Worker('data:text/javascript,');
            worker.onmessage = function(e) { console.log(e); };
            typeof worker.onmessage;
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Some(s) = value.as_string() {
            assert_eq!(s.to_std_string_escaped(), "function");
        }
    }

    /// Test Worker onerror property
    #[test]
    fn test_worker_onerror_property() {
        let mut context = setup_context();

        let script = r#"
            var worker = new Worker('data:text/javascript,');
            worker.onerror === null;
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.as_boolean().unwrap_or(false), "onerror should be null initially");
    }

    /// Test setting Worker onerror handler
    #[test]
    fn test_worker_set_onerror() {
        let mut context = setup_context();

        let script = r#"
            var worker = new Worker('data:text/javascript,');
            worker.onerror = function(e) { console.error(e); };
            typeof worker.onerror;
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Some(s) = value.as_string() {
            assert_eq!(s.to_std_string_escaped(), "function");
        }
    }

    /// Test Worker.postMessage can be called
    #[test]
    fn test_worker_post_message_callable() {
        let mut context = setup_context();

        let script = r#"
            var worker = new Worker('data:text/javascript,self.onmessage=()=>{}');
            try {
                worker.postMessage("test");
                true;
            } catch (e) {
                false;
            }
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.as_boolean().unwrap_or(false), "postMessage should be callable");
    }

    /// Test Worker.terminate can be called
    #[test]
    fn test_worker_terminate_callable() {
        let mut context = setup_context();

        let script = r#"
            var worker = new Worker('data:text/javascript,');
            try {
                worker.terminate();
                true;
            } catch (e) {
                false;
            }
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.as_boolean().unwrap_or(false), "terminate should be callable");
    }

    /// Test Worker constructor with options
    #[test]
    fn test_worker_constructor_with_options() {
        let mut context = setup_context();

        let script = r#"
            var worker = new Worker('data:text/javascript,', {
                name: 'test-worker',
                type: 'classic'
            });
            typeof worker;
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok(), "Worker constructor should accept options");

        let value = result.unwrap();
        if let Some(s) = value.as_string() {
            assert_eq!(s.to_std_string_escaped(), "object");
        }
    }

    /// Test Worker constructor with module type
    #[test]
    fn test_worker_constructor_module_type() {
        let mut context = setup_context();

        let script = r#"
            var worker = new Worker('data:text/javascript,', { type: 'module' });
            typeof worker;
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok(), "Worker constructor should accept module type");

        let value = result.unwrap();
        if let Some(s) = value.as_string() {
            assert_eq!(s.to_std_string_escaped(), "object");
        }
    }

    /// Test multiple Worker instances can be created
    #[test]
    fn test_multiple_workers() {
        let mut context = setup_context();

        let script = r#"
            var worker1 = new Worker('data:text/javascript,');
            var worker2 = new Worker('data:text/javascript,');
            worker1 !== worker2;
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.as_boolean().unwrap_or(false), "Each Worker should be a unique instance");
    }

    /// Test Worker postMessage with different data types
    #[test]
    fn test_worker_post_message_types() {
        let mut context = setup_context();

        let script = r#"
            var worker = new Worker('data:text/javascript,self.onmessage=()=>{}');
            var success = true;

            try {
                worker.postMessage("string");
                worker.postMessage(123);
                worker.postMessage(true);
                worker.postMessage({key: "value"});
                worker.postMessage([1, 2, 3]);
            } catch (e) {
                success = false;
            }

            success;
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.as_boolean().unwrap_or(false), "postMessage should accept various data types");
    }

    /// Test Worker instance properties are enumerable
    #[test]
    fn test_worker_properties_enumerable() {
        let mut context = setup_context();

        let script = r#"
            var worker = new Worker('data:text/javascript,');
            var props = [];
            for (var key in worker) {
                props.push(key);
            }
            props.includes('onmessage') && props.includes('onerror');
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.as_boolean().unwrap_or(false), "Event handler properties should be enumerable");
    }

    /// Test Worker methods are on prototype
    #[test]
    fn test_worker_methods_on_prototype() {
        let mut context = setup_context();

        let script = r#"
            var worker = new Worker('data:text/javascript,');
            worker.hasOwnProperty('postMessage') === false &&
            worker.hasOwnProperty('terminate') === false;
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.as_boolean().unwrap_or(false), "Methods should be on prototype, not own properties");
    }

    /// Test setting onmessage to null
    #[test]
    fn test_worker_onmessage_null() {
        let mut context = setup_context();

        let script = r#"
            var worker = new Worker('data:text/javascript,');
            worker.onmessage = function() {};
            worker.onmessage = null;
            worker.onmessage === null;
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.as_boolean().unwrap_or(false), "onmessage should be settable to null");
    }

    /// Test setting onerror to null
    #[test]
    fn test_worker_onerror_null() {
        let mut context = setup_context();

        let script = r#"
            var worker = new Worker('data:text/javascript,');
            worker.onerror = function() {};
            worker.onerror = null;
            worker.onerror === null;
        "#;

        let result = context.eval(Source::from_bytes(script));
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.as_boolean().unwrap_or(false), "onerror should be settable to null");
    }
}
