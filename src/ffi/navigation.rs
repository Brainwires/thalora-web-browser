//! Navigation FFI functions.
//!
//! These functions wrap the browser's async navigation methods,
//! blocking on the internal tokio runtime to provide sync C FFI.

use std::ffi::c_char;
use std::panic;
use std::ptr;
use std::time::Instant;

use super::instance::{
    ThalorInstance, c_str_to_rust_safe, instance_ref, instance_ref_const, rust_string_to_c,
};
use crate::engine::browser::types::NavigationMode;

/// Navigate to a URL. Returns the page HTML as a C string, or null on error.
/// The caller must free the returned string with `thalora_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_navigate(
    instance: *mut ThalorInstance,
    url: *const c_char,
) -> *mut c_char {
    let inst = match instance_ref(instance) {
        Some(i) => i,
        None => return ptr::null_mut(),
    };
    inst.clear_error();

    let url_str = match c_str_to_rust_safe(url) {
        Some(s) => s,
        None => {
            inst.set_error("Invalid or null URL string".into());
            return ptr::null_mut();
        }
    };

    // Wrap in catch_unwind to prevent Rust panics from crossing the FFI boundary
    // (undefined behavior). thalora_compute_styled_tree already does this.
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        match inst
            .browser
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))
        {
            Ok(mut browser) => inst
                .runtime
                .block_on(browser.navigate_to_with_js_option(url_str, true, true)),
            Err(e) => Err(e),
        }
    }));

    match result {
        Ok(Ok(html)) => rust_string_to_c(html),
        Ok(Err(e)) => {
            inst.set_error(format!("Navigation failed: {}", e));
            ptr::null_mut()
        }
        Err(_) => {
            eprintln!("[ERROR] FFI thalora_navigate panicked! Returning null.");
            inst.set_error("Navigation panicked (internal error)".into());
            ptr::null_mut()
        }
    }
}

/// Navigate to a URL without executing JavaScript. Returns the page HTML as a
/// C string (must be freed with `thalora_free_string`), or null on error.
///
/// Use this for the fast first phase of two-phase navigation. Call
/// `thalora_execute_page_scripts` afterward to run JS in the background while
/// the static page is already visible.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_navigate_static(
    instance: *mut ThalorInstance,
    url: *const c_char,
) -> *mut c_char {
    let inst = match instance_ref(instance) {
        Some(i) => i,
        None => return ptr::null_mut(),
    };
    inst.clear_error();

    let url_str = match c_str_to_rust_safe(url) {
        Some(s) => s,
        None => {
            inst.set_error("Invalid or null URL string".into());
            return ptr::null_mut();
        }
    };

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        match inst
            .browser
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))
        {
            Ok(mut browser) => inst
                .runtime
                .block_on(browser.navigate_to_with_js_option(url_str, true, false)),
            Err(e) => Err(e),
        }
    }));

    match result {
        Ok(Ok(html)) => rust_string_to_c(html),
        Ok(Err(e)) => {
            inst.set_error(format!("Navigation failed: {}", e));
            ptr::null_mut()
        }
        Err(_) => {
            eprintln!("[ERROR] FFI thalora_navigate_static panicked! Returning null.");
            inst.set_error("Navigation panicked (internal error)".into());
            ptr::null_mut()
        }
    }
}

/// Execute page scripts on the already-loaded page (Phase 2 of two-phase navigation).
/// Updates the internal DOM with JS modifications.
/// Returns 1 if the DOM was modified, 0 if no change, -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_execute_page_scripts(instance: *mut ThalorInstance) -> i32 {
    let inst = match instance_ref(instance) {
        Some(i) => i,
        None => return -1,
    };
    inst.clear_error();

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        match inst
            .browser
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))
        {
            Ok(mut browser) => inst
                .runtime
                .block_on(browser.execute_current_page_scripts()),
            Err(e) => Err(e),
        }
    }));

    match result {
        Ok(Ok(changed)) => {
            if changed {
                1
            } else {
                0
            }
        }
        Ok(Err(e)) => {
            inst.set_error(format!("Script execution failed: {}", e));
            -1
        }
        Err(_) => {
            eprintln!("[ERROR] FFI thalora_execute_page_scripts panicked! Returning -1.");
            inst.set_error("Script execution panicked (internal error)".into());
            -1
        }
    }
}

