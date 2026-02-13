//! Native platform implementation for Thalora Web Browser
//!
//! Uses reqwest for HTTP, tokio-tungstenite for WebSocket, and sled for storage.

use super::{HttpClient, HttpRequest, HttpResponse, HttpError, HttpErrorKind, HttpMethod};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Native HTTP client using reqwest
pub struct NativeHttpClient {
    client: reqwest::Client,
}

impl NativeHttpClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub fn with_config(config: reqwest::ClientBuilder) -> Self {
        Self {
            client: config.build().expect("Failed to create HTTP client"),
        }
    }
}

impl Default for NativeHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient for NativeHttpClient {
    fn request(&self, request: HttpRequest) -> Pin<Box<dyn Future<Output = Result<HttpResponse, HttpError>> + Send + '_>> {
        Box::pin(async move {
            let method = match request.method {
                HttpMethod::Get => reqwest::Method::GET,
                HttpMethod::Post => reqwest::Method::POST,
                HttpMethod::Put => reqwest::Method::PUT,
                HttpMethod::Delete => reqwest::Method::DELETE,
                HttpMethod::Head => reqwest::Method::HEAD,
                HttpMethod::Options => reqwest::Method::OPTIONS,
                HttpMethod::Patch => reqwest::Method::PATCH,
            };

            let mut req_builder = self.client.request(method, &request.url);

            // Add headers
            for (key, value) in &request.headers {
                req_builder = req_builder.header(key.as_str(), value.as_str());
            }

            // Add body if present
            if let Some(body) = request.body {
                req_builder = req_builder.body(body);
            }

            // Set timeout if specified
            if let Some(timeout_ms) = request.timeout_ms {
                req_builder = req_builder.timeout(std::time::Duration::from_millis(timeout_ms));
            }

            // Execute request
            let response = req_builder.send().await.map_err(|e| {
                if e.is_timeout() {
                    HttpError {
                        message: format!("Request timed out: {}", e),
                        kind: HttpErrorKind::Timeout,
                    }
                } else if e.is_connect() {
                    HttpError {
                        message: format!("Connection failed: {}", e),
                        kind: HttpErrorKind::Network,
                    }
                } else {
                    HttpError {
                        message: format!("Request failed: {}", e),
                        kind: HttpErrorKind::Other,
                    }
                }
            })?;

            let status = response.status().as_u16();
            let url = response.url().to_string();

            // Collect headers
            let mut headers = HashMap::new();
            for (key, value) in response.headers() {
                if let Ok(v) = value.to_str() {
                    headers.insert(key.as_str().to_string(), v.to_string());
                }
            }

            // Get body
            let body = response.bytes().await.map_err(|e| HttpError {
                message: format!("Failed to read response body: {}", e),
                kind: HttpErrorKind::InvalidResponse,
            })?.to_vec();

            Ok(HttpResponse {
                status,
                headers,
                body,
                url,
            })
        })
    }
}

/// Native storage using in-memory HashMap (can be extended to use sled)
pub struct NativeStorage {
    data: HashMap<String, Vec<u8>>,
}

impl NativeStorage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Default for NativeStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Storage for NativeStorage {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: &[u8]) -> Result<(), String> {
        self.data.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn delete(&mut self, key: &str) -> Result<(), String> {
        self.data.remove(key);
        Ok(())
    }

    fn clear(&mut self) -> Result<(), String> {
        self.data.clear();
        Ok(())
    }

    fn keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

/// Native timer implementation using tokio
pub struct NativeTimer;

impl super::Timer for NativeTimer {
    fn set_timeout<F>(callback: F, delay_ms: u32) -> u32
    where
        F: FnOnce() + Send + 'static,
    {
        // In native, we use tokio spawn for timeouts
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms as u64)).await;
            callback();
        });
        // Return a pseudo-ID (in production, track handles)
        0
    }

    fn clear_timeout(_id: u32) {
        // In production, would cancel the spawned task
    }

    fn set_interval<F>(callback: F, interval_ms: u32) -> u32
    where
        F: Fn() + Send + Sync + 'static,
    {
        let callback = Arc::new(callback);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(interval_ms as u64));
            loop {
                interval.tick().await;
                (callback)();
            }
        });
        0
    }

    fn clear_interval(_id: u32) {
        // In production, would cancel the spawned task
    }
}
