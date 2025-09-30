#[tokio::test]
async fn test_polyfill_functions_exist() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test that all ES2024 polyfill functions exist
    let result = engine.execute_enhanced(r#"
        [
            typeof Promise.withResolvers === 'function',
            typeof Object.groupBy === 'function',
            typeof Map.groupBy === 'function',
            typeof 'test'.isWellFormed === 'function',
            typeof 'test'.toWellFormed === 'function'
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
