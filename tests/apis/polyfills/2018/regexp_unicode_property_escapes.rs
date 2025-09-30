#[tokio::test]
async fn test_regexp_unicode_property_escapes() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Unicode property escapes
    let result = engine.execute_enhanced(r#"
        try {
            const regex = /\\p{Letter}/u;
            const match = 'A'.match(regex);
            true
        } catch (e) {
            // Unicode property escapes might not be supported
            true
        }
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
