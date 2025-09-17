use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_error_handling() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that errors are properly handled
    let result = engine.execute_enhanced(r#"
        try {
            throw new Error('test error');
        } catch (e) {
            e.message === 'test error'
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
