use anyhow::{Result, anyhow};
use regex::Regex;
use std::sync::OnceLock;

/// Execution context for JavaScript security validation.
///
/// Page-loaded scripts (from `<script>` tags in HTML) are trusted because they come
/// from the website itself — blocking eval/Function/document.write in page scripts
/// breaks Webpack, Google Tag Manager, analytics, and most real-world websites.
///
/// AI-injected scripts (from MCP tools, CDP evaluate, etc.) are untrusted and get
/// the full restrictive security policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityContext {
    /// Scripts from `<script>` tags, external JS files, and page-initiated code.
    /// Allows eval(), Function(), document.write(), WebAssembly — standard browser behavior.
    /// Still blocks prototype pollution, constructor chains, and Node.js APIs.
    PageScript,
    /// Scripts injected by AI agents via MCP tools or CDP.
    /// Full restrictive policy: blocks eval, Function, document.write, WebAssembly, etc.
    AiInjected,
}

/// JavaScript security validator using comprehensive regex patterns
/// This provides strong security without external AST parsing dependencies
pub struct JavaScriptSecurityValidator {
    /// Maximum code size in bytes
    max_code_size: usize,
}

impl JavaScriptSecurityValidator {
    pub fn new() -> Self {
        Self {
            max_code_size: 10_000_000, // 10 MB
        }
    }

    /// Validate JavaScript code for security risks (AI-injected context by default).
    ///
    /// SECURITY POLICY (HARD BLOCKS for AiInjected context):
    /// - Block eval() calls (arbitrary code execution)
    /// - Block Function() constructor (dynamic code generation)
    /// - Block setTimeout/setInterval with string arguments (code execution)
    /// - Block __proto__ access (prototype pollution)
    /// - Block constructor.constructor chains (Function access)
    /// - Block document.write (XSS vector)
    /// - Block WebAssembly instantiation (sandboxing)
    /// - Block Node.js-specific APIs (host access)
    ///
    /// ALLOWED (legitimate browser JS features):
    /// - import() and import statements (used by Astro, React, Vue for hydration/code-splitting)
    /// - window[key], self[key] (standard JS pattern; eval-bracket already caught separately)
    /// - Symbol, Reflect, Proxy (standard ES6+ features used by Vue, React, etc.)
    ///
    /// For PageScript context, eval/Function/document.write/WebAssembly are allowed
    /// since they are standard browser features used by real websites (Webpack, GTM, etc.).
    pub fn is_safe_javascript(&self, js_code: &str) -> Result<()> {
        self.validate(js_code, SecurityContext::AiInjected)
    }

    /// Validate JavaScript from page-loaded scripts (`<script>` tags, external JS files).
    /// Uses a relaxed policy that allows eval, Function, document.write, and WebAssembly
    /// since these are standard browser features used by real websites.
    pub fn is_safe_page_javascript(&self, js_code: &str) -> Result<()> {
        self.validate(js_code, SecurityContext::PageScript)
    }

    /// Core validation with configurable security context.
    pub fn validate(&self, js_code: &str, context: SecurityContext) -> Result<()> {
        // Size limit check
        if js_code.len() > self.max_code_size {
            return Err(anyhow!(
                "JavaScript code too large: {} bytes (max: {} bytes)",
                js_code.len(),
                self.max_code_size
            ));
        }

        // Empty code is safe
        if js_code.trim().is_empty() {
            return Ok(());
        }

        // Always check for bypass vectors in original code
        self.check_proto_bracket(js_code)?;

        // Remove comments and strings to prevent false positives for other checks
        let code_without_comments = self.remove_comments_and_strings(js_code);

        // Always block prototype pollution and Node.js APIs regardless of context
        self.check_proto_pollution(&code_without_comments)?;
        self.check_constructor_access(&code_without_comments)?;
        self.check_node_apis(&code_without_comments)?;

        // Additional restrictions for AI-injected scripts only
        if context == SecurityContext::AiInjected {
            self.check_eval_bracket(js_code)?;
            self.check_escape_sequences(js_code)?;
            self.check_eval(&code_without_comments)?;
            self.check_function_constructor(&code_without_comments)?;
            self.check_timeout_with_strings(js_code)?;
            self.check_constructor_after_literal(&code_without_comments)?;
            self.check_async_generator_constructor(&code_without_comments)?;
            self.check_with_statement(&code_without_comments)?;
            self.check_document_write(&code_without_comments)?;
            self.check_webassembly(&code_without_comments)?;
        }

        Ok(())
    }

