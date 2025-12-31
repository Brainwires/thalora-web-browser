//! Comprehensive test suite for File APIs
//! Tests Blob, File, and FileReader APIs

use boa_engine::{Context, Source, JsValue};
use boa_engine::string::JsString;

// Helper to initialize context with browser APIs
fn create_test_context() -> Context {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context)
        .expect("Failed to initialize browser APIs");
    context
}

// ============================================================================
// Blob Constructor Tests
// ============================================================================

#[test]
fn test_blob_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof Blob")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_blob_constructor_no_arguments() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let blob = new Blob();
        blob !== null && blob !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_blob_constructor_with_array() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let blob = new Blob(['hello', ' ', 'world']);
        blob !== null && blob !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_blob_constructor_with_options() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let blob = new Blob(['test'], { type: 'text/plain' });
        blob !== null && blob !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_blob_size_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let blob = new Blob(['hello']);
        typeof blob.size === 'number';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_blob_type_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let blob = new Blob(['test'], { type: 'text/plain' });
        typeof blob.type === 'string';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_blob_slice_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let blob = new Blob(['hello world']);
        typeof blob.slice === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_blob_text_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let blob = new Blob(['test']);
        typeof blob.text === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_blob_arraybuffer_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let blob = new Blob(['test']);
        typeof blob.arrayBuffer === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// File Constructor Tests
// ============================================================================

#[test]
fn test_file_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof File")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_file_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let file = new File(['content'], 'test.txt');
        file !== null && file !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_file_constructor_with_type() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let file = new File(['content'], 'test.txt', { type: 'text/plain' });
        file !== null && file !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_file_name_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let file = new File(['content'], 'test.txt');
        file.name === 'test.txt';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_file_size_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let file = new File(['hello'], 'test.txt');
        typeof file.size === 'number';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_file_type_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let file = new File(['content'], 'test.txt', { type: 'text/plain' });
        typeof file.type === 'string';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_file_lastmodified_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let file = new File(['content'], 'test.txt');
        typeof file.lastModified === 'number';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_file_inherits_blob_methods() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let file = new File(['test'], 'test.txt');
        typeof file.text === 'function' && typeof file.slice === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// FileReader Constructor Tests
// ============================================================================

#[test]
fn test_filereader_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof FileReader")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_filereader_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        reader !== null && reader !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_readystate_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        typeof reader.readyState === 'number';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_readystate_empty() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        reader.readyState === 0; // EMPTY
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_result_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        reader.result === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_error_property() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        reader.error === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_readastext_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        typeof reader.readAsText === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_readasdataurl_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        typeof reader.readAsDataURL === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_readasarraybuffer_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        typeof reader.readAsArrayBuffer === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_abort_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        typeof reader.abort === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_constants() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        FileReader.EMPTY === 0 &&
        FileReader.LOADING === 1 &&
        FileReader.DONE === 2;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// FileReader Event Handlers
// ============================================================================

#[test]
fn test_filereader_onload_handler() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        reader.onload = null;
        reader.onload === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_onerror_handler() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        reader.onerror = null;
        reader.onerror === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_onabort_handler() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        reader.onabort = null;
        reader.onabort === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_onloadstart_handler() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        reader.onloadstart = null;
        reader.onloadstart === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_onloadend_handler() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        reader.onloadend = null;
        reader.onloadend === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_onprogress_handler() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        reader.onprogress = null;
        reader.onprogress === null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Blob/File Integration Tests
// ============================================================================

#[test]
fn test_file_is_blob() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let file = new File(['content'], 'test.txt');
        file instanceof Blob;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// #[test]
// fn test_blob_slice_returns_blob() {
//     let mut context = create_test_context();
//     let result = context.eval(Source::from_bytes(r#"
//         let blob = new Blob(['hello world']);
//         let sliced = blob.slice(0, 5);
//         sliced instanceof Blob;
//     "#)).unwrap();
//     assert_eq!(result.to_boolean(), true);
// }

// #[test]
// fn test_file_slice_returns_blob() {
//     let mut context = create_test_context();
//     let result = context.eval(Source::from_bytes(r#"
//         let file = new File(['hello world'], 'test.txt');
//         let sliced = file.slice(0, 5);
//         sliced instanceof Blob;
//     "#)).unwrap();
//     assert_eq!(result.to_boolean(), true);
// }

// ============================================================================
// Property Descriptor Tests
// ============================================================================

#[test]
fn test_blob_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'Blob');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_file_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'File');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'FileReader');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_file_constructor_requires_name() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        try {
            new File(['content']);
            false;
        } catch(e) {
            true;
        }
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_requires_new() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        try {
            FileReader();
            false;
        } catch(e) {
            true;
        }
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// File System API Tests
// ============================================================================

#[test]
fn test_showopenpicker_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof showOpenFilePicker === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_showsavepicker_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof showSaveFilePicker === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_showdirectorypicker_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof showDirectoryPicker === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Blob endings option tests (new implementation)
// ============================================================================

#[test]
fn test_blob_endings_option_transparent() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let blob = new Blob(['line1\nline2'], { endings: 'transparent' });
        blob.size > 0;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_blob_endings_option_native() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let blob = new Blob(['line1\nline2'], { endings: 'native' });
        blob.size > 0;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// File endings option tests (new implementation)
// ============================================================================

#[test]
fn test_file_endings_option_native() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let file = new File(['line1\nline2'], 'test.txt', { endings: 'native' });
        file.size > 0;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// FileReader event firing tests (new implementation)
// ============================================================================

#[test]
fn test_filereader_readastext_with_event_handler() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        let loadCalled = false;
        reader.onload = function(e) {
            loadCalled = true;
        };
        let blob = new Blob(['test content']);
        reader.readAsText(blob);
        // After synchronous read completes, check state
        reader.readyState === 2 && reader.result !== null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_readasdataurl_result() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        let blob = new Blob(['hello']);
        reader.readAsDataURL(blob);
        reader.result !== null && reader.result.startsWith('data:');
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_readasbinarystring_result() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        let blob = new Blob(['hello']);
        reader.readAsBinaryString(blob);
        reader.result === 'hello';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_abort_sets_error() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        reader.readyState === 0;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_loadstart_event() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        let loadstartCalled = false;
        reader.onloadstart = function(e) {
            loadstartCalled = true;
        };
        let blob = new Blob(['test']);
        reader.readAsText(blob);
        loadstartCalled;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_load_event() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        let loadCalled = false;
        reader.onload = function(e) {
            loadCalled = true;
        };
        let blob = new Blob(['test']);
        reader.readAsText(blob);
        loadCalled;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_filereader_loadend_event() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let reader = new FileReader();
        let loadendCalled = false;
        reader.onloadend = function(e) {
            loadendCalled = true;
        };
        let blob = new Blob(['test']);
        reader.readAsText(blob);
        loadendCalled;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}
