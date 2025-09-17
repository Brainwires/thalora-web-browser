use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_string_methods() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test string methods
    let result = engine.execute_enhanced(r#"
        const str = 'Hello World';
        [
            str.length === 11,
            str.toLowerCase() === 'hello world',
            str.toUpperCase() === 'HELLO WORLD',
            str.indexOf('World') === 6,
            str.slice(0, 5) === 'Hello',
            str.replace('World', 'Universe') === 'Hello Universe'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