    /// Remove comments and string literals to prevent bypass via comments/strings
    fn remove_comments_and_strings(&self, code: &str) -> String {
        let mut result = String::with_capacity(code.len());
        let chars: Vec<char> = code.chars().collect();
        let mut i = 0;
        let len = chars.len();

        while i < len {
            // Single-line comment
            if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
                // Skip until end of line
                while i < len && chars[i] != '\n' {
                    i += 1;
                }
                result.push(' '); // Replace with space
                continue;
            }

            // Multi-line comment
            if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
                // Skip until */
                i += 2;
                while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2; // Skip */
                result.push(' '); // Replace with space
                continue;
            }

            // Single-quoted string
            if chars[i] == '\'' {
                result.push(' '); // Replace string with space
                i += 1;
                while i < len && chars[i] != '\'' {
                    if chars[i] == '\\' && i + 1 < len {
                        i += 2; // Skip escaped character
                    } else {
                        i += 1;
                    }
                }
                i += 1; // Skip closing quote
                continue;
            }

            // Double-quoted string
            if chars[i] == '"' {
                result.push(' '); // Replace string with space
                i += 1;
                while i < len && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < len {
                        i += 2; // Skip escaped character
                    } else {
                        i += 1;
                    }
                }
                i += 1; // Skip closing quote
                continue;
            }

            // Template literal
            if chars[i] == '`' {
                result.push(' '); // Replace template with space
                i += 1;
                while i < len && chars[i] != '`' {
                    if chars[i] == '\\' && i + 1 < len {
                        i += 2; // Skip escaped character
                    } else {
                        i += 1;
                    }
                }
                i += 1; // Skip closing backtick
                continue;
            }

            // Regular character
            result.push(chars[i]);
            i += 1;
        }

        result
    }

    /// Check for eval() calls (direct calls)
    fn check_eval(&self, code: &str) -> Result<()> {
        static EVAL_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = EVAL_REGEX.get_or_init(|| {
            // Match: eval(, window.eval(, globalThis.eval(, this.eval(
            Regex::new(
                r"(?:^|[^a-zA-Z0-9_$])(?:window\s*\.\s*|globalThis\s*\.\s*|this\s*\.\s*)?eval\s*\(",
            )
            .unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: eval() is not allowed (arbitrary code execution)"
            ));
        }

        Ok(())
    }

    /// Check for eval accessed via brackets: window["eval"]
    fn check_eval_bracket(&self, code: &str) -> Result<()> {
        static EVAL_BRACKET_REGEX: OnceLock<Regex> = OnceLock::new();
        let bracket_regex = EVAL_BRACKET_REGEX.get_or_init(|| {
            Regex::new(r#"(?:window|globalThis|this)\s*\[\s*["']eval["']\s*\]"#).unwrap()
        });

        if bracket_regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: eval() is not allowed (arbitrary code execution)"
            ));
        }

        Ok(())
    }

    /// Check for Function constructor
    fn check_function_constructor(&self, code: &str) -> Result<()> {
        static FUNCTION_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = FUNCTION_REGEX.get_or_init(|| {
            // Match: Function(, new Function(, window.Function(
            Regex::new(r"(?:^|[^a-zA-Z0-9_$])(?:new\s+)?(?:window\s*\.\s*|globalThis\s*\.\s*)?Function\s*\(").unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: Function() constructor is not allowed (code generation)"
            ));
        }

        Ok(())
    }

    /// Check for setTimeout/setInterval with string arguments
    fn check_timeout_with_strings(&self, code: &str) -> Result<()> {
        // This is harder to detect perfectly without AST, but we can catch common patterns
        static TIMEOUT_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = TIMEOUT_REGEX.get_or_init(|| {
            // Match: setTimeout( followed by a quote within reasonable distance
            Regex::new(r#"(?:setTimeout|setInterval)\s*\(\s*["'`]"#).unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: setTimeout/setInterval with string argument is not allowed (code execution)"
            ));
        }

        Ok(())
    }

    /// Check for __proto__ access via dot notation
    fn check_proto_pollution(&self, code: &str) -> Result<()> {
        static PROTO_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = PROTO_REGEX.get_or_init(|| {
            // Match: .__proto__
            Regex::new(r"\.\s*__proto__").unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: __proto__ access is not allowed (prototype pollution)"
            ));
        }

        Ok(())
    }

    /// Check for __proto__ access via bracket notation: ['__proto__'], ["__proto__"]
    fn check_proto_bracket(&self, code: &str) -> Result<()> {
        static PROTO_BRACKET_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = PROTO_BRACKET_REGEX.get_or_init(|| {
            // Match: ['__proto__'], ["__proto__"]
            Regex::new(r#"\[\s*["']__proto__["']\s*\]"#).unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: __proto__ access is not allowed (prototype pollution)"
            ));
        }

        Ok(())
    }

    /// Check for constructor.constructor access
    fn check_constructor_access(&self, code: &str) -> Result<()> {
        static CONSTRUCTOR_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = CONSTRUCTOR_REGEX.get_or_init(|| {
            // Match: .constructor.constructor
            Regex::new(r"\.\s*constructor\s*\.\s*constructor").unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: constructor.constructor is not allowed (Function access)"
            ));
        }

        Ok(())
    }

    /// Check for with statement
    fn check_with_statement(&self, code: &str) -> Result<()> {
        static WITH_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = WITH_REGEX.get_or_init(|| {
            // Match: with (
            Regex::new(r"(?:^|[^a-zA-Z0-9_$])with\s*\(").unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: 'with' statements are not allowed (ambiguous scope)"
            ));
        }

        Ok(())
    }

    /// Check for import statements (DISABLED — import/import() is standard browser JS
    /// needed by Astro, React, Vue for hydration and code-splitting)
    #[allow(dead_code)]
    fn check_import_statements(&self, code: &str) -> Result<()> {
        static IMPORT_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = IMPORT_REGEX.get_or_init(|| {
            // Match: import { or import * or import " or import word
            Regex::new(r#"(?:^|[^a-zA-Z0-9_$])import\s+(?:\{|\*|["']|\w)"#).unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: import statements are not allowed (dynamic module loading)"
            ));
        }

        // Also check for dynamic import()
        static DYNAMIC_IMPORT_REGEX: OnceLock<Regex> = OnceLock::new();
        let dynamic_regex = DYNAMIC_IMPORT_REGEX
            .get_or_init(|| Regex::new(r"(?:^|[^a-zA-Z0-9_$])import\s*\(").unwrap());

        if dynamic_regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: dynamic import() is not allowed (module loading)"
            ));
        }

        Ok(())
    }

    /// Check for document.write
    fn check_document_write(&self, code: &str) -> Result<()> {
        static DOC_WRITE_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = DOC_WRITE_REGEX.get_or_init(|| {
            // Match: document.write(, document.writeln(
            Regex::new(r"document\s*\.\s*(?:write|writeln)\s*\(").unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: document.write() is not allowed (XSS vector)"
            ));
        }

        Ok(())
    }

    /// Check for WebAssembly instantiation
    fn check_webassembly(&self, code: &str) -> Result<()> {
        static WASM_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = WASM_REGEX.get_or_init(|| {
            // Match: WebAssembly.instantiate, new WebAssembly.
            Regex::new(r"(?:WebAssembly\s*\.\s*(?:instantiate|compile|Module|Instance)|new\s+WebAssembly\s*\.)").unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: WebAssembly instantiation is not allowed"
            ));
        }

        Ok(())
    }

    /// Check for Node.js-specific APIs
    fn check_node_apis(&self, code: &str) -> Result<()> {
        static NODE_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = NODE_REGEX.get_or_init(|| {
            // Match: require(, process., child_process, fs.
            Regex::new(r"(?:^|[^a-zA-Z0-9_$])(?:require\s*\(|process\s*\.|child_process|fs\s*\.)")
                .unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: Node.js APIs are not allowed (require, process, fs, etc.)"
            ));
        }

        Ok(())
    }

    /// Block ALL bracket notation access to global objects (DISABLED — too aggressive,
    /// blocks legitimate code like OneTrust. window["eval"] is caught by check_eval_bracket)
    #[allow(dead_code)]
    fn check_global_bracket_access(&self, code: &str) -> Result<()> {
        static GLOBAL_BRACKET_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = GLOBAL_BRACKET_REGEX.get_or_init(|| {
            // Match: window[, globalThis[, self[
            Regex::new(r"(?:window|globalThis|self)\s*\[").unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: Bracket notation access to global objects is not allowed (prevents computed property bypass)"
            ));
        }

        Ok(())
    }

    /// CRITICAL FIX #2: Block Unicode and hex escape sequences in strings
    /// Prevents: window['\x65\x76\x61\x6c'], obj['\u005f\u005fproto\u005f\u005f']
    fn check_escape_sequences(&self, code: &str) -> Result<()> {
        // Check for hex escapes (\x) in bracket notation
        static HEX_ESCAPE_REGEX: OnceLock<Regex> = OnceLock::new();
        let hex_regex = HEX_ESCAPE_REGEX.get_or_init(|| {
            // Match: strings with \x or \u escape sequences in bracket notation
            Regex::new(r#"\[['"](?:[^'"]*\\[xu][0-9a-fA-F]+[^'"]*)+['"]\]"#).unwrap()
        });

        if hex_regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: Escape sequences (\\x, \\u) in bracket notation are not allowed (prevents encoding bypass)"
            ));
        }

        // Also block standalone hex/unicode escapes near dangerous keywords
        static ESCAPE_NEAR_DANGEROUS_REGEX: OnceLock<Regex> = OnceLock::new();
        let dangerous_regex = ESCAPE_NEAR_DANGEROUS_REGEX.get_or_init(|| {
            // Match: \x or \u near eval, proto, etc.
            Regex::new(r#"\\[xu][0-9a-fA-F]+.*(?:eval|proto|constructor|Function)"#).unwrap()
        });

        if dangerous_regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: Escape sequences near dangerous keywords are not allowed"
            ));
        }

        Ok(())
    }

    /// CRITICAL FIX #3: Block .constructor access after literals (primitives)
    /// Prevents: (0).constructor.constructor, [].constructor.constructor, ({}).constructor.constructor
    fn check_constructor_after_literal(&self, code: &str) -> Result<()> {
        static CONSTRUCTOR_LITERAL_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = CONSTRUCTOR_LITERAL_REGEX.get_or_init(|| {
            // Match: ).constructor or ].constructor or }.constructor
            Regex::new(r#"[\)\]\}]\s*\.\s*constructor"#).unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: Accessing .constructor on literals is not allowed (prevents Function constructor access)"
            ));
        }

        Ok(())
    }

    /// HIGH PRIORITY FIX: Block async and generator function constructor access
    /// Prevents: (async function(){}).constructor, (function*(){}).constructor
    fn check_async_generator_constructor(&self, code: &str) -> Result<()> {
        static ASYNC_GENERATOR_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = ASYNC_GENERATOR_REGEX.get_or_init(|| {
            // Match: async function patterns or generator patterns followed by .constructor
            Regex::new(r"(?:async\s+function|function\s*\*)[^}]*\}\s*\)\s*\.\s*constructor")
                .unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: Async/Generator function constructor access is not allowed"
            ));
        }

        Ok(())
    }

    /// Block Reflect API (DISABLED — standard ES6 feature used by Vue, MobX, etc.)
    #[allow(dead_code)]
    fn check_reflect_api(&self, code: &str) -> Result<()> {
        static REFLECT_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = REFLECT_REGEX.get_or_init(|| {
            // Match: Reflect.get, Reflect.apply, Reflect.construct, Reflect.defineProperty, Reflect.setPrototypeOf
            Regex::new(r"Reflect\s*\.\s*(?:get|apply|construct|defineProperty|setPrototypeOf|getOwnPropertyDescriptor)\s*\(").unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: Reflect API methods are not allowed (prevents indirect access bypass)"
            ));
        }

        Ok(())
    }

    /// Block Symbol API (DISABLED — standard ES6 feature used by React, etc.)
    #[allow(dead_code)]
    fn check_symbol_api(&self, code: &str) -> Result<()> {
        static SYMBOL_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = SYMBOL_REGEX.get_or_init(|| {
            // Match: Symbol.for(, Symbol(
            Regex::new(r"Symbol\s*(?:\.\s*for\s*)?\(").unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: Symbol API is not allowed (prevents dynamic property key bypass)"
            ));
        }

        Ok(())
    }

    /// Block Proxy usage (DISABLED — standard ES6 feature used by Vue reactivity, MobX, etc.)
    #[allow(dead_code)]
    fn check_proxy_usage(&self, code: &str) -> Result<()> {
        static PROXY_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = PROXY_REGEX.get_or_init(|| {
            // Match: new Proxy(
            Regex::new(r"(?:^|[^a-zA-Z0-9_$])new\s+Proxy\s*\(").unwrap()
        });

        if regex.is_match(code) {
            return Err(anyhow!(
                "SECURITY: Proxy objects are not allowed (prevents property access interception)"
            ));
        }

        Ok(())
    }
}

