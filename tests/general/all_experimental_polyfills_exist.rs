use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_all_experimental_polyfills_exist() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that all ES2025+ experimental polyfills exist
    let result = engine.execute_enhanced(r#"
        [
            typeof Array.fromAsync === 'function',
            typeof Record === 'function',
            typeof Tuple === 'function',
            typeof match === 'function',
            typeof pipe === 'function',
            typeof Observable === 'function',
            typeof AsyncContext === 'function',
            typeof Math.sumPrecise === 'function',
            typeof Error.isError === 'function',
            typeof using === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
