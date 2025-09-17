#[tokio::test]
async fn test_object_from_entries() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Object.fromEntries
    let result = engine.execute_enhanced(r#"
        const entries = [['a', 1], ['b', 2], ['c', 3]];
        const obj = Object.fromEntries(entries);
        obj.a === 1 && obj.b === 2 && obj.c === 3
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
