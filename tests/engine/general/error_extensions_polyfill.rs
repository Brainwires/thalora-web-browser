use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_error_extensions_polyfill() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Error extensions exist
    let result = engine.execute_enhanced(r#"
        typeof Error.isError === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
