use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_string_dedent_mixed_indentation() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String.prototype.dedent with mixed indentation
    let result = engine.execute_enhanced(r#"
        const text = "\\n      hello\\n        world\\n      test\\n";
        const dedented = text.dedent();

        [
            dedented.includes('hello'),
            dedented.includes('world'),
            dedented.includes('test'),
            !dedented.startsWith(' '), // Should not start with spaces
            typeof dedented === 'string'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
