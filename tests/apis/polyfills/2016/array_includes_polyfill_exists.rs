#[tokio::test]
async fn test_array_includes_polyfill_exists() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Verify the polyfill is properly loaded
    let result = engine.execute_enhanced(r#"
        typeof Array.prototype.includes === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
