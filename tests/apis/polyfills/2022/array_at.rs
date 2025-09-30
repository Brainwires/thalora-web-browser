#[tokio::test]
async fn test_array_at() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Array.prototype.at
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3, 4, 5];
        [
            arr.at(0),
            arr.at(-1),
            arr.at(2),
            arr.at(-2),
            arr.at(10)
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
