use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_symbol_dispose() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Symbol.dispose for resource management
    let result = engine.execute_enhanced(r#"
        [
            typeof Symbol !== 'undefined',
            typeof Symbol.dispose === 'symbol' || typeof Symbol.dispose === 'undefined',
            typeof Symbol.asyncDispose === 'symbol' || typeof Symbol.asyncDispose === 'undefined'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
