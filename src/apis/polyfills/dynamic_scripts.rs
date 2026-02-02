//! Dynamic Script Execution Polyfill
//!
//! This polyfill intercepts appendChild/insertBefore calls to execute
//! dynamically created script elements, mimicking real browser behavior.
//!
//! In real browsers, when you appendChild a <script> element:
//! - If it has inline content (textContent/innerHTML), it's executed immediately
//! - If it has a src attribute, the script is fetched and executed
//!
//! This is critical for Cloudflare challenges which dynamically create scripts.

use thalora_browser_apis::boa_engine::{Context, JsResult, Source};

/// Setup dynamic script execution hooks
pub fn setup_dynamic_script_execution(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        (function() {
            'use strict';

            // Queue for scripts that need to be fetched (src attribute)
            // These will be processed by the browser's fetch mechanism
            if (typeof window !== 'undefined') {
                window.__pendingScripts = window.__pendingScripts || [];
            }

            // Helper to check if an element is a script
            function isScriptElement(node) {
                return node && node.tagName && node.tagName.toLowerCase() === 'script';
            }

            // Helper to execute inline script content
            function executeInlineScript(scriptElement) {
                try {
                    var content = scriptElement.textContent || scriptElement.innerHTML || '';
                    if (content && content.trim()) {
                        // Check script type - only execute JavaScript
                        var type = scriptElement.type || scriptElement.getAttribute('type') || 'text/javascript';

                        // Handle various JavaScript MIME types
                        var isJS = type === '' ||
                                   type === 'text/javascript' ||
                                   type === 'application/javascript' ||
                                   type === 'module' ||
                                   type.endsWith('-text/javascript'); // Cloudflare Rocket Loader

                        if (isJS) {
                            // Use indirect eval for global scope execution
                            var globalEval = eval;
                            globalEval(content);
                        }
                    }
                } catch (e) {
                    console.error('Dynamic script execution error:', e.message || e);
                }
            }

            // Helper to handle script with src attribute
            function handleExternalScript(scriptElement) {
                var src = scriptElement.src || scriptElement.getAttribute('src');
                if (src && typeof window !== 'undefined' && window.__pendingScripts) {
                    // Queue for external fetching by the browser
                    window.__pendingScripts.push({
                        src: src,
                        async: scriptElement.async || scriptElement.hasAttribute('async'),
                        defer: scriptElement.defer || scriptElement.hasAttribute('defer'),
                        type: scriptElement.type || 'text/javascript'
                    });
                }
            }

            // Process a script element when it's added to the DOM
            function processScriptElement(scriptElement) {
                if (!isScriptElement(scriptElement)) return;

                var src = scriptElement.src || scriptElement.getAttribute('src');
                if (src) {
                    // External script - queue for fetch
                    handleExternalScript(scriptElement);
                } else {
                    // Inline script - execute immediately
                    executeInlineScript(scriptElement);
                }
            }

            // Hook appendChild on Node.prototype
            if (typeof Node !== 'undefined' && Node.prototype && Node.prototype.appendChild) {
                var originalAppendChild = Node.prototype.appendChild;
                Node.prototype.appendChild = function(child) {
                    var result = originalAppendChild.call(this, child);

                    // Process if it's a script element
                    processScriptElement(child);

                    return result;
                };
            }

            // Hook appendChild on Element.prototype (some implementations use this)
            if (typeof Element !== 'undefined' && Element.prototype && Element.prototype.appendChild) {
                var originalElementAppendChild = Element.prototype.appendChild;
                Element.prototype.appendChild = function(child) {
                    var result = originalElementAppendChild.call(this, child);

                    // Process if it's a script element
                    processScriptElement(child);

                    return result;
                };
            }

            // Hook insertBefore on Node.prototype
            if (typeof Node !== 'undefined' && Node.prototype && Node.prototype.insertBefore) {
                var originalInsertBefore = Node.prototype.insertBefore;
                Node.prototype.insertBefore = function(newNode, referenceNode) {
                    var result = originalInsertBefore.call(this, newNode, referenceNode);

                    // Process if it's a script element
                    processScriptElement(newNode);

                    return result;
                };
            }

            // Hook insertBefore on Element.prototype
            if (typeof Element !== 'undefined' && Element.prototype && Element.prototype.insertBefore) {
                var originalElementInsertBefore = Element.prototype.insertBefore;
                Element.prototype.insertBefore = function(newNode, referenceNode) {
                    var result = originalElementInsertBefore.call(this, newNode, referenceNode);

                    // Process if it's a script element
                    processScriptElement(newNode);

                    return result;
                };
            }

            // Hook replaceChild on Node.prototype
            if (typeof Node !== 'undefined' && Node.prototype && Node.prototype.replaceChild) {
                var originalReplaceChild = Node.prototype.replaceChild;
                Node.prototype.replaceChild = function(newChild, oldChild) {
                    var result = originalReplaceChild.call(this, newChild, oldChild);

                    // Process if it's a script element
                    processScriptElement(newChild);

                    return result;
                };
            }

            // Also hook document.createElement to track script elements
            // and set up their execution when added
            if (typeof document !== 'undefined' && document.createElement) {
                var originalCreateElement = document.createElement;
                document.createElement = function(tagName) {
                    var element = originalCreateElement.call(document, tagName);

                    // Mark dynamically created scripts
                    if (tagName && tagName.toLowerCase() === 'script') {
                        element.__dynamicallyCreated = true;
                    }

                    return element;
                };
            }
        })();
    "#))?;

    Ok(())
}
