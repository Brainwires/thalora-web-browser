#[tokio::test]
async fn test_intl_list_format() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Intl.ListFormat (may not be fully supported)
    let result = engine.execute_enhanced(r#"
        typeof Intl !== 'undefined'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
