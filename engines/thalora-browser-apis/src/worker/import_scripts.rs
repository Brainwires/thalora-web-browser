//! importScripts() implementation for Web Workers
//!
//! Implements the synchronous script loading mechanism for workers as defined in:
//! https://html.spec.whatwg.org/multipage/workers.html#dom-workerglobalscope-importscripts

use boa_engine::{Context, JsNativeError, JsResult, JsValue, Source};
use std::time::Duration;
use url::Url;

/// Script fetcher for importScripts()
pub struct ScriptImporter {
    /// Base URL for relative script resolution
    base_url: Option<Url>,
    /// HTTP client for fetching scripts
    client: reqwest::blocking::Client,
}

impl ScriptImporter {
    /// Create a new script importer with a base URL
    pub fn new(base_url: Option<String>) -> Self {
        let parsed_base_url = base_url.and_then(|url_str| Url::parse(&url_str).ok());

        // Create HTTP client with reasonable timeout
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client for importScripts");

        Self {
            base_url: parsed_base_url,
            client,
        }
    }

    /// Import multiple scripts synchronously
    ///
    /// This function blocks until all scripts are fetched and executed.
    /// According to the spec, importScripts() must be synchronous.
    pub fn import_scripts(&self, urls: &[String], context: &mut Context) -> JsResult<()> {
        eprintln!("[importScripts] Importing {} script(s)", urls.len());

        for url_str in urls {
            // Resolve the URL (handle relative URLs)
            let absolute_url = self.resolve_url(url_str)?;

            eprintln!("[importScripts] Fetching: {}", absolute_url);

            // Fetch the script content
            let script_content = self.fetch_script(&absolute_url)?;

            eprintln!("[importScripts] Fetched {} bytes", script_content.len());

            // Execute the script in the current context
            self.execute_script(&script_content, &absolute_url, context)?;

            eprintln!("[importScripts] Executed: {}", absolute_url);
        }

        Ok(())
    }

    /// Resolve a URL relative to the base URL
    fn resolve_url(&self, url_str: &str) -> JsResult<String> {
        // Try to parse as absolute URL first
        if let Ok(parsed) = Url::parse(url_str) {
            return Ok(parsed.to_string());
        }

        // If we have a base URL, try relative resolution
        if let Some(ref base) = self.base_url {
            match base.join(url_str) {
                Ok(resolved) => Ok(resolved.to_string()),
                Err(e) => Err(JsNativeError::uri()
                    .with_message(format!("Failed to resolve URL '{}': {}", url_str, e))
                    .into()),
            }
        } else {
            // No base URL and not absolute - this is an error
            Err(JsNativeError::uri()
                .with_message(format!(
                    "Cannot resolve relative URL '{}' without base URL",
                    url_str
                ))
                .into())
        }
    }

    /// Fetch a script from a URL
    fn fetch_script(&self, url: &str) -> JsResult<String> {
        // Check URL scheme - only http/https are supported
        let parsed_url = Url::parse(url)
            .map_err(|e| JsNativeError::uri().with_message(format!("Invalid URL: {}", e)))?;

        match parsed_url.scheme() {
            "http" | "https" => {
                // Fetch via HTTP/HTTPS
                self.fetch_http(url)
            }
            "file" => {
                // File URLs not supported in workers for security reasons
                Err(JsNativeError::typ()
                    .with_message("file:// URLs are not supported in importScripts()")
                    .into())
            }
            "data" => {
                // Data URLs - extract the content
                self.extract_from_data_url(url)
            }
            scheme => Err(JsNativeError::typ()
                .with_message(format!("Unsupported URL scheme: {}", scheme))
                .into()),
        }
    }

    /// Fetch a script via HTTP/HTTPS
    fn fetch_http(&self, url: &str) -> JsResult<String> {
        let response = self.client.get(url).send().map_err(|e| {
            JsNativeError::error().with_message(format!("Network error fetching script: {}", e))
        })?;

        // Check HTTP status
        if !response.status().is_success() {
            return Err(JsNativeError::error()
                .with_message(format!(
                    "HTTP error {} fetching script from {}",
                    response.status(),
                    url
                ))
                .into());
        }

        // Get the text content
        let content = response.text().map_err(|e| {
            JsNativeError::error().with_message(format!("Failed to read script content: {}", e))
        })?;

        Ok(content)
    }

