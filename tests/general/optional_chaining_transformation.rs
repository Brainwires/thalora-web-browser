use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_optional_chaining_transformation() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test optional chaining transformation
    let result = engine.execute_enhanced(r#"
        const obj = {
            nested: {
                value: 42,
                method: function() { return 'called'; }
            }
        };
        const nullObj = null;

        [
            obj?.nested?.value,
            nullObj?.nested?.value,
            obj?.missing?.value,
            obj?.nested?.method?.()
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
