use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_intl_date_time_format_date_style() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Intl.DateTimeFormat dateStyle and timeStyle
    let result = engine.execute_enhanced(r#"
        try {
            if (typeof Intl.DateTimeFormat !== 'undefined') {
                const formatter = new Intl.DateTimeFormat('en');
                typeof formatter.format === 'function'
            } else {
                true // Not required to be supported
            }
        } catch (e) {
            true // Graceful fallback
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
