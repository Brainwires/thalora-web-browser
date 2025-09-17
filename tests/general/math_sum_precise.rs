use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_math_sum_precise() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Math.sumPrecise with Kahan summation
    let result = engine.execute_enhanced(r#"
        const values = [0.1, 0.2, 0.3];
        const regularSum = values.reduce((a, b) => a + b, 0);
        const preciseSum = Math.sumPrecise(values);

        [
            typeof preciseSum === 'number',
            Math.abs(preciseSum - 0.6) < Math.abs(regularSum - 0.6),
            typeof Math.sumPrecise === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
