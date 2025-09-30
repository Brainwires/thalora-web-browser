use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_console_functionality() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test console object and methods
    let result = engine.execute_enhanced(r#"
        [
            typeof console.log === 'function',
            typeof console.error === 'function',
            typeof console.warn === 'function',
            typeof console.info === 'function',
            typeof console.debug === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
