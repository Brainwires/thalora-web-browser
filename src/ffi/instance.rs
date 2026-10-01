//! Core FFI types and lifecycle management.
//!
//! Provides the ThalorInstance struct that owns a browser running on its own
//! thread, bridging async Rust to synchronous C FFI calls.

use std::ffi::{CStr, CString, c_char};
use std::ptr;
use std::sync::Mutex;

use futures::FutureExt;
use futures::future::LocalBoxFuture;

#[cfg(unix)]
extern crate libc;

use crate::engine::HeadlessWebBrowser;
use crate::engine::browser::BrowserThread;
use crate::engine::browser::types::NavigationMode;
use crate::engine::engine_trait::EngineType;

/// Opaque instance holding the browser.
///
/// The browser (and its Boa JS heap, whose GC is thread-local) lives on a
/// dedicated 16 MB-stack thread for the instance's whole life. FFI entry
/// points, which may arrive on any caller thread, hand work to that thread
/// and block until it finishes.
pub struct ThalorInstance {
    pub(crate) browser: BrowserThread,
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

    /// Run `f` with the browser on the browser's thread and wait for it.
    // Runs on the browser's own thread, one job at a time (see BrowserThread)
    #[allow(clippy::await_holding_lock)]
    pub(crate) fn with_browser<R, F>(&self, f: F) -> anyhow::Result<R>
    where
        R: Send + 'static,
        F: for<'a> FnOnce(&'a mut HeadlessWebBrowser) -> LocalBoxFuture<'a, anyhow::Result<R>>
            + Send
            + 'static,
    {
        self.browser.call_blocking(move |browser| {
            async move {
                let mut guard = browser
                    .lock()
                    .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
                f(&mut guard).await
            }
            .boxed_local()
        })?
    }

    /// Synchronous access to the browser (still on the browser's thread).
    pub(crate) fn read_browser<R, F>(&self, f: F) -> anyhow::Result<R>
    where
        R: Send + 'static,
        F: FnOnce(&mut HeadlessWebBrowser) -> R + Send + 'static,
    {
        self.with_browser(move |browser| {
            let value = f(browser);
            async move { Ok(value) }.boxed_local()
        })
    }
}

/// Run pure-Rust FFI work (no JS, no browser access) on a fresh thread with a
/// 16 MB stack and panic-catching so a stack overflow or panic can't take
/// down the process. Browser and JS work runs on the instance's
/// [`BrowserThread`] instead, which has the same stack size.
///
/// Why: the .NET ThreadPool worker that drives our FFI calls has a ~512 KB
/// stack on macOS. The CSS selector matcher on real-world pages (Google,
/// GitHub) recurses deeply enough to overflow that stack.
///
/// `catch_unwind` does NOT catch stack overflows — the large stack is what
/// prevents the overflow; `catch_unwind` only catches normal panics.
///
/// Returns `Ok(T)` on success or `Err(msg)` describing spawn / join / panic /
/// inner failures so callers can surface a diagnostic via `set_error`.
pub(crate) fn on_large_stack<F, T>(name: &'static str, f: F) -> Result<T, String>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let handle = std::thread::Builder::new()
        .name(name.into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)))
        .map_err(|e| format!("Failed to spawn FFI worker '{}': {}", name, e))?;

    match handle.join() {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(_)) => Err(format!("FFI worker '{}' panicked", name)),
        Err(_) => Err(format!(
            "FFI worker '{}' aborted (likely stack overflow)",
            name
        )),
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
/// When the engine crashes, the default OS behavior is to generate a crash
/// report and show a dialog to the user. Instead, we handle the signal
/// ourselves: log the signal and call `_exit(0)`. Exiting with code 0 prevents
/// macOS CrashReporter from activating, and the BrowserController's PID-watcher
/// detects the exit and relaunches the GUI.
///
/// The main historical cause (Boa objects used after the thread that created
/// them exited) is gone now that each instance's JS runs on one long-lived
/// thread, so any crash caught here is a real bug: the message says so loudly
/// and names the signal. Scheduled for removal once that has been confirmed in
/// the field.
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
            libc::SIGSEGV => {
                b"[thalora] FATAL: SIGSEGV in the browser engine (bug - please report)\n"
            }
            libc::SIGBUS => {
                b"[thalora] FATAL: SIGBUS in the browser engine (bug - please report)\n"
            }
            libc::SIGABRT => {
                b"[thalora] FATAL: SIGABRT in the browser engine (bug - please report)\n"
            }
            _ => b"[thalora] FATAL: signal in the browser engine (bug - please report)\n",
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
            sa.sa_sigaction = crash_handler as *const () as libc::sighandler_t;
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

    // The browser is created on, and only ever touched from, its own thread
    let browser = match BrowserThread::spawn("ffi", EngineType::Boa) {
        Ok(browser) => browser,
        Err(e) => {
            eprintln!("[ERROR] FFI thalora_init: {}", e);
            return ptr::null_mut();
        }
    };

    let instance = ThalorInstance {
        browser,
        last_error: Mutex::new(None),
    };

    // FFI is only used by the GUI — set Interactive mode to skip anti-bot delays
    if let Err(e) =
        instance.read_browser(|browser| browser.set_navigation_mode(NavigationMode::Interactive))
    {
        eprintln!("[ERROR] FFI thalora_init: {}", e);
        instance.browser.shutdown_blocking();
        return ptr::null_mut();
    }

    Box::into_raw(Box::new(instance))
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

    // Stop the browser thread; the browser (and its JS heap) is dropped on
    // the thread that created it, whichever thread calls destroy.
    let ThalorInstance { browser, .. } = *inst;
    browser.shutdown_blocking();
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

/// Tell the CSS engine whether the OS/app currently prefers a dark color scheme.
/// Pages that use `@media (prefers-color-scheme: dark)` will match those rules
/// when `dark` is 1; otherwise the default light branch is used. Process-global;
/// safe to call on any thread. Passing 0 reverts to light.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_set_prefers_dark(dark: i32) {
    crate::engine::renderer::css::set_prefers_dark(dark != 0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::{thalora_execute_js, thalora_get_current_url};

    fn eval(instance: usize, code: &str) -> Option<String> {
        let code = CString::new(code).unwrap();
        let result = thalora_execute_js(instance as *mut ThalorInstance, code.as_ptr());
        reclaim_c_string(result).map(|s| s.to_string_lossy().into_owned())
    }

    /// FFI calls arrive on arbitrary threads; JS must still run on the
    /// instance's own thread against a live heap.
    #[test]
    fn js_calls_from_different_threads_share_one_browser() {
        let instance = thalora_init();
        assert!(!instance.is_null());
        let addr = instance as usize;

        let first = std::thread::spawn(move || eval(addr, "globalThis.__ffiTest = 41; 1 + 1"))
            .join()
            .unwrap();
        assert_eq!(first.as_deref(), Some("2"));

        let second = std::thread::spawn(move || eval(addr, "globalThis.__ffiTest + 1"))
            .join()
            .unwrap();
        assert_eq!(second.as_deref(), Some("42"));

        assert!(thalora_get_current_url(instance).is_null());
        thalora_destroy(instance);
    }

    #[test]
    fn destroy_from_another_thread() {
        let addr = thalora_init() as usize;
        assert_eq!(eval(addr, "1 + 1").as_deref(), Some("2"));
        std::thread::spawn(move || thalora_destroy(addr as *mut ThalorInstance))
            .join()
            .unwrap();
    }
}
