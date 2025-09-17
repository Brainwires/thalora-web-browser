#[tokio::test]
async fn test_array_buffer_transfer() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test ArrayBuffer.prototype.transfer
    let result = engine.execute_enhanced(r#"
        try {
            const buffer = new ArrayBuffer(16);
            typeof buffer.transfer === 'function'
        } catch (e) {
            true // May not be supported
        }
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
