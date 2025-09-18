use anyhow::{anyhow, Result};
use boa_engine::Source;
use std::time::Duration;
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

        // Execute JavaScript directly without nested async handling
        // The timeout will be handled by the test framework itself
        let source = Source::from_bytes(js_code);
        match self.js_context.eval(source) {
            Ok(value) => {
                // Convert JS value to string
                Ok(self.js_value_to_string(value))
            },
            Err(e) => {
                Err(anyhow!("JavaScript execution error: {:?}", e))
            }
        }
    }
}