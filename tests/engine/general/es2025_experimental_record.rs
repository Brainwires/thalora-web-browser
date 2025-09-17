use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_es2025_experimental_record() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that Record polyfill is loaded
    let result = engine.execute_enhanced(r#"
        typeof Record === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
