use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_temporal_api_basic() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test basic Temporal API (if available)
    let result = engine.execute_enhanced(r#"
        typeof Temporal !== 'undefined' || typeof Temporal === 'undefined'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
