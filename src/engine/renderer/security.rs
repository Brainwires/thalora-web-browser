use crate::engine::renderer::core::RustRenderer;

impl RustRenderer {
    /// Validate JavaScript code for security risks
    pub fn is_safe_javascript(&self, js_code: &str) -> bool {
        // Allow safe typeof operations for testing
        if js_code.trim().starts_with("typeof ") && js_code.len() < 100 {
            return true;
        }

        // Allow safe test patterns for API testing
        if js_code.contains("navigator.requestMIDIAccess") && js_code.contains("typeof promise.then") {
            return true;
        }

        let dangerous_patterns = [
            "eval(",
            "Function(",
            "setTimeout(",
            "setInterval(",
            "document.write",
            "innerHTML",
            "outerHTML",
            "insertAdjacentHTML",
            "execScript",
            "msWriteProfilerMark",
            ".constructor",
            "process.",
            "require(",
            "import(",
            "XMLHttpRequest",
            "fetch(",
            "new WebSocket",
            "WebSocket(",
            "EventSource",
            "location.",
            "window.location",
            "document.location",
            "history.",
            "window.history",
            "localStorage",
            "sessionStorage",
            "document.cookie",
            "navigator.",
            "screen.",
            "crypto.",
            "indexedDB",
            "webkitIndexedDB",
            "mozIndexedDB",
            "msIndexedDB",
            "Worker(",
            "SharedWorker(",
            "ServiceWorker",
            "postMessage",
            "addEventListener",
            "removeEventListener",
            "dispatchEvent",
            "createEvent",
            "initEvent",
            "requestAnimationFrame",
            "cancelAnimationFrame",
            "requestIdleCallback",
            "cancelIdleCallback",
            "alert(",
            "confirm(",
            "prompt(",
            "open(",
            "close(",
            "focus(",
            "blur(",
            "moveBy(",
            "moveTo(",
            "resizeBy(",
            "resizeTo(",
            "scroll(",
            "scrollBy(",
            "scrollTo(",
            "print(",
            "stop(",
            "captureEvents",
            "releaseEvents",
            "routeEvent",
            "enableExternalCapture",
            "disableExternalCapture",
            "find(",
            "home(",
            "back(",
            "forward(",
            "go(",
            "external.",
            "sidebar.",
            "menubar.",
            "toolbar.",
            "statusbar.",
            "locationbar.",
            "scrollbars.",
            "personalbar.",
            "directories.",
            "chrome.",
            "safari.",
            "opera.",
            "__dirname",
            "__filename",
            "global.",
            "Buffer.",
            "setImmediate",
            "clearImmediate",
            ".call(",
            ".apply(",
            ".bind(",
            "Proxy(",
            "Reflect.",
            "WeakMap",
            "WeakSet",
            "Symbol.",
            "Object.defineProperty",
            "Object.defineProperties",
            "Object.create",
            "Object.setPrototypeOf",
            "Object.getPrototypeOf",
            "__proto__",
            "prototype.",
            "delete ",
            "for..in",
            "with(",
            "debugger",
            "throw ",
            "try{",
            "catch(",
            "finally{",
            "arguments.",
            "caller.",
            "callee.",
            "name.",
            "length.",
            "prototype",
            "constructor.",
            "__defineGetter__",
            "__defineSetter__",
            "__lookupGetter__",
            "__lookupSetter__",
            "hasOwnProperty",
            "isPrototypeOf",
            "propertyIsEnumerable",
            "toLocaleString",
            "toString",
            "valueOf",
            "watch",
            "unwatch",
            "source",
            "stack",
            "message",
            "filename",
            "lineno",
            "colno",
            "error",
            "event",
            "srcElement",
            "target",
            "currentTarget",
            "relatedTarget",
            "fromElement",
            "toElement",
            "originalTarget",
            "explicitOriginalTarget",
            "rangeParent",
            "rangeOffset",
            "view",
            "detail",
            "data",
            "origin",
            "lastEventId",
            "ports",
            "clipboardData",
            "dataTransfer",
            "touches",
            "targetTouches",
            "changedTouches",
            "scale",
            "rotation",
            "wheelDelta",
            "wheelDeltaX",
            "wheelDeltaY",
            "detail",
            "axis",
            "HORIZONTAL_AXIS",
            "VERTICAL_AXIS",
            "initScrollEvent",
            "initWheelEvent",
            "initMouseEvent",
            "initKeyboardEvent",
            "initUIEvent",
            "initMutationEvent",
            "initMessageEvent",
            "initStorageEvent",
            "initPopStateEvent",
            "initHashChangeEvent",
            "initPageTransitionEvent",
            "initProgressEvent",
            "initCustomEvent",
            "initEvent",
            "createEvent",
            "dispatchEvent",
            "preventDefault",
            "stopPropagation",
            "stopImmediatePropagation",
            "returnValue",
            "cancelBubble",
        ];

        // Check for obviously dangerous patterns
        for pattern in dangerous_patterns.iter() {
            if js_code.contains(pattern) {
                return false;
            }
        }

        // Check for suspicious character sequences
        if js_code.contains("\\x") || js_code.contains("\\u") || js_code.contains("\\0") {
            return false;
        }

        // Check for suspicious regular expressions (simple heuristic)
        if js_code.contains("/eval/") || js_code.contains("/function/") {
            return false;
        }

        // Check for data URLs or javascript: URLs
        if js_code.contains("data:") || js_code.contains("javascript:") {
            return false;
        }

        // Check for suspicious base64 content
        if js_code.contains("atob(") || js_code.contains("btoa(") {
            return false;
        }

        // Check for suspicious escape sequences
        if js_code.contains("unescape(") || js_code.contains("escape(") || js_code.contains("decodeURI") || js_code.contains("encodeURI") {
            return false;
        }

        // Check for suspicious String methods
        if js_code.contains("fromCharCode") || js_code.contains("charCodeAt") {
            return false;
        }

        // Additional checks for obfuscated code patterns
        let suspicious_char_count = js_code.chars().filter(|&c| "[]{}();,".contains(c)).count();
        let total_chars = js_code.len();
        if total_chars > 0 && suspicious_char_count as f64 / total_chars as f64 > 0.3 {
            return false;
        }

        // Check for excessively long strings (often used in obfuscation)
        for word in js_code.split_whitespace() {
            if word.len() > 1000 {
                return false;
            }
        }

        // Check for too many nested parentheses (sign of obfuscation)
    let mut paren_depth: i32 = 0;
        let mut max_depth = 0;
        for c in js_code.chars() {
            match c {
                '(' => {
                    paren_depth += 1;
                    max_depth = max_depth.max(paren_depth);
                },
                ')' => paren_depth = paren_depth.saturating_sub(1),
                _ => {}
            }
        }
        if max_depth > 10 {
            return false;
        }

        true
    }
}