//! Script element detection and execution helpers

use boa_engine::{
    object::JsObject,
    value::JsValue,
    Context, JsResult, js_string,
};

/// Fire a `load` event on a script element after successful execution.
/// Handles both addEventListener listeners (via dispatchEvent) and
/// property handlers (script.onload = fn) used by Webpack chunk loading.
fn fire_script_load_event(script_obj: &JsObject, context: &mut Context) {
    // Create a load Event
    let event = match context
        .intrinsics()
        .constructors()
        .event()
        .constructor()
        .construct(&[js_string!("load").into()], None, context)
    {
        Ok(ev) => ev,
        Err(_) => return,
    };

    // 1. Dispatch via dispatchEvent (for addEventListener listeners)
    if let Ok(dispatch_fn) = script_obj.get(js_string!("dispatchEvent"), context) {
        if let Some(callable) = dispatch_fn.as_callable() {
            let _ = callable.call(
                &script_obj.clone().into(),
                &[event.clone().into()],
                context,
            );
        }
    }

    // 2. Call onload property handler (for Webpack-style: script.onload = fn)
    if let Ok(onload) = script_obj.get(js_string!("onload"), context) {
        if let Some(callable) = onload.as_callable() {
            let _ = callable.call(
                &script_obj.clone().into(),
                &[event.into()],
                context,
            );
        }
    }
}

/// Fire an `error` event on a script element after failed execution or fetch.
/// Handles both addEventListener listeners and property handlers (script.onerror).
fn fire_script_error_event(script_obj: &JsObject, context: &mut Context) {
    // Create an error Event
    let event = match context
        .intrinsics()
        .constructors()
        .event()
        .constructor()
        .construct(&[js_string!("error").into()], None, context)
    {
        Ok(ev) => ev,
        Err(_) => return,
    };

    // 1. Dispatch via dispatchEvent (for addEventListener listeners)
    if let Ok(dispatch_fn) = script_obj.get(js_string!("dispatchEvent"), context) {
        if let Some(callable) = dispatch_fn.as_callable() {
            let _ = callable.call(
                &script_obj.clone().into(),
                &[event.clone().into()],
                context,
            );
        }
    }

    // 2. Call onerror property handler
    if let Ok(onerror) = script_obj.get(js_string!("onerror"), context) {
        if let Some(callable) = onerror.as_callable() {
            let _ = callable.call(
                &script_obj.clone().into(),
                &[event.into()],
                context,
            );
        }
    }
}

/// Check if a JsObject is a script element (by tagName or by HTMLScriptElementData)
pub(super) fn is_script_element(obj: &JsObject, context: &mut Context) -> JsResult<bool> {
    // First, try to check by HTMLScriptElementData
    if obj.downcast_ref::<crate::dom::html_script_element::HTMLScriptElementData>().is_some() {
        return Ok(true);
    }

    // Fall back to checking tagName property
    if let Ok(tag_name_value) = obj.get(js_string!("tagName"), context) {
        if let Ok(tag_name) = tag_name_value.to_string(context) {
            let tag_name_str = tag_name.to_std_string_escaped();
            return Ok(tag_name_str.eq_ignore_ascii_case("SCRIPT"));
        }
    }

    // Also check ElementData's tagName (dispatches across all element types)
    if let Ok(result) = super::with_element_data(obj, |element_data| {
        element_data.get_tag_name().eq_ignore_ascii_case("SCRIPT")
    }, "not element") {
        return Ok(result);
    }

    Ok(false)
}

/// Get the script type attribute value
fn get_script_type(obj: &JsObject, context: &mut Context) -> JsResult<String> {
    // First, try HTMLScriptElementData
    if let Some(script_data) = obj.downcast_ref::<crate::dom::html_script_element::HTMLScriptElementData>() {
        // Access the type field - but it's private, so check via JS property
    }

    // Check the 'type' property
    if let Ok(type_value) = obj.get(js_string!("type"), context) {
        if !type_value.is_undefined() && !type_value.is_null() {
            let type_str = type_value.to_string(context)?.to_std_string_escaped();
            return Ok(type_str);
        }
    }

    // Default to text/javascript
    Ok(String::new())
}

