#[tokio::test]
async fn test_array_flat_map_edge_cases() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Array.prototype.flatMap edge cases
    let result = engine.execute_enhanced(r#"
        [
            [1, 2, 3].flatMap(x => []),
            [1, 2, 3].flatMap(x => x),
            ['hello', 'world'].flatMap(str => str.split('')),
            [].flatMap(x => [x, x])
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
