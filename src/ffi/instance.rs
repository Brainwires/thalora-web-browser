//! Core FFI types and lifecycle management.
//!
//! Provides the ThalorInstance struct that holds a persistent tokio runtime
//! and browser instance, bridging async Rust to synchronous C FFI calls.

use std::ffi::{CStr, CString, c_char, c_void};
use std::ptr;
use std::sync::{Arc, Mutex};

use crate::engine::HeadlessWebBrowser;

/// Opaque instance holding the browser and async runtime.
/// Each instance owns its own tokio runtime so that FFI callers
/// (who have no async runtime) can call blocking functions.
pub struct ThalorInstance {
    pub(crate) runtime: tokio::runtime::Runtime,
    pub(crate) browser: Arc<Mutex<HeadlessWebBrowser>>,
    pub(crate) last_error: Mutex<Option<String>>,
}

impl ThalorInstance {
    /// Set the last error message.
    pub(crate) fn set_error(&self, msg: String) {
        if let Ok(mut err) = self.last_error.lock() {
            *err = Some(msg);
        }
    }

    /// Clear the last error.
    pub(crate) fn clear_error(&self) {
        if let Ok(mut err) = self.last_error.lock() {
            *err = None;
        }
    }
}

/// Helper: convert a Rust String into a heap-allocated C string.
/// The caller must free it with `thalora_free_string`.
pub(crate) fn rust_string_to_c(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(cs) => cs.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Helper: convert a C string pointer to a Rust &str.
/// Returns None if the pointer is null or not valid UTF-8.
pub(crate) unsafe fn c_str_to_rust<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }.to_str().ok()
}

// ---------------------------------------------------------------------------
// Lifecycle FFI functions
// ---------------------------------------------------------------------------

/// Create a new Thalora browser instance.
///
/// Returns an opaque pointer that must be passed to all other FFI functions.
/// The caller must eventually call `thalora_destroy` to free resources.
/// Returns null on failure.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_init() -> *mut ThalorInstance {
    // Build a multi-threaded tokio runtime for this instance
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    // Create the browser (HeadlessWebBrowser::new() returns Arc<Mutex<..>>)
    let browser = HeadlessWebBrowser::new();

    let instance = Box::new(ThalorInstance {
        runtime,
        browser,
        last_error: Mutex::new(None),
    });

    Box::into_raw(instance)
}

/// Destroy a Thalora browser instance and free all resources.
///
/// After calling this, the pointer is invalid and must not be used.
/// Passing a null pointer is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_destroy(instance: *mut ThalorInstance) {
    if instance.is_null() {
        return;
    }
    // Safety: we created this pointer in thalora_init via Box::into_raw
    unsafe {
        let _ = Box::from_raw(instance);
    }
}

/// Get the last error message from the instance.
///
/// Returns a pointer to a C string describing the last error, or null if
/// no error has occurred. The returned string is valid until the next FFI
/// call on this instance. The caller must NOT free this string.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_last_error(instance: *const ThalorInstance) -> *const c_char {
    if instance.is_null() {
        return ptr::null();
    }
    let inst = unsafe { &*instance };
    if let Ok(err) = inst.last_error.lock() {
        match err.as_ref() {
            Some(msg) => {
                // We leak a CString here for simplicity — the caller doesn't free it,
                // and it gets replaced on the next error. For a production system
                // we'd use a pre-allocated buffer, but this is fine for FFI.
                match CString::new(msg.as_str()) {
                    Ok(cs) => cs.into_raw() as *const c_char,
                    Err(_) => ptr::null(),
                }
            }
            None => ptr::null(),
        }
    } else {
        ptr::null()
    }
}

/// Free a string that was returned by a Thalora FFI function.
///
/// All `*mut c_char` pointers returned by navigation/interaction functions
/// must be freed with this function. Passing null is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    // Safety: this pointer was created by CString::into_raw in rust_string_to_c
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}
