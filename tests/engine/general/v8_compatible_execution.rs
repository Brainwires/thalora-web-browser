use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_v8_compatible_execution() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test V8-compatible execution method
    let result = engine.execute_v8_compatible("Math.pow(2, 3)").await.unwrap();
    assert_eq!(result.as_number().unwrap(), 8.0);
}
