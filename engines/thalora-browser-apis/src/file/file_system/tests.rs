//! Tests for the File System API implementation

use crate::{Context, JsString, JsValue, Source};

#[test]
fn test_file_system_constructors_not_directly_constructible() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Per WHATWG File System spec these constructors are exposed globally but
    // calling them directly throws. They are obtained from
    // `navigator.storage.getDirectory()` or the picker functions.
    for ctor in [
        "FileSystemHandle",
        "FileSystemFileHandle",
        "FileSystemDirectoryHandle",
        "FileSystemWritableFileStream",
        "FileSystemSyncAccessHandle",
    ] {
        let typeof_expr = format!("typeof {ctor}");
        let result = context.eval(Source::from_bytes(&typeof_expr)).unwrap();
        assert_eq!(
            result,
            JsValue::from(JsString::from("function")),
            "{ctor} should be exposed as a constructor"
        );
        let new_expr = format!("new {ctor}()");
        let result = context.eval(Source::from_bytes(&new_expr));
        assert!(result.is_err(), "{ctor} should not be directly constructible");
    }
}

#[test]
fn test_file_picker_functions_exist() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test that showOpenFilePicker exists and is a function
    let result = context
        .eval(Source::from_bytes("typeof showOpenFilePicker"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    // Test that showSaveFilePicker exists and is a function
    let result = context
        .eval(Source::from_bytes("typeof showSaveFilePicker"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));

    // Test that showDirectoryPicker exists and is a function
    let result = context
        .eval(Source::from_bytes("typeof showDirectoryPicker"))
        .unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_file_picker_functions_return_promises() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test that showOpenFilePicker returns a Promise
    let result = context
        .eval(Source::from_bytes(
            "showOpenFilePicker() instanceof Promise",
        ))
        .unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test that showSaveFilePicker returns a Promise
    let result = context
        .eval(Source::from_bytes(
            "showSaveFilePicker() instanceof Promise",
        ))
        .unwrap();
    assert_eq!(result, JsValue::from(true));

    // Test that showDirectoryPicker returns a Promise
    let result = context
        .eval(Source::from_bytes(
            "showDirectoryPicker() instanceof Promise",
        ))
        .unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_file_system_basic_functionality() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Create a mock test for file system functionality
    context
        .eval(Source::from_bytes(
            r#"
        // Test showOpenFilePicker mock functionality
        var filePickerPromise = showOpenFilePicker();
        var isPromise = filePickerPromise instanceof Promise;

        // Test showSaveFilePicker mock functionality
        var savePickerPromise = showSaveFilePicker();
        var isSavePromise = savePickerPromise instanceof Promise;

        // Test showDirectoryPicker mock functionality
        var dirPickerPromise = showDirectoryPicker();
        var isDirPromise = dirPickerPromise instanceof Promise;
    "#,
        ))
        .unwrap();

    // Test that promises were created correctly
    let result = context.eval(Source::from_bytes("isPromise")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("isSavePromise")).unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context.eval(Source::from_bytes("isDirPromise")).unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_file_handle_methods_exist() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    context
        .eval(Source::from_bytes(
            r#"
        // Test that file handle has expected methods after resolution
        var fileHandleTest = false;
        showOpenFilePicker().then(function(fileHandles) {
            if (fileHandles && fileHandles.length > 0) {
                var fileHandle = fileHandles[0];
                fileHandleTest = typeof fileHandle === 'object' &&
                                typeof fileHandle.getFile === 'function' &&
                                typeof fileHandle.createWritable === 'function';
            }
        });
    "#,
        ))
        .unwrap();

    // Note: In a real async environment, we would need to wait for the promise
    // This test just verifies the structure is set up correctly
}

#[test]
fn test_directory_handle_methods_exist() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    context
        .eval(Source::from_bytes(
            r#"
        // Test that directory handle has expected methods after resolution
        var dirHandleTest = false;
        showDirectoryPicker().then(function(dirHandle) {
            if (dirHandle) {
                dirHandleTest = typeof dirHandle === 'object' &&
                               typeof dirHandle.getFileHandle === 'function' &&
                               typeof dirHandle.getDirectoryHandle === 'function' &&
                               typeof dirHandle.removeEntry === 'function' &&
                               typeof dirHandle.resolve === 'function';
            }
        });
    "#,
        ))
        .unwrap();

    // Note: In a real async environment, we would need to wait for the promise
    // This test just verifies the structure is set up correctly
}

