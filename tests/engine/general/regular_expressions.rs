use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_regular_expressions() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test regular expression support
    let result = engine.execute_enhanced(r#"
        const regex = /hello/gi;
        const str = 'Hello HELLO hello';
        const matches = str.match(regex);

        [
            regex instanceof RegExp,
            matches !== null,
            matches.length >= 2
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
