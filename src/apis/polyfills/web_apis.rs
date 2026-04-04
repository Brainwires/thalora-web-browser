use thalora_browser_apis::boa_engine::{Context, JsResult, Source};

// Import the modular polyfill components
use super::{
    chrome_features::setup_chrome_features, performance::setup_performance_apis,
    security::setup_security_apis,
};

/// Setup Web APIs polyfills
///
/// ⚠️  CRITICAL WARNING: MOST OF THESE ARE MOCK/STUB IMPLEMENTATIONS! ⚠️
///
/// These polyfills provide API shape compatibility but NOT real functionality.
/// They return hardcoded fake values and are primarily for compatibility testing.
/// For production use, these need to be replaced with real implementations.
///
/// KNOWN MOCKS:
/// - Performance API: Returns fake timing data, no real measurements
/// - WebSocketStream: Logs but doesn't create real connections
/// - WebAuthn: Returns fake credentials, no real authentication
/// - IndexedDB: Fake database operations
/// - WebRTC: Mock media streams and connections
/// - Web Workers: Log messages but don't execute scripts
/// - And many more... (see MOCKED_APIS.md for complete list)
///
/// NATIVE IMPLEMENTATIONS (no longer polyfilled):
/// - fetch, Request, Response, Headers: Native HTTP client in Boa
/// - WebSocket: Native WebSocket with real networking in Boa
/// - ReadableStream: Native WHATWG Streams implementation in Boa
/// - DOM, Document, Element, History: Native DOM implementation in Boa
pub fn setup_web_apis(context: &mut Context) -> JsResult<()> {
    // Setup modular polyfill components
    setup_performance_apis(context)?;
    setup_security_apis(context)?;
    // DOM and CSS APIs are now natively implemented in Boa engine
    // Worker APIs are now natively implemented in Boa engine
    setup_chrome_features(context)?;

    // Setup remaining APIs that don't fit in the above modules
    setup_misc_apis(context)?;

    // Additional functional shims for tests
    setup_additional_shims(context)?;

    Ok(())
}

/// Setup miscellaneous APIs that don't fit in other modules
fn setup_misc_apis(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Document and DOM APIs are now natively implemented in Boa engine

        // No polyfill needed - async/await is natively supported by Boa engine
        if (typeof window !== 'undefined' && typeof window.requestAnimationFrame === 'undefined') {
            window.requestAnimationFrame = function(callback) {
                return setTimeout(function() {
                    callback(Date.now());
                }, 16); // ~60fps
            };

            window.cancelAnimationFrame = function(id) {
                clearTimeout(id);
            };
        }

        // URL and URLSearchParams are now natively implemented in Boa engine


        // TextEncoder/TextDecoder are now natively implemented in Boa engine

        // requestIdleCallback / cancelIdleCallback (used by React concurrent mode)
        if (typeof window !== 'undefined' && typeof window.requestIdleCallback === 'undefined') {
            window.requestIdleCallback = function(callback, options) {
                var timeout = (options && options.timeout) || 50;
                var start = Date.now();
                return setTimeout(function() {
                    callback({
                        didTimeout: Date.now() - start >= timeout,
                        timeRemaining: function() {
                            return Math.max(0, 50 - (Date.now() - start));
                        }
                    });
                }, 1);
            };

            window.cancelIdleCallback = function(id) {
                clearTimeout(id);
            };
        }

        // navigator.sendBeacon (used by analytics on page unload)
        if (typeof navigator !== 'undefined' && typeof navigator.sendBeacon === 'undefined') {
            navigator.sendBeacon = function(url, data) {
                try {
                    // Fire-and-forget POST request using fetch
                    var init = {
                        method: 'POST',
                        keepalive: true,
                        credentials: 'include'
                    };
                    if (data !== undefined && data !== null) {
                        if (typeof data === 'string') {
                            init.body = data;
                            init.headers = { 'Content-Type': 'text/plain;charset=UTF-8' };
                        } else if (typeof Blob !== 'undefined' && data instanceof Blob) {
                            init.body = data;
                            if (data.type) {
                                init.headers = { 'Content-Type': data.type };
                            }
                        } else if (typeof URLSearchParams !== 'undefined' && data instanceof URLSearchParams) {
                            init.body = data.toString();
                            init.headers = { 'Content-Type': 'application/x-www-form-urlencoded;charset=UTF-8' };
                        } else if (typeof FormData !== 'undefined' && data instanceof FormData) {
                            init.body = data;
                            // Let fetch set the Content-Type with boundary for FormData
                        } else {
                            init.body = String(data);
                        }
                    }
                    fetch(url, init).catch(function() {});
                    return true;
                } catch(e) {
                    return false;
                }
            };
        }

        // Basic Web Animations API (Chrome 133)
        if (typeof Element !== 'undefined' && typeof Element.prototype.animate === 'undefined') {
            Element.prototype.animate = function(keyframes, options) {
                // MOCK - Real implementation would create actual animations
                console.log('Element.animate called with keyframes:', keyframes, 'options:', options);

                var animation = {
                    currentTime: 0,
                    playbackRate: 1,
                    playState: 'running',
                    startTime: Date.now(),

                    play: function() {
                        this.playState = 'running';
                        console.log('Animation play()');
                    },

                    pause: function() {
                        this.playState = 'paused';
                        console.log('Animation pause()');
                    },

                    cancel: function() {
                        this.playState = 'idle';
                        console.log('Animation cancel()');
                    },

                    finish: function() {
                        this.playState = 'finished';
                        console.log('Animation finish()');
                    },

                    reverse: function() {
                        this.playbackRate *= -1;
                        console.log('Animation reverse()');
                    },

                    addEventListener: function(type, listener) {
                        console.log('Animation event listener added:', type);
                    },

                    removeEventListener: function(type, listener) {
                        console.log('Animation event listener removed:', type);
                    }
                };

                // Set duration from options
                if (options && typeof options === 'object') {
                    if (typeof options.duration === 'number') {
                        animation.duration = options.duration;
                    }
                    if (typeof options.fill === 'string') {
                        animation.fill = options.fill;
                    }
                    if (typeof options.easing === 'string') {
                        animation.easing = options.easing;
                    }
                } else if (typeof options === 'number') {
                    animation.duration = options;
                }

                return animation;
            };
        }

        // Basic navigation.userAgentData (Chrome 89+)
        if (typeof navigator !== 'undefined' && typeof navigator.userAgentData === 'undefined') {
            Object.defineProperty(navigator, 'userAgentData', {
                value: {
                    brands: [
                        { brand: 'Chromium', version: '140' },
                        { brand: 'Chrome', version: '140' },
                        { brand: 'Not_A Brand', version: '99' }
                    ],
                    mobile: false,
                    platform: 'Linux',

                    getHighEntropyValues: function(hints) {
                        return Promise.resolve({
                            architecture: 'x86',
                            bitness: '64',
                            model: '',
                            platform: 'Linux',
                            platformVersion: '6.0.0',
                            uaFullVersion: '140.0.0.0',
                            fullVersionList: [
                                { brand: 'Thalora', version: '1.0.0.0' },
                                { brand: 'Chromium', version: '140.0.0.0' },
                                { brand: 'Chrome', version: '140.0.0.0' }
                            ]
                        });
                    },

                    toJSON: function() {
                        return {
                            brands: this.brands,
                            mobile: this.mobile,
                            platform: this.platform
                        };
                    }
                },
                writable: false,
                enumerable: true,
                configurable: false
            });
        }
    "#))?;

    Ok(())
}