impl Default for JavaScriptSecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_javascript() {
        let validator = JavaScriptSecurityValidator::new();

        // Safe code examples
        assert!(validator.is_safe_javascript("const x = 1 + 2;").is_ok());
        assert!(
            validator
                .is_safe_javascript("function add(a, b) { return a + b; }")
                .is_ok()
        );
        assert!(
            validator
                .is_safe_javascript("const arr = [1, 2, 3]; arr.map(x => x * 2);")
                .is_ok()
        );
        assert!(
            validator
                .is_safe_javascript("console.log('Hello, world!');")
                .is_ok()
        );
    }

    #[test]
    fn test_eval_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // Direct eval
        assert!(validator.is_safe_javascript("eval('alert(1)')").is_err());

        // Indirect eval
        assert!(
            validator
                .is_safe_javascript("window.eval('alert(1)')")
                .is_err()
        );
        assert!(
            validator
                .is_safe_javascript("globalThis.eval('alert(1)')")
                .is_err()
        );

        // Bracket notation
        assert!(
            validator
                .is_safe_javascript("window['eval']('alert(1)')")
                .is_err()
        );
    }

    #[test]
    fn test_eval_in_comment_allowed() {
        let validator = JavaScriptSecurityValidator::new();

        // eval in comment should be allowed
        assert!(
            validator
                .is_safe_javascript("// This is eval in comment\nconst x = 1;")
                .is_ok()
        );
        assert!(
            validator
                .is_safe_javascript("/* eval */ const x = 1;")
                .is_ok()
        );
    }

    #[test]
    fn test_eval_in_string_allowed() {
        let validator = JavaScriptSecurityValidator::new();

        // eval in string should be allowed
        assert!(validator.is_safe_javascript("const x = 'eval';").is_ok());
        assert!(
            validator
                .is_safe_javascript("console.log(\"eval is dangerous\");")
                .is_ok()
        );
    }

    #[test]
    fn test_function_constructor_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // Function constructor
        assert!(
            validator
                .is_safe_javascript("Function('return 1')()")
                .is_err()
        );
        assert!(
            validator
                .is_safe_javascript("new Function('return 1')()")
                .is_err()
        );
        assert!(
            validator
                .is_safe_javascript("window.Function('return 1')()")
                .is_err()
        );
    }

    #[test]
    fn test_settimeout_with_string_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // setTimeout with string (code execution)
        assert!(
            validator
                .is_safe_javascript("setTimeout('alert(1)', 1000)")
                .is_err()
        );
        assert!(
            validator
                .is_safe_javascript("setInterval(\"alert(1)\", 1000)")
                .is_err()
        );

        // setTimeout with function is OK
        assert!(
            validator
                .is_safe_javascript("setTimeout(() => console.log('ok'), 1000)")
                .is_ok()
        );
        assert!(
            validator
                .is_safe_javascript("setTimeout(function() { console.log('ok'); }, 1000)")
                .is_ok()
        );
    }

    #[test]
    fn test_proto_pollution_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // __proto__ access
        assert!(validator.is_safe_javascript("obj.__proto__ = {}").is_err());
        assert!(
            validator
                .is_safe_javascript("obj['__proto__'] = {}")
                .is_err()
        );
        assert!(
            validator
                .is_safe_javascript("obj[\"__proto__\"] = {}")
                .is_err()
        );
    }

    #[test]
    fn test_constructor_constructor_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // constructor.constructor access
        assert!(
            validator
                .is_safe_javascript("obj.constructor.constructor('return 1')()")
                .is_err()
        );
    }

    #[test]
    fn test_with_statement_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // with statement
        assert!(
            validator
                .is_safe_javascript("with (obj) { x = 1; }")
                .is_err()
        );
    }

    #[test]
    fn test_import_allowed() {
        let validator = JavaScriptSecurityValidator::new();

        // import statements are now ALLOWED — needed for Astro/React/Vue hydration
        assert!(
            validator
                .is_safe_javascript("import { foo } from 'bar';")
                .is_ok()
        );
        assert!(validator.is_safe_javascript("import * from 'bar';").is_ok());
        assert!(validator.is_safe_javascript("import 'bar';").is_ok());

        // Dynamic import is now ALLOWED — used by every modern framework
        assert!(validator.is_safe_javascript("import('bar')").is_ok());
    }

    #[test]
    fn test_document_write_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // document.write
        assert!(
            validator
                .is_safe_javascript("document.write('<script>alert(1)</script>')")
                .is_err()
        );
        assert!(
            validator
                .is_safe_javascript("document.writeln('text')")
                .is_err()
        );
    }

    #[test]
    fn test_webassembly_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // WebAssembly instantiation
        assert!(
            validator
                .is_safe_javascript("WebAssembly.instantiate(buffer)")
                .is_err()
        );
        assert!(
            validator
                .is_safe_javascript("new WebAssembly.Module(buffer)")
                .is_err()
        );
    }

    #[test]
    fn test_node_apis_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // Node.js APIs
        assert!(validator.is_safe_javascript("require('fs')").is_err());
        assert!(validator.is_safe_javascript("process.exit()").is_err());
        assert!(validator.is_safe_javascript("fs.readFile('file')").is_err());
    }

    #[test]
    fn test_code_size_limit() {
        let validator = JavaScriptSecurityValidator::new();

        // Generate code larger than limit
        let large_code = "x = 1;".repeat(2_000_000); // > 10 MB
        assert!(validator.is_safe_javascript(&large_code).is_err());
    }

    // === NEW SECURITY FIXES TESTS ===

    #[test]
    fn test_global_bracket_access_allowed() {
        let validator = JavaScriptSecurityValidator::new();

        // Bracket access to globals is now ALLOWED — standard JS pattern
        // used by OneTrust, analytics, and many legitimate libraries.
        // The specific window["eval"] attack is still caught by check_eval_bracket.
        assert!(validator.is_safe_javascript("window[x]").is_ok());
        assert!(validator.is_safe_javascript("globalThis[key]").is_ok());
        assert!(validator.is_safe_javascript("self['property']").is_ok());

        // But window["eval"] is still blocked by check_eval_bracket
        assert!(
            validator
                .is_safe_javascript("window['eval']('alert(1)')")
                .is_err()
        );
    }

    #[test]
    fn test_escape_sequences_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // Block hex and unicode escapes in bracket notation (prevents eval/proto via encoding)
        assert!(
            validator
                .is_safe_javascript(r#"window['\x65\x76\x61\x6c']"#)
                .is_err()
        );
        assert!(
            validator
                .is_safe_javascript(r#"obj['\u005f\u005fproto\u005f\u005f']"#)
                .is_err()
        );

        // Escapes assigned to a variable and then used via window[x] — this pattern
        // is no longer caught since check_global_bracket_access is disabled.
        // The direct bracket-with-escape case above is still blocked.
        assert!(
            validator
                .is_safe_javascript(r#"const x = '\x65\x76\x61\x6c'; window[x]('code')"#)
                .is_ok()
        );
    }

    #[test]
    fn test_constructor_after_literal_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // CRITICAL FIX #3: Block constructor chain via literals
        assert!(
            validator
                .is_safe_javascript("(0).constructor.constructor")
                .is_err()
        );
        assert!(
            validator
                .is_safe_javascript("[].constructor.constructor")
                .is_err()
        );
        assert!(
            validator
                .is_safe_javascript("({}).constructor.constructor")
                .is_err()
        );
        assert!(
            validator
                .is_safe_javascript("(function(){}).constructor")
                .is_err()
        );
    }

    #[test]
    fn test_async_generator_constructor_blocked() {
        let validator = JavaScriptSecurityValidator::new();

        // HIGH PRIORITY: Block async/generator constructor access
        let async_attack = r#"
            AsyncFunction = (async function(){}).constructor;
            AsyncFunction('return alert(1)')();
        "#;
        assert!(validator.is_safe_javascript(async_attack).is_err());

        let generator_attack = r#"
            GeneratorFunction = (function*(){}).constructor;
            GeneratorFunction('yield alert(1)')();
        "#;
        assert!(validator.is_safe_javascript(generator_attack).is_err());
    }

    #[test]
    fn test_reflect_api_allowed() {
        let validator = JavaScriptSecurityValidator::new();

        // Reflect API is now ALLOWED — standard ES6 feature used by Vue, MobX, etc.
        assert!(
            validator
                .is_safe_javascript("Reflect.get(window, 'eval')")
                .is_ok()
        );
        assert!(
            validator
                .is_safe_javascript("Reflect.apply(eval, null, ['code'])")
                .is_ok()
        );
        assert!(
            validator
                .is_safe_javascript("Reflect.defineProperty(obj, 'x', {})")
                .is_ok()
        );
    }

    #[test]
    fn test_symbol_api_allowed() {
        let validator = JavaScriptSecurityValidator::new();

        // Symbol API is now ALLOWED — standard ES6 feature used by React (@@iterator), etc.
        assert!(validator.is_safe_javascript("Symbol.for('myKey')").is_ok());
        assert!(validator.is_safe_javascript("Symbol('key')").is_ok());
    }

    #[test]
    fn test_proxy_usage_allowed() {
        let validator = JavaScriptSecurityValidator::new();

        // Proxy is now ALLOWED — standard ES6 feature used by Vue reactivity, MobX, etc.
        assert!(
            validator
                .is_safe_javascript("new Proxy({}, handler)")
                .is_ok()
        );
        assert!(
            validator
                .is_safe_javascript("const p = new Proxy(target, { get: (t, k) => t[k] })")
                .is_ok()
        );
    }

    #[test]
    fn test_complex_safe_code() {
        let validator = JavaScriptSecurityValidator::new();

        let safe_code = r#"
            class Calculator {
                constructor() {
                    this.value = 0;
                }
                add(n) {
                    this.value += n;
                    return this;
                }
                subtract(n) {
                    this.value -= n;
                    return this;
                }
                getValue() {
                    return this.value;
                }
            }
            const calc = new Calculator();
            const result = calc.add(10).subtract(3).getValue();
            console.log(result);
        "#;

        assert!(validator.is_safe_javascript(safe_code).is_ok());
    }

    // === SecurityContext tests ===

    #[test]
    fn test_page_script_allows_eval() {
        let validator = JavaScriptSecurityValidator::new();

        // eval is blocked for AI-injected scripts
        assert!(
            validator
                .validate("eval('1+1')", SecurityContext::AiInjected)
                .is_err()
        );

        // eval is allowed for page scripts (Webpack, GTM, analytics use it)
        assert!(
            validator
                .validate("eval('1+1')", SecurityContext::PageScript)
                .is_ok()
        );
    }

    #[test]
    fn test_page_script_allows_function_constructor() {
        let validator = JavaScriptSecurityValidator::new();

        // Function() blocked for AI, allowed for page
        assert!(
            validator
                .validate("new Function('return 1')()", SecurityContext::AiInjected)
                .is_err()
        );
        assert!(
            validator
                .validate("new Function('return 1')()", SecurityContext::PageScript)
                .is_ok()
        );
    }

    #[test]
    fn test_page_script_allows_document_write() {
        let validator = JavaScriptSecurityValidator::new();

        // document.write blocked for AI, allowed for page
        assert!(
            validator
                .validate(
                    "document.write('<p>Hello</p>')",
                    SecurityContext::AiInjected
                )
                .is_err()
        );
        assert!(
            validator
                .validate(
                    "document.write('<p>Hello</p>')",
                    SecurityContext::PageScript
                )
                .is_ok()
        );
    }

    #[test]
    fn test_page_script_allows_webassembly() {
        let validator = JavaScriptSecurityValidator::new();

        // WebAssembly blocked for AI, allowed for page
        assert!(
            validator
                .validate(
                    "WebAssembly.instantiate(buffer)",
                    SecurityContext::AiInjected
                )
                .is_err()
        );
        assert!(
            validator
                .validate(
                    "WebAssembly.instantiate(buffer)",
                    SecurityContext::PageScript
                )
                .is_ok()
        );
    }

    #[test]
    fn test_page_script_still_blocks_proto_pollution() {
        let validator = JavaScriptSecurityValidator::new();

        // Prototype pollution blocked for BOTH contexts
        assert!(
            validator
                .validate("obj.__proto__ = {}", SecurityContext::PageScript)
                .is_err()
        );
        assert!(
            validator
                .validate("obj['__proto__'] = {}", SecurityContext::PageScript)
                .is_err()
        );
    }

    #[test]
    fn test_page_script_still_blocks_node_apis() {
        let validator = JavaScriptSecurityValidator::new();

        // Node.js APIs blocked for BOTH contexts
        assert!(
            validator
                .validate("require('fs')", SecurityContext::PageScript)
                .is_err()
        );
        assert!(
            validator
                .validate("process.exit()", SecurityContext::PageScript)
                .is_err()
        );
    }

    #[test]
    fn test_page_script_still_blocks_constructor_chains() {
        let validator = JavaScriptSecurityValidator::new();

        // constructor.constructor blocked for BOTH contexts
        assert!(
            validator
                .validate(
                    "obj.constructor.constructor('code')()",
                    SecurityContext::PageScript
                )
                .is_err()
        );
    }

    #[test]
    fn test_is_safe_page_javascript_convenience() {
        let validator = JavaScriptSecurityValidator::new();

        // Convenience method should use PageScript context
        assert!(validator.is_safe_page_javascript("eval('1+1')").is_ok());
        assert!(
            validator
                .is_safe_page_javascript("new Function('return 1')()")
                .is_ok()
        );
        assert!(
            validator
                .is_safe_page_javascript("document.write('<p>hi</p>')")
                .is_ok()
        );

        // But still block dangerous patterns
        assert!(
            validator
                .is_safe_page_javascript("obj.__proto__ = {}")
                .is_err()
        );
        assert!(
            validator
                .is_safe_page_javascript("require('child_process')")
                .is_err()
        );
    }
}
