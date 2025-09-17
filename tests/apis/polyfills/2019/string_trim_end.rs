use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_string_trim_end() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String.prototype.trimEnd/trimRight
    let result = engine.execute_enhanced(r#"
        const str = '   hello   ';
        [
            str.trimEnd(),
            str.trimRight(),
            '   '.trimEnd(),
            'hello'.trimEnd()
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
