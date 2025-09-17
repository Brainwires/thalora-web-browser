use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_optional_chaining() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test optional chaining operator (?.)
    let result = engine.execute_enhanced(r#"
        const obj = { nested: { value: 42 } };
        const empty = null;
        [
            obj?.nested?.value,
            empty?.nested?.value,
            obj?.missing?.value,
            obj?.nested?.missing
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
