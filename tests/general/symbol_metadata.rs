use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_symbol_metadata() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Symbol.metadata
    let result = engine.execute_enhanced(r#"
        typeof Symbol !== 'undefined' && typeof Symbol.metadata === 'symbol'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
