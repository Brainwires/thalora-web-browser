use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_promise_any() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Promise.any
    let result = engine.execute_enhanced(r#"
        typeof Promise.any === 'function'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
