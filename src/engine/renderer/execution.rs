use crate::engine::renderer::core::RustRenderer;
use anyhow::{Result, anyhow};
use std::error::Error;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use thalora_browser_apis::boa_engine::{Source, module::Module};

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
            Err(e) => Err(anyhow!("Google challenge execution failed: {}", e)),
        }
    }

    pub fn evaluate_javascript(&mut self, js_code: &str) -> Result<String> {
        self.evaluate_javascript_with_timeout(js_code, Duration::from_secs(5))
    }

    /// Execute JavaScript from page-loaded `<script>` tags (trusted context).
    /// Uses relaxed security that allows eval, Function, document.write, WebAssembly
    /// since these are standard browser features used by real websites.
    pub fn evaluate_page_javascript(&mut self, js_code: &str) -> Result<String> {
        self.evaluate_page_javascript_with_timeout(js_code, Duration::from_secs(5))
    }

    /// Execute JavaScript for browser interactions without wrapper interference
    pub fn evaluate_javascript_direct(&mut self, js_code: &str) -> Result<String> {
        // Security check
        if !self.is_safe_javascript(js_code) {
            return Err(anyhow!("JavaScript contains potentially dangerous code"));
        }

        // Execute JavaScript directly without wrapper for form interactions
        let source = Source::from_bytes(js_code);
        eprintln!(
            "🔍 DEBUG: About to eval direct JavaScript: {}",
            if js_code.len() > 200 {
                &js_code[..200]
            } else {
                js_code
            }
        );

        if let Some(ctx) = &mut self.js_context {
            match ctx.eval(source) {
                Ok(value) => {
                    eprintln!(
                        "🔍 DEBUG: Direct JavaScript eval succeeded, value type: {:?}",
                        value.get_type()
                    );
                    // Convert JS value to string - this should preserve JSON strings
                    let result = self.js_value_to_string(value);
                    eprintln!("🔍 DEBUG: Direct conversion to string: {}", result);
                    Ok(result)
                }
                Err(e) => {
                    eprintln!("🔍 DEBUG: Direct JavaScript execution error: {:?}", e);
                    Err(anyhow!("JavaScript execution failed: {}", e))
                }
            }
        } else {
            Err(anyhow!("JavaScript context not available"))
        }
    }

    /// Execute JavaScript safely for tests and polyfill environments
    pub async fn execute_javascript_safely(
        &mut self,
        js_code: &str,
    ) -> Result<thalora_browser_apis::boa_engine::JsValue> {
        use thalora_browser_apis::boa_engine::Source;

        // Security check
        if !self.is_safe_javascript(js_code) {
            return Err(anyhow!("JavaScript contains potentially dangerous code"));
        }

        // Execute JavaScript without wrapper for testing
        let source = Source::from_bytes(js_code);
        if let Some(ctx) = &mut self.js_context {
            match ctx.eval(source) {
                Ok(value) => Ok(value),
                Err(e) => Err(anyhow!("JavaScript execution failed: {}", e)),
            }
        } else {
            Err(anyhow!("JavaScript context not available"))
        }
    }

    pub fn inject_minimal_form_for_google(&mut self) -> Result<()> {
        let form_injection_code = r#"
            try {
                // Only inject if no forms exist (for Google challenge pages)
                if (document.forms.length === 0) {
                    var form = document.createElement('form');
                    form.id = 'google_compatibility_form';

                    // Add the search input that Google expects
                    var searchInput = document.createElement('input');
                    searchInput.name = 'q';
                    searchInput.type = 'text';
                    searchInput.value = '';

                    // Add to document body if it exists
                    if (document.body) {
                        document.body.appendChild(form);
                        form.appendChild(searchInput);
                    }
                }
                return 'Forms injected: ' + document.forms.length;
            } catch(e) {
                return 'Form injection error: ' + e.message;
            }
        "#;

        let result =
            self.evaluate_javascript_with_timeout(form_injection_code, Duration::from_secs(2))?;
        eprintln!("🔍 DEBUG: Form injection result: {}", result);
        Ok(())
    }

    pub fn test_shadow_dom_apis(&mut self) -> Result<String> {
        // Shadow DOM BorrowMutError fix: GcRefCell operations now use try_borrow/try_borrow_mut
        // to prevent panics from re-entrant access during attachShadow and property accessors.
        let js_code = r#"
            (function() {
                try {
                    var results = [];

                    // Test attachShadow exists
                    var div = document.createElement('div');
                    results.push('attachShadow: ' + (typeof div.attachShadow === 'function'));

                    // Test creating open shadow root
                    var shadow = div.attachShadow({mode: 'open'});
                    results.push('shadow created: ' + (shadow !== null && shadow !== undefined));

                    // Test shadowRoot property accessible
                    results.push('shadowRoot accessible: ' + (div.shadowRoot !== null));

                    // Test shadow root mode
                    results.push('mode: ' + (shadow.mode || 'unknown'));

                    return results.join(', ');
                } catch(e) {
                    return 'Shadow DOM error: ' + e.message;
                }
            })()
        "#;

        self.evaluate_javascript(js_code)
    }

    fn evaluate_javascript_with_timeout(
        &mut self,
        js_code: &str,
        timeout_duration: Duration,
    ) -> Result<String> {
        // Security check
        if !self.is_safe_javascript(js_code) {
            return Err(anyhow!("JavaScript contains potentially dangerous code"));
        }

        // Simple error-safe wrapper that prevents page JavaScript from crashing the engine.
        // Uses async IIFE when the script contains `await` to support top-level await.
        // Execute JavaScript directly without IIFE wrapper.
        // The IIFE wrapper was causing issues because:
        // 1. Expression results were lost (no `return` statement in IIFE)
        // 2. try/catch completion values differ between eval and function scope
        // Using eval() directly preserves expression return values correctly.
        let safe_wrapper = js_code.to_string();

        // Execute JavaScript directly without nested async handling
        let source = Source::from_bytes(&safe_wrapper);
        eprintln!(
            "🔍 DEBUG: About to eval JavaScript: {}",
            if safe_wrapper.len() > 200 {
                &safe_wrapper[..200]
            } else {
                &safe_wrapper
            }
        );

        // Run bot detection test BEFORE executing page scripts
        if safe_wrapper.contains("window.google") {
            eprintln!("🤖 RUNNING BOT DETECTION TEST ON GOOGLE PAGE");
            if let Some(ctx) = &mut self.js_context {
                let test_script = r#"
                console.log("=== BOT DETECTION ===");
                console.log("navigator:", typeof navigator);
                console.log("navigator.webdriver:", typeof navigator.webdriver, navigator.webdriver);
                console.log("navigator.plugins:", typeof navigator.plugins, navigator.plugins);
                console.log("navigator.plugins.length:", navigator.plugins ? navigator.plugins.length : "plugins is undefined/null");
                console.log("window.chrome:", typeof window.chrome);
                console.log("window.outerWidth:", typeof window.outerWidth, window.outerWidth);
                console.log("Image constructor:", typeof Image);
                console.log("screen:", typeof screen);
                console.log("=== END TEST ===");
                "#;
                let _ = ctx.eval(Source::from_bytes(test_script));
            }
        }

        if let Some(ctx) = &mut self.js_context {
            // SECURITY: Set execution deadline to enforce timeout
            let deadline = Instant::now() + timeout_duration;
            ctx.runtime_limits_mut().set_execution_deadline(deadline);

            let result = ctx.eval(source);

            // SECURITY: Always clear the deadline after execution
            ctx.runtime_limits_mut().clear_execution_deadline();

            match result {
                Ok(value) => {
                    eprintln!(
                        "🔍 DEBUG: JavaScript eval succeeded, value type: {:?}",
                        value.get_type()
                    );
                    // Convert JS value to string
                    let result = self.js_value_to_string(value);
                    eprintln!("🔍 DEBUG: Converted to string: {}", result);
                    Ok(result)
                }
                Err(e) => {
                    // Check if this was a timeout error
                    let error_str = format!("{}", e);
                    if error_str.contains("timeout") || error_str.contains("ExecutionTimeout") {
                        return Err(anyhow!(
                            "JavaScript execution timeout after {:?}",
                            timeout_duration
                        ));
                    }
                    // For Google's JavaScript, we'll be more forgiving of errors
                    eprintln!(
                        "🔍 DEBUG: JavaScript execution had recoverable error: {:?}",
                        e
                    );
                    eprintln!("🔴 JS ERROR DETAILS:");
                    eprintln!("   Error type: {}", e);
                    if let Some(cause) = e.source() {
                        eprintln!("   Caused by: {:?}", cause);
                    }
                    // Try to get more info from the JsError
                    eprintln!("   Full error: {:#?}", e);
                    Ok("undefined".to_string()) // Return success with undefined result
                }
            }
        } else {
            Ok("undefined".to_string())
        }
    }

    /// Execute page-loaded JavaScript with timeout (trusted context).
    /// Allows eval, Function, document.write, WebAssembly — standard browser behavior.
    /// Still blocks prototype pollution, constructor chains, and Node.js APIs.
    fn evaluate_page_javascript_with_timeout(
        &mut self,
        js_code: &str,
        timeout_duration: Duration,
    ) -> Result<String> {
        // Page script security check (relaxed — allows eval/Function/document.write/WASM)
        if !self.is_safe_page_javascript(js_code) {
            return Err(anyhow!("Page script contains dangerous code (prototype pollution or Node.js APIs)"));
        }

        let source = Source::from_bytes(js_code);

        if let Some(ctx) = &mut self.js_context {
            // SECURITY: Set execution deadline to enforce timeout
            let deadline = Instant::now() + timeout_duration;
            ctx.runtime_limits_mut().set_execution_deadline(deadline);

            let result = ctx.eval(source);

            // SECURITY: Always clear the deadline after execution
            ctx.runtime_limits_mut().clear_execution_deadline();

            match result {
                Ok(value) => {
                    let result = self.js_value_to_string(value);
                    Ok(result)
                }
                Err(e) => {
                    let error_str = format!("{}", e);
                    if error_str.contains("timeout") || error_str.contains("ExecutionTimeout") {
                        return Err(anyhow!(
                            "JavaScript execution timeout after {:?}",
                            timeout_duration
                        ));
                    }
                    // Page scripts often have recoverable errors — don't abort
                    eprintln!("⚠️  Page script execution error: {}", e);
                    Ok("undefined".to_string())
                }
            }
        } else {
            Ok("undefined".to_string())
        }
    }

    /// Evaluate JavaScript source as an ES module.
    ///
    /// Parses the source as a module (not a classic script), then loads
    /// dependencies, links, and evaluates. The URL is used as the module's
    /// path for relative import resolution in subsequent imports.
    pub fn evaluate_module(&mut self, source: &str, url: &str) -> Result<String> {
        if let Some(ctx) = &mut self.js_context {
            // Use URL-as-PathBuf so referrer.path() works for child imports
            let path = PathBuf::from(url);
            let src = Source::from_bytes(source.as_bytes()).with_path(&path);

            // Parse as ES module
            let module = Module::parse(src, None, ctx)
                .map_err(|e| anyhow!("Module parse error for '{}': {}", url, e))?;

            // Load dependencies, link, and evaluate
            let _promise = module.load_link_evaluate(ctx);

            // Flush the job queue to drive async module loading
            if let Err(e) = ctx.run_jobs() {
                eprintln!("⚠️  Module job queue error for '{}': {}", url, e);
            }

            Ok("module evaluated".to_string())
        } else {
            Err(anyhow!("JavaScript context not available"))
        }
    }

    /// Update the HTTP module loader's base URL for relative import resolution.
    pub fn set_module_base_url(&mut self, url: &str) {
        if let Some(ctx) = &self.js_context {
            if let Some(loader) = ctx.downcast_module_loader::<crate::engine::browser::module_loader::HttpModuleLoader>() {
                loader.set_base_url(url);
            }
        }
    }
}
