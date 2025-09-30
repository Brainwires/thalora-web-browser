#[tokio::test]
async fn test_bigint_support() {
    let mut engine = JavaScriptEngine::new().unwrap();
    // Test BigInt support
    let result = engine.execute_enhanced(r#"
        typeof BigInt === 'function' && typeof BigInt('123') === 'bigint'
    "#).await.unwrap();
    assert!(result.as_boolean().unwrap());
}
