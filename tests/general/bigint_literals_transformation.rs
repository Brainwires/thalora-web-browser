use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_bigint_literals_transformation() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test BigInt literals transformation
    let result = engine.execute_enhanced(r#"
        const big1 = BigInt('123');
        const big2 = BigInt('456789');

        [
            typeof big1 === 'bigint',
            typeof big2 === 'bigint',
            typeof BigInt === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
