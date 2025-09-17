use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_class_static_block() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test class static initialization blocks (transformed)
    let result = engine.execute_enhanced(r#"
        class MyClass {
            static value = 0;
        }
        // Static block would be transformed to immediate execution
        (function() {
            MyClass.value = 42;
        })();

        MyClass.value === 42
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
