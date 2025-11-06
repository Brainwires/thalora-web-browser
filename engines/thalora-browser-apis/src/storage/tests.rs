//! Comprehensive test suite for Storage APIs
//! Tests Storage (localStorage/sessionStorage), StorageManager, and StorageEvent

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
// Storage Constructor Tests
// ============================================================================

#[test]
fn test_storage_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Storage")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

// ============================================================================
// localStorage Tests
// ============================================================================

#[test]
fn test_localstorage_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof localStorage === 'object';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_localstorage_is_storage() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage instanceof Storage;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_localstorage_setitem() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.setItem('test', 'value');
        localStorage.getItem('test') === 'value';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_localstorage_getitem() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.setItem('key', 'value');
        localStorage.getItem('key') === 'value';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_localstorage_getitem_nonexistent() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.getItem('nonexistent') === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_localstorage_removeitem() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.setItem('test', 'value');
        localStorage.removeItem('test');
        localStorage.getItem('test') === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_localstorage_clear() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.setItem('key1', 'value1');
        localStorage.setItem('key2', 'value2');
        localStorage.clear();
        localStorage.getItem('key1') === null && localStorage.getItem('key2') === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_localstorage_length() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.clear();
        localStorage.setItem('key1', 'value1');
        localStorage.setItem('key2', 'value2');
        localStorage.length === 2;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_localstorage_key() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.clear();
        localStorage.setItem('test', 'value');
        localStorage.key(0) !== null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_localstorage_key_out_of_bounds() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.clear();
        localStorage.key(100) === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// sessionStorage Tests
// ============================================================================

#[test]
fn test_sessionstorage_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof sessionStorage === 'object';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_sessionstorage_is_storage() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        sessionStorage instanceof Storage;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_sessionstorage_setitem() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        sessionStorage.setItem('test', 'value');
        sessionStorage.getItem('test') === 'value';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_sessionstorage_independent_from_localstorage() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.clear();
        sessionStorage.clear();
        localStorage.setItem('key', 'local');
        sessionStorage.setItem('key', 'session');
        localStorage.getItem('key') === 'local' && sessionStorage.getItem('key') === 'session';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Storage Methods Tests
// ============================================================================

#[test]
fn test_storage_setitem_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof localStorage.setItem === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storage_getitem_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof localStorage.getItem === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storage_removeitem_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof localStorage.removeItem === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storage_clear_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof localStorage.clear === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storage_key_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof localStorage.key === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Storage Value Conversion Tests
// ============================================================================

#[test]
fn test_storage_stores_as_string() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.setItem('number', 123);
        localStorage.getItem('number') === '123';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storage_boolean_conversion() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.setItem('bool', true);
        localStorage.getItem('bool') === 'true';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storage_object_conversion() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.setItem('obj', {a: 1});
        localStorage.getItem('obj') === '[object Object]';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// StorageEvent Tests
// ============================================================================

#[test]
fn test_storageevent_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof StorageEvent")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_storageevent_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let event = new StorageEvent('storage');
        event !== null && event !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storageevent_type_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let event = new StorageEvent('storage');
        event.type === 'storage';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storageevent_key_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let event = new StorageEvent('storage', { key: 'test' });
        event.key === 'test';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storageevent_oldvalue_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let event = new StorageEvent('storage', { oldValue: 'old' });
        event.oldValue === 'old';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storageevent_newvalue_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let event = new StorageEvent('storage', { newValue: 'new' });
        event.newValue === 'new';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storageevent_url_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let event = new StorageEvent('storage', { url: 'http://example.com' });
        event.url === 'http://example.com';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storageevent_storagearea_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let event = new StorageEvent('storage', { storageArea: localStorage });
        event.storageArea === localStorage;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// StorageManager Tests
// ============================================================================

#[test]
fn test_storagemanager_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof StorageManager")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_navigator_storage_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof navigator.storage === 'object';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storagemanager_estimate_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof navigator.storage.estimate === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storagemanager_persist_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof navigator.storage.persist === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storagemanager_persisted_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof navigator.storage.persisted === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Property Descriptor Tests
// ============================================================================

#[test]
fn test_storage_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'Storage');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_localstorage_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'localStorage');
        desc !== undefined && typeof desc.value === 'object';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_sessionstorage_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'sessionStorage');
        desc !== undefined && typeof desc.value === 'object';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storageevent_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'StorageEvent');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storagemanager_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'StorageManager');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_storage_persistence() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.setItem('persist', 'test');
        let value = localStorage.getItem('persist');
        value === 'test';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storage_iteration() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.clear();
        localStorage.setItem('a', '1');
        localStorage.setItem('b', '2');
        localStorage.setItem('c', '3');
        let count = 0;
        for (let i = 0; i < localStorage.length; i++) {
            if (localStorage.key(i)) count++;
        }
        count === 3;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storage_overwrite() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.setItem('key', 'value1');
        localStorage.setItem('key', 'value2');
        localStorage.getItem('key') === 'value2';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storage_empty_string_value() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.setItem('empty', '');
        localStorage.getItem('empty') === '';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storage_null_handling() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.setItem('null', null);
        localStorage.getItem('null') === 'null';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_storage_undefined_handling() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        localStorage.setItem('undef', undefined);
        localStorage.getItem('undef') === 'undefined';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}
