use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_pattern_matching_basic() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test basic pattern matching
    let result = engine.execute_enhanced(r#"
        const value = 42;
        const result1 = match(value)
            .with(42, x => 'found forty-two')
            .otherwise(x => 'something else');

        const result2 = match(value)
            .with(x => x > 40, x => 'greater than 40')
            .otherwise(x => 'not greater than 40');

        [
            result1.result === 'found forty-two',
            result1.matched === true,
            result2.result === 'greater than 40',
            result2.matched === true,
            typeof match === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
