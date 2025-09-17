use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_syntax_transformation_numeric_separators() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test numeric separators transformation
    let result = engine.execute_enhanced(r#"
        const big = 1_000_000;
        big
    "#).await.unwrap();
    assert_eq!(result.as_number().unwrap(), 1000000.0);
}
