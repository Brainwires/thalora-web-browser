#[tokio::test]
async fn test_array_to_sorted() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Array.prototype.toSorted
    let result = engine.execute_enhanced(r#"
        const arr = [3, 1, 4, 1, 5];
        const sorted = arr.toSorted();
        const customSorted = arr.toSorted((a, b) => b - a);
        [
            JSON.stringify(arr),
            JSON.stringify(sorted),
            JSON.stringify(customSorted),
            arr !== sorted
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
