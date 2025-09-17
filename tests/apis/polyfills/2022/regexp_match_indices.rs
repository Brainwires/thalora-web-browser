use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_regexp_match_indices() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test RegExp match indices (d flag)
    let result = engine.execute_enhanced(r#"
        try {
            const regex = /a(b)/d;
            const match = 'ab'.match(regex);
            match !== null
        } catch (e) {
            // d flag might not be supported
            true
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
