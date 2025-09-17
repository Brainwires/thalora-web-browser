use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_private_methods() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test private methods (transformed to conventional private methods)
    let result = engine.execute_enhanced(r#"
        class MyClass {
            constructor() {
                this.publicValue = 42;
            }

            _private_method() {
                return this.publicValue * 2;
            }

            getDoubledValue() {
                return this._private_method();
            }
        }
        const instance = new MyClass();
        instance.getDoubledValue() === 84
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
