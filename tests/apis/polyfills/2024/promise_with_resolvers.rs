use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_promise_with_resolvers() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Promise.withResolvers
    let result = engine.execute_enhanced(r#"
        const { promise, resolve, reject } = Promise.withResolvers();
        [
            typeof promise === 'object',
            typeof resolve === 'function',
            typeof reject === 'function',
            promise instanceof Promise
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
