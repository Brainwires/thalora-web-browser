#[tokio::test]
async fn test_global_this() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test globalThis availability
    let result = engine.execute_enhanced(r#"
        typeof globalThis === 'object'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
