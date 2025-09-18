use anyhow::{anyhow, Result};
use boa_engine::Source;
use std::time::Duration;
use tokio::time::timeout;
use crate::engine::renderer::core::RustRenderer;

impl RustRenderer {
    pub fn handle_google_challenge(&mut self, js_code: &str) -> Result<String> {
        // Security check for Google challenge JavaScript
        if !self.is_safe_javascript(js_code) {
            return Err(anyhow!("JavaScript contains potentially dangerous code"));
        }

        // Execute with timeout
        let result = self.evaluate_javascript_with_timeout(js_code, Duration::from_secs(5));
        match result {
            Ok(result) => Ok(result),
            Err(e) => Err(anyhow!("Google challenge execution failed: {}", e))
        }
    }

    pub fn evaluate_javascript(&mut self, js_code: &str) -> Result<String> {
        self.evaluate_javascript_with_timeout(js_code, Duration::from_secs(5))
    }

    fn evaluate_javascript_with_timeout(&mut self, js_code: &str, timeout_duration: Duration) -> Result<String> {
        // Security check
        if !self.is_safe_javascript(js_code) {
            return Err(anyhow!("JavaScript contains potentially dangerous code"));
        }

        // Use tokio::time::timeout for async timeout handling
        let handle = tokio::runtime::Handle::current();
        let js_context = &mut self.js_context;

        let result = handle.block_on(async {
            timeout(timeout_duration, async {
                // Execute JavaScript synchronously within the async block
                let source = Source::from_bytes(js_code);
                js_context.eval(source)
            }).await
        });

        match result {
            Ok(Ok(value)) => {
                // Convert JS value to string
                Ok(self.js_value_to_string(value))
            },
            Ok(Err(e)) => {
                Err(anyhow!("JavaScript execution error: {:?}", e))
            },
            Err(_) => {
                Err(anyhow!("JavaScript execution timed out after {:?}", timeout_duration))
            }
        }
    }
}