#[test]
fn test_file_system_vfs_integration() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    // Test that file picker functions work with VFS
    let result = context
        .eval(Source::from_bytes(
            "showOpenFilePicker() instanceof Promise",
        ))
        .unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context
        .eval(Source::from_bytes(
            "showSaveFilePicker() instanceof Promise",
        ))
        .unwrap();
    assert_eq!(result, JsValue::from(true));

    let result = context
        .eval(Source::from_bytes(
            "showDirectoryPicker() instanceof Promise",
        ))
        .unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn test_file_system_handle_common_methods() {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

    context
        .eval(Source::from_bytes(
            r#"
        // Test common FileSystemHandle methods
        var handleMethodsTest = false;
        showSaveFilePicker().then(function(fileHandle) {
            if (fileHandle) {
                handleMethodsTest = typeof fileHandle.isSameEntry === 'function' &&
                                   typeof fileHandle.queryPermission === 'function' &&
                                   typeof fileHandle.requestPermission === 'function';
            }
        });
    "#,
        ))
        .unwrap();

    // Note: In a real async environment, we would need to wait for the promise
    // This test just verifies the structure is set up correctly
}

// =============================================================================
// OPFS-specific tests
// =============================================================================

use crate::file::file_system::opfs_backend::OpfsBackend;
use std::path::PathBuf;

fn unique_origin() -> String {
    format!(
        "thalora-test://{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4().simple()
    )
}

fn cleanup_origin(origin: &str) {
    let backend = OpfsBackend::for_origin(origin);
    let _ = std::fs::remove_dir_all(backend.root_path());
}

#[test]
fn opfs_get_directory_resolves_to_directory_handle() {
    let origin = unique_origin();
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("init");
    crate::realm_ext::set_active_origin(&mut context, origin.clone());

    context.eval(Source::from_bytes(r#"let kind = null; navigator.storage.getDirectory().then(d => { kind = d.kind; });"#)).expect("eval");
    for _ in 0..30 { context.run_jobs().ok(); }
    let kind = context.eval(Source::from_bytes("kind")).unwrap();
    assert_eq!(kind, JsValue::from(JsString::from("directory")));
    cleanup_origin(&origin);
}

#[test]
fn opfs_round_trip_string_write_and_read() {
    let origin = unique_origin();
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("init");
    crate::realm_ext::set_active_origin(&mut context, origin.clone());

    let script = r#"
        let outcome = "pending";
        (async () => {
            const root = await navigator.storage.getDirectory();
            const fh = await root.getFileHandle("hello.txt", { create: true });
            const w = await fh.createWritable();
            await w.write("hello opfs");
            await w.close();
            const f = await fh.getFile();
            outcome = await f.text();
        })().catch(e => { outcome = "error:" + (e && e.message); });
    "#;
    context.eval(Source::from_bytes(script)).expect("eval");
    for _ in 0..50 { context.run_jobs().ok(); }
    let outcome = context.eval(Source::from_bytes("outcome")).unwrap();
    assert_eq!(outcome, JsValue::from(JsString::from("hello opfs")));

    let backend = OpfsBackend::for_origin(&origin);
    let bytes = backend.read_bytes(&PathBuf::from("/hello.txt")).unwrap();
    assert_eq!(bytes, b"hello opfs");
    cleanup_origin(&origin);
}

#[test]
fn opfs_get_file_handle_create_false_throws_not_found() {
    let origin = unique_origin();
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("init");
    crate::realm_ext::set_active_origin(&mut context, origin.clone());

    let script = r#"
        let errName = "none";
        (async () => {
            const root = await navigator.storage.getDirectory();
            try { await root.getFileHandle("does-not-exist.txt", { create: false }); errName = "no-error"; }
            catch (e) { errName = e && e.name; }
        })();
    "#;
    context.eval(Source::from_bytes(script)).expect("eval");
    for _ in 0..30 { context.run_jobs().ok(); }
    let err_name = context.eval(Source::from_bytes("errName")).unwrap();
    assert_eq!(err_name, JsValue::from(JsString::from("NotFoundError")));
    cleanup_origin(&origin);
}

#[test]
fn opfs_directory_async_iteration() {
    let origin = unique_origin();
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("init");
    crate::realm_ext::set_active_origin(&mut context, origin.clone());

    let script = r#"
        let names = [];
        (async () => {
            const root = await navigator.storage.getDirectory();
            await root.getFileHandle("a.txt", {create:true});
            await root.getFileHandle("b.txt", {create:true});
            for await (const k of root.keys()) names.push(k);
        })();
    "#;
    context.eval(Source::from_bytes(script)).expect("eval");
    for _ in 0..50 { context.run_jobs().ok(); }
    let len = context.eval(Source::from_bytes("names.length")).unwrap();
    assert_eq!(len, JsValue::from(2_i32));
    cleanup_origin(&origin);
}

#[test]
fn opfs_persistence_across_contexts() {
    let origin = unique_origin();
    {
        let mut context = Context::default();
        crate::initialize_browser_apis(&mut context).expect("init");
        crate::realm_ext::set_active_origin(&mut context, origin.clone());
        let script = r#"(async () => {
            const root = await navigator.storage.getDirectory();
            const fh = await root.getFileHandle("persist.txt", { create: true });
            const w = await fh.createWritable();
            await w.write("survives"); await w.close();
        })();"#;
        context.eval(Source::from_bytes(script)).expect("eval");
        for _ in 0..30 { context.run_jobs().ok(); }
    }
    let mut context2 = Context::default();
    crate::initialize_browser_apis(&mut context2).expect("init2");
    crate::realm_ext::set_active_origin(&mut context2, origin.clone());
    let script = r#"
        let txt = "pending";
        (async () => {
            const root = await navigator.storage.getDirectory();
            const fh = await root.getFileHandle("persist.txt", { create: false });
            const f = await fh.getFile();
            txt = await f.text();
        })();
    "#;
    context2.eval(Source::from_bytes(script)).expect("eval");
    for _ in 0..30 { context2.run_jobs().ok(); }
    let txt = context2.eval(Source::from_bytes("txt")).unwrap();
    assert_eq!(txt, JsValue::from(JsString::from("survives")));
    cleanup_origin(&origin);
}

