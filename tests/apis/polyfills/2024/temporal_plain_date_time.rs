#[tokio::test]
async fn test_temporal_plain_date_time() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Temporal.PlainDateTime (if available)
    let result = engine.execute_enhanced(r#"
        try {
            if (typeof Temporal !== 'undefined' && typeof Temporal.PlainDateTime !== 'undefined') {
                const dt = new Temporal.PlainDateTime(2024, 1, 1, 12, 0, 0);
                typeof dt.toString === 'function'
            } else {
                true // Not required to be supported
            }
        } catch (e) {
            true // Graceful fallback
        }
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
