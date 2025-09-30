#[tokio::test]
async fn test_string_pad_end() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test String.prototype.padEnd
    let result = engine.execute_enhanced(r#"
        [
            'hello'.padEnd(10),
            'hello'.padEnd(10, '*'),
            'hello'.padEnd(3),
            'abc'.padEnd(6, '123456')
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