// Additional functional shims that provide minimal but working implementations
// for APIs expected by tests. These are intentionally small and safe; they
// do not perform network I/O but provide correct shapes and simple behaviors.
fn setup_additional_shims(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        (function(){
            // WebSocketStream minimal functional shim (local echo)
            if (typeof WebSocketStream === 'undefined') {
                function WebSocketStream(url) {
                    if (!(this instanceof WebSocketStream)) return new WebSocketStream(url);
                    this.url = url || '';
                    this._events = {};
                    this.readyState = 1;
                }

                WebSocketStream.prototype.addEventListener = function(type, cb) {
                    if (!this._events[type]) this._events[type] = [];
                    this._events[type].push(cb);
                };

                WebSocketStream.prototype.removeEventListener = function(type, cb) {
                    if (!this._events[type]) return;
                    this._events[type] = this._events[type].filter(function(f){ return f !== cb; });
                };

                WebSocketStream.prototype.send = function(data) {
                    // Echo to message listeners synchronously
                    var evs = this._events['message'] || [];
                    for (var i = 0; i < evs.length; i++) { try { evs[i]({data: data}); } catch(e){} }
                };

                WebSocketStream.prototype.close = function() {
                    this.readyState = 3;
                    var evs = this._events['close'] || [];
                    for (var i = 0; i < evs.length; i++) { try { evs[i](); } catch(e){} }
                };

                try { this.WebSocketStream = WebSocketStream; } catch(e) { window.WebSocketStream = WebSocketStream; }
            }

            // Ensure Element.prototype.setHTMLUnsafe exists and sets innerHTML when possible
            try {
                if (typeof Element !== 'undefined' && typeof Element.prototype.setHTMLUnsafe === 'undefined') {
                    Element.prototype.setHTMLUnsafe = function(html) {
                        try {
                            if (typeof this.innerHTML !== 'undefined') { this.innerHTML = html; }
                            else { this._setHTMLUnsafeValue = html; }
                            return true;
                        } catch(e) { return false; }
                    };
                }
            } catch(e) {}

            // Selection/Range helpers: provide getComposedRanges, direction, and modify
            try {
                if (typeof Selection !== 'undefined') {
                    if (typeof Selection.prototype.getComposedRanges === 'undefined') {
                        Selection.prototype.getComposedRanges = function() {
                            if (this._ranges && this._ranges.length) return this._ranges.slice();
                            return [];
                        };
                    }

                    if (typeof Selection.prototype.modify === 'undefined') {
                        Selection.prototype.modify = function(alter, direction, granularity) {
                            // Minimal behavior: adjust internal type and return
                            if (alter === 'move' || alter === 'extend') { this.type = 'Caret'; }
                        };
                    }
                }

                if (typeof Range !== 'undefined') {
                    if (typeof Range.prototype.setStart === 'undefined') {
                        Range.prototype.setStart = function(node, offset) { this.startContainer = node; this.startOffset = offset || 0; };
                    }
                    if (typeof Range.prototype.setEnd === 'undefined') {
                        Range.prototype.setEnd = function(node, offset) { this.endContainer = node; this.endOffset = offset || 0; };
                    }
                }
            } catch(e) {}
        })();
    "#))?;

    Ok(())
}
