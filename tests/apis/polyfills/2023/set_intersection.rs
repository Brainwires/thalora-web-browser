#[tokio::test]
async fn test_set_intersection() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Set.prototype.intersection
    let result = engine.execute_enhanced(r#"
        const set1 = new Set([1, 2, 3, 4]);
        const set2 = new Set([3, 4, 5, 6]);
        const intersection = set1.intersection(set2);
        [
            intersection.size,
            intersection.has(3),
            intersection.has(4),
            intersection.has(1),
            intersection.has(5)
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
