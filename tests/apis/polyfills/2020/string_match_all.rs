#[tokio::test]
async fn test_string_match_all() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test String.prototype.matchAll
    let result = engine.execute_enhanced(r#"
        const str = 'test1 test2 test3';
        const matches = [...str.matchAll(/test\\d/g)];
        matches.length === 3
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
