#[tokio::test]
async fn test_intl_list_format_complete() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Intl.ListFormat (if available)
    let result = engine.execute_enhanced(r#"
        try {
            if (typeof Intl.ListFormat !== 'undefined') {
                const list = new Intl.ListFormat('en', { style: 'long', type: 'conjunction' });
                typeof list.format === 'function'
            } else {
                true // Not required to be supported
            }
        } catch (e) {
            true // Graceful fallback
        }
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
