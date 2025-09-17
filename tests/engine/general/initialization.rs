use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_engine_initialization() {
    let engine = JavaScriptEngine::new();
    assert!(engine.is_ok());
}
