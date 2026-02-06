//! Blocking HTTP client wrapper for rquest
//!
//! rquest doesn't have a blocking module like reqwest, so we provide a simple
//! blocking wrapper that uses tokio's runtime to execute async HTTP requests.

use std::time::Duration;
use rquest::header::{HeaderMap, HeaderName, HeaderValue};

/// A blocking HTTP client that wraps rquest's async client
pub struct BlockingClient {
    client: rquest::Client,
    runtime: tokio::runtime::Runtime,
}

impl BlockingClient {
    /// Create a new blocking client with default settings
    pub fn new() -> Result<Self, rquest::Error> {
        Self::builder().build()
    }

    /// Create a new builder for configuring the client
    pub fn builder() -> BlockingClientBuilder {
        BlockingClientBuilder::new()
    }

    /// Perform a blocking GET request
    pub fn get(&self, url: &str) -> BlockingRequestBuilder {
        BlockingRequestBuilder {
            runtime: &self.runtime,
            request: self.client.get(url),
        }
    }
}

impl Default for BlockingClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default blocking client")
    }
}

/// Builder for creating a BlockingClient
pub struct BlockingClientBuilder {
    timeout: Option<Duration>,
}

impl BlockingClientBuilder {
    fn new() -> Self {
        Self { timeout: None }
    }

    /// Set the request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Build the blocking client
    pub fn build(self) -> Result<BlockingClient, rquest::Error> {
        let mut builder = rquest::Client::builder()
            .redirect(rquest::redirect::Policy::limited(10));

        if let Some(timeout) = self.timeout {
            builder = builder.timeout(timeout);
        }

        let client = builder.build()?;

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime for blocking client");

        Ok(BlockingClient { client, runtime })
    }
}

/// A request builder for blocking requests
pub struct BlockingRequestBuilder<'a> {
    runtime: &'a tokio::runtime::Runtime,
    request: rquest::RequestBuilder,
}

impl<'a> BlockingRequestBuilder<'a> {
    /// Add a header to the request
    pub fn header(mut self, key: &str, value: &str) -> Self {
        if let (Ok(name), Ok(val)) = (
            HeaderName::try_from(key),
            HeaderValue::try_from(value),
        ) {
            self.request = self.request.header(name, val);
        }
        self
    }

    /// Send the request and get a blocking response
    pub fn send(self) -> Result<BlockingResponse, rquest::Error> {
        self.runtime.block_on(async {
            let response = self.request.send().await?;
            Ok(BlockingResponse {
                response,
            })
        })
    }
}

/// A blocking response wrapper
pub struct BlockingResponse {
    response: rquest::Response,
}

impl BlockingResponse {
    /// Get the HTTP status code
    pub fn status(&self) -> rquest::StatusCode {
        self.response.status()
    }

    /// Get the response headers
    pub fn headers(&self) -> &HeaderMap {
        self.response.headers()
    }

    /// Get the response body as text (blocking)
    pub fn text(self) -> Result<String, rquest::Error> {
        // We need a runtime to block on the async text() method
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime for response text");

        rt.block_on(async {
            self.response.text().await
        })
    }

    /// Get the response body as bytes (blocking)
    pub fn bytes(self) -> Result<bytes::Bytes, rquest::Error> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime for response bytes");

        rt.block_on(async {
            self.response.bytes().await
        })
    }
}
