use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_promise_all_settled() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Promise.allSettled
    let result = engine.execute_enhanced(r#"
        typeof Promise.allSettled === 'function'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
