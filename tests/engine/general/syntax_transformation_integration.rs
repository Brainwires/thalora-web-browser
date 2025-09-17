use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_syntax_transformation_integration() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that syntax transformations work end-to-end
    let result = engine.execute_enhanced(r#"
        // This uses multiple transformed syntax features
        const obj = { nested: { value: 42 } };
        let result = null;

        result ??= obj?.nested?.value || 0;
        const nums = [1_000, 2_000];
        const sum = nums[0] + nums[1];

        result === 42 && sum === 3000
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
