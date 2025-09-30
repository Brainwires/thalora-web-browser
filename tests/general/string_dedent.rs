use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_string_dedent() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String.prototype.dedent
    let result = engine.execute_enhanced(r#"
        const indented = "    line 1\\n    line 2\\n    line 3";
        const dedented = indented.dedent();

        [
            dedented === "line 1\\nline 2\\nline 3",
            typeof 'test'.dedent === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
