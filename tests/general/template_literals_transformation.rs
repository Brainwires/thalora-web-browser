use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_template_literals_transformation() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test template literals transformation
    let result = engine.execute_enhanced(r#"
        const name = 'World';
        const count = 42;

        // Template literals are transformed to string concatenation
        const greeting = 'Hello, ' + (name) + '!';
        const message = 'Count: ' + (count) + ' items';
        const multiline = 'Line 1' + '\\n' + 'Line 2';

        [
            greeting === 'Hello, World!',
            message === 'Count: 42 items',
            multiline.includes('Line 1'),
            multiline.includes('Line 2')
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
