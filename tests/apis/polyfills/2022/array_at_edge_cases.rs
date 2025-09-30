#[tokio::test]
async fn test_array_at_edge_cases() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Array.prototype.at edge cases
    let result = engine.execute_enhanced(r#"
        [
            [].at(0),
            [1].at(-1),
            [1, 2, 3].at(1.5), // Should floor to 1
            'test'.at(-1)
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
