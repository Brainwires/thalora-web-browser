#[tokio::test]
async fn test_array_to_spliced() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Array.prototype.toSpliced
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3, 4, 5];
        const spliced1 = arr.toSpliced(2, 1);
        const spliced2 = arr.toSpliced(1, 2, 'a', 'b');
        [
            JSON.stringify(arr),
            JSON.stringify(spliced1),
            JSON.stringify(spliced2),
            arr !== spliced1
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
