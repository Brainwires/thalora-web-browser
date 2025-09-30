use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_console_polyfill() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test console object exists
    let result = engine.execute_enhanced(r#"
        typeof console === 'object' && typeof console.log === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
