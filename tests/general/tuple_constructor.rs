use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_tuple_constructor() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Tuple constructor
    let result = engine.execute_enhanced(r#"
        const tup = Tuple(1, 2, 3, 4);
        [
            tup.length === 4,
            tup[0] === 1,
            tup[3] === 4,
            typeof tup === 'object',
            typeof Tuple === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
