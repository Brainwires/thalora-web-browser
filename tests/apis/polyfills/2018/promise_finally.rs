#[tokio::test]
async fn test_promise_finally() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Promise.prototype.finally
    let result = engine.execute_enhanced(r#"
        typeof Promise.prototype.finally === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
