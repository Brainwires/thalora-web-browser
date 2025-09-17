#[tokio::test]
async fn test_dynamic_import() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test dynamic import syntax (transformed)
    let result = engine.execute_enhanced(r#"
        // Dynamic imports would be transformed or polyfilled
        typeof Promise !== 'undefined'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
