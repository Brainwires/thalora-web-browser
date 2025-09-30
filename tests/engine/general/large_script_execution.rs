use thalora::js::JavaScriptEngine;
use boa_engine::JsValue;

#[tokio::test]
async fn test_large_script_execution() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test execution of larger script with multiple features
    let result = engine.execute_enhanced(r#"
        // Complex script using many ES features
        class Calculator {
            constructor() {
                this.history = [];
            }

            calculate(a, b, operation) {
                let result;
                switch (operation) {
                    case 'add':
                        result = a + b;
                        break;
                    case 'multiply':
                        result = a * b;
                        break;
                    default:
                        result = 0;
                }

                this.history.push({ a, b, operation, result });
                return result;
            }

            getHistory() {
                return this.history.slice();
            }
        }

        const calc = new Calculator();
        const sum = calc.calculate(10, 5, 'add');
        const product = calc.calculate(3, 4, 'multiply');
        const history = calc.getHistory();

        sum === 15 && product === 12 && history.length === 2
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}
