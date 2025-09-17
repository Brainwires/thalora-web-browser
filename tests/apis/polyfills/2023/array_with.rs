#[tokio::test]
async fn test_array_with() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Array.prototype.with
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3];
        const modified1 = arr.with(1, 'changed');
        const modified2 = arr.with(-1, 'last');
        [
            JSON.stringify(arr),
            JSON.stringify(modified1),
            JSON.stringify(modified2),
            arr !== modified1
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
