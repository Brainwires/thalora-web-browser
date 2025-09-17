#[tokio::test]
async fn test_string_replace_all() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test String.prototype.replaceAll
    let result = engine.execute_enhanced(r#"
        const str = 'hello world hello';
        [
            str.replaceAll('hello', 'hi'),
            str.replaceAll('world', 'universe'),
            'test test test'.replaceAll('test', 'demo')
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
