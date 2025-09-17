use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_pattern_matching_otherwise() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test pattern matching with otherwise clause
    let result = engine.execute_enhanced(r#"
        const value = 'unmatched';
        const result = match(value)
            .with(42, x => 'number')
            .with('hello', x => 'greeting')
            .otherwise(x => 'fallback: ' + x);

        [
            result.result === 'fallback: unmatched',
            result.matched === true
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
