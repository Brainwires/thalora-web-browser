use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_numeric_separators_transformation() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test numeric separators transformation
    let result = engine.execute_enhanced(r#"
        const million = 1_000_000;
        const binary = 0b1010_0001;
        const octal = 0o755_444;
        const hex = 0xFF_EC_DE_5E;
        const decimal = 123.456_789;

        [
            million === 1000000,
            binary === 161,
            hex === 4293713502,
            typeof million === 'number'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
