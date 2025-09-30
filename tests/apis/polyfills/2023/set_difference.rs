#[tokio::test]
async fn test_set_difference() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Set.prototype.difference
    let result = engine.execute_enhanced(r#"
        const set1 = new Set([1, 2, 3, 4]);
        const set2 = new Set([3, 4, 5, 6]);
        const difference = set1.difference(set2);
        [
            difference.size,
            difference.has(1),
            difference.has(2),
            difference.has(3),
            difference.has(4)
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
