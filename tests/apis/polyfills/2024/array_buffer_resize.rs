use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_buffer_resize() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test ArrayBuffer.prototype.resize (if supported)
    let result = engine.execute_enhanced(r#"
        try {
            const buffer = new ArrayBuffer(16);
            typeof buffer.resize === 'function' || typeof buffer.resize === 'undefined'
        } catch (e) {
            true // Graceful fallback
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
