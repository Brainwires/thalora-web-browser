use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_math_extensions_polyfill() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Math extensions exist
    let result = engine.execute_enhanced(r#"
        typeof Math.sumPrecise === 'function'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
