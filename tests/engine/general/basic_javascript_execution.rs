use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_basic_javascript_execution() {
    let mut engine = JavaScriptEngine::new().unwrap();

    let result = engine.execute_enhanced("2 + 3").await.unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0);

    let result = engine.execute_enhanced("'hello ' + 'world'").await.unwrap();
    assert!(result.is_string());

    let result = engine.execute_enhanced("true && false").await.unwrap();
    assert_eq!(result.as_boolean().unwrap(), false);
}
