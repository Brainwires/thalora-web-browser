use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_set_is_disjoint_from() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Set.prototype.isDisjointFrom
    let result = engine.execute_enhanced(r#"
        const set1 = new Set([1, 2, 3]);
        const set2 = new Set([4, 5, 6]);
        const set3 = new Set([3, 4, 5]);
        [
            set1.isDisjointFrom(set2),
            set1.isDisjointFrom(set3),
            set2.isDisjointFrom(set3),
            new Set().isDisjointFrom(set1)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
