#[tokio::test]
async fn test_string_pad_start() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test String.prototype.padStart
    let result = engine.execute_enhanced(r#"
        [
            'hello'.padStart(10),
            'hello'.padStart(10, '*'),
            'hello'.padStart(3),
            '123'.padStart(5, '0')
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
