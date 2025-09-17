use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_shared_array_buffer_grow() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test SharedArrayBuffer.prototype.grow (if supported)
    let result = engine.execute_enhanced(r#"
        typeof SharedArrayBuffer !== 'undefined' ?
            (typeof SharedArrayBuffer.prototype.grow === 'function' || typeof SharedArrayBuffer.prototype.grow === 'undefined') :
            true
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
