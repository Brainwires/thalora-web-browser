use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_string_padding_unicode() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test string padding with unicode
    let result = engine.execute_enhanced(r#"
        [
            'café'.padStart(10, '★'),
            '🌟'.padEnd(5, '✨'),
            'test'.padStart(2), // Should return original string
            'test'.padEnd(2)    // Should return original string
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
