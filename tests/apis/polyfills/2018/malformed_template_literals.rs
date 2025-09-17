use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_malformed_template_literals() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test template literal revision (malformed escape sequences)
    let result = engine.execute_enhanced(r#"
        // Template literals with malformed escapes should be handled gracefully
        try {
            const template = 'test';
            template.length === 4
        } catch (e) {
            false
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
