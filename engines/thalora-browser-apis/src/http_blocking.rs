//! Blocking HTTP client wrapper for rquest
//!
//! rquest doesn't have a blocking module like reqwest, so we provide a simple
//! blocking wrapper that uses tokio's runtime to execute async HTTP requests.

use std::sync::OnceLock;
use std::time::Duration;
use rquest::header::{HeaderMap, HeaderName, HeaderValue};

/// Global shared HTTP client, initialized from the browser's configured client.
/// This allows script fetching and other APIs to reuse the browser's Chrome131-emulated
/// client instead of creating bare new clients that lack TLS fingerprinting, cookies,
/// and compression support.
static SHARED_CLIENT: OnceLock<rquest::Client> = OnceLock::new();

/// Initialize the shared client (called once during browser setup).
/// If already set, this is a no-op.
pub fn set_shared_client(client: rquest::Client) {
    let _ = SHARED_CLIENT.set(client);
}

/// Get the shared client, or create a basic fallback if not initialized.
pub fn get_shared_client() -> rquest::Client {
    SHARED_CLIENT.get().cloned().unwrap_or_else(|| {
        rquest::Client::builder()
            .redirect(rquest::redirect::Policy::limited(10))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create fallback HTTP client")
    })
}

/// Execute a future in a blocking context, handling nested runtime detection.
/// If already inside a tokio runtime, spawns a separate OS thread with its own runtime
/// to avoid the "Cannot start a runtime from within a runtime" panic.
pub fn block_on_compat<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        // Inside an existing runtime — spawn thread with fresh runtime
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");
            rt.block_on(future)
        })
        .join()
        .expect("Blocking thread panicked")
    } else {
        // Not in a runtime — create one directly
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");
        rt.block_on(future)
    }
}

/// A blocking HTTP client that wraps rquest's async client
pub struct BlockingClient {
    client: rquest::Client,
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

        Ok(BlockingClient { client })
    }
}

/// A request builder for blocking requests
pub struct BlockingRequestBuilder {
    request: rquest::RequestBuilder,
}

impl BlockingRequestBuilder {
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
        let request = self.request;
        block_on_compat(async {
            let response = request.send().await?;
            Ok(BlockingResponse { response })
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
        let response = self.response;
        block_on_compat(async { response.text().await })
    }

    /// Get the response body as bytes (blocking)
    pub fn bytes(self) -> Result<bytes::Bytes, rquest::Error> {
        let response = self.response;
        block_on_compat(async { response.bytes().await })
    }
}
