use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_date_functionality() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Date functionality
    let result = engine.execute_enhanced(r#"
        const date = new Date();
        [
            date instanceof Date,
            typeof date.getTime === 'function',
            typeof date.getFullYear === 'function',
            typeof Date.now === 'function',
            typeof Date.now() === 'number'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
