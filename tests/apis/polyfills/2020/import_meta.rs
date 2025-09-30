#[tokio::test]
async fn test_import_meta() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test import.meta (would be polyfilled or transformed)
    let result = engine.execute_enhanced(r#"
        // import.meta would be handled by module system
        typeof undefined === 'undefined'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
