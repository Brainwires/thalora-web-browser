use boa_engine::{Context, JsResult, Source};

// Import the modular polyfill components
use super::{
    performance::setup_performance_apis,
    security::setup_security_apis,
    worker::setup_worker_apis,
    storage::setup_storage_apis,
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
    setup_worker_apis(context)?;
    setup_storage_apis(context)?;
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

        // No polyfill needed
        // Async/await polyfill using Promises
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

        // URL Constructor (basic)
        if (typeof URL === 'undefined') {
            var URL = function(url, base) {
                if (typeof url !== 'string') {
                    throw new TypeError('URL constructor requires a string');
                }

                // Simple URL parsing - real implementation would be much more comprehensive
                var fullUrl = url;
                if (base && !url.includes('://')) {
                    if (base.endsWith('/') && url.startsWith('/')) {
                        fullUrl = base.slice(0, -1) + url;
                    } else if (!base.endsWith('/') && !url.startsWith('/')) {
                        fullUrl = base + '/' + url;
                    } else {
                        fullUrl = base + url;
                    }
                }

                var match = fullUrl.match(/^(https?:)\/\/([^\/]+)(\/.*)?$/);
                if (match) {
                    this.protocol = match[1];
                    this.host = match[2];
                    this.hostname = match[2].split(':')[0];
                    this.port = match[2].includes(':') ? match[2].split(':')[1] : '';
                    this.pathname = match[3] || '/';
                    this.href = fullUrl;
                    this.origin = this.protocol + '//' + this.host;
                } else {
                    this.protocol = 'http:';
                    this.host = 'localhost';
                    this.hostname = 'localhost';
                    this.port = '';
                    this.pathname = '/' + url;
                    this.href = 'http://localhost/' + url;
                    this.origin = 'http://localhost';
                }

                this.search = '';
                this.hash = '';
                this.username = '';
                this.password = '';

                // Extract search and hash if present
                var pathParts = this.pathname.split('?');
                if (pathParts.length > 1) {
                    this.pathname = pathParts[0];
                    var searchHash = pathParts[1].split('#');
                    this.search = '?' + searchHash[0];
                    if (searchHash.length > 1) {
                        this.hash = '#' + searchHash[1];
                    }
                } else {
                    var hashParts = this.pathname.split('#');
                    if (hashParts.length > 1) {
                        this.pathname = hashParts[0];
                        this.hash = '#' + hashParts[1];
                    }
                }

                this.searchParams = {
                    get: function(name) {
                        // Basic implementation
                        if (!this.search) return null;
                        var params = this.search.substring(1).split('&');
                        for (var i = 0; i < params.length; i++) {
                            var param = params[i].split('=');
                            if (param[0] === name) {
                                return decodeURIComponent(param[1] || '');
                            }
                        }
                        return null;
                    }.bind(this),
                    set: function(name, value) {
                        console.log('URLSearchParams.set called:', name, value);
                    },
                    append: function(name, value) {
                        console.log('URLSearchParams.append called:', name, value);
                    },
                    delete: function(name) {
                        console.log('URLSearchParams.delete called:', name);
                    },
                    has: function(name) {
                        return this.get(name) !== null;
                    }.bind(this),
                    keys: function() {
                        return [];
                    },
                    values: function() {
                        return [];
                    },
                    entries: function() {
                        return [];
                    },
                    toString: function() {
                        return this.search.substring(1);
                    }.bind(this)
                };
            };

            URL.createObjectURL = function(object) {
                // MOCK - Real implementation would create blob URL
                return 'blob:' + Date.now();
            };

            URL.revokeObjectURL = function(url) {
                // MOCK - Real implementation would revoke blob URL
                console.log('URL.revokeObjectURL called for:', url);
            };
        }

        // URLSearchParams constructor
        if (typeof URLSearchParams === 'undefined') {
            var URLSearchParams = function(init) {
                this.params = new Map();

                if (typeof init === 'string') {
                    if (init.startsWith('?')) {
                        init = init.substring(1);
                    }
                    var pairs = init.split('&');
                    for (var i = 0; i < pairs.length; i++) {
                        if (pairs[i]) {
                            var pair = pairs[i].split('=');
                            var key = decodeURIComponent(pair[0]);
                            var value = pair.length > 1 ? decodeURIComponent(pair[1]) : '';
                            this.append(key, value);
                        }
                    }
                }
            };

            URLSearchParams.prototype.append = function(name, value) {
                var existing = this.params.get(name);
                if (existing) {
                    if (Array.isArray(existing)) {
                        existing.push(String(value));
                    } else {
                        this.params.set(name, [existing, String(value)]);
                    }
                } else {
                    this.params.set(name, String(value));
                }
            };

            URLSearchParams.prototype.delete = function(name) {
                this.params.delete(name);
            };

            URLSearchParams.prototype.get = function(name) {
                var value = this.params.get(name);
                if (Array.isArray(value)) {
                    return value[0];
                }
                return value || null;
            };

            URLSearchParams.prototype.getAll = function(name) {
                var value = this.params.get(name);
                if (Array.isArray(value)) {
                    return value.slice();
                }
                return value ? [value] : [];
            };

            URLSearchParams.prototype.has = function(name) {
                return this.params.has(name);
            };

            URLSearchParams.prototype.set = function(name, value) {
                this.params.set(name, String(value));
            };

            URLSearchParams.prototype.toString = function() {
                var result = [];
                this.params.forEach(function(value, key) {
                    if (Array.isArray(value)) {
                        value.forEach(function(v) {
                            result.push(encodeURIComponent(key) + '=' + encodeURIComponent(v));
                        });
                    } else {
                        result.push(encodeURIComponent(key) + '=' + encodeURIComponent(value));
                    }
                });
                return result.join('&');
            };

            URLSearchParams.prototype.keys = function() {
                var keys = [];
                this.params.forEach(function(value, key) {
                    if (Array.isArray(value)) {
                        value.forEach(function() {
                            keys.push(key);
                        });
                    } else {
                        keys.push(key);
                    }
                });
                return keys;
            };

            URLSearchParams.prototype.values = function() {
                var values = [];
                this.params.forEach(function(value) {
                    if (Array.isArray(value)) {
                        values = values.concat(value);
                    } else {
                        values.push(value);
                    }
                });
                return values;
            };

            URLSearchParams.prototype.entries = function() {
                var entries = [];
                this.params.forEach(function(value, key) {
                    if (Array.isArray(value)) {
                        value.forEach(function(v) {
                            entries.push([key, v]);
                        });
                    } else {
                        entries.push([key, value]);
                    }
                });
                return entries;
            };
        }

        // TextEncoder/TextDecoder for string encoding
        if (typeof TextEncoder === 'undefined') {
            var TextEncoder = function(encoding) {
                this.encoding = encoding || 'utf-8';
            };

            TextEncoder.prototype.encode = function(string) {
                // Simple UTF-8 encoding simulation
                var utf8 = [];
                for (var i = 0; i < string.length; i++) {
                    var charcode = string.charCodeAt(i);
                    if (charcode < 0x80) {
                        utf8.push(charcode);
                    } else if (charcode < 0x800) {
                        utf8.push(0xc0 | (charcode >> 6), 0x80 | (charcode & 0x3f));
                    } else if (charcode < 0xd800 || charcode >= 0xe000) {
                        utf8.push(0xe0 | (charcode >> 12), 0x80 | ((charcode>>6) & 0x3f), 0x80 | (charcode & 0x3f));
                    }
                }
                return new Uint8Array(utf8);
            };
        }

        if (typeof TextDecoder === 'undefined') {
            var TextDecoder = function(encoding) {
                this.encoding = encoding || 'utf-8';
            };

            TextDecoder.prototype.decode = function(buffer) {
                // Simple UTF-8 decoding simulation
                var result = '';
                var view = new Uint8Array(buffer);
                for (var i = 0; i < view.length; i++) {
                    result += String.fromCharCode(view[i]);
                }
                return result;
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
                        { brand: 'Thalora', version: '1.0' },
                        { brand: 'Chromium', version: '140' },
                        { brand: 'Chrome', version: '140' }
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
