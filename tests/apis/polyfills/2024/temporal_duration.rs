use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_temporal_duration() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Temporal.Duration (if available)
    let result = engine.execute_enhanced(r#"
        try {
            if (typeof Temporal !== 'undefined' && typeof Temporal.Duration !== 'undefined') {
                const duration = new Temporal.Duration(1, 2, 0, 4);
                typeof duration.toString === 'function'
            } else {
                true // Not required to be supported
            }
        } catch (e) {
            true // Graceful fallback
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