    /// Extract content from a data URL
    fn extract_from_data_url(&self, data_url: &str) -> JsResult<String> {
        // Basic data URL parsing: data:[<mediatype>][;base64],<data>
        if let Some(comma_pos) = data_url.find(',') {
            let content = &data_url[comma_pos + 1..];

            // Check if base64 encoded
            if data_url[..comma_pos].contains(";base64") {
                use base64::{Engine as _, engine::general_purpose};
                let decoded = general_purpose::STANDARD.decode(content).map_err(|e| {
                    JsNativeError::error()
                        .with_message(format!("Failed to decode base64 data URL: {}", e))
                })?;
                String::from_utf8(decoded).map_err(|e| {
                    JsNativeError::error()
                        .with_message(format!("Invalid UTF-8 in data URL: {}", e))
                        .into()
                })
            } else {
                // URL decode the content
                urlencoding::decode(content)
                    .map(|s| s.to_string())
                    .map_err(|e| {
                        JsNativeError::error()
                            .with_message(format!("Failed to decode data URL: {}", e))
                            .into()
                    })
            }
        } else {
            Err(JsNativeError::error()
                .with_message("Invalid data URL format")
                .into())
        }
    }

    /// Execute a script in the given context
    fn execute_script(
        &self,
        script_content: &str,
        script_url: &str,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Create source with the script URL as a comment for better error messages
        let source_with_comment = format!("// Source: {}\n{}", script_url, script_content);
        let source = Source::from_bytes(&source_with_comment);

        // Execute the script
        context.eval(source).map_err(|e| {
            eprintln!(
                "[importScripts] Script execution error in {}: {:?}",
                script_url, e
            );
            e
        })
    }
}

/// Implementation of the global importScripts() function
///
/// This is called from WorkerGlobalScope.importScripts()
pub fn import_scripts_impl(
    urls: Vec<String>,
    base_url: Option<String>,
    context: &mut Context,
) -> JsResult<()> {
    let importer = ScriptImporter::new(base_url);
    importer.import_scripts(&urls, context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_url_extraction() {
        let importer = ScriptImporter::new(None);

        // Test plain data URL
        let data_url = "data:application/javascript,console.log('test');";
        let result = importer.extract_from_data_url(data_url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "console.log('test');");

        // Test base64 data URL
        let script = "console.log('base64');";
        use base64::{Engine as _, engine::general_purpose};
        let encoded = general_purpose::STANDARD.encode(script);
        let data_url_b64 = format!("data:application/javascript;base64,{}", encoded);
        let result = importer.extract_from_data_url(&data_url_b64);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), script);
    }

    #[test]
    fn test_url_resolution() {
        // Test absolute URL
        let importer = ScriptImporter::new(None);
        let result = importer.resolve_url("https://example.com/script.js");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com/script.js");

        // Test relative URL with base
        let importer = ScriptImporter::new(Some("https://example.com/workers/".to_string()));
        let result = importer.resolve_url("script.js");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://example.com/workers/script.js");

        // Test relative URL without base (should fail)
        let importer = ScriptImporter::new(None);
        let result = importer.resolve_url("script.js");
        assert!(result.is_err());
    }

    #[test]
    fn test_import_data_url_script() {
        let mut context = Context::default();
        let importer = ScriptImporter::new(None);

        // Create a simple test script
        let script = "var testVar = 42;";
        let data_url = format!(
            "data:application/javascript,{}",
            urlencoding::encode(script)
        );

        let result = importer.import_scripts(&[data_url], &mut context);
        assert!(result.is_ok());

        // Verify the script was executed
        let test_var = context.eval(Source::from_bytes("testVar")).unwrap();
        assert_eq!(test_var.to_number(&mut context).unwrap(), 42.0);
    }
}
