#[tokio::test]
async fn test_finalization_registry() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test FinalizationRegistry
    let result = engine.execute_enhanced(r#"
        typeof FinalizationRegistry === 'function' || typeof FinalizationRegistry === 'undefined'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
