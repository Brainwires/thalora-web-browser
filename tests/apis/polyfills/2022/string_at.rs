use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_string_at() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String.prototype.at
    let result = engine.execute_enhanced(r#"
        const str = 'hello';
        [
            str.at(0),
            str.at(-1),
            str.at(2),
            str.at(-2),
            str.at(10)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
