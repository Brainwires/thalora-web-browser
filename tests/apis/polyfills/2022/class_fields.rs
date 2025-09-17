use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_class_fields() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test class fields (transformed)
    let result = engine.execute_enhanced(r#"
        class MyClass {
            constructor() {
                this.publicField = 'public';
                this._private_privateField = 'private';
            }
        }
        const instance = new MyClass();
        instance.publicField === 'public'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
