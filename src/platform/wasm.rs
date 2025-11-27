//! WASM platform implementation for Thalora Web Browser
//!
//! Uses web-sys fetch for HTTP, web-sys WebSocket for WebSocket,
//! and web-sys localStorage/IndexedDB for storage.

use super::{HttpClient, HttpRequest, HttpResponse, HttpError, HttpErrorKind, HttpMethod};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

/// WASM HTTP client using web-sys fetch API
pub struct WasmHttpClient;

impl WasmHttpClient {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WasmHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient for WasmHttpClient {
    fn request(&self, request: HttpRequest) -> Pin<Box<dyn Future<Output = Result<HttpResponse, HttpError>> + '_>> {
        Box::pin(async move {
            let window = web_sys::window().ok_or_else(|| HttpError {
                message: "No window object available".to_string(),
                kind: HttpErrorKind::Other,
            })?;

            let mut opts = RequestInit::new();
            opts.method(request.method.as_str());
            opts.mode(RequestMode::Cors);

            // Add body if present
            if let Some(body) = &request.body {
                let uint8_array = js_sys::Uint8Array::from(body.as_slice());
                opts.body(Some(&uint8_array));
            }

            // Create request
            let req = Request::new_with_str_and_init(&request.url, &opts).map_err(|e| HttpError {
                message: format!("Failed to create request: {:?}", e),
                kind: HttpErrorKind::InvalidUrl,
            })?;

            // Add headers
            let headers = req.headers();
            for (key, value) in &request.headers {
                headers.set(key, value).map_err(|e| HttpError {
                    message: format!("Failed to set header: {:?}", e),
                    kind: HttpErrorKind::Other,
                })?;
            }

            // Execute fetch
            let resp_value = JsFuture::from(window.fetch_with_request(&req))
                .await
                .map_err(|e| HttpError {
                    message: format!("Fetch failed: {:?}", e),
                    kind: HttpErrorKind::Network,
                })?;

            let resp: Response = resp_value.dyn_into().map_err(|e| HttpError {
                message: format!("Invalid response: {:?}", e),
                kind: HttpErrorKind::InvalidResponse,
            })?;

            let status = resp.status();
            let url = resp.url();

            // Get headers
            let mut headers = HashMap::new();
            // Note: In WASM, accessing all headers is restricted by CORS
            // We can only access CORS-safelisted headers

            // Get body as bytes
            let array_buffer = JsFuture::from(resp.array_buffer().map_err(|e| HttpError {
                message: format!("Failed to get array buffer: {:?}", e),
                kind: HttpErrorKind::InvalidResponse,
            })?)
            .await
            .map_err(|e| HttpError {
                message: format!("Failed to read response: {:?}", e),
                kind: HttpErrorKind::InvalidResponse,
            })?;

            let uint8_array = js_sys::Uint8Array::new(&array_buffer);
            let body = uint8_array.to_vec();

            Ok(HttpResponse {
                status,
                headers,
                body,
                url,
            })
        })
    }
}

/// WASM storage using localStorage
pub struct WasmStorage {
    prefix: String,
}

impl WasmStorage {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }

    fn get_storage(&self) -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    fn prefixed_key(&self, key: &str) -> String {
        format!("{}:{}", self.prefix, key)
    }
}

impl Default for WasmStorage {
    fn default() -> Self {
        Self::new("thalora")
    }
}

impl super::Storage for WasmStorage {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        let storage = self.get_storage()?;
        let value = storage.get_item(&self.prefixed_key(key)).ok()??;
        // Decode from base64
        base64_decode(&value)
    }

    fn set(&mut self, key: &str, value: &[u8]) -> Result<(), String> {
        let storage = self.get_storage().ok_or("No storage available")?;
        // Encode as base64 for localStorage
        let encoded = base64_encode(value);
        storage.set_item(&self.prefixed_key(key), &encoded)
            .map_err(|_| "Failed to set item".to_string())
    }

    fn delete(&mut self, key: &str) -> Result<(), String> {
        let storage = self.get_storage().ok_or("No storage available")?;
        storage.remove_item(&self.prefixed_key(key))
            .map_err(|_| "Failed to remove item".to_string())
    }

    fn clear(&mut self) -> Result<(), String> {
        let storage = self.get_storage().ok_or("No storage available")?;
        // Only clear items with our prefix
        let keys = self.keys();
        for key in keys {
            storage.remove_item(&self.prefixed_key(&key))
                .map_err(|_| "Failed to remove item".to_string())?;
        }
        Ok(())
    }

    fn keys(&self) -> Vec<String> {
        let storage = match self.get_storage() {
            Some(s) => s,
            None => return Vec::new(),
        };

        let prefix = format!("{}:", self.prefix);
        let len = storage.length().unwrap_or(0);
        let mut keys = Vec::new();

        for i in 0..len {
            if let Ok(Some(key)) = storage.key(i) {
                if key.starts_with(&prefix) {
                    keys.push(key[prefix.len()..].to_string());
                }
            }
        }

        keys
    }
}

/// WASM timer implementation
pub struct WasmTimer;

impl super::Timer for WasmTimer {
    fn set_timeout<F>(callback: F, delay_ms: u32) -> u32
    where
        F: FnOnce() + 'static,
    {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return 0,
        };

        let closure = Closure::once(callback);
        let id = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            delay_ms as i32,
        ).unwrap_or(0);

        // Leak the closure to prevent it from being dropped
        closure.forget();

        id as u32
    }

    fn clear_timeout(id: u32) {
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(id as i32);
        }
    }

    fn set_interval<F>(callback: F, interval_ms: u32) -> u32
    where
        F: Fn() + 'static,
    {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return 0,
        };

        let closure = Closure::wrap(Box::new(callback) as Box<dyn Fn()>);
        let id = window.set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            interval_ms as i32,
        ).unwrap_or(0);

        // Leak the closure to prevent it from being dropped
        closure.forget();

        id as u32
    }

    fn clear_interval(id: u32) {
        if let Some(window) = web_sys::window() {
            window.clear_interval_with_handle(id as i32);
        }
    }
}

/// Base64 encoding helper
fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

/// Base64 decoding helper
fn base64_decode(data: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(data).ok()
}
