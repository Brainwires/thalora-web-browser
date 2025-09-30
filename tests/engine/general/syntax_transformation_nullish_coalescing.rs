use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_syntax_transformation_nullish_coalescing() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test nullish coalescing transformation
    let result = engine.execute_enhanced(r#"
        const a = null;
        const b = "default";
        a ?? b
    "#).await.unwrap();

    // Check if result is a string
    assert!(result.is_string());
}
