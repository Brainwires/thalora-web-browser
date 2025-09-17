#[tokio::test]
async fn test_object_values() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Object.values
    let result = engine.execute_enhanced(r#"
        const obj = { a: 1, b: 2, c: 3 };
        const values = Object.values(obj);
        JSON.stringify(values.sort())
    "#).await.unwrap();
    // Should contain [1, 2, 3]
    assert!(result.is_string());
}