/// Get the current URL. Returns a C string or null if no page is loaded.
/// The caller must free the returned string with `thalora_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_get_current_url(instance: *mut ThalorInstance) -> *mut c_char {
    let inst = match instance_ref(instance) {
        Some(i) => i,
        None => return ptr::null_mut(),
    };
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
    let inst = match instance_ref(instance) {
        Some(i) => i,
        None => return ptr::null_mut(),
    };
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
    let inst = match instance_ref(instance) {
        Some(i) => i,
        None => return -1,
    };
    inst.clear_error();

    // Wrap in catch_unwind to prevent Rust panics from crossing the FFI boundary
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        match inst
            .browser
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))
        {
            Ok(mut browser) => inst.runtime.block_on(browser.go_back()),
            Err(e) => Err(e),
        }
    }));

    match result {
        Ok(Ok(Some(_))) => 0,
        Ok(Ok(None)) => {
            inst.set_error("Already at beginning of history".into());
            -1
        }
        Ok(Err(e)) => {
            inst.set_error(format!("Go back failed: {}", e));
            -1
        }
        Err(_) => {
            eprintln!("[ERROR] FFI thalora_go_back panicked! Returning -1.");
            inst.set_error("Go back panicked (internal error)".into());
            -1
        }
    }
}

/// Go forward in navigation history.
/// Returns 0 on success, -1 on error (check `thalora_last_error`).
#[unsafe(no_mangle)]
pub extern "C" fn thalora_go_forward(instance: *mut ThalorInstance) -> i32 {
    let inst = match instance_ref(instance) {
        Some(i) => i,
        None => return -1,
    };
    inst.clear_error();

    // Wrap in catch_unwind to prevent Rust panics from crossing the FFI boundary
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        match inst
            .browser
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))
        {
            Ok(mut browser) => inst.runtime.block_on(browser.go_forward()),
            Err(e) => Err(e),
        }
    }));

    match result {
        Ok(Ok(Some(_))) => 0,
        Ok(Ok(None)) => {
            inst.set_error("Already at end of history".into());
            -1
        }
        Ok(Err(e)) => {
            inst.set_error(format!("Go forward failed: {}", e));
            -1
        }
        Err(_) => {
            eprintln!("[ERROR] FFI thalora_go_forward panicked! Returning -1.");
            inst.set_error("Go forward panicked (internal error)".into());
            -1
        }
    }
}

/// Reload the current page. Returns the page HTML as a C string, or null on error.
/// The caller must free the returned string with `thalora_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_reload(instance: *mut ThalorInstance) -> *mut c_char {
    let inst = match instance_ref(instance) {
        Some(i) => i,
        None => return ptr::null_mut(),
    };
    inst.clear_error();

    let result = match inst
        .browser
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))
    {
        Ok(mut browser) => inst.runtime.block_on(browser.reload()),
        Err(e) => Err(e),
    };

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
    let inst = match instance_ref_const(instance) {
        Some(i) => i,
        None => return 0,
    };

    match inst.browser.lock() {
        Ok(browser) => {
            if browser.can_go_back() {
                1
            } else {
                0
            }
        }
        Err(_) => 0,
    }
}

/// Check if the browser can go forward in history.
/// Returns 1 if true, 0 if false.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_can_go_forward(instance: *const ThalorInstance) -> i32 {
    let inst = match instance_ref_const(instance) {
        Some(i) => i,
        None => return 0,
    };

    match inst.browser.lock() {
        Ok(browser) => {
            if browser.can_go_forward() {
                1
            } else {
                0
            }
        }
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
    let inst = match instance_ref(instance) {
        Some(i) => i,
        None => return ptr::null_mut(),
    };
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
        Ok(layout_result) => match serde_json::to_string(&layout_result) {
            Ok(json) => rust_string_to_c(json),
            Err(e) => {
                inst.set_error(format!("Failed to serialize layout: {}", e));
                ptr::null_mut()
            }
        },
        Err(e) => {
            inst.set_error(format!("Layout computation failed: {}", e));
            ptr::null_mut()
        }
    }
}

