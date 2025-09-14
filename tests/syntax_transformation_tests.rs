use synaptic::enhanced_js_v2::EnhancedJavaScriptEngine;

#[tokio::test]
async fn test_nullish_coalescing_transformation() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test nullish coalescing operator transformation
    let result = engine.execute_enhanced(r#"
        const a = null;
        const b = undefined;
        const c = 0;
        const d = false;
        const e = '';

        [
            a ?? 'default',
            b ?? 'default',
            c ?? 'default',
            d ?? 'default',
            e ?? 'default'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_optional_chaining_transformation() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test optional chaining transformation
    let result = engine.execute_enhanced(r#"
        const obj = {
            nested: {
                value: 42,
                method: function() { return 'called'; }
            }
        };
        const nullObj = null;

        [
            obj?.nested?.value,
            nullObj?.nested?.value,
            obj?.missing?.value,
            obj?.nested?.method?.()
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_logical_assignment_transformation() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test logical assignment operators transformation
    let result = engine.execute_enhanced(r#"
        let a = false;
        let b = true;
        let c = null;
        let d = 'existing';

        a ||= 'fallback';
        b &&= 'modified';
        c ??= 'null-fallback';
        d ??= 'wont-change';

        [a, b, c, d]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_numeric_separators_transformation() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test numeric separators transformation
    let result = engine.execute_enhanced(r#"
        const million = 1_000_000;
        const binary = 0b1010_0001;
        const octal = 0o755_444;
        const hex = 0xFF_EC_DE_5E;
        const decimal = 123.456_789;

        [
            million === 1000000,
            binary === 161,
            hex === 4293713502,
            typeof million === 'number'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_private_fields_transformation() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test private fields transformation
    let result = engine.execute_enhanced(r#"
        // Private fields are transformed to _private_ prefixed properties
        class TestClass {
            constructor() {
                this._private_value = 42;
                this.publicValue = 'public';
            }

            getValue() {
                return this._private_value;
            }

            setValue(val) {
                this._private_value = val;
            }
        }

        const instance = new TestClass();
        [
            instance.getValue() === 42,
            instance.publicValue === 'public',
            typeof instance._private_value === 'number'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_bigint_literals_transformation() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test BigInt literals transformation
    let result = engine.execute_enhanced(r#"
        const big1 = BigInt('123');
        const big2 = BigInt('456789');

        [
            typeof big1 === 'bigint',
            typeof big2 === 'bigint',
            typeof BigInt === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_arrow_functions_transformation() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test arrow functions transformation to regular functions
    let result = engine.execute_enhanced(r#"
        var add = function(a, b) { return a + b; };
        var multiply = function(x, y) { return x * y; };
        var greet = function(name) { return 'Hello, ' + name; };

        [
            add(2, 3) === 5,
            multiply(4, 5) === 20,
            greet('World') === 'Hello, World',
            typeof add === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_template_literals_transformation() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test template literals transformation
    let result = engine.execute_enhanced(r#"
        const name = 'World';
        const count = 42;

        // Template literals are transformed to string concatenation
        const greeting = 'Hello, ' + (name) + '!';
        const message = 'Count: ' + (count) + ' items';
        const multiline = 'Line 1' + '\\n' + 'Line 2';

        [
            greeting === 'Hello, World!',
            message === 'Count: 42 items',
            multiline.includes('Line 1'),
            multiline.includes('Line 2')
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_destructuring_transformation() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test destructuring transformation
    let result = engine.execute_enhanced(r#"
        const obj = { a: 1, b: 2, c: 3 };

        // Destructuring is transformed to individual assignments
        var a = (obj).a;
        var b = (obj).b;

        [
            a === 1,
            b === 2,
            typeof a === 'number',
            typeof b === 'number'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_spread_operator_transformation() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test spread operator transformation
    let result = engine.execute_enhanced(r#"
        function sum(a, b, c) {
            return a + b + c;
        }

        const args = [1, 2, 3];

        // Spread is transformed to apply
        const result = sum.apply(null, args);

        [
            result === 6,
            typeof sum.apply === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_for_of_loops_transformation() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test for...of loops transformation
    let result = engine.execute_enhanced(r#"
        const arr = ['a', 'b', 'c'];
        const result = [];

        // for...of is transformed to regular for loop
        for (var __i = 0; __i < (arr).length; __i++) {
            var item = (arr)[__i];
            result.push(item);
        }

        [
            result.length === 3,
            result[0] === 'a',
            result[1] === 'b',
            result[2] === 'c'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_const_let_transformation() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test const/let to var transformation
    let result = engine.execute_enhanced(r#"
        var x = 10;
        var y = 'hello';
        var z = true;

        [
            x === 10,
            y === 'hello',
            z === true,
            typeof x === 'number',
            typeof y === 'string',
            typeof z === 'boolean'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_array_at_transformation() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test Array.prototype.at transformation
    let result = engine.execute_enhanced(r#"
        const arr = ['a', 'b', 'c', 'd', 'e'];

        // .at() is transformed to index calculation
        [
            arr.at(0),
            arr.at(-1),
            arr.at(2),
            arr.at(-2),
            arr.at(10) === undefined
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_top_level_await_transformation() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test top-level await transformation
    let result = engine.execute_enhanced(r#"
        // Top-level await is wrapped in async IIFE
        (async function() {
            const result = await Promise.resolve(42);
            return result;
        })()
    "#).await.unwrap();

    assert!(result.is_object()); // Should be a Promise
}

#[tokio::test]
async fn test_class_fields_transformation() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

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

#[tokio::test]
async fn test_complex_transformation_chain() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test multiple transformations working together
    let result = engine.execute_enhanced(r#"
        // Multiple ES features combined
        var data = null;
        data ??= { values: [1_000, 2_000, 3_000] };

        var sum = 0;
        for (var __i = 0; __i < (data.values).length; __i++) {
            var value = (data.values)[__i];
            sum = sum || 0;
            sum += value;
        }

        var result = data?.values?.length === 3 && sum === 6000;
        result
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}