/// Get the script src attribute value (for external scripts)
fn get_script_src(obj: &JsObject, context: &mut Context) -> JsResult<Option<String>> {
    // Check the 'src' property
    if let Ok(src_value) = obj.get(js_string!("src"), context) {
        if !src_value.is_undefined() && !src_value.is_null() {
            let src_str = src_value.to_string(context)?.to_std_string_escaped();
            if !src_str.is_empty() {
                return Ok(Some(src_str));
            }
        }
    }

    Ok(None)
}

/// Get the inline script content (text or innerHTML)
fn get_script_content(obj: &JsObject, context: &mut Context) -> JsResult<String> {
    // Try 'text' property first (specific to script elements)
    if let Ok(text_value) = obj.get(js_string!("text"), context) {
        if !text_value.is_undefined() && !text_value.is_null() {
            let text_str = text_value.to_string(context)?.to_std_string_escaped();
            if !text_str.is_empty() {
                return Ok(text_str);
            }
        }
    }

    // Fall back to textContent
    if let Ok(text_content_value) = obj.get(js_string!("textContent"), context) {
        if !text_content_value.is_undefined() && !text_content_value.is_null() {
            let text_str = text_content_value.to_string(context)?.to_std_string_escaped();
            if !text_str.is_empty() {
                return Ok(text_str);
            }
        }
    }

    // Try innerHTML
    if let Ok(inner_html_value) = obj.get(js_string!("innerHTML"), context) {
        if !inner_html_value.is_undefined() && !inner_html_value.is_null() {
            let html_str = inner_html_value.to_string(context)?.to_std_string_escaped();
            if !html_str.is_empty() {
                return Ok(html_str);
            }
        }
    }

    // Try ElementData (dispatches across all element types)
    if let Ok(result) = super::with_element_data(obj, |element_data| {
        let text_content = element_data.get_text_content();
        if !text_content.is_empty() {
            return Some(text_content);
        }
        let inner_html = element_data.get_inner_html();
        if !inner_html.is_empty() {
            return Some(inner_html);
        }
        None
    }, "not element") {
        if let Some(content) = result {
            return Ok(content);
        }
    }

    Ok(String::new())
}

/// Check if a script type is executable JavaScript
fn is_executable_script_type(script_type: &str) -> bool {
    if script_type.is_empty() {
        return true; // Default is JavaScript
    }

    let script_type_lower = script_type.to_lowercase();

    // Standard JavaScript MIME types
    if script_type_lower == "text/javascript" ||
       script_type_lower == "application/javascript" ||
       script_type_lower == "application/x-javascript" ||
       script_type_lower == "text/ecmascript" ||
       script_type_lower == "application/ecmascript" {
        return true;
    }

    // Cloudflare Rocket Loader pattern (e.g., "text/javascript-obfuscated")
    if script_type_lower.contains("javascript") || script_type_lower.contains("ecmascript") {
        return true;
    }

    // Module scripts
    if script_type_lower == "module" {
        return true;
    }

    false
}

/// Execute a script element after it's appended to the DOM
/// This is the core function that actually runs the script
pub fn execute_script_element(script_obj: &JsObject, context: &mut Context) -> JsResult<()> {
    // Get script type
    let script_type = get_script_type(script_obj, context)?;

    // Check if this is an executable script type
    if !is_executable_script_type(&script_type) {
        eprintln!("DEBUG: Skipping non-executable script type: {}", script_type);
        return Ok(());
    }

    // Check for external script (src attribute)
    if let Some(src_url) = get_script_src(script_obj, context)? {
        eprintln!("DEBUG: Executing external script from: {}", src_url);
        let success = execute_external_script(&src_url, context);
        if success {
            fire_script_load_event(script_obj, context);
        } else {
            fire_script_error_event(script_obj, context);
        }
        return Ok(());
    }

    // Inline script - get content and execute
    let script_content = get_script_content(script_obj, context)?;

    if script_content.is_empty() {
        eprintln!("DEBUG: Script element has no content to execute");
        return Ok(());
    }

    eprintln!("DEBUG: Executing inline script ({} chars)", script_content.len());

    // Execute the script content
    match context.eval(boa_engine::Source::from_bytes(&script_content)) {
        Ok(_result) => {
            // Microtask checkpoint: process Promise.then callbacks per HTML spec.
            // Scripts may enqueue microtasks (e.g., webpack chunk registration) that
            // must resolve before the next script runs.
            let _ = context.run_jobs();
            eprintln!("DEBUG: Script executed successfully");
            fire_script_load_event(script_obj, context);
            Ok(())
        }
        Err(e) => {
            eprintln!("DEBUG: Script execution error: {}", e);
            // Don't propagate the error - scripts with errors shouldn't break DOM operations
            fire_script_error_event(script_obj, context);
            Ok(())
        }
    }
}

