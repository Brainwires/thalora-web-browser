use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_global_object_operations() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that global object operations don't crash
    let missing = engine.get_global_object("missingValue").unwrap();
    assert!(missing.is_none());
}
