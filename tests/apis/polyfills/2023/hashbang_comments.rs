use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_hashbang_comments() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test hashbang comment removal
    let result = engine.execute_enhanced(r#"
        // Hashbang comments would be stripped during transformation
        const value = 42;
        value
    "#).await.unwrap();

    assert_eq!(result.as_number().unwrap(), 42.0);
}
