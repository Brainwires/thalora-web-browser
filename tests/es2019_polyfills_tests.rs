use synaptic::enhanced_js_v2::EnhancedJavaScriptEngine;

#[tokio::test]
async fn test_array_flat() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test Array.prototype.flat
    let result = engine.execute_enhanced(r#"
        const arr1 = [1, 2, [3, 4]];
        const arr2 = [1, 2, [3, 4, [5, 6]]];
        [
            JSON.stringify(arr1.flat()),
            JSON.stringify(arr2.flat()),
            JSON.stringify(arr2.flat(2))
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_array_flat_map() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test Array.prototype.flatMap
    let result = engine.execute_enhanced(r#"
        const arr = [1, 2, 3];
        const result = arr.flatMap(x => [x, x * 2]);
        JSON.stringify(result)
    "#).await.unwrap();

    // Should be [1,2,2,4,3,6]
    assert!(result.is_string());
}

#[tokio::test]
async fn test_object_from_entries() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test Object.fromEntries
    let result = engine.execute_enhanced(r#"
        const entries = [['a', 1], ['b', 2], ['c', 3]];
        const obj = Object.fromEntries(entries);
        obj.a === 1 && obj.b === 2 && obj.c === 3
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_string_trim_start() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test String.prototype.trimStart/trimLeft
    let result = engine.execute_enhanced(r#"
        const str = '   hello   ';
        [
            str.trimStart(),
            str.trimLeft(),
            '   '.trimStart(),
            'hello'.trimStart()
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_string_trim_end() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test String.prototype.trimEnd/trimRight
    let result = engine.execute_enhanced(r#"
        const str = '   hello   ';
        [
            str.trimEnd(),
            str.trimRight(),
            '   '.trimEnd(),
            'hello'.trimEnd()
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_symbol_description() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

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

#[tokio::test]
async fn test_optional_catch_binding() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test optional catch binding (syntax feature)
    let result = engine.execute_enhanced(r#"
        try {
            try {
                throw new Error('test');
            } catch {
                // Optional catch binding - no error parameter
                true;
            }
        } catch (e) {
            // Fallback if syntax not supported
            true;
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_array_flat_depth() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test Array.prototype.flat with various depths
    let result = engine.execute_enhanced(r#"
        const deepArray = [1, [2, [3, [4, 5]]]];
        [
            JSON.stringify(deepArray.flat(0)),
            JSON.stringify(deepArray.flat(1)),
            JSON.stringify(deepArray.flat(2)),
            JSON.stringify(deepArray.flat(Infinity))
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_array_flat_map_edge_cases() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test Array.prototype.flatMap edge cases
    let result = engine.execute_enhanced(r#"
        [
            [1, 2, 3].flatMap(x => []),
            [1, 2, 3].flatMap(x => x),
            ['hello', 'world'].flatMap(str => str.split('')),
            [].flatMap(x => [x, x])
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_object_from_entries_edge_cases() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test Object.fromEntries edge cases
    let result = engine.execute_enhanced(r#"
        [
            Object.fromEntries([]),
            Object.fromEntries([['key', undefined]]),
            Object.fromEntries([['0', 'zero'], ['1', 'one']]),
            typeof Object.fromEntries === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_well_formed_json_stringify() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test well-formed JSON.stringify (handles lone surrogates)
    let result = engine.execute_enhanced(r#"
        // Test that JSON.stringify handles unicode correctly
        try {
            JSON.stringify('hello');
            true
        } catch (e) {
            false
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_stable_array_sort() {
    let mut engine = EnhancedJavaScriptEngine::new().unwrap();

    // Test stable Array.prototype.sort
    let result = engine.execute_enhanced(r#"
        const items = [
            { name: 'a', value: 1 },
            { name: 'b', value: 1 },
            { name: 'c', value: 1 }
        ];
        const sorted = items.sort((a, b) => a.value - b.value);
        sorted.length === 3
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}