#[tokio::test]
async fn test_async_iteration() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Symbol.asyncIterator
    let result = engine.execute_enhanced(r#"
        typeof Symbol.asyncIterator === 'symbol'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
