use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_static_class_fields() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test static class fields
    let result = engine.execute_enhanced(r#"
        class MyClass {
            static staticField = 'static';

            static getStaticField() {
                return this.staticField;
            }
        }
        MyClass.staticField = 'static'; // Polyfilled assignment
        MyClass.getStaticField() === 'static'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
