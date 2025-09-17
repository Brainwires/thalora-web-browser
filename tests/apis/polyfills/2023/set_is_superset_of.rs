use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_set_is_superset_of() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Set.prototype.isSupersetOf
    let result = engine.execute_enhanced(r#"
        const set1 = new Set([1, 2, 3, 4]);
        const set2 = new Set([1, 2]);
        const set3 = new Set([5, 6]);
        [
            set1.isSupersetOf(set2),
            set2.isSupersetOf(set1),
            set1.isSupersetOf(set3),
            set1.isSupersetOf(new Set())
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
