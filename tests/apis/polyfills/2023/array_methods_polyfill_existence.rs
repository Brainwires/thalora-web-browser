#[tokio::test]
async fn test_array_methods_polyfill_existence() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test that all ES2023 array methods exist
    let result = engine.execute_enhanced(r#"
        const arr = [];
        [
            typeof arr.findLast === 'function',
            typeof arr.findLastIndex === 'function',
            typeof arr.toReversed === 'function',
            typeof arr.toSorted === 'function',
            typeof arr.toSpliced === 'function',
            typeof arr.with === 'function'
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
