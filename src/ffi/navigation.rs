//! Navigation FFI functions.
//!
//! These functions run the browser's async navigation methods on the
//! instance's browser thread and block until they finish, to provide sync
//! C FFI.

use std::ffi::c_char;
use std::ptr;
use std::time::Instant;

use futures::FutureExt;

use super::instance::{
    ThalorInstance, c_str_to_rust_safe, instance_ref, instance_ref_const, on_large_stack,
    rust_string_to_c,
};
use crate::engine::browser::types::NavigationMode;

/// Navigate to a URL. Returns the page HTML as a C string, or null on error.
/// The caller must free the returned string with `thalora_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_navigate(
    instance: *mut ThalorInstance,
    url: *const c_char,
) -> *mut c_char {
    let Some(inst) = instance_ref(instance) else {
        return ptr::null_mut();
    };

    let url_str = match c_str_to_rust_safe(url) {
        Some(s) => s.to_owned(),
        None => {
            inst.set_error("Invalid or null URL string".into());
            return ptr::null_mut();
        }
    };

    inst.clear_error();
    let outcome = inst
        .with_browser(move |browser| {
            async move {
                browser
                    .navigate_to_with_js_option(&url_str, true, true)
                    .await
            }
            .boxed_local()
        })
        .map_err(|e| format!("Navigation failed: {}", e));

    match outcome {
        Ok(html) => rust_string_to_c(html),
        Err(msg) => {
            eprintln!("[ERROR] FFI thalora_navigate: {}", msg);
            if let Some(inst) = instance_ref(instance) {
                inst.set_error(msg);
            }
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
    let Some(inst) = instance_ref(instance) else {
        return ptr::null_mut();
    };

    let url_str = match c_str_to_rust_safe(url) {
        Some(s) => s.to_owned(),
        None => {
            inst.set_error("Invalid or null URL string".into());
            return ptr::null_mut();
        }
    };

    inst.clear_error();
    let outcome = inst
        .with_browser(move |browser| {
            async move {
                browser
                    .navigate_to_with_js_option(&url_str, true, false)
                    .await
            }
            .boxed_local()
        })
        .map_err(|e| format!("Navigation failed: {}", e));

    match outcome {
        Ok(html) => rust_string_to_c(html),
        Err(msg) => {
            eprintln!("[ERROR] FFI thalora_navigate_static: {}", msg);
            if let Some(inst) = instance_ref(instance) {
                inst.set_error(msg);
            }
            ptr::null_mut()
        }
    }
}

/// Execute page scripts on the already-loaded page (Phase 2 of two-phase navigation).
/// Updates the internal DOM with JS modifications.
/// Returns 1 if the DOM was modified, 0 if no change, -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_execute_page_scripts(instance: *mut ThalorInstance) -> i32 {
    let Some(inst) = instance_ref(instance) else {
        return -1;
    };

    inst.clear_error();
    let outcome = inst
        .with_browser(|browser| {
            async move { browser.execute_current_page_scripts().await }.boxed_local()
        })
        .map_err(|e| format!("Script execution failed: {}", e));

    match outcome {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(msg) => {
            eprintln!("[ERROR] FFI thalora_execute_page_scripts: {}", msg);
            if let Some(inst) = instance_ref(instance) {
                inst.set_error(msg);
            }
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

    match inst.read_browser(|browser| browser.get_current_url()) {
        Ok(Some(url)) => rust_string_to_c(url),
        Ok(None) => ptr::null_mut(),
        Err(e) => {
            inst.set_error(e.to_string());
            ptr::null_mut()
        }
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

    match inst.read_browser(|browser| browser.get_current_content()) {
        Ok(content) if !content.is_empty() => rust_string_to_c(content),
        Ok(_) => ptr::null_mut(),
        Err(e) => {
            inst.set_error(e.to_string());
            ptr::null_mut()
        }
    }
}

/// Go back in navigation history.
/// Returns 0 on success, -1 on error (check `thalora_last_error`).
#[unsafe(no_mangle)]
pub extern "C" fn thalora_go_back(instance: *mut ThalorInstance) -> i32 {
    let Some(inst) = instance_ref(instance) else {
        return -1;
    };

    inst.clear_error();
    let outcome = inst
        .with_browser(|browser| async move { browser.go_back().await }.boxed_local())
        .map_err(|e| format!("Go back failed: {}", e));

    match outcome {
        Ok(Some(_)) => 0,
        Ok(None) => {
            if let Some(inst) = instance_ref(instance) {
                inst.set_error("Already at beginning of history".into());
            }
            -1
        }
        Err(msg) => {
            eprintln!("[ERROR] FFI thalora_go_back: {}", msg);
            if let Some(inst) = instance_ref(instance) {
                inst.set_error(msg);
            }
            -1
        }
    }
}

/// Go forward in navigation history.
/// Returns 0 on success, -1 on error (check `thalora_last_error`).
#[unsafe(no_mangle)]
pub extern "C" fn thalora_go_forward(instance: *mut ThalorInstance) -> i32 {
    let Some(inst) = instance_ref(instance) else {
        return -1;
    };

    inst.clear_error();
    let outcome = inst
        .with_browser(|browser| async move { browser.go_forward().await }.boxed_local())
        .map_err(|e| format!("Go forward failed: {}", e));

    match outcome {
        Ok(Some(_)) => 0,
        Ok(None) => {
            if let Some(inst) = instance_ref(instance) {
                inst.set_error("Already at end of history".into());
            }
            -1
        }
        Err(msg) => {
            eprintln!("[ERROR] FFI thalora_go_forward: {}", msg);
            if let Some(inst) = instance_ref(instance) {
                inst.set_error(msg);
            }
            -1
        }
    }
}

/// Reload the current page. Returns the page HTML as a C string, or null on error.
/// The caller must free the returned string with `thalora_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_reload(instance: *mut ThalorInstance) -> *mut c_char {
    let Some(inst) = instance_ref(instance) else {
        return ptr::null_mut();
    };

    inst.clear_error();
    let outcome = inst
        .with_browser(|browser| async move { browser.reload().await }.boxed_local())
        .map_err(|e| format!("Reload failed: {}", e));

    match outcome {
        Ok(html) => rust_string_to_c(html),
        Err(msg) => {
            eprintln!("[ERROR] FFI thalora_reload: {}", msg);
            if let Some(inst) = instance_ref(instance) {
                inst.set_error(msg);
            }
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

    match inst.read_browser(|browser| browser.can_go_back()) {
        Ok(true) => 1,
        _ => 0,
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

    match inst.read_browser(|browser| browser.can_go_forward()) {
        Ok(true) => 1,
        _ => 0,
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

    // Copy the content off the browser thread; layout is pure Rust and runs
    // on the caller's thread without blocking the browser.
    let content = match inst.read_browser(|browser| browser.get_current_content()) {
        Ok(content) => content,
        Err(e) => {
            inst.set_error(e.to_string());
            return ptr::null_mut();
        }
    };
    if content.is_empty() {
        inst.set_error("No page content loaded".into());
        return ptr::null_mut();
    }

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
    // Copy the content and stylesheets off the browser thread; the styled
    // tree is pure Rust and is computed without blocking the browser.
    let (content, external_css) = match inst.read_browser(|browser| {
        (
            browser.get_current_content(),
            browser.get_external_stylesheets().to_vec(),
        )
    }) {
        Ok(page) => page,
        Err(e) => {
            inst.set_error(e.to_string());
            return ptr::null_mut();
        }
    };
    if content.is_empty() {
        inst.set_error("No page content loaded".into());
        return ptr::null_mut();
    }
    eprintln!(
        "[TIMING] FFI get_current_content: {}ms ({} bytes, {} external CSS)",
        content_start.elapsed().as_millis(),
        content.len(),
        external_css.len()
    );

    let compute_start = Instant::now();

    // The CSS cascade + selector matching + style-tree walk on large pages (GitHub
    // has ~18k rules; Google's `compile_selectors` alone takes ~1s) recurse deeply
    // enough to blow the .NET ThreadPool worker's ~512 KB stack. Run on a 16 MB
    // stack via the shared helper — catch_unwind can't recover stack overflows.
    let worker_result = on_large_stack("thalora-styled-tree", move || {
        crate::engine::renderer::compute_styled_tree_with_css(
            &content,
            viewport_w,
            viewport_h,
            &external_css,
        )
        .map_err(|e| format!("Styled tree computation failed: {}", e))
        .and_then(|styled_tree| {
            eprintln!(
                "[TIMING] FFI compute_styled_tree_with_css: {}ms",
                compute_start.elapsed().as_millis()
            );
            let serialize_start = Instant::now();
            serde_json::to_string(&styled_tree)
                .map(|json| {
                    eprintln!(
                        "[TIMING] FFI serde_json::to_string: {}ms ({} bytes output)",
                        serialize_start.elapsed().as_millis(),
                        json.len()
                    );
                    json
                })
                .map_err(|e| format!("Failed to serialize styled tree: {}", e))
        })
    });

    match worker_result.and_then(|r| r) {
        Ok(json) => {
            eprintln!(
                "[TIMING] FFI Total styled tree: {}ms",
                ffi_start.elapsed().as_millis()
            );
            rust_string_to_c(json)
        }
        Err(msg) => {
            eprintln!("[ERROR] FFI thalora_compute_styled_tree: {}", msg);
            inst.set_error(msg);
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

    let events = match inst.read_browser(|browser| browser.drain_history_events()) {
        Ok(events) => events,
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

    match inst.read_browser(move |browser| browser.set_navigation_mode(nav_mode)) {
        Ok(()) => 0,
        Err(e) => {
            inst.set_error(e.to_string());
            -1
        }
    }
}
