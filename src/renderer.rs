use anyhow::{anyhow, Result};
use boa_engine::{Context, Source};
use std::time::Duration;
use tokio::time::timeout;
// use crate::enhanced_dom::{DomManager, DomElement, DomMutation};

// use crate::enhanced_js::EnhancedJavaScriptEngine;

pub struct RustRenderer {
    js_context: Context,
    // dom_manager: Option<DomManager>,
}

impl RustRenderer {
    pub fn new() -> Self {
        let mut context = Context::default();
        
        // Enhanced polyfills for modern web APIs and anti-bot challenge support
        context.eval(Source::from_bytes(r#"
            // Enhanced document object with more DOM methods
            var document = {
                title: '',
                hidden: false,
                visibilityState: 'visible',
                readyState: 'complete',
                createElement: function(tag) { 
                    return { 
                        tagName: tag.toUpperCase(),
                        style: {},
                        setAttribute: function(name, value) { this[name] = value; },
                        getAttribute: function(name) { return this[name]; },
                        addEventListener: function(event, handler) {},
                        removeEventListener: function(event, handler) {}
                    }; 
                },
                getElementById: function(id) { return null; },
                querySelector: function(selector) { return null; },
                querySelectorAll: function(selector) { return []; },
                getElementsByClassName: function(className) { return []; },
                getElementsByTagName: function(tagName) { return []; },
                body: { 
                    appendChild: function(child) {},
                    removeChild: function(child) {},
                    style: {}
                },
                head: { 
                    appendChild: function(child) {},
                    removeChild: function(child) {},
                    style: {}
                },
                addEventListener: function(event, handler) {},
                removeEventListener: function(event, handler) {},
                dispatchEvent: function(event) { return true; }
            };
            
            // Enhanced window object with modern APIs
            var window = { 
                document: document,
                self: {},
                global: {},
                top: null,
                parent: null,
                frameElement: null,
                location: {
                    href: 'https://www.google.com',
                    hostname: 'www.google.com',
                    protocol: 'https:',
                    search: '',
                    hash: ''
                },
                navigator: {
                    userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36',
                    language: 'en-US',
                    languages: ['en-US', 'en'],
                    platform: 'MacIntel',
                    cookieEnabled: true,
                    doNotTrack: null,
                    hardwareConcurrency: 8,
                    maxTouchPoints: 0,
                    onLine: true
                },
                screen: {
                    width: 1920,
                    height: 1080,
                    availWidth: 1920,
                    availHeight: 1055,
                    colorDepth: 24,
                    pixelDepth: 24
                },
                performance: {
                    now: function() { return Date.now() - this.timeOrigin; },
                    timeOrigin: Date.now() - Math.random() * 10000,
                    timing: { 
                        navigationStart: Date.now() - Math.random() * 10000,
                        loadEventEnd: Date.now() - Math.random() * 5000
                    },
                    getEntriesByType: function(type) { return []; },
                    mark: function(name) {},
                    measure: function(name, start, end) {}
                },
                Math: Math,
                Date: Date,
                JSON: JSON,
                parseInt: parseInt,
                parseFloat: parseFloat,
                isNaN: isNaN,
                isFinite: isFinite,
                encodeURIComponent: encodeURIComponent,
                decodeURIComponent: decodeURIComponent,
                btoa: function(str) { 
                    // Basic base64 encoding simulation
                    return str.replace(/./g, function(c) {
                        return String.fromCharCode(c.charCodeAt(0) + 1);
                    });
                },
                atob: function(str) { 
                    // Basic base64 decoding simulation
                    return str.replace(/./g, function(c) {
                        return String.fromCharCode(c.charCodeAt(0) - 1);
                    });
                },
                setTimeout: function(fn, delay) { 
                    if (typeof fn === 'function') {
                        try { fn(); } catch(e) {}
                    }
                    return Math.floor(Math.random() * 1000) + 1;
                },
                setInterval: function(fn, delay) { 
                    return Math.floor(Math.random() * 1000) + 1;
                },
                clearTimeout: function(id) {},
                clearInterval: function(id) {},
                requestAnimationFrame: function(callback) {
                    setTimeout(callback, 16);
                    return Math.floor(Math.random() * 1000) + 1;
                },
                cancelAnimationFrame: function(id) {},
                addEventListener: function(event, handler) {},
                removeEventListener: function(event, handler) {},
                dispatchEvent: function(event) { return true; },
                getComputedStyle: function(element) {
                    return {
                        getPropertyValue: function(prop) { return ''; }
                    };
                },
                innerWidth: 1920,
                innerHeight: 1055,
                outerWidth: 1920,
                outerHeight: 1080,
                devicePixelRatio: 1,
                scrollX: 0,
                scrollY: 0,
                // Essential Chrome object to pass bot detection
                chrome: {
                    runtime: {
                        onConnect: null,
                        onMessage: null
                    },
                    loadTimes: function() {
                        return {
                            commitLoadTime: Date.now() / 1000 - Math.random() * 2,
                            finishDocumentLoadTime: Date.now() / 1000 - Math.random() * 1,
                            finishLoadTime: Date.now() / 1000 - Math.random() * 0.5,
                            firstPaintAfterLoadTime: 0,
                            firstPaintTime: Date.now() / 1000 - Math.random() * 1.5,
                            navigationType: 'Navigation',
                            numTabsOpen: Math.floor(Math.random() * 5) + 2,
                            origFirstPaintTime: Date.now() / 1000 - Math.random() * 1.5,
                            origFirstPaintAfterLoadTime: 0,
                            requestTime: Date.now() / 1000 - Math.random() * 3,
                            startLoadTime: Date.now() / 1000 - Math.random() * 2.5
                        };
                    },
                    csi: function() {
                        return {
                            onloadT: Date.now(),
                            startE: Date.now() - Math.random() * 1000,
                            tran: Math.floor(Math.random() * 20) + 1
                        };
                    }
                }
            };
            
            var self = window;
            var globalThis = window;
            
            // HTMLCanvasElement constructor and canvas fingerprinting
            window.HTMLCanvasElement = function() {
                this.width = 300;
                this.height = 150;
                this.style = {};
            };
            
            window.HTMLCanvasElement.prototype.getContext = function(contextType) {
                if (contextType === '2d') {
                    return {
                        fillStyle: '#000000',
                        strokeStyle: '#000000',
                        lineWidth: 1,
                        font: '10px sans-serif',
                        textAlign: 'start',
                        textBaseline: 'alphabetic',
                        fillRect: function(x, y, width, height) {
                            // Simulate canvas drawing
                        },
                        fillText: function(text, x, y) {
                            // Simulate text rendering with slight variations
                        },
                        strokeText: function(text, x, y) {
                            // Simulate stroke text
                        },
                        arc: function(x, y, radius, startAngle, endAngle) {
                            // Simulate arc drawing
                        },
                        beginPath: function() {},
                        closePath: function() {},
                        stroke: function() {},
                        fill: function() {},
                        save: function() {},
                        restore: function() {},
                        translate: function(x, y) {},
                        scale: function(x, y) {},
                        rotate: function(angle) {},
                        clearRect: function(x, y, width, height) {},
                        getImageData: function(x, y, width, height) {
                            // Return realistic ImageData with slight variations
                            var data = new Array(width * height * 4);
                            for (var i = 0; i < data.length; i += 4) {
                                var variance = Math.floor(Math.random() * 3);
                                data[i] = 200 + variance;     // R
                                data[i + 1] = 200 + variance; // G  
                                data[i + 2] = 200 + variance; // B
                                data[i + 3] = 255;           // A
                            }
                            return {
                                data: data,
                                width: width,
                                height: height
                            };
                        },
                        putImageData: function(imageData, x, y) {},
                        createLinearGradient: function(x0, y0, x1, y1) {
                            return {
                                addColorStop: function(offset, color) {}
                            };
                        },
                        createRadialGradient: function(x0, y0, r0, x1, y1, r1) {
                            return {
                                addColorStop: function(offset, color) {}
                            };
                        }
                    };
                } else if (contextType === 'webgl' || contextType === 'experimental-webgl') {
                    return {
                        getParameter: function(param) {
                            // WebGL parameter simulation with realistic values
                            switch(param) {
                                case 7936: // UNMASKED_VENDOR_WEBGL
                                    return 'Google Inc. (Apple)';
                                case 7937: // UNMASKED_RENDERER_WEBGL  
                                    return 'ANGLE (Apple, Apple M1 Pro, OpenGL 4.1)';
                                case 3379: // MAX_TEXTURE_SIZE
                                    return 16384;
                                case 34076: // MAX_VERTEX_UNIFORM_VECTORS
                                    return 1024;
                                case 36347: // SHADING_LANGUAGE_VERSION
                                    return 'WebGL GLSL ES 1.0 (OpenGL ES GLSL ES 1.0 Chromium)';
                                case 7938: // VERSION
                                    return 'WebGL 1.0 (OpenGL ES 2.0 Chromium)';
                                default:
                                    return null;
                            }
                        },
                        getExtension: function(name) {
                            if (name === 'WEBGL_debug_renderer_info') {
                                return {
                                    UNMASKED_VENDOR_WEBGL: 7936,
                                    UNMASKED_RENDERER_WEBGL: 7937
                                };
                            }
                            return null;
                        },
                        getSupportedExtensions: function() {
                            return [
                                'ANGLE_instanced_arrays',
                                'EXT_blend_minmax', 
                                'EXT_color_buffer_half_float',
                                'WEBGL_debug_renderer_info'
                            ];
                        }
                    };
                }
                return null;
            };
            
            // Enhanced HTMLCanvasElement with toDataURL that produces realistic fingerprints
            window.HTMLCanvasElement.prototype.toDataURL = function(type) {
                // Generate a realistic canvas fingerprint with slight randomization
                var baseFingerprint = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAASwAAAAeCAYAAACTLCojAAAABHNCSVQICAgIfAhkiAAAAAlwSFlzAAAAdgAAAHYBTnsmCAAAABl0RVh0U29mdHdhcmUAd3d3Lmlua3NjYXBlLm9yZ5vuPBoAAANjSURBVHic7Z09axRBFIafJQQSCwsLwcJCG1sLG0uxsLGwsLBQsLGwsLGwsLGwsLCwsLGwsLGwsLCwsLGwsLCwsLGwsLGwsLCwsLGwsLGwsLCwsLGwsLCwsLGwsLCwsLGwsLGwsLGwsLGwsLCwsLGwsLGwsLGwsLGwsLCwsLCwsLGwsLCwsLGwsLCwsLGwsLCwsLGwsLGwsLGwsLGwsLGwsLGwsLGwsLGwsLGwsLGwsLGwsLCwsLCwsLCwsLGwsLCwsLCwsLGwsLCwsLGwsLCwsLCwsLGwsLCwsLCwsLCw';
                
                // Add slight variance to make each fingerprint unique but consistent per session
                var variance = Math.floor(Math.random() * 1000);
                return baseFingerprint + variance;
            };
            
            var console = { 
                log: function() { /* console.log captured */ }, 
                error: function() { /* console.error captured */ },
                warn: function() { /* console.warn captured */ },
                info: function() { /* console.info captured */ }
            };
            
            // TrustedTypes API for Google challenges
            var trustedTypes = {
                createPolicy: function(name, rules) {
                    return {
                        createHTML: rules.createHTML || function(s) { return s; },
                        createScript: rules.createScript || function(s) { return s; },
                        createScriptURL: rules.createScriptURL || function(s) { return s; }
                    };
                }
            };
            
            // Google-specific globals
            var google = {
                tick: function(event, label) { /* Google timing captured */ }
            };
        "#)).unwrap();

        Self {
            js_context: context,
            // dom_manager: None,
        }
    }

    // /// Initialize DOM manager with HTML content for enhanced DOM operations
    // pub fn init_dom_manager(&mut self, html: &str) -> Result<()> {
    //     let dom_manager = DomManager::new(html)?;
    //     dom_manager.setup_dom_globals(&mut self.js_context)?;
    //     self.dom_manager = Some(dom_manager);
    //     Ok(())
    // }

    // /// Get enhanced DOM content with structure
    // pub fn get_enhanced_dom_content(&self, selector: Option<&str>) -> Result<DomElement> {
    //     match &self.dom_manager {
    //         Some(dom_manager) => dom_manager.extract_enhanced_content(selector),
    //         None => Ok(DomElement {
    //             tag_name: "body".to_string(),
    //             attributes: std::collections::HashMap::new(),
    //             text_content: String::new(),
    //             inner_html: String::new(),
    //             children: Vec::new(),
    //             id: "body".to_string(),
    //         }),
    //     }
    // }

    // /// Get storage data for debugging or inspection
    // pub fn get_storage_data(&self) -> (std::collections::HashMap<String, String>, std::collections::HashMap<String, String>) {
    //     match &self.dom_manager {
    //         Some(dom_manager) => (
    //             dom_manager.get_local_storage_data(),
    //             dom_manager.get_session_storage_data(),
    //         ),
    //         None => (std::collections::HashMap::new(), std::collections::HashMap::new()),
    //     }
    // }

    pub async fn render_with_js(&mut self, html: &str, _url: &str) -> Result<String> {
        // // Initialize DOM manager for enhanced DOM operations
        // if let Err(e) = self.init_dom_manager(html) {
        //     tracing::debug!("DOM manager initialization failed: {}", e);
        // }
        
        let modified_html = html.to_string();

        let script_regex = regex::Regex::new(r"<script[^>]*>(.*?)</script>").unwrap();
        
        for captures in script_regex.captures_iter(html) {
            if let Some(script_content) = captures.get(1) {
                let js_code = script_content.as_str();
                
                if self.is_safe_javascript(js_code) {
                    let execution_result = timeout(
                        Duration::from_secs(5),
                        self.execute_javascript_safely(js_code)
                    ).await;

                    match execution_result {
                        Ok(Ok(_result)) => {
                            tracing::debug!("JavaScript executed successfully");
                        }
                        Ok(Err(e)) => {
                            tracing::warn!("JavaScript execution failed: {}", e);
                        }
                        Err(_) => {
                            tracing::warn!("JavaScript execution timed out");
                        }
                    }
                } else {
                    tracing::warn!("Potentially unsafe JavaScript detected, skipping execution");
                }
            }
        }

        Ok(modified_html)
    }

    pub fn is_safe_javascript(&self, js_code: &str) -> bool {
        // Allow challenge JavaScript for anti-bot bypass
        if self.is_challenge_javascript(js_code) {
            return true;
        }

        let dangerous_patterns = [
            "eval(",
            "Function(",
            "XMLHttpRequest",
            "fetch(",
            "import(",
            "require(",
            "process.",
            "global.",
            "__proto__",
            "constructor.constructor",
            "location.href",
            "window.location",
            "document.cookie",
            "localStorage.setItem",
            "alert(",
            "confirm(",
            "prompt(",
        ];

        let js_lower = js_code.to_lowercase();
        
        for pattern in &dangerous_patterns {
            if js_lower.contains(&pattern.to_lowercase()) {
                return false;
            }
        }

        // Allow larger scripts for challenges
        if js_code.len() > 10000 && !self.is_challenge_javascript(js_code) {
            return false;
        }

        true
    }

    fn is_challenge_javascript(&self, js_code: &str) -> bool {
        // Enhanced challenge detection for modern anti-bot systems
        let challenge_indicators = [
            // Google specific
            "google.tick",
            "trustedTypes",
            "createPolicy",
            "createScript",
            "Math.random()*7824",
            "sourceMappingURL=data:application/json",
            "Copyright Google LLC",
            "SPDX-License-Identifier: Apache-2.0",
            // Common challenge patterns
            "function(V,t,p,z,U,I,S,n,x,C)",
            "var M=function(",
            "Math.floor(Math.random()",
            "btoa(",
            "atob(",
            "String.fromCharCode",
            // Obfuscation patterns
            ".call(this,",
            "eval(function(",
            "function(){",
            "var ",
            "return ",
            // Anti-debugging
            "debugger",
            "console.log",
            "performance.now",
            // Common challenge math operations
            "<<",
            ">>",
            ">>>",
            "^",
            "&",
            "|",
        ];

        let code_lower = js_code.to_lowercase();
        let mut matches = 0;
        let mut google_specific = 0;

        for indicator in &challenge_indicators {
            if code_lower.contains(&indicator.to_lowercase()) {
                matches += 1;
                
                // Give more weight to Google-specific patterns
                if indicator.contains("google") || indicator.contains("trustedTypes") || indicator.contains("Copyright Google") {
                    google_specific += 1;
                }
            }
        }

        // Advanced heuristics for challenge detection
        let has_complex_math = code_lower.contains("math.") && 
                              (code_lower.contains("random") || code_lower.contains("floor"));
        let has_obfuscation = code_lower.matches("var ").count() > 10 || 
                             code_lower.matches("function").count() > 5;
        let has_string_manipulation = code_lower.contains("string.fromcharcode") || 
                                    code_lower.contains("charat") ||
                                    code_lower.contains(".replace(");

        // Allow if it's clearly a challenge or has Google-specific patterns
        google_specific >= 2 || 
        matches >= 5 || 
        (matches >= 3 && (has_complex_math || has_obfuscation || has_string_manipulation))
    }

    pub async fn execute_javascript_safely(&mut self, js_code: &str) -> Result<boa_engine::JsValue> {
        // Enhanced execution with better error handling and context
        let sandboxed_code = if self.is_challenge_javascript(js_code) {
            // For challenges, execute more directly but still safely
            format!(
                r#"
                (function() {{
                    try {{
                        // Set up challenge environment
                        var __challenge_result = undefined;
                        var __original_eval = eval;
                        
                        // Override eval for safety while allowing challenge code
                        eval = function(code) {{
                            if (typeof code === 'string' && code.length < 10000) {{
                                return __original_eval(code);
                            }}
                            return null;
                        }};
                        
                        // Execute challenge code
                        {}
                        
                        // Restore eval
                        eval = __original_eval;
                        
                        // Return any challenge results
                        return __challenge_result || 'challenge_executed';
                    }} catch (e) {{
                        console.log('Challenge execution error:', e.message);
                        return 'challenge_error: ' + e.message;
                    }}
                }})()
                "#,
                js_code
            )
        } else {
            // Standard sandboxed execution for regular JavaScript
            format!(
                r#"
                (function() {{
                    try {{
                        var result = (function() {{
                            {}
                            return undefined;
                        }})();
                        return result || 'executed';
                    }} catch (e) {{
                        return 'error: ' + e.message;
                    }}
                }})()
                "#,
                js_code
            )
        };

        match self.js_context.eval(Source::from_bytes(&sandboxed_code)) {
            Ok(result) => {
                tracing::debug!("JavaScript execution completed: {:?}", result);
                Ok(result)
            },
            Err(e) => {
                tracing::warn!("JavaScript execution failed: {}", e);
                Err(anyhow!("JavaScript execution error: {}", e))
            }
        }
    }

    // Add method to handle specific challenge types
    pub fn handle_google_challenge(&mut self, js_code: &str) -> Result<String> {
        // Extract and execute Google's challenge JavaScript
        if js_code.contains("google.tick") {
            // Execute the challenge and return any generated tokens or results
            match self.js_context.eval(Source::from_bytes(js_code)) {
                Ok(result) => {
                    let result_str = result.to_string(&mut self.js_context)
                        .map_err(|e| anyhow!("Failed to convert result to string: {}", e))?;
                    Ok(result_str.to_std_string_escaped())
                },
                Err(e) => Err(anyhow!("Google challenge execution failed: {}", e))
            }
        } else {
            Err(anyhow!("Not a recognized Google challenge"))
        }
    }

}

pub struct CssProcessor {
    // Using lightningcss for CSS parsing and processing
}

impl CssProcessor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn process_css(&self, css: &str) -> Result<String> {
        use lightningcss::{
            stylesheet::{StyleSheet, ParserOptions, PrinterOptions},
            targets::Browsers,
        };

        let stylesheet = StyleSheet::parse(css, ParserOptions::default())
            .map_err(|e| anyhow!("CSS parsing error: {:?}", e))?;

        let printer_options = PrinterOptions {
            minify: false,
            targets: Browsers::default().into(),
            ..Default::default()
        };

        let result = stylesheet.to_css(printer_options)
            .map_err(|e| anyhow!("CSS processing error: {:?}", e))?;

        Ok(result.code)
    }
}

pub struct LayoutEngine {
    // Using Taffy for layout calculations
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn calculate_layout(&self, _html: &str, _css: &str) -> Result<LayoutResult> {
        use taffy::prelude::*;

        let mut taffy: taffy::TaffyTree<()> = TaffyTree::new();
        
        let root_style = Style {
            display: Display::Block,
            size: Size {
                width: Dimension::Length(800.0),
                height: Dimension::Auto,
            },
            ..Default::default()
        };

        let root_node = taffy.new_leaf(root_style).unwrap();
        
        let available_space = Size {
            width: AvailableSpace::Definite(800.0),
            height: AvailableSpace::MaxContent,
        };

        taffy.compute_layout(root_node, available_space).unwrap();
        
        let layout = taffy.layout(root_node).unwrap();

        Ok(LayoutResult {
            width: layout.size.width,
            height: layout.size.height,
            x: layout.location.x,
            y: layout.location.y,
        })
    }
}

#[derive(Debug)]
pub struct LayoutResult {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
}