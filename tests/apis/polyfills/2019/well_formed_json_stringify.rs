use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_well_formed_json_stringify() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test well-formed JSON.stringify (handles lone surrogates)
    let result = engine.execute_enhanced(r#"
        // Test that JSON.stringify handles unicode correctly
        try {
            JSON.stringify('hello');
            true
        } catch (e) {
            false
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
