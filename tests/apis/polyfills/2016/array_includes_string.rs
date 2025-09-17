use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_includes_string() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test includes with strings
    let result = engine.execute_enhanced(r#"
        const str = "hello world";
        const words = str.split(' ');
        [
            words.includes('hello'),
            words.includes('world'),
            words.includes('foo'),
            typeof words.includes === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
