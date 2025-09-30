use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_from_async() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.fromAsync
    let result = engine.execute_enhanced(r#"
        typeof Array.fromAsync === 'function'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
