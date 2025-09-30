#[tokio::test]
async fn test_numeric_separators() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test numeric separators
    let result = engine.execute_enhanced(r#"
        const million = 1_000_000;
        const binary = 0b1010_0001;
        const hex = 0xFF_EC_DE_5E;
        [million, binary, hex]
    "#).await.unwrap();
    assert!(result.is_object());
}
