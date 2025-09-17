use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_symbol_description() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Symbol.prototype.description
    let result = engine.execute_enhanced(r#"
        const sym1 = Symbol('test');
        const sym2 = Symbol();
        [
            sym1.description === 'test',
            sym2.description === undefined,
            typeof Symbol.prototype.description !== 'undefined'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
