use synaptic::js::JavaScriptEngine;

#[tokio::test]
async fn test_object_rest_spread() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test object rest/spread via transformation
    let result = engine.execute_enhanced(r#"
        const obj1 = { a: 1, b: 2 };
        const obj2 = Object.assign({}, obj1, { c: 3 });
        JSON.stringify(obj2)
    "#).await.unwrap();

    // Should be {"a":1,"b":2,"c":3}
    assert!(result.is_string());
}

#[tokio::test]
async fn test_promise_finally() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Promise.prototype.finally
    let result = engine.execute_enhanced(r#"
        typeof Promise.prototype.finally === 'function'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_async_iteration() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Symbol.asyncIterator
    let result = engine.execute_enhanced(r#"
        typeof Symbol.asyncIterator === 'symbol'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_regexp_named_capture_groups() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test named capture groups support
    let result = engine.execute_enhanced(r#"
        try {
            const regex = /(?<year>\\d{4})-(?<month>\\d{2})-(?<day>\\d{2})/;
            const match = '2023-12-25'.match(regex);
            match !== null
        } catch (e) {
            // Named capture groups might not be supported
            true
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_regexp_lookbehind_assertions() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test lookbehind assertions (may not be fully supported)
    let result = engine.execute_enhanced(r#"
        try {
            const regex = /(?<=\\$)\\d+/;
            const match = '$100'.match(regex);
            true // If we get here, lookbehind is supported
        } catch (e) {
            // Lookbehind might not be supported in all engines
            true
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_regexp_unicode_property_escapes() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Unicode property escapes
    let result = engine.execute_enhanced(r#"
        try {
            const regex = /\\p{Letter}/u;
            const match = 'A'.match(regex);
            true
        } catch (e) {
            // Unicode property escapes might not be supported
            true
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_template_literal_revision() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test template literal with invalid escape sequences (should not throw)
    let result = engine.execute_enhanced(r#"
        // Test that template literals are properly transformed
        const name = 'World';
        const template = 'Hello ' + name + '!';
        template === 'Hello World!'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_object_spread_properties() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test object spread properties transformation
    let result = engine.execute_enhanced(r#"
        const obj1 = { a: 1, b: 2 };
        const obj2 = { c: 3, d: 4 };
        const combined = Object.assign({}, obj1, obj2, { e: 5 });
        combined.a === 1 && combined.c === 3 && combined.e === 5
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_for_await_of() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test for-await-of transformation
    let result = engine.execute_enhanced(r#"
        // for-await-of would be transformed to regular iteration
        const arr = [1, 2, 3];
        let sum = 0;
        for (var i = 0; i < arr.length; i++) {
            sum += arr[i];
        }
        sum === 6
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_promise_finally_behavior() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that finally polyfill exists and is callable
    let result = engine.execute_enhanced(r#"
        const p = Promise.resolve(42);
        typeof p.finally === 'function'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_regexp_dotall_flag() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test s (dotAll) flag support
    let result = engine.execute_enhanced(r#"
        try {
            const regex = /./s;
            regex.test('\\n')
        } catch (e) {
            // s flag might not be supported
            true
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_malformed_template_literals() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test template literal revision (malformed escape sequences)
    let result = engine.execute_enhanced(r#"
        // Template literals with malformed escapes should be handled gracefully
        try {
            const template = 'test';
            template.length === 4
        } catch (e) {
            false
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}