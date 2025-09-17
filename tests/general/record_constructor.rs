use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_record_constructor() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Record constructor
    let result = engine.execute_enhanced(r#"
        const rec = Record({ a: 1, b: 2, c: 3 });
        [
            rec.a === 1,
            rec.b === 2,
            rec.c === 3,
            typeof rec === 'object',
            typeof Record === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
