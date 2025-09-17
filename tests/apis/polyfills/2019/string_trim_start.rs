use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_string_trim_start() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String.prototype.trimStart/trimLeft
    let result = engine.execute_enhanced(r#"
        const str = '   hello   ';
        [
            str.trimStart(),
            str.trimLeft(),
            '   '.trimStart(),
            'hello'.trimStart()
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
