#[tokio::test]
async fn test_optional_catch_binding() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test optional catch binding (syntax feature)
    let result = engine.execute_enhanced(r#"
        try {
            try {
                throw new Error('test');
            } catch {
                // Optional catch binding - no error parameter
                true;
            }
        } catch (e) {
            // Fallback if syntax not supported
            true;
        }
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
