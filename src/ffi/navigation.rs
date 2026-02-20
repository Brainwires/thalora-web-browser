//! Navigation FFI functions.
//!
//! These functions wrap the browser's async navigation methods,
//! blocking on the internal tokio runtime to provide sync C FFI.

use std::ffi::c_char;
use std::ptr;

use super::instance::{ThalorInstance, c_str_to_rust, rust_string_to_c};

/// Navigate to a URL. Returns the page HTML as a C string, or null on error.
/// The caller must free the returned string with `thalora_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_navigate(
    instance: *mut ThalorInstance,
    url: *const c_char,
) -> *mut c_char {
    if instance.is_null() {
        return ptr::null_mut();
    }
    let inst = unsafe { &*instance };
    inst.clear_error();

    let url_str = match unsafe { c_str_to_rust(url) } {
        Some(s) => s,
        None => {
            inst.set_error("Invalid or null URL string".into());
            return ptr::null_mut();
        }
    };

    let result = inst.runtime.block_on(async {
        let mut browser = inst.browser.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        browser.navigate_to(url_str).await
    });

    match result {
        Ok(html) => rust_string_to_c(html),
        Err(e) => {
            inst.set_error(format!("Navigation failed: {}", e));
            ptr::null_mut()
        }
    }
}

/// Get the current URL. Returns a C string or null if no page is loaded.
/// The caller must free the returned string with `thalora_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_get_current_url(instance: *mut ThalorInstance) -> *mut c_char {
    if instance.is_null() {
        return ptr::null_mut();
    }
    let inst = unsafe { &*instance };
    inst.clear_error();

    let browser = match inst.browser.lock() {
        Ok(b) => b,
        Err(e) => {
            inst.set_error(format!("Lock poisoned: {}", e));
            return ptr::null_mut();
        }
    };

    match browser.get_current_url() {
        Some(url) => rust_string_to_c(url),
        None => ptr::null_mut(),
    }
}

/// Get the current page HTML content. Returns a C string or null.
/// The caller must free the returned string with `thalora_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_get_page_html(instance: *mut ThalorInstance) -> *mut c_char {
    if instance.is_null() {
        return ptr::null_mut();
    }
    let inst = unsafe { &*instance };
    inst.clear_error();

    let browser = match inst.browser.lock() {
        Ok(b) => b,
        Err(e) => {
            inst.set_error(format!("Lock poisoned: {}", e));
            return ptr::null_mut();
        }
    };

    let content = browser.get_current_content();
    if content.is_empty() {
        ptr::null_mut()
    } else {
        rust_string_to_c(content)
    }
}

/// Go back in navigation history.
/// Returns 0 on success, -1 on error (check `thalora_last_error`).
#[unsafe(no_mangle)]
pub extern "C" fn thalora_go_back(instance: *mut ThalorInstance) -> i32 {
    if instance.is_null() {
        return -1;
    }
    let inst = unsafe { &*instance };
    inst.clear_error();

    let result = inst.runtime.block_on(async {
        let mut browser = inst.browser.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        browser.go_back().await
    });

    match result {
        Ok(Some(_)) => 0,
        Ok(None) => {
            inst.set_error("Already at beginning of history".into());
            -1
        }
        Err(e) => {
            inst.set_error(format!("Go back failed: {}", e));
            -1
        }
    }
}

/// Go forward in navigation history.
/// Returns 0 on success, -1 on error (check `thalora_last_error`).
#[unsafe(no_mangle)]
pub extern "C" fn thalora_go_forward(instance: *mut ThalorInstance) -> i32 {
    if instance.is_null() {
        return -1;
    }
    let inst = unsafe { &*instance };
    inst.clear_error();

    let result = inst.runtime.block_on(async {
        let mut browser = inst.browser.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        browser.go_forward().await
    });

    match result {
        Ok(Some(_)) => 0,
        Ok(None) => {
            inst.set_error("Already at end of history".into());
            -1
        }
        Err(e) => {
            inst.set_error(format!("Go forward failed: {}", e));
            -1
        }
    }
}

/// Reload the current page. Returns the page HTML as a C string, or null on error.
/// The caller must free the returned string with `thalora_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_reload(instance: *mut ThalorInstance) -> *mut c_char {
    if instance.is_null() {
        return ptr::null_mut();
    }
    let inst = unsafe { &*instance };
    inst.clear_error();

    let result = inst.runtime.block_on(async {
        let mut browser = inst.browser.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        browser.reload().await
    });

    match result {
        Ok(html) => rust_string_to_c(html),
        Err(e) => {
            inst.set_error(format!("Reload failed: {}", e));
            ptr::null_mut()
        }
    }
}

/// Check if the browser can go back in history.
/// Returns 1 if true, 0 if false.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_can_go_back(instance: *const ThalorInstance) -> i32 {
    if instance.is_null() {
        return 0;
    }
    let inst = unsafe { &*instance };

    match inst.browser.lock() {
        Ok(browser) => if browser.can_go_back() { 1 } else { 0 },
        Err(_) => 0,
    }
}

/// Check if the browser can go forward in history.
/// Returns 1 if true, 0 if false.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_can_go_forward(instance: *const ThalorInstance) -> i32 {
    if instance.is_null() {
        return 0;
    }
    let inst = unsafe { &*instance };

    match inst.browser.lock() {
        Ok(browser) => if browser.can_go_forward() { 1 } else { 0 },
        Err(_) => 0,
    }
}

/// Compute the layout for the current page content.
///
/// Takes viewport dimensions and returns a JSON string containing the full
/// layout tree with positions, sizes, and visual properties.
/// Returns null if no page is loaded or on error.
/// The caller must free the returned string with `thalora_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_compute_layout(
    instance: *mut ThalorInstance,
    viewport_w: f32,
    viewport_h: f32,
) -> *mut c_char {
    if instance.is_null() {
        return ptr::null_mut();
    }
    let inst = unsafe { &*instance };
    inst.clear_error();

    let browser = match inst.browser.lock() {
        Ok(b) => b,
        Err(e) => {
            inst.set_error(format!("Lock poisoned: {}", e));
            return ptr::null_mut();
        }
    };

    let content = browser.get_current_content();
    if content.is_empty() {
        inst.set_error("No page content loaded".into());
        return ptr::null_mut();
    }

    // Drop the lock before computing layout (which can take time)
    drop(browser);

    match crate::engine::renderer::compute_page_layout(&content, viewport_w, viewport_h) {
        Ok(layout_result) => {
            match serde_json::to_string(&layout_result) {
                Ok(json) => rust_string_to_c(json),
                Err(e) => {
                    inst.set_error(format!("Failed to serialize layout: {}", e));
                    ptr::null_mut()
                }
            }
        }
        Err(e) => {
            inst.set_error(format!("Layout computation failed: {}", e));
            ptr::null_mut()
        }
    }
}
