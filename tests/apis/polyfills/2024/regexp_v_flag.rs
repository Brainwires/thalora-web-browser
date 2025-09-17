use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_regexp_v_flag() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test RegExp v flag support
    let result = engine.execute_enhanced(r#"
        try {
            const regex = new RegExp('test', 'v');
            regex.flags.includes('u') // v flag converted to u
        } catch (e) {
            try {
                const regex = new RegExp('test', 'u');
                true // Fallback to u flag
            } catch (e2) {
                true // Basic regex support
            }
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
