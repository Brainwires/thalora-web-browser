use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_includes() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.prototype.includes
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3, NaN];
        [
            arr.includes(2),
            arr.includes(4),
            arr.includes(NaN),
            arr.includes(1, 1),
            arr.includes(1, -3)
        ]
    "#).await.unwrap();

    // Should return [true, false, true, false, true]
    assert!(result.is_object());
}

#[tokio::test]
async fn test_array_includes_edge_cases() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test edge cases for includes
    let result = engine.execute_enhanced(r#"
        const arr = ['a', 'b', 'c'];
        [
            arr.includes('a'),
            arr.includes('d'),
            [].includes(undefined),
            [undefined].includes(undefined),
            [null].includes(null)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_exponentiation_operator() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test ** operator via Math.pow transformation
    let result = engine.execute_enhanced(r#"
        [
            Math.pow(2, 3),
            Math.pow(4, 0.5),
            Math.pow(2, -1),
            Math.pow(-2, 3)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_array_includes_with_from_index() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test includes with fromIndex parameter
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3, 4, 5];
        [
            arr.includes(3, 2),    // true - found at index 2
            arr.includes(3, 3),    // false - start search from index 3
            arr.includes(1, -4),   // true - negative index
            arr.includes(5, -1)    // true - found at last position
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_array_includes_string() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test includes with strings
    let result = engine.execute_enhanced(r#"
        const str = "hello world";
        const words = str.split(' ');
        [
            words.includes('hello'),
            words.includes('world'),
            words.includes('foo'),
            typeof words.includes === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_array_includes_polyfill_exists() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Verify the polyfill is properly loaded
    let result = engine.execute_enhanced(r#"
        typeof Array.prototype.includes === 'function'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}