#[tokio::test]
async fn test_for_in_mechanics() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test for-in mechanics improvements
    let result = engine.execute_enhanced(r#"
        const obj = { a: 1, b: 2, c: 3 };
        const keys = [];
        for (const key in obj) {
            keys.push(key);
        }
        keys.length === 3
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
