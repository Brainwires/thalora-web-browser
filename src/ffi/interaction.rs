//! DOM interaction FFI functions.
//!
//! JavaScript execution, element clicking, text input, form submission,
//! and page title extraction.

use std::ffi::c_char;
use std::ptr;

use super::instance::{ThalorInstance, c_str_to_rust_safe, instance_ref, rust_string_to_c};

/// Execute JavaScript code in the browser context.
/// Returns the result as a C string, or null on error.
/// The caller must free the returned string with `thalora_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_execute_js(
    instance: *mut ThalorInstance,
    code: *const c_char,
) -> *mut c_char {
    let inst = match instance_ref(instance) {
        Some(i) => i,
        None => return ptr::null_mut(),
    };
    inst.clear_error();

    let code_str = match c_str_to_rust_safe(code) {
        Some(s) => s.to_owned(),
        None => {
            inst.set_error("Invalid or null JavaScript code string".into());
            return ptr::null_mut();
        }
    };

    let result = match inst
        .browser
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))
    {
        Ok(mut browser) => inst.runtime.block_on(browser.execute_javascript(&code_str)),
        Err(e) => Err(e),
    };

    match result {
        Ok(output) => rust_string_to_c(output),
        Err(e) => {
            inst.set_error(format!("JS execution failed: {}", e));
            ptr::null_mut()
        }
    }
}

/// Click an element identified by CSS selector.
/// Returns 0 on success, -1 on error (check `thalora_last_error`).
#[unsafe(no_mangle)]
pub extern "C" fn thalora_click_element(
    instance: *mut ThalorInstance,
    selector: *const c_char,
) -> i32 {
    let inst = match instance_ref(instance) {
        Some(i) => i,
        None => return -1,
    };
    inst.clear_error();

    let sel_str = match c_str_to_rust_safe(selector) {
        Some(s) => s.to_owned(),
        None => {
            inst.set_error("Invalid or null selector string".into());
            return -1;
        }
    };

    let result = match inst
        .browser
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))
    {
        Ok(mut browser) => inst.runtime.block_on(browser.click_element(&sel_str)),
        Err(e) => Err(e),
    };

    match result {
        Ok(response) => {
            if response.success {
                0
            } else {
                -1
            }
        }
        Err(e) => {
            inst.set_error(format!("Click failed: {}", e));
            -1
        }
    }
}

/// Type text into an element identified by CSS selector.
/// `clear_first`: 1 to clear existing text before typing, 0 to append.
/// Returns 0 on success, -1 on error (check `thalora_last_error`).
#[unsafe(no_mangle)]
pub extern "C" fn thalora_type_text(
    instance: *mut ThalorInstance,
    selector: *const c_char,
    text: *const c_char,
    clear_first: i32,
) -> i32 {
    let inst = match instance_ref(instance) {
        Some(i) => i,
        None => return -1,
    };
    inst.clear_error();

    let sel_str = match c_str_to_rust_safe(selector) {
        Some(s) => s.to_owned(),
        None => {
            inst.set_error("Invalid or null selector string".into());
            return -1;
        }
    };

    let text_str = match c_str_to_rust_safe(text) {
        Some(s) => s.to_owned(),
        None => {
            inst.set_error("Invalid or null text string".into());
            return -1;
        }
    };

    let clear = clear_first != 0;

    let result = match inst
        .browser
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))
    {
        Ok(mut browser) => inst
            .runtime
            .block_on(browser.type_text_into_element(&sel_str, &text_str, clear)),
        Err(e) => Err(e),
    };

    match result {
        Ok(response) => {
            if response.success {
                0
            } else {
                -1
            }
        }
        Err(e) => {
            inst.set_error(format!("Type text failed: {}", e));
            -1
        }
    }
}

/// Submit a form identified by CSS selector with optional JSON data.
/// `json_data` can be null (submit with existing form data) or a JSON object
/// string mapping field names to values.
/// Returns 0 on success, -1 on error (check `thalora_last_error`).
#[unsafe(no_mangle)]
pub extern "C" fn thalora_submit_form(
    instance: *mut ThalorInstance,
    form_selector: *const c_char,
    json_data: *const c_char,
) -> i32 {
    let inst = match instance_ref(instance) {
        Some(i) => i,
        None => return -1,
    };
    inst.clear_error();

    let sel_str = match c_str_to_rust_safe(form_selector) {
        Some(s) => s.to_owned(),
        None => {
            inst.set_error("Invalid or null form selector string".into());
            return -1;
        }
    };

    // Parse optional JSON data into field name→value pairs
    let field_data: Option<std::collections::HashMap<String, String>> =
        c_str_to_rust_safe(json_data).and_then(|json_str| serde_json::from_str(json_str).ok());

    let result = match inst
        .browser
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))
    {
        Ok(mut browser) => {
            // First fill in any form data if provided
            let mut fill_result = Ok(());
            if let Some(fields) = &field_data {
                for (name, value) in fields {
                    let field_selector = format!("{} [name=\"{}\"]", sel_str, name);
                    if let Err(e) = inst.runtime.block_on(browser.type_text_into_element(
                        &field_selector,
                        value,
                        true,
                    )) {
                        fill_result = Err(e);
                        break;
                    }
                }
            }

            match fill_result {
                Ok(()) => {
                    // Then click the submit button within the form
                    let submit_selector = format!(
                        "{} [type=\"submit\"], {} button[type=\"submit\"], {} button:not([type])",
                        sel_str, sel_str, sel_str
                    );
                    inst.runtime
                        .block_on(browser.click_element(&submit_selector))
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    };

    match result {
        Ok(response) => {
            if response.success {
                0
            } else {
                -1
            }
        }
        Err(e) => {
            inst.set_error(format!("Form submission failed: {}", e));
            -1
        }
    }
}

/// Get the current page title.
/// Returns a C string or null if no title is available.
/// The caller must free the returned string with `thalora_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn thalora_get_page_title(instance: *mut ThalorInstance) -> *mut c_char {
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

    match browser.get_current_title() {
        Some(title) => rust_string_to_c(title),
        None => ptr::null_mut(),
    }
}
