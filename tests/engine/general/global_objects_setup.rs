use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_global_objects_setup() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that global objects are properly set up
    let result = engine.execute_enhanced(r#"
        [
            typeof console === 'object',
            typeof global === 'object',
            typeof globalThis === 'object',
            typeof Promise === 'function',
            typeof Array === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
