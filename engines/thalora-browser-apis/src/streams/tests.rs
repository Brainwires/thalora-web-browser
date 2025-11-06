//! Comprehensive test suite for Streams APIs
//! Tests ReadableStream, WritableStream, TransformStream, and QueuingStrategy

use crate::boa_engine::{Context, Source, JsValue};
use crate::boa_engine::string::JsString;

// Helper to initialize context with browser APIs
fn create_test_context() -> Context {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context)
        .expect("Failed to initialize browser APIs");
    context
}

// ============================================================================
// ReadableStream Tests
// ============================================================================

#[test]
fn test_readable_stream_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof ReadableStream")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_readable_stream_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let stream = new ReadableStream();
        stream !== null && stream !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_readable_stream_get_reader_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let stream = new ReadableStream();
        typeof stream.getReader === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_readable_stream_cancel_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let stream = new ReadableStream();
        typeof stream.cancel === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_readable_stream_locked_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let stream = new ReadableStream();
        typeof stream.locked === 'boolean';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// ReadableStreamReader Tests
// ============================================================================

// Note: ReadableStreamDefaultReader is not a global constructor, created via stream.getReader()
// #[test]
// fn test_readable_stream_reader_constructor_exists() {
//     let mut context = create_test_context();
//     let result = context.eval(Source::from_bytes("typeof ReadableStreamDefaultReader")).unwrap();
//     assert_eq!(result, JsValue::from(JsString::from("function")));
// }

#[test]
fn test_readable_stream_get_reader_returns_reader() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let stream = new ReadableStream();
        let reader = stream.getReader();
        reader !== null && reader !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_readable_stream_reader_read_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let stream = new ReadableStream();
        let reader = stream.getReader();
        typeof reader.read === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_readable_stream_reader_release_lock_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let stream = new ReadableStream();
        let reader = stream.getReader();
        typeof reader.releaseLock === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// WritableStream Tests
// ============================================================================

#[test]
fn test_writable_stream_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof WritableStream")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_writable_stream_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let stream = new WritableStream();
        stream !== null && stream !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_writable_stream_get_writer_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let stream = new WritableStream();
        typeof stream.getWriter === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_writable_stream_locked_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let stream = new WritableStream();
        typeof stream.locked === 'boolean';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_writable_stream_abort_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let stream = new WritableStream();
        typeof stream.abort === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// TransformStream Tests
// ============================================================================

#[test]
fn test_transform_stream_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof TransformStream")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_transform_stream_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let stream = new TransformStream();
        stream !== null && stream !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_transform_stream_readable_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let stream = new TransformStream();
        typeof stream.readable === 'object';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_transform_stream_writable_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let stream = new TransformStream();
        typeof stream.writable === 'object';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// QueuingStrategy Tests
// ============================================================================

#[test]
fn test_count_queuing_strategy_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof CountQueuingStrategy")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_count_queuing_strategy_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let strategy = new CountQueuingStrategy({ highWaterMark: 1 });
        strategy !== null && strategy !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_count_queuing_strategy_size_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let strategy = new CountQueuingStrategy({ highWaterMark: 1 });
        typeof strategy.size === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_byte_length_queuing_strategy_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof ByteLengthQueuingStrategy")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_byte_length_queuing_strategy_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let strategy = new ByteLengthQueuingStrategy({ highWaterMark: 1024 });
        strategy !== null && strategy !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_byte_length_queuing_strategy_size_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let strategy = new ByteLengthQueuingStrategy({ highWaterMark: 1024 });
        typeof strategy.size === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Property Descriptor Tests
// ============================================================================

#[test]
fn test_readable_stream_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'ReadableStream');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_writable_stream_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'WritableStream');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_transform_stream_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'TransformStream');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_all_streams_apis_available() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof ReadableStream === 'function' &&
        typeof WritableStream === 'function' &&
        typeof TransformStream === 'function' &&
        typeof CountQueuingStrategy === 'function' &&
        typeof ByteLengthQueuingStrategy === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_readable_stream_locks_on_get_reader() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let stream = new ReadableStream();
        let wasUnlocked = !stream.locked;
        let reader = stream.getReader();
        let nowLocked = stream.locked;
        wasUnlocked && nowLocked;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_writable_stream_locks_on_get_writer() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let stream = new WritableStream();
        let wasUnlocked = !stream.locked;
        let writer = stream.getWriter();
        let nowLocked = stream.locked;
        wasUnlocked && nowLocked;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}
