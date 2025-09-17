use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_json_support() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test basic JSON object existence only (avoiding complex operations)
    let result = engine.execute_enhanced(r#"
        typeof JSON === 'object'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
