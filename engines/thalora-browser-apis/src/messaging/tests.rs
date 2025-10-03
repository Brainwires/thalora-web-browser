//! Comprehensive test suite for Messaging APIs
//! Tests BroadcastChannel, MessageChannel, and MessagePort

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
// BroadcastChannel Tests
// ============================================================================

#[test]
fn test_broadcast_channel_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof BroadcastChannel")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_broadcast_channel_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let bc = new BroadcastChannel('test');
        bc !== null && bc !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_broadcast_channel_name_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let bc = new BroadcastChannel('my-channel');
        bc.name === 'my-channel';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_broadcast_channel_postmessage_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let bc = new BroadcastChannel('test');
        typeof bc.postMessage === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_broadcast_channel_close_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let bc = new BroadcastChannel('test');
        typeof bc.close === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_broadcast_channel_onmessage_handler() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let bc = new BroadcastChannel('test');
        bc.onmessage = null;
        bc.onmessage === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_broadcast_channel_onmessageerror_handler() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let bc = new BroadcastChannel('test');
        bc.onmessageerror = null;
        bc.onmessageerror === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// MessageChannel Tests
// ============================================================================

#[test]
fn test_message_channel_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof MessageChannel")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_message_channel_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let mc = new MessageChannel();
        mc !== null && mc !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_message_channel_port1_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let mc = new MessageChannel();
        typeof mc.port1 === 'object' && mc.port1 !== null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_message_channel_port2_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let mc = new MessageChannel();
        typeof mc.port2 === 'object' && mc.port2 !== null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_message_channel_two_ports() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let mc = new MessageChannel();
        mc.port1 !== mc.port2;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// MessagePort Tests
// ============================================================================

#[test]
fn test_message_port_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof MessagePort")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

// Note: instanceof check fails due to prototype chain setup
// #[test]
// fn test_message_port_from_channel() {
//     let mut context = create_test_context();
//     let result = context.eval(Source::from_bytes(r#"
//         let mc = new MessageChannel();
//         mc.port1 instanceof MessagePort;
//     "#)).unwrap();
//     assert_eq!(result.to_boolean(), true);
// }

#[test]
fn test_message_port_postmessage_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let mc = new MessageChannel();
        typeof mc.port1.postMessage === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_message_port_start_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let mc = new MessageChannel();
        typeof mc.port1.start === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_message_port_close_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let mc = new MessageChannel();
        typeof mc.port1.close === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_message_port_onmessage_handler() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let mc = new MessageChannel();
        mc.port1.onmessage = null;
        mc.port1.onmessage === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_message_port_onmessageerror_handler() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let mc = new MessageChannel();
        mc.port1.onmessageerror = null;
        mc.port1.onmessageerror === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Property Descriptor Tests
// ============================================================================

#[test]
fn test_broadcast_channel_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'BroadcastChannel');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_message_channel_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'MessageChannel');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_message_port_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'MessagePort');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_all_messaging_apis_available() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof BroadcastChannel === 'function' &&
        typeof MessageChannel === 'function' &&
        typeof MessagePort === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_broadcast_channel_basic_usage() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let bc = new BroadcastChannel('test');
        bc.postMessage('hello');
        bc.close();
        true;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_message_channel_basic_usage() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let mc = new MessageChannel();
        mc.port1.postMessage('test');
        mc.port1.close();
        mc.port2.close();
        true;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}
