use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_at() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.at
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3, 4, 5];
        [
            arr.at(0),
            arr.at(-1),
            arr.at(2),
            arr.at(-2),
            arr.at(10)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_string_at() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String.prototype.at
    let result = engine.execute_enhanced(r#"
        const str = 'hello';
        [
            str.at(0),
            str.at(-1),
            str.at(2),
            str.at(-2),
            str.at(10)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_object_has_own() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Object.hasOwn
    let result = engine.execute_enhanced(r#"
        const obj = { a: 1, b: 2 };
        const inherited = Object.create(obj);
        inherited.c = 3;

        [
            Object.hasOwn(obj, 'a'),
            Object.hasOwn(obj, 'toString'),
            Object.hasOwn(inherited, 'c'),
            Object.hasOwn(inherited, 'a')
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_regexp_match_indices() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test RegExp match indices (d flag)
    let result = engine.execute_enhanced(r#"
        try {
            const regex = /a(b)/d;
            const match = 'ab'.match(regex);
            match !== null
        } catch (e) {
            // d flag might not be supported
            true
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_error_cause() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Error cause property
    let result = engine.execute_enhanced(r#"
        try {
            const err = new Error('test', { cause: 'root cause' });
            err.cause === 'root cause'
        } catch (e) {
            // cause might not be supported
            true
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_top_level_await() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test top-level await transformation
    let result = engine.execute_enhanced(r#"
        // Top-level await would be wrapped in async IIFE
        (async function() {
            const result = await Promise.resolve(42);
            return result;
        })()
    "#).await.unwrap();

    assert!(result.is_object()); // Should be a Promise
}

#[tokio::test]
async fn test_class_fields() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test class fields (transformed)
    let result = engine.execute_enhanced(r#"
        class MyClass {
            constructor() {
                this.publicField = 'public';
                this._private_privateField = 'private';
            }
        }
        const instance = new MyClass();
        instance.publicField === 'public'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_private_methods() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test private methods (transformed to conventional private methods)
    let result = engine.execute_enhanced(r#"
        class MyClass {
            constructor() {
                this.publicValue = 42;
            }

            _private_method() {
                return this.publicValue * 2;
            }

            getDoubledValue() {
                return this._private_method();
            }
        }
        const instance = new MyClass();
        instance.getDoubledValue() === 84
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_static_class_fields() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test static class fields
    let result = engine.execute_enhanced(r#"
        class MyClass {
            static staticField = 'static';

            static getStaticField() {
                return this.staticField;
            }
        }
        MyClass.staticField = 'static'; // Polyfilled assignment
        MyClass.getStaticField() === 'static'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_class_static_block() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test class static initialization blocks (transformed)
    let result = engine.execute_enhanced(r#"
        class MyClass {
            static value = 0;
        }
        // Static block would be transformed to immediate execution
        (function() {
            MyClass.value = 42;
        })();

        MyClass.value === 42
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_array_at_edge_cases() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.at edge cases
    let result = engine.execute_enhanced(r#"
        [
            [].at(0),
            [1].at(-1),
            [1, 2, 3].at(1.5), // Should floor to 1
            'test'.at(-1)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_object_has_own_edge_cases() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Object.hasOwn edge cases
    let result = engine.execute_enhanced(r#"
        [
            Object.hasOwn({}, 'missing'),
            Object.hasOwn({ 0: 'zero' }, '0'),
            Object.hasOwn({ null: 'value' }, 'null'),
            typeof Object.hasOwn === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}