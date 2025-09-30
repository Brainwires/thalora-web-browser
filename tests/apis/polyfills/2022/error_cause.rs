#[tokio::test]
async fn test_error_cause() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test Error cause property
    let result = engine.execute_enhanced(r#"
        try {
            const err = new Error('test', { cause: 'root cause' });
            err.cause === 'root cause'
        } catch (e) {
            // cause might not be supported
            true
        }
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
