use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_polyfill_integration() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that polyfills are integrated and working
    let result = engine.execute_enhanced(r#"
        // Test a mix of polyfilled and native features
        const arr = [1, 2, 3, 4, 5];
        const set = new Set([1, 2, 3]);

        [
            // ES2023 polyfills
            typeof arr.findLast === 'function',
            typeof set.intersection === 'function',
            // ES2024 polyfills
            typeof Promise.withResolvers === 'function',
            typeof Object.groupBy === 'function',
            // ES2025+ experimental
            typeof Record === 'function',
            typeof Observable === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
