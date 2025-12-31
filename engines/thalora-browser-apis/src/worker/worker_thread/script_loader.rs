//! Script loading utilities for worker threads

use boa_engine::{Context, JsResult, JsValue, JsNativeError, Source};
use url::Url;
use std::sync::Arc;

use crate::worker::worker_global_scope::WorkerGlobalScope;

/// Load and execute a script in the worker context
pub fn load_and_execute_script(
    script_url: &str,
    context: &mut Context,
    worker_scope: &Arc<WorkerGlobalScope>,
) -> JsResult<JsValue> {
    eprintln!("[Worker] Loading script from: {}", script_url);

    // Check if this is an inline script (starts with "data:" or is raw JS)
    let script_content = if script_url.starts_with("data:") {
        // Extract content from data URL
        extract_from_data_url(script_url)?
    } else if script_url.starts_with("http://") || script_url.starts_with("https://") {
        // Fetch the script content via HTTP
        fetch_script_from_url(script_url)?
    } else {
        // Treat as inline script content
        script_url.to_string()
    };

    // Execute the script in the worker context
    worker_scope.execute_script(context, &script_content)
}

/// Extract content from a data URL
pub fn extract_from_data_url(data_url: &str) -> JsResult<String> {
    // Basic data URL parsing: data:[<mediatype>][;base64],<data>
    if let Some(comma_pos) = data_url.find(',') {
        let content = &data_url[comma_pos + 1..];

        // Check if base64 encoded
        if data_url[..comma_pos].contains(";base64") {
            use base64::{Engine as _, engine::general_purpose};
            let decoded = general_purpose::STANDARD.decode(content)
                .map_err(|e| JsNativeError::error()
                    .with_message(format!("Failed to decode base64 data URL: {}", e)))?;
            String::from_utf8(decoded)
                .map_err(|e| JsNativeError::error()
                    .with_message(format!("Invalid UTF-8 in data URL: {}", e))
                    .into())
        } else {
            // URL decode the content
            urlencoding::decode(content)
                .map(|s| s.to_string())
                .map_err(|e| JsNativeError::error()
                    .with_message(format!("Failed to decode data URL: {}", e))
                    .into())
        }
    } else {
        Err(JsNativeError::error()
            .with_message("Invalid data URL format")
            .into())
    }
}

/// Fetch a script from an HTTP(S) URL
fn fetch_script_from_url(url: &str) -> JsResult<String> {
    eprintln!("[Worker] Fetching script from URL: {}", url);

    // Use reqwest blocking client for synchronous fetch
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| JsNativeError::error()
            .with_message(format!("Failed to create HTTP client: {}", e)))?;

    let response = client.get(url)
        .header("Accept", "application/javascript, text/javascript, */*")
        .send()
        .map_err(|e| JsNativeError::error()
            .with_message(format!("Failed to fetch worker script: {}", e)))?;

    // Check for successful response
    if !response.status().is_success() {
        return Err(JsNativeError::error()
            .with_message(format!("Failed to fetch worker script: HTTP {}", response.status()))
            .into());
    }

    // Get content type to verify it's JavaScript
    let content_type = response.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Allow common JavaScript MIME types
    let is_javascript = content_type.contains("javascript")
        || content_type.contains("text/plain")
        || content_type.is_empty(); // Some servers don't set content-type

    if !is_javascript && !content_type.contains("application/octet-stream") {
        eprintln!("[Worker] Warning: Script content-type is '{}', expected JavaScript", content_type);
    }

    // Read the response body as text
    response.text()
        .map_err(|e| JsNativeError::error()
            .with_message(format!("Failed to read worker script response: {}", e))
            .into())
}
