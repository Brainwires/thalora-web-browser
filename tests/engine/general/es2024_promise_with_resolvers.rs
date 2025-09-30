use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_es2024_promise_with_resolvers() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that Promise.withResolvers polyfill is loaded
    let result = engine.execute_enhanced(r#"
        typeof Promise.withResolvers === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
