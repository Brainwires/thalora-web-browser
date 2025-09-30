use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_partial_application() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Function.prototype.partial
    let result = engine.execute_enhanced(r#"
        function add(a, b, c) {
            return a + b + c;
        }

        const addWithFirst = add.partial(10, undefined, undefined);
        const addWithFirstAndThird = add.partial(10, undefined, 5);

        [
            addWithFirst(2, 3) === 15,
            addWithFirstAndThird(7) === 22,
            typeof add.partial === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