/// Fetch and execute an external script.
/// Returns `true` on success, `false` on fetch or execution failure.
#[cfg(feature = "native")]
fn execute_external_script(url: &str, context: &mut Context) -> bool {
    use crate::http_blocking::{get_shared_client, block_on_compat};
    use url::Url;

    eprintln!("DEBUG: Fetching external script: {}", url);

    // Resolve relative URLs against the current page's base URL
    let resolved_url = match Url::parse(url) {
        Ok(_) => url.to_string(), // Already absolute
        Err(_) => {
            // Try to resolve against window.location.href
            let base_url = crate::fetch::fetch::get_base_url_from_context(context);
            if let Some(base) = base_url {
                if let Ok(base_parsed) = Url::parse(&base) {
                    match base_parsed.join(url) {
                        Ok(resolved) => {
                            let resolved_str = resolved.to_string();
                            eprintln!("DEBUG: Resolved relative script URL '{}' -> '{}'", url, resolved_str);
                            resolved_str
                        }
                        Err(e) => {
                            eprintln!("DEBUG: Failed to resolve script URL '{}': {:?}", url, e);
                            return false;
                        }
                    }
                } else {
                    eprintln!("DEBUG: Invalid base URL for script resolution: {}", base);
                    return false;
                }
            } else {
                eprintln!("DEBUG: No base URL available to resolve relative script URL: {}", url);
                return false;
            }
        }
    };

    // Fetch the script in a separate thread to avoid "cannot start a runtime
    // from within a runtime" panic when called from async navigation code.
    // Uses the shared browser client for Chrome131 TLS fingerprint, cookies,
    // and compression support. Retries up to 2 times for transient IO errors
    // (BrokenPipe, connection reset) that occur with HTTP/2 stream resets.
    let fetch_url = resolved_url.clone();
    let script_content = std::thread::spawn(move || -> Option<String> {
        let client = get_shared_client();

        for attempt in 0u32..3 {
            let request = client.get(&fetch_url);
            let result = block_on_compat(async move {
                let response = request.send().await?.error_for_status()?;
                response.text().await
            });

            match result {
                Ok(text) => return Some(text),
                Err(e) => {
                    // Check if the error is transient (IO/connection/decode errors)
                    let err_debug = format!("{:?}", e);
                    let is_transient = err_debug.contains("BrokenPipe")
                        || err_debug.contains("ConnectionReset")
                        || err_debug.contains("Io(")
                        || err_debug.contains("Decode")
                        || err_debug.contains("hyper")
                        || err_debug.contains("connection closed");

                    if is_transient && attempt < 2 {
                        eprintln!(
                            "DEBUG: [thread] Retry {}/2 for {}: {:?}",
                            attempt + 1,
                            fetch_url,
                            e
                        );
                        std::thread::sleep(std::time::Duration::from_millis(
                            100 * (attempt as u64 + 1),
                        ));
                        continue;
                    }
                    eprintln!("DEBUG: [thread] Failed to fetch script: {:?}", e);
                    return None;
                }
            }
        }
        None
    })
    .join()
    .ok()
    .flatten();

    match script_content {
        Some(content) => {
            eprintln!("DEBUG: Fetched {} bytes of script content", content.len());

            // Execute the fetched script
            match context.eval(boa_engine::Source::from_bytes(&content)) {
                Ok(_) => {
                    // Microtask checkpoint: process Promise.then callbacks per HTML spec.
                    let _ = context.run_jobs();
                    eprintln!("DEBUG: External script executed successfully");
                    true
                }
                Err(e) => {
                    eprintln!("DEBUG: External script execution error: {}", e);
                    false
                }
            }
        }
        None => {
            eprintln!("DEBUG: Failed to fetch script from: {}", resolved_url);
            false
        }
    }
}

#[cfg(not(feature = "native"))]
fn execute_external_script(_url: &str, _context: &mut Context) -> bool {
    eprintln!("DEBUG: External script execution not supported in WASM mode");
    false
}
