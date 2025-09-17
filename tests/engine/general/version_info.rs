use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_engine_version_info() {
    let engine = JavaScriptEngine::new().unwrap();
    let version = engine.version_info();
    assert_eq!(version, "Enhanced JavaScript Engine v3.0 (ES2025+ Compatible)");
}
