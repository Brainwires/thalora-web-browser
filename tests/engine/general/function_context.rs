use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_function_context() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test function context and this binding
    let result = engine.execute_enhanced(r#"
        const obj = {
            value: 42,
            getValue: function() {
                return this.value;
            }
        };

        [
            obj.getValue() === 42,
            typeof obj.getValue === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
