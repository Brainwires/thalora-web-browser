#[tokio::test]
async fn test_object_entries() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Object.entries
    let result = engine.execute_enhanced(r#"
        const obj = { foo: 'bar', baz: 42 };
        const entries = Object.entries(obj);
        entries.length === 2 && Array.isArray(entries[0])
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
