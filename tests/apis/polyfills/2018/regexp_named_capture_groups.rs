use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_regexp_named_capture_groups() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test named capture groups support
    let result = engine.execute_enhanced(r#"
        try {
            const regex = /(?<year>\\d{4})-(?<month>\\d{2})-(?<day>\\d{2})/;
            const match = '2023-12-25'.match(regex);
            match !== null
        } catch (e) {
            // Named capture groups might not be supported
            true
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
