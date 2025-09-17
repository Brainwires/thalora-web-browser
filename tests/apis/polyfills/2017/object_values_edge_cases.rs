#[tokio::test]
async fn test_object_values_edge_cases() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Object.values edge cases
    let result = engine.execute_enhanced(r#"
        [
            Object.values({}),
            Object.values([1, 2, 3]),
            Object.values('hello'),
            Object.values(null) || [],
            Object.values(undefined) || []
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
