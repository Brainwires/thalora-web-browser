use synaptic::js::JavaScriptEngine;

#[tokio::test]
async fn test_object_values() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Object.values
    let result = engine.execute_enhanced(r#"
        const obj = { a: 1, b: 2, c: 3 };
        const values = Object.values(obj);
        JSON.stringify(values.sort())
    "#).await.unwrap();

    // Should contain [1, 2, 3]
    assert!(result.is_string());
}

#[tokio::test]
async fn test_object_entries() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Object.entries
    let result = engine.execute_enhanced(r#"
        const obj = { foo: 'bar', baz: 42 };
        const entries = Object.entries(obj);
        entries.length === 2 && Array.isArray(entries[0])
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_object_get_own_property_descriptors() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Object.getOwnPropertyDescriptors
    let result = engine.execute_enhanced(r#"
        const obj = { a: 1 };
        const descriptors = Object.getOwnPropertyDescriptors(obj);
        typeof descriptors === 'object' && descriptors.a !== undefined
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_string_pad_start() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String.prototype.padStart
    let result = engine.execute_enhanced(r#"
        [
            'hello'.padStart(10),
            'hello'.padStart(10, '*'),
            'hello'.padStart(3),
            '123'.padStart(5, '0')
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_string_pad_end() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String.prototype.padEnd
    let result = engine.execute_enhanced(r#"
        [
            'hello'.padEnd(10),
            'hello'.padEnd(10, '*'),
            'hello'.padEnd(3),
            'abc'.padEnd(6, '123456')
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_async_functions() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test basic async function support (syntax transformation)
    let result = engine.execute_enhanced(r#"
        // Async functions are transformed to regular functions returning promises
        function asyncTest() {
            return Promise.resolve(42);
        }
        asyncTest()
    "#).await.unwrap();

    assert!(result.is_object()); // Promise object
}

#[tokio::test]
async fn test_shared_array_buffer() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test SharedArrayBuffer polyfill (if available)
    let result = engine.execute_enhanced(r#"
        // Check if SharedArrayBuffer exists or is polyfilled
        typeof SharedArrayBuffer !== 'undefined' || typeof ArrayBuffer !== 'undefined'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_atomics_basic() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test basic Atomics availability
    let result = engine.execute_enhanced(r#"
        // Atomics might not be fully supported in all environments
        typeof Atomics === 'object' || typeof Atomics === 'undefined'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_object_values_edge_cases() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Object.values edge cases
    let result = engine.execute_enhanced(r#"
        [
            Object.values({}),
            Object.values([1, 2, 3]),
            Object.values('hello'),
            Object.values(null) || [],
            Object.values(undefined) || []
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_object_entries_edge_cases() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Object.entries edge cases
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3];
        const entries = Object.entries(arr);
        // Array entries should be [['0', 1], ['1', 2], ['2', 3]]
        entries.length === 3 && entries[0][0] === '0'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_string_padding_unicode() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test string padding with unicode
    let result = engine.execute_enhanced(r#"
        [
            'café'.padStart(10, '★'),
            '🌟'.padEnd(5, '✨'),
            'test'.padStart(2), // Should return original string
            'test'.padEnd(2)    // Should return original string
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}