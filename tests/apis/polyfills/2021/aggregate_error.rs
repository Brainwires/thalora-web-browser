use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_aggregate_error() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test AggregateError
    let result = engine.execute_enhanced(r#"
        typeof AggregateError === 'function'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
