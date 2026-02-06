use thalora_browser_apis::boa_engine::{Context, JsResult, Source};

// Import the modular polyfill components
use super::{
    performance::setup_performance_apis,
    security::setup_security_apis,
    chrome_features::setup_chrome_features,
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

        // Basic navigation.userAgentData (Chrome 89+) - must match Chrome 131/Windows
        if (typeof navigator !== 'undefined' && typeof navigator.userAgentData === 'undefined') {
            Object.defineProperty(navigator, 'userAgentData', {
                value: {
                    brands: [
                        { brand: 'Google Chrome', version: '131' },
                        { brand: 'Chromium', version: '131' },
                        { brand: 'Not_A Brand', version: '24' }
                    ],
                    mobile: false,
                    platform: 'Windows',

                    getHighEntropyValues: function(hints) {
                        return Promise.resolve({
                            architecture: 'x86',
                            bitness: '64',
                            model: '',
                            platform: 'Windows',
                            platformVersion: '15.0.0',
                            uaFullVersion: '131.0.0.0',
                            fullVersionList: [
                                { brand: 'Google Chrome', version: '131.0.0.0' },
                                { brand: 'Chromium', version: '131.0.0.0' },
                                { brand: 'Not_A Brand', version: '24.0.0.0' }
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

        // navigator.connection (NetworkInformation API) - Cloudflare checks this exists
        if (typeof navigator !== 'undefined' && typeof navigator.connection === 'undefined') {
            Object.defineProperty(navigator, 'connection', {
                value: {
                    effectiveType: '4g',
                    downlink: 10,
                    rtt: 50,
                    saveData: false,
                    type: 'wifi',
                    addEventListener: function() {},
                    removeEventListener: function() {}
                },
                writable: false,
                enumerable: true,
                configurable: true
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
