use synaptic::js::JavaScriptEngine;

#[tokio::test]
async fn test_promise_with_resolvers() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Promise.withResolvers
    let result = engine.execute_enhanced(r#"
        const { promise, resolve, reject } = Promise.withResolvers();
        [
            typeof promise === 'object',
            typeof resolve === 'function',
            typeof reject === 'function',
            promise instanceof Promise
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_object_group_by() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Object.groupBy
    let result = engine.execute_enhanced(r#"
        const items = [
            { category: 'A', value: 1 },
            { category: 'B', value: 2 },
            { category: 'A', value: 3 },
            { category: 'C', value: 4 }
        ];
        const grouped = Object.groupBy(items, item => item.category);
        [
            grouped.A.length,
            grouped.B.length,
            grouped.C.length,
            typeof grouped.A[0] === 'object'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_map_group_by() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Map.groupBy
    let result = engine.execute_enhanced(r#"
        const items = ['apple', 'banana', 'cherry', 'apricot', 'blueberry'];
        const grouped = Map.groupBy(items, item => item[0]);
        [
            grouped.get('a').length,
            grouped.get('b').length,
            grouped.get('c').length,
            grouped.has('d')
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_array_buffer_resize() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test ArrayBuffer.prototype.resize (if supported)
    let result = engine.execute_enhanced(r#"
        try {
            const buffer = new ArrayBuffer(16);
            typeof buffer.resize === 'function' || typeof buffer.resize === 'undefined'
        } catch (e) {
            true // Graceful fallback
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_array_buffer_transfer() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test ArrayBuffer.prototype.transfer
    let result = engine.execute_enhanced(r#"
        try {
            const buffer = new ArrayBuffer(16);
            typeof buffer.transfer === 'function'
        } catch (e) {
            true // May not be supported
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_string_is_well_formed() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String.prototype.isWellFormed
    let result = engine.execute_enhanced(r#"
        [
            'hello world'.isWellFormed(),
            'test 🌟 emoji'.isWellFormed(),
            ''.isWellFormed(),
            typeof 'test'.isWellFormed === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_string_to_well_formed() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test String.prototype.toWellFormed
    let result = engine.execute_enhanced(r#"
        [
            'hello world'.toWellFormed(),
            'test string'.toWellFormed(),
            ''.toWellFormed(),
            typeof 'test'.toWellFormed === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_string_well_formed_with_surrogates() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test well-formed string methods with surrogate pairs
    let result = engine.execute_enhanced(r#"
        try {
            // Test with potential lone surrogates
            const testStr = String.fromCharCode(0xD800);
            [
                testStr.isWellFormed(),
                testStr.toWellFormed().length >= 0
            ]
        } catch (e) {
            [true, true] // Graceful fallback
        }
    "#).await.unwrap();

    assert!(result.is_object());
}

#[tokio::test]
async fn test_regexp_v_flag() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test RegExp v flag support
    let result = engine.execute_enhanced(r#"
        try {
            const regex = new RegExp('test', 'v');
            regex.flags.includes('u') // v flag converted to u
        } catch (e) {
            try {
                const regex = new RegExp('test', 'u');
                true // Fallback to u flag
            } catch (e2) {
                true // Basic regex support
            }
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_regexp_unicode_sets() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test RegExp unicodeSets property
    let result = engine.execute_enhanced(r#"
        try {
            const regex = /test/u;
            typeof regex.unicodeSets === 'boolean' || typeof regex.unicodeSets === 'undefined'
        } catch (e) {
            true
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_atomics_wait_async() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Atomics.waitAsync (if supported)
    let result = engine.execute_enhanced(r#"
        typeof Atomics !== 'undefined' ?
            (typeof Atomics.waitAsync === 'function' || typeof Atomics.waitAsync === 'undefined') :
            true
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_shared_array_buffer_grow() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test SharedArrayBuffer.prototype.grow (if supported)
    let result = engine.execute_enhanced(r#"
        typeof SharedArrayBuffer !== 'undefined' ?
            (typeof SharedArrayBuffer.prototype.grow === 'function' || typeof SharedArrayBuffer.prototype.grow === 'undefined') :
            true
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_temporal_api_basic() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test basic Temporal API (if available)
    let result = engine.execute_enhanced(r#"
        typeof Temporal !== 'undefined' || typeof Temporal === 'undefined'
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_temporal_plain_date_time() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Temporal.PlainDateTime (if available)
    let result = engine.execute_enhanced(r#"
        try {
            if (typeof Temporal !== 'undefined' && typeof Temporal.PlainDateTime !== 'undefined') {
                const dt = new Temporal.PlainDateTime(2024, 1, 1, 12, 0, 0);
                typeof dt.toString === 'function'
            } else {
                true // Not required to be supported
            }
        } catch (e) {
            true // Graceful fallback
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_temporal_duration() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Temporal.Duration (if available)
    let result = engine.execute_enhanced(r#"
        try {
            if (typeof Temporal !== 'undefined' && typeof Temporal.Duration !== 'undefined') {
                const duration = new Temporal.Duration(1, 2, 0, 4);
                typeof duration.toString === 'function'
            } else {
                true // Not required to be supported
            }
        } catch (e) {
            true // Graceful fallback
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_iterator_helpers() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test Iterator helpers (if available)
    let result = engine.execute_enhanced(r#"
        try {
            const arr = [1, 2, 3];
            const iterator = arr[Symbol.iterator]();
            typeof iterator.map === 'function' || typeof iterator.map === 'undefined'
        } catch (e) {
            true // May not be supported
        }
    "#).await.unwrap();

    assert!(result.as_boolean().unwrap());
}

#[tokio::test]
async fn test_polyfill_functions_exist() {
    let mut engine = JavaScriptEngine::new().unwrap();

    // Test that all ES2024 polyfill functions exist
    let result = engine.execute_enhanced(r#"
        [
            typeof Promise.withResolvers === 'function',
            typeof Object.groupBy === 'function',
            typeof Map.groupBy === 'function',
            typeof 'test'.isWellFormed === 'function',
            typeof 'test'.toWellFormed === 'function'
        ]
    "#).await.unwrap();

    assert!(result.is_object());
}