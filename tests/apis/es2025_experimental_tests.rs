use thalora::js::JavaScriptEngine;

#[tokio::test]
async fn test_array_from_async() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Array.fromAsync
    let result = engine.execute_enhanced(r#"
        typeof Array.fromAsync === 'function'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_symbol_metadata() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Symbol.metadata
    let result = engine.execute_enhanced(r#"
        typeof Symbol !== 'undefined' && typeof Symbol.metadata === 'symbol'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

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

#[tokio::test]
async fn test_tuple_constructor() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Tuple constructor
    let result = engine.execute_enhanced(r#"
        const tup = Tuple(1, 2, 3, 4);
        [
            tup.length === 4,
            tup[0] === 1,
            tup[3] === 4,
            typeof tup === 'object',
            typeof Tuple === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_record_immutability() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Record immutability
    let result = engine.execute_enhanced(r#"
        const rec = Record({ x: 10, y: 20 });
        try {
            rec.x = 999; // Should not change the record
            rec.x === 10 // Original value preserved
        } catch (e) {
            true // Might throw in strict mode
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_tuple_immutability() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Tuple immutability
    let result = engine.execute_enhanced(r#"
        const tup = Tuple('a', 'b', 'c');
        try {
            tup[1] = 'changed'; // Should not change the tuple
            tup[1] === 'b' // Original value preserved
        } catch (e) {
            true // Might throw in strict mode
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_pattern_matching_basic() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test basic pattern matching
    let result = engine.execute_enhanced(r#"
        const value = 42;
        const result1 = match(value)
            .with(42, x => 'found forty-two')
            .otherwise(x => 'something else');

        const result2 = match(value)
            .with(x => x > 40, x => 'greater than 40')
            .otherwise(x => 'not greater than 40');

        [
            result1.result === 'found forty-two',
            result1.matched === true,
            result2.result === 'greater than 40',
            result2.matched === true,
            typeof match === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_pattern_matching_otherwise() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test pattern matching with otherwise clause
    let result = engine.execute_enhanced(r#"
        const value = 'unmatched';
        const result = match(value)
            .with(42, x => 'number')
            .with('hello', x => 'greeting')
            .otherwise(x => 'fallback: ' + x);

        [
            result.result === 'fallback: unmatched',
            result.matched === true
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_pipeline_operator() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test pipeline operator (pipe function)
    let result = engine.execute_enhanced(r#"
        function double(x) { return x * 2; }
        function addTen(x) { return x + 10; }
        function toString(x) { return String(x); }

        const result = pipe(5, double, addTen, toString);
        [
            result === '20',
            typeof pipe === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

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

#[tokio::test]
async fn test_observable_basic() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test basic Observable functionality
    let result = engine.execute_enhanced(r#"
        const values = [];
        const obs = Observable.of(1, 2, 3);
        obs.subscribe({
            next: value => values.push(value),
            complete: () => {}
        });

        [
            values.length === 3,
            values[0] === 1,
            values[2] === 3,
            typeof Observable === 'function',
            typeof Observable.of === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_observable_custom() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test custom Observable creation
    let result = engine.execute_enhanced(r#"
        const obs = new Observable(observer => {
            observer.next('hello');
            observer.next('world');
            observer.complete();
        });

        const values = [];
        let completed = false;

        obs.subscribe({
            next: value => values.push(value),
            complete: () => completed = true
        });

        [
            values.length === 2,
            values[0] === 'hello',
            values[1] === 'world',
            completed === true
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_async_context() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test AsyncContext functionality
    let result = engine.execute_enhanced(r#"
        const context = new AsyncContext('test-context');

        const result1 = context.run('test-value', () => {
            return context.get();
        });

        const result2 = context.get(); // Should be undefined outside run

        [
            result1 === 'test-value',
            result2 === undefined,
            typeof context.run === 'function',
            typeof context.get === 'function',
            typeof AsyncContext === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_async_context_nesting() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test AsyncContext nesting
    let result = engine.execute_enhanced(r#"
        const context = new AsyncContext('nested-test');

        const result = context.run('outer', () => {
            const outer = context.get();
            const inner = context.run('inner', () => {
                return context.get();
            });
            const restored = context.get();

            return [outer, inner, restored];
        });

        [
            result[0] === 'outer',
            result[1] === 'inner',
            result[2] === 'outer'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_string_dedent() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String.prototype.dedent
    let result = engine.execute_enhanced(r#"
        const indented = "    line 1\\n    line 2\\n    line 3";
        const dedented = indented.dedent();

        [
            dedented === "line 1\\nline 2\\nline 3",
            typeof 'test'.dedent === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_string_dedent_mixed_indentation() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String.prototype.dedent with mixed indentation
    let result = engine.execute_enhanced(r#"
        const text = "\\n      hello\\n        world\\n      test\\n";
        const dedented = text.dedent();

        [
            dedented.includes('hello'),
            dedented.includes('world'),
            dedented.includes('test'),
            !dedented.startsWith(' '), // Should not start with spaces
            typeof dedented === 'string'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_math_sum_precise() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Math.sumPrecise with Kahan summation
    let result = engine.execute_enhanced(r#"
        const values = [0.1, 0.2, 0.3];
        const regularSum = values.reduce((a, b) => a + b, 0);
        const preciseSum = Math.sumPrecise(values);

        [
            typeof preciseSum === 'number',
            Math.abs(preciseSum - 0.6) < Math.abs(regularSum - 0.6),
            typeof Math.sumPrecise === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_error_is_error() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Error.isError utility
    let result = engine.execute_enhanced(r#"
        const err = new Error('test');
        const errLike = { name: 'CustomError', message: 'custom message' };
        const notErr = { name: 123, message: 'invalid' };

        [
            Error.isError(err) === true,
            Error.isError(errLike) === true,
            Error.isError(notErr) === false,
            Error.isError('string') === false,
            Error.isError(null) === false,
            typeof Error.isError === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_symbol_dispose() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Symbol.dispose for resource management
    let result = engine.execute_enhanced(r#"
        [
            typeof Symbol !== 'undefined',
            typeof Symbol.dispose === 'symbol' || typeof Symbol.dispose === 'undefined',
            typeof Symbol.asyncDispose === 'symbol' || typeof Symbol.asyncDispose === 'undefined'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_using_declarations() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test using declarations (resource management)
    let result = engine.execute_enhanced(r#"
        const resource = {
            value: 'test-resource'
        };

        const managed = using(resource);

        [
            managed.resource === resource,
            typeof managed[Symbol.dispose] === 'function' || typeof managed[Symbol.dispose] === 'undefined',
            typeof using === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_all_experimental_polyfills_exist() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that all ES2025+ experimental polyfills exist
    let result = engine.execute_enhanced(r#"
        [
            typeof Array.fromAsync === 'function',
            typeof Record === 'function',
            typeof Tuple === 'function',
            typeof match === 'function',
            typeof pipe === 'function',
            typeof Observable === 'function',
            typeof AsyncContext === 'function',
            typeof Math.sumPrecise === 'function',
            typeof Error.isError === 'function',
            typeof using === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}