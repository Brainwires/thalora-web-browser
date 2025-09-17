use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_set_symmetric_difference() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Set.prototype.symmetricDifference
    let result = engine.execute_enhanced(r#"
        const set1 = new Set([1, 2, 3]);
        const set2 = new Set([3, 4, 5]);
        const symDiff = set1.symmetricDifference(set2);
        [
            symDiff.size,
            symDiff.has(1),
            symDiff.has(2),
            symDiff.has(3),
            symDiff.has(4),
            symDiff.has(5)
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
