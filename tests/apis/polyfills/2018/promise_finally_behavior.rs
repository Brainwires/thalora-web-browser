#[tokio::test]
async fn test_promise_finally_behavior() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test that finally polyfill exists and is callable
    let result = engine.execute_enhanced(r#"
        const p = Promise.resolve(42);
        typeof p.finally === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
