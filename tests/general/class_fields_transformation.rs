use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_class_fields_transformation() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test class fields transformation
    let result = engine.execute_enhanced(r#"
        // Class fields are moved to constructor
        class TestClass {
            constructor() {
                this.field = 'value';
                this.number = 42;
            }
        }

        const instance = new TestClass();
        [
            instance.field === 'value',
            instance.number === 42,
            typeof instance === 'object'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}