/// Compute the styled element tree for the current page content (new pipeline).
///
/// Unlike `thalora_compute_layout` which returns pixel-positioned elements (taffy),
/// this returns a styled tree with CSS properties resolved but no positions computed.
/// The C# side converts this to Avalonia native controls for layout and rendering.
///
/// Returns null if no page is loaded or on error.
/// The caller must free the returned string with `thalora_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_compute_styled_tree(
    instance: *mut ThalorInstance,
    viewport_w: f32,
    viewport_h: f32,
) -> *mut c_char {
    let inst = match instance_ref(instance) {
        Some(i) => i,
        None => return ptr::null_mut(),
    };
    inst.clear_error();

    let ffi_start = Instant::now();

    let content_start = Instant::now();
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

    // Collect external stylesheets before dropping the lock
    let external_css: Vec<String> = browser.get_external_stylesheets().to_vec();

    // Drop the lock before computing (which can take time)
    drop(browser);
    eprintln!(
        "[TIMING] FFI get_current_content: {}ms ({} bytes, {} external CSS)",
        content_start.elapsed().as_millis(),
        content.len(),
        external_css.len()
    );

    let compute_start = Instant::now();

    // Wrap in catch_unwind to prevent Rust panics from crossing the FFI boundary
    // (undefined behavior). Note: catch_unwind does NOT catch stack overflows —
    // that's handled by MAX_RECURSION_DEPTH in build_styled_element_from_dom().
    let compute_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        crate::engine::renderer::compute_styled_tree_with_css(
            &content,
            viewport_w,
            viewport_h,
            &external_css,
        )
    }));

    match compute_result {
        Ok(Ok(styled_tree)) => {
            eprintln!(
                "[TIMING] FFI compute_styled_tree_with_css: {}ms",
                compute_start.elapsed().as_millis()
            );

            let serialize_start = Instant::now();
            match serde_json::to_string(&styled_tree) {
                Ok(json) => {
                    eprintln!(
                        "[TIMING] FFI serde_json::to_string: {}ms ({} bytes output)",
                        serialize_start.elapsed().as_millis(),
                        json.len()
                    );
                    eprintln!(
                        "[TIMING] FFI Total styled tree: {}ms",
                        ffi_start.elapsed().as_millis()
                    );
                    rust_string_to_c(json)
                }
                Err(e) => {
                    inst.set_error(format!("Failed to serialize styled tree: {}", e));
                    ptr::null_mut()
                }
            }
        }
        Ok(Err(e)) => {
            inst.set_error(format!("Styled tree computation failed: {}", e));
            ptr::null_mut()
        }
        Err(_) => {
            eprintln!("[ERROR] FFI compute_styled_tree_with_css panicked! Returning null.");
            inst.set_error("Styled tree computation panicked (internal error)".into());
            ptr::null_mut()
        }
    }
}

/// Poll for History API events (pushState, replaceState, popstate).
///
/// Returns a JSON array of events as a C string, or null if no events are pending.
/// The caller must free the returned string with `thalora_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_poll_history_events(instance: *mut ThalorInstance) -> *mut c_char {
    let inst = match instance_ref(instance) {
        Some(i) => i,
        None => return ptr::null_mut(),
    };

    let events = match inst.browser.lock() {
        Ok(browser) => browser.drain_history_events(),
        Err(_) => return ptr::null_mut(),
    };

    if events.is_empty() {
        return ptr::null_mut();
    }

    match serde_json::to_string(&events) {
        Ok(json) => rust_string_to_c(json),
        Err(_) => ptr::null_mut(),
    }
}

/// Set the navigation mode for the browser instance.
///
/// mode: 0 = Interactive (no delays, for GUI), 1 = Stealth (human-like delays, for MCP)
/// Returns 0 on success, -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_set_navigation_mode(instance: *mut ThalorInstance, mode: i32) -> i32 {
    let inst = match instance_ref(instance) {
        Some(i) => i,
        None => return -1,
    };
    inst.clear_error();

    let nav_mode = match mode {
        0 => NavigationMode::Interactive,
        1 => NavigationMode::Stealth,
        _ => {
            inst.set_error(format!(
                "Invalid navigation mode: {} (expected 0=Interactive, 1=Stealth)",
                mode
            ));
            return -1;
        }
    };

    match inst.browser.lock() {
        Ok(mut browser) => {
            browser.set_navigation_mode(nav_mode);
            0
        }
        Err(e) => {
            inst.set_error(format!("Lock poisoned: {}", e));
            -1
        }
    }
}
