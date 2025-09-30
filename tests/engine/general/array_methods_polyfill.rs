use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_methods_polyfill() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test multiple array methods exist
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3];
        typeof arr.findLast === 'function' &&
        typeof arr.findLastIndex === 'function' &&
        typeof arr.toReversed === 'function' &&
        typeof arr.toSorted === 'function' &&
        typeof arr.toSpliced === 'function' &&
        typeof arr.with === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