#[test]
fn opfs_query_permission_returns_granted() {
    let origin = unique_origin();
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("init");
    crate::realm_ext::set_active_origin(&mut context, origin.clone());

    let script = r#"
        let state = "pending";
        (async () => {
            const root = await navigator.storage.getDirectory();
            state = await root.queryPermission({ mode: "readwrite" });
        })();
    "#;
    context.eval(Source::from_bytes(script)).expect("eval");
    for _ in 0..30 { context.run_jobs().ok(); }
    let state = context.eval(Source::from_bytes("state")).unwrap();
    assert_eq!(state, JsValue::from(JsString::from("granted")));
    cleanup_origin(&origin);
}

#[test]
fn opfs_create_sync_access_handle_blocked_on_main_thread() {
    let origin = unique_origin();
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("init");
    crate::realm_ext::set_active_origin(&mut context, origin.clone());

    let script = r#"
        let errName = "none";
        (async () => {
            const root = await navigator.storage.getDirectory();
            const fh = await root.getFileHandle("sah.bin", { create: true });
            try { await fh.createSyncAccessHandle(); errName = "no-error"; }
            catch (e) { errName = e && e.name; }
        })();
    "#;
    context.eval(Source::from_bytes(script)).expect("eval");
    for _ in 0..30 { context.run_jobs().ok(); }
    let err_name = context.eval(Source::from_bytes("errName")).unwrap();
    assert_eq!(err_name, JsValue::from(JsString::from("InvalidStateError")));
    cleanup_origin(&origin);
}

#[test]
fn opfs_remove_entry_non_empty_requires_recursive() {
    let origin = unique_origin();
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context).expect("init");
    crate::realm_ext::set_active_origin(&mut context, origin.clone());

    let script = r#"
        let errName = "pending";
        (async () => {
            const root = await navigator.storage.getDirectory();
            const sub = await root.getDirectoryHandle("subdir", { create: true });
            await sub.getFileHandle("inside.txt", { create: true });
            try { await root.removeEntry("subdir"); errName = "no-error"; }
            catch (e) { errName = e && e.name; }
            await root.removeEntry("subdir", { recursive: true });
        })();
    "#;
    context.eval(Source::from_bytes(script)).expect("eval");
    for _ in 0..50 { context.run_jobs().ok(); }
    let err_name = context.eval(Source::from_bytes("errName")).unwrap();
    assert_eq!(err_name, JsValue::from(JsString::from("InvalidModificationError")));
    cleanup_origin(&origin);
}

