use crate::engine::renderer::core::RustRenderer;
use crate::engine::renderer::js_security::{JavaScriptSecurityValidator, SecurityContext};
use anyhow::Result;

impl RustRenderer {
    /// Validate JavaScript code for security risks (AI-injected context).
    ///
    /// This applies the full restrictive policy: blocks eval, Function, document.write,
    /// WebAssembly, prototype pollution, Node.js APIs, etc.
    ///
    /// Use `is_safe_page_javascript` for scripts from `<script>` tags which need
    /// eval/Function/document.write to work (Webpack, GTM, analytics).
    pub fn is_safe_javascript(&self, js_code: &str) -> bool {
        let validator = JavaScriptSecurityValidator::new();
        match validator.is_safe_javascript(js_code) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("🔒 SECURITY: JavaScript validation failed: {}", e);
                false
            }
        }
    }

    /// Validate JavaScript from page-loaded scripts (`<script>` tags, external JS).
    ///
    /// Uses a relaxed policy that allows eval, Function, document.write, and WebAssembly
    /// since these are standard browser features used by real websites (Webpack, GTM, etc.).
    /// Still blocks prototype pollution, constructor chains, and Node.js APIs.
    pub fn is_safe_page_javascript(&self, js_code: &str) -> bool {
        let validator = JavaScriptSecurityValidator::new();
        match validator.is_safe_page_javascript(js_code) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("🔒 SECURITY: Page script validation failed: {}", e);
                false
            }
        }
    }

    /// Get detailed security validation result with error message
    pub fn validate_javascript_detailed(&self, js_code: &str) -> Result<()> {
        let validator = JavaScriptSecurityValidator::new();
        validator.is_safe_javascript(js_code)
    }

    /// Get detailed security validation result for page scripts
    pub fn validate_page_javascript_detailed(&self, js_code: &str) -> Result<()> {
        let validator = JavaScriptSecurityValidator::new();
        validator.is_safe_page_javascript(js_code)
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

        // eval should be blocked for AI-injected scripts
        assert!(!renderer.is_safe_javascript("eval('alert(1)')"));
    }

    #[test]
    fn test_eval_allowed_for_page_scripts() {
        let renderer = create_test_renderer();

        // eval should be allowed for page scripts
        assert!(renderer.is_safe_page_javascript("eval('alert(1)')"));
        assert!(renderer.is_safe_page_javascript("new Function('return 1')()"));
        assert!(renderer.is_safe_page_javascript("document.write('<p>hi</p>')"));
    }

    #[test]
    fn test_function_constructor_blocked() {
        let renderer = create_test_renderer();

        // Function constructor should be blocked for AI-injected scripts
        assert!(!renderer.is_safe_javascript("Function('return 1')()"));
        assert!(!renderer.is_safe_javascript("new Function('return 1')()"));
    }

    #[test]
    fn test_proto_pollution_blocked_both_contexts() {
        let renderer = create_test_renderer();

        // __proto__ access should be blocked in both contexts
        assert!(!renderer.is_safe_javascript("obj.__proto__ = {}"));
        assert!(!renderer.is_safe_page_javascript("obj.__proto__ = {}"));
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
