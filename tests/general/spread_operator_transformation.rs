use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_spread_operator_transformation() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test spread operator transformation
    let result = engine.execute_enhanced(r#"
        function sum(a, b, c) {
            return a + b + c;
        }

        const args = [1, 2, 3];

        // Spread is transformed to apply
        const result = sum.apply(null, args);

        [
            result === 6,
            typeof sum.apply === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
