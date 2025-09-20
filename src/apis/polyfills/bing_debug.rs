//! Temporary debugging polyfill to investigate what APIs Bing search is missing
//!
//! This is a TEMPORARY HACK for debugging purposes only.
//! Once we identify the missing APIs, we'll implement them properly in Boa.

use boa_engine::{Context, JsResult, Source};

pub fn setup_bing_debug_polyfill(context: &mut Context) -> JsResult<()> {
    let polyfill_code = r#"
        // TEMPORARY DEBUGGING POLYFILL: Track missing APIs that Bing is trying to use
        window._BING_DEBUG = {
            missingAPIs: [],
            errors: [],
            ajaxCalls: [],
            timers: [],
            log: function(msg) {
                this.errors.push(new Date().toISOString() + ': ' + msg);
                if (typeof console !== 'undefined' && console.log) {
                    console.log('[BING_DEBUG]', msg);
                }
            },

            // Report all findings
            report: function() {
                return {
                    missingAPIs: this.missingAPIs,
                    errors: this.errors,
                    ajaxCalls: this.ajaxCalls,
                    timers: this.timers
                };
            }
        };

        // XMLHttpRequest polyfill to catch AJAX usage
        if (typeof XMLHttpRequest === 'undefined') {
            window._BING_DEBUG.log('XMLHttpRequest missing - implementing debug version');

            window.XMLHttpRequest = function() {
                window._BING_DEBUG.log('XMLHttpRequest constructor called');

                this.readyState = 0;
                this.status = 0;
                this.statusText = '';
                this.responseText = '';
                this.responseXML = null;
                this.onreadystatechange = null;
                this.onload = null;
                this.onerror = null;
                this.onabort = null;
                this.ontimeout = null;
                this.timeout = 0;
                this.withCredentials = false;
                this.upload = {};

                var self = this;

                this.open = function(method, url, async, user, password) {
                    window._BING_DEBUG.log('XHR.open called: ' + method + ' ' + url + ' (async: ' + async + ')');
                    window._BING_DEBUG.ajaxCalls.push({
                        method: method,
                        url: url,
                        async: async,
                        timestamp: new Date().toISOString()
                    });
                    this.readyState = 1;
                    if (this.onreadystatechange) {
                        this.onreadystatechange();
                    }
                };

                this.send = function(data) {
                    window._BING_DEBUG.log('XHR.send called with data: ' + (data ? 'has data' : 'no data'));

                    // Simulate async response for search results
                    setTimeout(function() {
                        self.readyState = 4;
                        self.status = 200;
                        self.statusText = 'OK';

                        // Mock Bing search results response
                        self.responseText = JSON.stringify({
                            results: [
                                {
                                    title: "Rust Programming Language - Official Site",
                                    url: "https://www.rust-lang.org/",
                                    snippet: "Rust is a programming language focused on safety, speed, and concurrency."
                                },
                                {
                                    title: "Rust (programming language) - Wikipedia",
                                    url: "https://en.wikipedia.org/wiki/Rust_(programming_language)",
                                    snippet: "Rust is a multi-paradigm programming language designed for performance and safety."
                                }
                            ]
                        });

                        if (self.onreadystatechange) {
                            self.onreadystatechange();
                        }
                        if (self.onload) {
                            self.onload();
                        }
                    }, 500); // Simulate network delay
                };

                this.setRequestHeader = function(name, value) {
                    window._BING_DEBUG.log('XHR.setRequestHeader: ' + name + '=' + value);
                };

                this.getAllResponseHeaders = function() {
                    return 'Content-Type: application/json\r\n';
                };

                this.getResponseHeader = function(name) {
                    if (name.toLowerCase() === 'content-type') {
                        return 'application/json';
                    }
                    return null;
                };

                this.abort = function() {
                    window._BING_DEBUG.log('XHR.abort called');
                    this.readyState = 0;
                    if (this.onabort) {
                        this.onabort();
                    }
                };
            };

            window._BING_DEBUG.missingAPIs.push('XMLHttpRequest');
        }

        // MutationObserver polyfill
        if (typeof MutationObserver === 'undefined') {
            window._BING_DEBUG.log('MutationObserver missing - implementing debug version');

            window.MutationObserver = function(callback) {
                window._BING_DEBUG.log('MutationObserver constructor called');
                this.callback = callback;
                this.observing = false;

                this.observe = function(target, options) {
                    window._BING_DEBUG.log('MutationObserver.observe called on: ' + (target.tagName || 'unknown'));
                    this.observing = true;

                    // Simulate some mutations for search results
                    setTimeout(() => {
                        if (this.observing && this.callback) {
                            this.callback([{
                                type: 'childList',
                                target: target,
                                addedNodes: [],
                                removedNodes: []
                            }]);
                        }
                    }, 1000);
                };

                this.disconnect = function() {
                    window._BING_DEBUG.log('MutationObserver.disconnect called');
                    this.observing = false;
                };

                this.takeRecords = function() {
                    return [];
                };
            };

            window._BING_DEBUG.missingAPIs.push('MutationObserver');
        }

        // IntersectionObserver polyfill
        if (typeof IntersectionObserver === 'undefined') {
            window._BING_DEBUG.log('IntersectionObserver missing');

            window.IntersectionObserver = function(callback, options) {
                window._BING_DEBUG.log('IntersectionObserver constructor called');
                this.callback = callback;

                this.observe = function(target) {
                    window._BING_DEBUG.log('IntersectionObserver.observe called');
                };

                this.unobserve = function(target) {
                    window._BING_DEBUG.log('IntersectionObserver.unobserve called');
                };

                this.disconnect = function() {
                    window._BING_DEBUG.log('IntersectionObserver.disconnect called');
                };
            };

            window._BING_DEBUG.missingAPIs.push('IntersectionObserver');
        }

        // ResizeObserver polyfill
        if (typeof ResizeObserver === 'undefined') {
            window._BING_DEBUG.log('ResizeObserver missing');

            window.ResizeObserver = function(callback) {
                this.observe = function(target) {
                    window._BING_DEBUG.log('ResizeObserver.observe called');
                };
                this.unobserve = function(target) {};
                this.disconnect = function() {};
            };

            window._BING_DEBUG.missingAPIs.push('ResizeObserver');
        }

        // AbortController and AbortSignal polyfills
        if (typeof AbortController === 'undefined') {
            window._BING_DEBUG.log('AbortController missing');

            window.AbortController = function() {
                this.signal = {
                    aborted: false,
                    addEventListener: function() {},
                    removeEventListener: function() {}
                };

                this.abort = function() {
                    this.signal.aborted = true;
                };
            };

            window.AbortSignal = function() {
                this.aborted = false;
            };

            window._BING_DEBUG.missingAPIs.push('AbortController');
            window._BING_DEBUG.missingAPIs.push('AbortSignal');
        }

        // requestAnimationFrame polyfill
        if (typeof requestAnimationFrame === 'undefined') {
            window._BING_DEBUG.log('requestAnimationFrame missing');

            window.requestAnimationFrame = function(callback) {
                window._BING_DEBUG.log('requestAnimationFrame called');
                var id = setTimeout(callback, 16); // ~60fps
                window._BING_DEBUG.timers.push(id);
                return id;
            };

            window.cancelAnimationFrame = function(id) {
                clearTimeout(id);
            };

            window._BING_DEBUG.missingAPIs.push('requestAnimationFrame');
        }

        // Track property access attempts
        var originalDescriptor = Object.getOwnPropertyDescriptor;
        if (originalDescriptor) {
            Object.getOwnPropertyDescriptor = function(obj, prop) {
                var result = originalDescriptor.call(this, obj, prop);
                if (!result && obj === window && typeof prop === 'string') {
                    window._BING_DEBUG.log('Attempted access to missing window property: ' + prop);
                    if (window._BING_DEBUG.missingAPIs.indexOf(prop) === -1) {
                        window._BING_DEBUG.missingAPIs.push(prop);
                    }
                }
                return result;
            };
        }

        // Override error handling to capture more info
        var originalOnError = window.onerror;
        window.onerror = function(message, source, lineno, colno, error) {
            window._BING_DEBUG.log('JavaScript Error: ' + message + ' at ' + source + ':' + lineno);
            if (originalOnError) {
                return originalOnError(message, source, lineno, colno, error);
            }
            return false;
        };

        // Track unhandled promise rejections
        if (typeof window.addEventListener === 'function') {
            window.addEventListener('unhandledrejection', function(event) {
                window._BING_DEBUG.log('Unhandled Promise Rejection: ' + (event.reason || 'Unknown'));
            });
        }

        window._BING_DEBUG.log('Bing debug polyfill initialized');
    "#;

    context.eval(Source::from_bytes(polyfill_code))?;
    Ok(())
}