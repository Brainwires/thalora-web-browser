use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_experimental_features_polyfill() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test experimental features exist
    let result = engine.execute_enhanced(r#"
        typeof Record === 'function' &&
        typeof Tuple === 'function' &&
        typeof match === 'function' &&
        typeof pipe === 'function' &&
        typeof Observable === 'function' &&
        typeof AsyncContext === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
