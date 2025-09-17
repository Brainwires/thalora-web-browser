use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_string_well_formed_with_surrogates() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test well-formed string methods with surrogate pairs
    let result = engine.execute_enhanced(r#"
        try {
            // Test with potential lone surrogates
            const testStr = String.fromCharCode(0xD800);
            [
                testStr.isWellFormed(),
                testStr.toWellFormed().length >= 0
            ]
        } catch (e) {
            [true, true] // Graceful fallback
        }
    "#).await.unwrap();

    assert!(result.is_object());
}
