use crate::engine::renderer::core::RustRenderer;
use crate::engine::renderer::js_security::JavaScriptSecurityValidator;
use anyhow::Result;

impl RustRenderer {
    /// Validate JavaScript code for security risks using regex-based analysis
    ///
    /// SECURITY POLICY (HARD BLOCKS - No feature flags):
    /// - eval() calls are BLOCKED
    /// - Function() constructor is BLOCKED
    /// - setTimeout/setInterval with strings are BLOCKED
    /// - __proto__ access is BLOCKED (prototype pollution)
    /// - constructor.constructor is BLOCKED (Function access)
    /// - with statements are BLOCKED
    /// - document.write is BLOCKED
    /// - WebAssembly instantiation is BLOCKED
    /// - Node.js APIs are BLOCKED
    ///
    /// ALLOWED (legitimate browser features):
    /// - import/import() — needed for Astro/React/Vue hydration
    /// - window[key]/self[key] — standard JS (eval-bracket caught separately)
    /// - Symbol, Reflect, Proxy — standard ES6+ APIs
    ///
    /// This is a HARD BLOCK implementation - no environment variables can override.
    /// For AI automation use cases that require dangerous operations, use dedicated
    /// trusted execution contexts with explicit user consent.
    pub fn is_safe_javascript(&self, js_code: &str) -> bool {
        // Create validator
        let validator = JavaScriptSecurityValidator::new();

        // Validate code - return false if validation fails
        match validator.is_safe_javascript(js_code) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("🔒 SECURITY: JavaScript validation failed: {}", e);
                false
            }
        }
    }

    /// Get detailed security validation result with error message
    pub fn validate_javascript_detailed(&self, js_code: &str) -> Result<()> {
        let validator = JavaScriptSecurityValidator::new();
        validator.is_safe_javascript(js_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_renderer() -> RustRenderer {
        RustRenderer::new()
    }

    #[test]
    fn test_safe_javascript_accepted() {
        let renderer = create_test_renderer();

        // Safe code should pass
        assert!(renderer.is_safe_javascript("const x = 1 + 2;"));
        assert!(renderer.is_safe_javascript("function test() { return 'safe'; }"));
        assert!(renderer.is_safe_javascript("console.log('Hello');"));
    }

    #[test]
    fn test_eval_blocked() {
        let renderer = create_test_renderer();

        // eval should be blocked
        assert!(!renderer.is_safe_javascript("eval('alert(1)')"));
    }

    #[test]
    fn test_function_constructor_blocked() {
        let renderer = create_test_renderer();

        // Function constructor should be blocked
        assert!(!renderer.is_safe_javascript("Function('return 1')()"));
        assert!(!renderer.is_safe_javascript("new Function('return 1')()"));
    }

    #[test]
    fn test_proto_pollution_blocked() {
        let renderer = create_test_renderer();

        // __proto__ access should be blocked
        assert!(!renderer.is_safe_javascript("obj.__proto__ = {}"));
    }

    #[test]
    fn test_detailed_validation_error() {
        let renderer = create_test_renderer();

        // Get detailed error message
        let result = renderer.validate_javascript_detailed("eval('test')");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("eval"));
    }
}
