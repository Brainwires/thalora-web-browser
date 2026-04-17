//! Core FFI types and lifecycle management.
//!
//! Provides the ThalorInstance struct that holds a persistent tokio runtime
//! and browser instance, bridging async Rust to synchronous C FFI calls.

use std::ffi::{CStr, CString, c_char, c_void};
use std::ptr;
use std::rc::Rc;
use std::sync::Mutex;

#[cfg(unix)]
extern crate libc;

use crate::engine::HeadlessWebBrowser;
use crate::engine::browser::types::NavigationMode;

/// Opaque instance holding the browser and async runtime.
/// Each instance owns its own tokio runtime so that FFI callers
/// (who have no async runtime) can call blocking functions.
pub struct ThalorInstance {
    pub(crate) runtime: tokio::runtime::Runtime,
    pub(crate) browser: Rc<Mutex<HeadlessWebBrowser>>,
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

/// Helper: safely convert a `*mut ThalorInstance` to `Option<&ThalorInstance>`.
/// Returns `None` if the pointer is null.
pub(crate) fn instance_ref(ptr: *mut ThalorInstance) -> Option<&'static ThalorInstance> {
    if ptr.is_null() {
        None
    } else {
        // Safety: caller guarantees the pointer was produced by `thalora_init`
        // and has not been destroyed yet.
        Some(unsafe { &*ptr })
    }
}

/// Helper: safely convert a `*const ThalorInstance` to `Option<&ThalorInstance>`.
/// Returns `None` if the pointer is null.
pub(crate) fn instance_ref_const(ptr: *const ThalorInstance) -> Option<&'static ThalorInstance> {
    if ptr.is_null() {
        None
    } else {
        // Safety: caller guarantees the pointer was produced by `thalora_init`
        // and has not been destroyed yet.
        Some(unsafe { &*ptr })
    }
}

/// Helper: reclaim a boxed `ThalorInstance` from a raw pointer.
/// Returns `None` if the pointer is null.
pub(crate) fn instance_into_box(ptr: *mut ThalorInstance) -> Option<Box<ThalorInstance>> {
    if ptr.is_null() {
        None
    } else {
        // Safety: pointer was created by `Box::into_raw` in `thalora_init`.
        Some(unsafe { Box::from_raw(ptr) })
    }
}

/// Helper: reclaim a `CString` from a raw `*mut c_char` pointer.
/// Returns `None` if the pointer is null.
pub(crate) fn reclaim_c_string(ptr: *mut c_char) -> Option<CString> {
    if ptr.is_null() {
        None
    } else {
        // Safety: pointer was created by `CString::into_raw`.
        Some(unsafe { CString::from_raw(ptr) })
    }
}

/// Helper: convert a C string pointer to a Rust &str (safe wrapper).
/// Returns None if the pointer is null or not valid UTF-8.
pub(crate) fn c_str_to_rust_safe<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    // Safety: caller guarantees the pointer is valid and null-terminated.
    unsafe { CStr::from_ptr(ptr) }.to_str().ok()
}

// ---------------------------------------------------------------------------
// Crash signal handler
// ---------------------------------------------------------------------------

/// Install Unix signal handlers for SIGSEGV, SIGBUS, and SIGABRT.
///
/// When the Boa JS engine crashes (e.g. GC corruption → SIGSEGV), the default
/// OS behavior is to generate a crash report and show a dialog to the user.
/// Instead, we handle the signal ourselves: log the signal number and call
/// `_exit(0)`. Exiting with code 0 prevents macOS CrashReporter from activating,
/// and the BrowserController's PID-watcher detects the exit and relaunches the GUI.
///
/// SAFETY: Signal handlers must only call async-signal-safe functions.
/// `libc::write` and `libc::_exit` are both async-signal-safe.
#[cfg(unix)]
fn install_crash_handlers() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static INSTALLED: AtomicBool = AtomicBool::new(false);
    if INSTALLED.swap(true, Ordering::SeqCst) {
        return; // Only install once per process
    }

    extern "C" fn crash_handler(sig: libc::c_int) {
        // Write a short message using write() — the only safe I/O in a signal handler.
        let msg: &[u8] = match sig {
            libc::SIGSEGV => b"[thalora] Caught SIGSEGV - exiting cleanly\n",
            libc::SIGBUS => b"[thalora] Caught SIGBUS - exiting cleanly\n",
            libc::SIGABRT => b"[thalora] Caught SIGABRT - exiting cleanly\n",
            _ => b"[thalora] Caught fatal signal - exiting cleanly\n",
        };
        unsafe {
            libc::write(2, msg.as_ptr() as *const libc::c_void, msg.len());
        }
        // Exit with 0 so macOS CrashReporter stays silent.
        // The BrowserController PID-watcher detects any exit and relaunches.
        unsafe {
            libc::_exit(0);
        }
    }

    let signals = [libc::SIGSEGV, libc::SIGBUS, libc::SIGABRT];
    for &sig in &signals {
        unsafe {
            let mut sa: libc::sigaction = std::mem::zeroed();
            sa.sa_sigaction = crash_handler as libc::sighandler_t;
            libc::sigemptyset(&mut sa.sa_mask);
            // SA_RESETHAND: restore default after first delivery (prevents infinite loops).
            // SA_ONSTACK: use alternate signal stack if one is registered (safer for SIGSEGV).
            sa.sa_flags = libc::SA_RESETHAND | libc::SA_ONSTACK;
            libc::sigaction(sig, &sa, std::ptr::null_mut());
        }
    }
}

#[cfg(not(unix))]
fn install_crash_handlers() {} // No-op on Windows

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
    // Install crash signal handlers once per process so SIGSEGV/SIGBUS/SIGABRT
    // exit cleanly instead of triggering OS crash dialogs.
    install_crash_handlers();

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

    // FFI is only used by the GUI — set Interactive mode to skip anti-bot delays
    if let Ok(mut b) = browser.lock() {
        b.set_navigation_mode(NavigationMode::Interactive);
    }

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
    let inst = match instance_into_box(instance) {
        Some(i) => i,
        None => return,
    };

    // Leak the Boa JS renderer before dropping the instance.
    //
    // WHY: The renderer (and its Boa GC state) was last used on an 8MB OS thread
    // created by NavigateAsync or ExecutePageScriptsAsync. thalora_destroy is called
    // from the C# disposal path (UI thread or finalizer thread). Dropping the renderer
    // on a different thread causes cross-thread Boa GC corruption → SIGSEGV, which
    // triggers a crash dialog even as the app is shutting down.
    //
    // Leaking is safe here: the process exits immediately after destroy, so the OS
    // reclaims the memory. The ~5–15MB per navigation is a one-time leak on exit.
    if let Ok(mut browser) = inst.browser.lock() {
        browser.leak_renderer();
    }

    drop(inst);
}

/// Get the last error message from the instance.
///
/// Returns a pointer to a C string describing the last error, or null if
/// no error has occurred. The returned string is valid until the next FFI
/// call on this instance. The caller must NOT free this string.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_last_error(instance: *const ThalorInstance) -> *const c_char {
    let inst = match instance_ref_const(instance) {
        Some(i) => i,
        None => return ptr::null(),
    };
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
    // Reclaim the CString (no-op if null)
    let _ = reclaim_c_string(ptr);
}
