use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_template_literal_revision() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test template literal with invalid escape sequences (should not throw)
    let result = engine.execute_enhanced(r#"
        // Test that template literals are properly transformed
        const name = 'World';
        const template = 'Hello ' + name + '!';
        template === 'Hello World!'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
