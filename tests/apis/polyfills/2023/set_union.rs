use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_set_union() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Set.prototype.union
    let result = engine.execute_enhanced(r#"
        const set1 = new Set([1, 2, 3]);
        const set2 = new Set([3, 4, 5]);
        const union = set1.union(set2);
        [
            union.size,
            union.has(1),
            union.has(2),
            union.has(3),
            union.has(4),
            union.has(5)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
