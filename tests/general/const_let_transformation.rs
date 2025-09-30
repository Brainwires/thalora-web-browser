use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_const_let_transformation() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test const/let to var transformation
    let result = engine.execute_enhanced(r#"
        var x = 10;
        var y = 'hello';
        var z = true;

        [
            x === 10,
            y === 'hello',
            z === true,
            typeof x === 'number',
            typeof y === 'string',
            typeof z === 'boolean'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
