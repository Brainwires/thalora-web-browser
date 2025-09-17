use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_record_immutability() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Record immutability
    let result = engine.execute_enhanced(r#"
        const rec = Record({ x: 10, y: 20 });
        try {
            rec.x = 999; // Should not change the record
            rec.x === 10 // Original value preserved
        } catch (e) {
            true // Might throw in strict mode
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
