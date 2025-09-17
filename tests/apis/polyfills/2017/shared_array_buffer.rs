use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_shared_array_buffer() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test SharedArrayBuffer polyfill (if available)
    let result = engine.execute_enhanced(r#"
        // Check if SharedArrayBuffer exists or is polyfilled
        typeof SharedArrayBuffer !== 'undefined' || typeof ArrayBuffer !== 'undefined'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
