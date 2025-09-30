#[tokio::test]
async fn test_set_is_subset_of() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Set.prototype.isSubsetOf
    let result = engine.execute_enhanced(r#"
        const set1 = new Set([1, 2]);
        const set2 = new Set([1, 2, 3, 4]);
        const set3 = new Set([5, 6]);
        [
            set1.isSubsetOf(set2),
            set2.isSubsetOf(set1),
            set3.isSubsetOf(set2),
            new Set().isSubsetOf(set1)
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
