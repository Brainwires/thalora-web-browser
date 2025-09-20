use crate::engine::renderer::core::RustRenderer;

impl RustRenderer {
    /// Validate JavaScript code for security risks - now much more permissive for V8 compliance
    pub fn is_safe_javascript(&self, js_code: &str) -> bool {
        // Size limit for safety (10MB should be more than enough)
        if js_code.len() > 10_000_000 {
            return false;
        }

        // Only block truly dangerous patterns that could escape the sandbox
        let truly_dangerous_patterns = [
            "process.exit",
            "process.abort",
            "process.kill",
            "require('fs')",
            "require(\"fs\")",
            "require('child_process')",
            "require(\"child_process\")",
            "require('os')",
            "require(\"os\")",
            "__dirname",
            "__filename",
            "global.process",
            "Buffer.allocUnsafe",
            ".constructor.constructor",
        ];

        // Check for truly dangerous patterns only
        for pattern in truly_dangerous_patterns.iter() {
            if js_code.contains(pattern) {
                return false;
            }
        }

        // Allow everything else - this is V8-compliant JavaScript execution
        true
    }
}