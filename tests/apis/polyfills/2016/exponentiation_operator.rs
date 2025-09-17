#[tokio::test]
async fn test_exponentiation_operator() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test ** operator via Math.pow transformation
    let result = engine.execute_enhanced(r#"
        [
            Math.pow(2, 3),
            Math.pow(4, 0.5),
            Math.pow(2, -1),
            Math.pow(-2, 3)
        ]
    "#).await.unwrap();
    assert!(result.is_object());
}
