use boa_engine::{Context, JsResult, Source};

/// Setup Web APIs (fetch, URL, etc.)
pub fn setup_web_apis(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Performance API
        if (typeof performance === 'undefined') {
            var performance = {
                now: function() {
                    return Date.now(); // Fallback to Date.now()
                },
                timeOrigin: Date.now() - 1000, // Mock time origin
                timing: {
                    navigationStart: Date.now() - 1000,
                    unloadEventStart: Date.now() - 950,
                    unloadEventEnd: Date.now() - 945,
                    redirectStart: 0,
                    redirectEnd: 0,
                    fetchStart: Date.now() - 940,
                    domainLookupStart: Date.now() - 935,
                    domainLookupEnd: Date.now() - 930,
                    connectStart: Date.now() - 925,
                    connectEnd: Date.now() - 920,
                    secureConnectionStart: Date.now() - 915,
                    requestStart: Date.now() - 910,
                    responseStart: Date.now() - 905,
                    responseEnd: Date.now() - 900,
                    domLoading: Date.now() - 895,
                    domInteractive: Date.now() - 890,
                    domContentLoadedEventStart: Date.now() - 885,
                    domContentLoadedEventEnd: Date.now() - 880,
                    domComplete: Date.now() - 875,
                    loadEventStart: Date.now() - 870,
                    loadEventEnd: Date.now() - 865
                },
                mark: function(name) {
                    // Mock implementation
                    return undefined;
                },
                measure: function(name, startMark, endMark) {
                    // Mock implementation
                    return undefined;
                },
                clearMarks: function(name) {
                    // Mock implementation
                    return undefined;
                },
                clearMeasures: function(name) {
                    // Mock implementation
                    return undefined;
                },
                getEntries: function() {
                    // Mock implementation - return empty array
                    return [];
                },
                getEntriesByType: function(type) {
                    // Mock implementation - return empty array
                    return [];
                },
                getEntriesByName: function(name, type) {
                    // Mock implementation - return empty array
                    return [];
                }
            };
        }

        // PerformanceObserver API
        if (typeof PerformanceObserver === 'undefined') {
            var PerformanceObserver = function(callback) {
                this.callback = callback;
                this.observing = false;
            };

            PerformanceObserver.prototype.observe = function(options) {
                this.observing = true;
                // Mock implementation - doesn't actually observe anything
            };

            PerformanceObserver.prototype.disconnect = function() {
                this.observing = false;
            };

            PerformanceObserver.prototype.takeRecords = function() {
                return [];
            };
        }

        // Chrome-specific APIs (should be undefined in non-Chrome browsers)
        // These should NOT exist in our implementation, but tests check for them
        if (typeof chrome === 'undefined') {
            // Chrome APIs should be undefined, not throw errors
            var chrome = undefined;
        }

        // Security Context APIs - detect actual context
        if (typeof window !== 'undefined') {
            // Detect if we're in a secure context (HTTPS, localhost, or file://)
            if (typeof window.isSecureContext === 'undefined') {
                var currentLocation = typeof location !== 'undefined' ? location.protocol : 'https:';
                window.isSecureContext = currentLocation === 'https:' ||
                                       currentLocation === 'file:' ||
                                       (typeof location !== 'undefined' && location.hostname === 'localhost');
            }

            // Set origin only if we have real location context
            if (typeof window.origin === 'undefined') {
                if (typeof location !== 'undefined' && location.protocol && location.host) {
                    window.origin = location.protocol + '//' + location.host;
                }
                // Otherwise leave it undefined - no fake origins
            }

            // History API - Real implementation will be provided by browser engine
            // The actual History API with navigation is set up by the browser renderer
            // This polyfill only provides minimal fallbacks if the real API isn't available
            if (typeof window.history === 'undefined') {
                window.history = {
                    length: 1,
                    state: null,
                    scrollRestoration: 'auto',

                    back: function() {
                        // Fallback - real implementation provided by browser engine
                        console.log('history.back() - fallback called (real navigation should be available via browser engine)');
                    },

                    forward: function() {
                        // Fallback - real implementation provided by browser engine
                        console.log('history.forward() - fallback called (real navigation should be available via browser engine)');
                    },

                    go: function(delta) {
                        // Fallback - real implementation provided by browser engine
                        console.log('history.go(' + delta + ') - fallback called (real navigation should be available via browser engine)');
                    },

                    pushState: function(state, title, url) {
                        // Fallback - real implementation provided by browser engine
                        this.state = state;
                        console.log('history.pushState() - fallback called (real implementation should be available via browser engine)');
                    },

                    replaceState: function(state, title, url) {
                        // Fallback - real implementation provided by browser engine
                        this.state = state;
                        console.log('history.replaceState() - fallback called (real implementation should be available via browser engine)');
                    }
                };
            }

        }

        // Document properties
        if (typeof document !== 'undefined') {
            if (typeof document.visibilityState === 'undefined') {
                document.visibilityState = 'visible';
            }
        }

        // Security and Privacy APIs
        if (typeof TrustedHTML === 'undefined') {
            var TrustedHTML = function(value) {
                this.toString = function() { return value; };
            };
        }

        if (typeof SecurityPolicyViolationEvent === 'undefined') {
            var SecurityPolicyViolationEvent = function(type, eventInitDict) {
                this.type = type;
                this.blockedURI = eventInitDict && eventInitDict.blockedURI || '';
                this.documentURI = eventInitDict && eventInitDict.documentURI || '';
                this.violatedDirective = eventInitDict && eventInitDict.violatedDirective || '';
            };
        }

        if (typeof crossOriginIsolated === 'undefined') {
            var crossOriginIsolated = false;
        }

        // Create a basic history object for global access
        var history = {
            length: 1,
            state: null,
            scrollRestoration: 'auto',
            back: function() { console.log('history.back() called'); return undefined; },
            forward: function() { console.log('history.forward() called'); return undefined; },
            go: function(delta) { console.log('history.go(' + delta + ') called'); return undefined; },
            pushState: function(state, title, url) {
                console.log('history.pushState called');
                this.state = state;
                return undefined;
            },
            replaceState: function(state, title, url) {
                console.log('history.replaceState called');
                this.state = state;
                return undefined;
            }
        };

        // Also make it available as window.history if window exists
        if (typeof window !== 'undefined') {
            window.history = history;
        }

        // Basic DOM API - Element constructor
        if (typeof Element === 'undefined') {
            var Element = function Element() {};
            Element.prototype = {
                constructor: Element,
                setHTMLUnsafe: function(html) {
                    console.log('Element.setHTMLUnsafe called with:', html);
                    this.innerHTML = html;
                    return this;
                },
                innerHTML: '',
                tagName: 'DIV'
            };
        }

        // Basic DOM API - Document constructor
        if (typeof Document === 'undefined') {
            var Document = function Document() {};
            Document.parseHTMLUnsafe = function(html) {
                console.log('Document.parseHTMLUnsafe called with:', html);
                var element = new Element();
                element.innerHTML = html;
                return element;
            };
        }

        // Basic document object if it doesn't exist
        if (typeof document === 'undefined') {
            var document = {
                visibilityState: 'visible',
                hidden: false,
                addEventListener: function(type, listener) {
                    console.log('document.addEventListener called for:', type);
                },
                removeEventListener: function(type, listener) {
                    console.log('document.removeEventListener called for:', type);
                }
            };
        }

        // Storage Access API (Chrome 125) - add to document using Object.defineProperty
        if (typeof document !== 'undefined' && typeof document.requestStorageAccess === 'undefined') {
            try {
                Object.defineProperty(document, 'requestStorageAccess', {
                    value: function(options) {
                        console.log('document.requestStorageAccess called with options:', options);
                        // In headless mode, assume storage access is granted
                        return Promise.resolve();
                    },
                    writable: true,
                    enumerable: false,
                    configurable: true
                });
            } catch (e) {
                // Fallback for environments where Object.defineProperty doesn't work
                document.requestStorageAccess = function(options) {
                    console.log('document.requestStorageAccess called with options:', options);
                    return Promise.resolve();
                };
            }
        }

        if (typeof document !== 'undefined' && typeof document.hasStorageAccess === 'undefined') {
            try {
                Object.defineProperty(document, 'hasStorageAccess', {
                    value: function() {
                        console.log('document.hasStorageAccess called');
                        // In headless mode, assume storage access is available
                        return Promise.resolve(true);
                    },
                    writable: true,
                    enumerable: false,
                    configurable: true
                });
            } catch (e) {
                // Fallback for environments where Object.defineProperty doesn't work
                document.hasStorageAccess = function() {
                    console.log('document.hasStorageAccess called');
                    return Promise.resolve(true);
                };
            }
        }

        // ReadableStream API
        if (typeof ReadableStream === 'undefined') {
            var ReadableStream = function ReadableStream(underlyingSource, strategy) {
                this.underlyingSource = underlyingSource || {};
                this.strategy = strategy || {};
                this.locked = false;
                this._controller = null;
                this._started = false;

                // Initialize the stream
                if (this.underlyingSource.start) {
                    this._controller = {
                        enqueue: function(chunk) {
                            console.log('ReadableStream: enqueuing chunk', chunk);
                        },
                        close: function() {
                            console.log('ReadableStream: closing stream');
                        },
                        error: function(error) {
                            console.log('ReadableStream: error', error);
                        }
                    };
                    try {
                        this.underlyingSource.start(this._controller);
                        this._started = true;
                    } catch (e) {
                        console.error('ReadableStream start error:', e);
                    }
                }
            };

            ReadableStream.prototype[Symbol.asyncIterator] = function() {
                console.log('ReadableStream: creating async iterator');
                return {
                    next: function() {
                        return Promise.resolve({ value: undefined, done: true });
                    },
                    return: function() {
                        return Promise.resolve({ done: true });
                    }
                };
            };

            ReadableStream.prototype.getReader = function() {
                return {
                    read: function() {
                        return Promise.resolve({ value: undefined, done: true });
                    },
                    cancel: function() {
                        return Promise.resolve();
                    }
                };
            };
        }

        // WebSocketStream API (Chrome 124)
        if (typeof WebSocketStream === 'undefined') {
            var WebSocketStream = function WebSocketStream(url, options) {
                console.log('WebSocketStream created:', url, options);
                this.url = url;
                this.options = options || {};
                this.readyState = 0; // CONNECTING

                // Simulate connection
                var self = this;
                setTimeout(function() {
                    self.readyState = 1; // OPEN
                }, 100);
            };

            WebSocketStream.prototype.connection = Promise.resolve({
                readable: new ReadableStream(),
                writable: {
                    getWriter: function() {
                        return {
                            write: function(data) {
                                console.log('WebSocketStream write:', data);
                                return Promise.resolve();
                            },
                            close: function() {
                                return Promise.resolve();
                            }
                        };
                    }
                }
            });
        }

        // PressureObserver API (Chrome 125 - Compute Pressure)
        if (typeof PressureObserver === 'undefined') {
            var PressureObserver = function PressureObserver(callback, options) {
                this.callback = callback;
                this.options = options || {};
                this.observing = false;
            };

            PressureObserver.prototype.observe = function(source) {
                console.log('PressureObserver.observe called for source:', source);
                this.observing = true;
                // Simulate no pressure data in headless mode
                setTimeout(() => {
                    if (this.observing && this.callback) {
                        this.callback([{
                            source: source || 'cpu',
                            state: 'nominal',
                            time: performance.now ? performance.now() : Date.now()
                        }], this);
                    }
                }, 100);
            };

            PressureObserver.prototype.unobserve = function(source) {
                console.log('PressureObserver.unobserve called for source:', source);
                this.observing = false;
            };

            PressureObserver.prototype.disconnect = function() {
                console.log('PressureObserver.disconnect called');
                this.observing = false;
            };

            PressureObserver.knownSources = ['cpu'];
        }

        // CSS.supports API for modern CSS feature detection
        if (typeof CSS === 'undefined') {
            var CSS = {};
        }

        if (!CSS.supports || typeof CSS.supports !== 'function') {
            CSS.supports = function(property, value) {
                // Basic implementation that supports common CSS features
                // In a real implementation, this would parse and validate CSS

                if (arguments.length === 1) {
                    // CSS.supports('selector(:has(div))')
                    var declaration = property;
                    if (typeof declaration === 'string') {
                        // Check for modern CSS features
                        if (declaration.includes('selector') || declaration.includes(':has')) {
                            return true; // :has() selector - basic support implemented
                        }
                        if (declaration.includes('@layer')) {
                            return true; // CSS layers - basic support implemented
                        }
                        if (declaration.includes('container-type')) {
                            return true; // Container queries - basic support implemented
                        }
                        if (declaration.includes('grid-template-rows') && declaration.includes('subgrid')) {
                            return true; // CSS subgrid - basic support implemented
                        }
                        return true; // Most CSS features are supported with basic implementation
                    }
                    return false;
                }

                if (arguments.length === 2) {
                    // CSS.supports('display', 'grid')
                    var prop = property;
                    var val = value;

                    // Support common CSS features that are widely supported
                    if (prop === 'display') {
                        if (val === 'grid' || val === 'flex' || val === 'block' || val === 'inline' || val === 'none') {
                            return true;
                        }
                    }

                    if (prop === '--custom-property' || prop.startsWith('--')) {
                        return true; // CSS custom properties supported
                    }

                    if (prop === 'gap' && (val === '1rem' || val === '10px' || typeof val === 'string')) {
                        return true; // CSS gap supported
                    }

                    if (prop === 'aspect-ratio') {
                        return true; // CSS aspect-ratio supported
                    }

                    if (prop === 'margin-inline-start' || prop.includes('inline') || prop.includes('block')) {
                        return true; // CSS logical properties supported
                    }

                    // Default: most properties are "supported" for basic compatibility
                    return true;
                }

                return false;
            };
        }

        // XMLHttpRequest implementation (basic)
        if (typeof XMLHttpRequest === 'undefined') {
            var XMLHttpRequest = function() {
                this.readyState = 0;
                this.status = 0;
                this.statusText = '';
                this.responseText = '';
                this.responseXML = null;
                this.response = '';
                this.responseType = '';
                this.timeout = 0;
                this.withCredentials = false;
                this.upload = {};

                this.onreadystatechange = null;
                this.onabort = null;
                this.onerror = null;
                this.onload = null;
                this.onloadend = null;
                this.onloadstart = null;
                this.onprogress = null;
                this.ontimeout = null;

                this.open = function(method, url, async, user, password) {
                    this.readyState = 1;
                    if (this.onreadystatechange) this.onreadystatechange();
                };

                this.send = function(data) {
                    var self = this;
                    setTimeout(function() {
                        self.readyState = 4;
                        self.status = 200;
                        self.statusText = 'OK';
                        self.responseText = '{"mock": "response"}';
                        self.response = self.responseText;
                        if (self.onreadystatechange) self.onreadystatechange();
                        if (self.onload) self.onload();
                    }, 10);
                };

                this.abort = function() {
                    this.readyState = 0;
                    if (this.onabort) this.onabort();
                };

                this.setRequestHeader = function(header, value) {
                    // Mock implementation
                };

                this.getResponseHeader = function(header) {
                    return null;
                };

                this.getAllResponseHeaders = function() {
                    return '';
                };

                this.overrideMimeType = function(mimeType) {
                    // Mock implementation
                };
            };

            XMLHttpRequest.UNSENT = 0;
            XMLHttpRequest.OPENED = 1;
            XMLHttpRequest.HEADERS_RECEIVED = 2;
            XMLHttpRequest.LOADING = 3;
            XMLHttpRequest.DONE = 4;
        }

        // Worker API (basic implementation)
        if (typeof Worker === 'undefined') {
            var Worker = function(scriptURL, options) {
                this.onerror = null;
                this.onmessage = null;
                this.onmessageerror = null;

                this.postMessage = function(message, transfer) {
                    // Mock implementation - doesn't actually run scripts
                    console.log('Worker.postMessage() called (mock implementation)');
                };

                this.terminate = function() {
                    // Mock implementation
                    console.log('Worker.terminate() called (mock implementation)');
                };

                // Simulate worker creation success
                var self = this;
                setTimeout(function() {
                    // Mock worker is "ready"
                }, 10);
            };
        }

        // ES6+ Syntax Workarounds for Boa engine limitations

        // Destructuring assignment helper
        if (typeof __destructure === 'undefined') {
            var __destructure = function(array, vars) {
                // Helper for array destructuring: __destructure([1, 2], ['a', 'b'])
                for (var i = 0; i < vars.length && i < array.length; i++) {
                    window[vars[i]] = array[i];
                }
                return array;
            };

            // Object destructuring helper
            var __destructureObj = function(obj, props) {
                // Helper for object destructuring: __destructureObj({a: 1, b: 2}, ['a', 'b'])
                var result = {};
                for (var i = 0; i < props.length; i++) {
                    var prop = props[i];
                    result[prop] = obj[prop];
                    window[prop] = obj[prop];
                }
                return result;
            };
        }

        // Async/await polyfill using Promises
        if (typeof __async === 'undefined') {
            var __async = function(generatorFunc) {
                // Converts generator-like function to Promise-based async function
                return function() {
                    var args = Array.prototype.slice.call(arguments);
                    return new Promise(function(resolve, reject) {
                        try {
                            var result = generatorFunc.apply(this, args);
                            if (result && typeof result.then === 'function') {
                                result.then(resolve, reject);
                            } else {
                                resolve(result);
                            }
                        } catch (e) {
                            reject(e);
                        }
                    });
                };
            };

            var __await = function(promise) {
                // Await helper for Promise chains
                if (promise && typeof promise.then === 'function') {
                    return promise;
                } else {
                    return Promise.resolve(promise);
                }
            };
        }

        // Enhanced Date.now() for consistency
        Date.now = Date.now || function() {
            return new Date().getTime();
        };

        // URL Constructor (basic)
        if (typeof URL === 'undefined') {
            var URL = function(url, base) {
                this.href = url;

                // Basic URL parsing
                var match = url.match(/^(https?:)\/\/([^\/]+)(\/[^?#]*)?(\?[^#]*)?(#.*)?$/);
                if (match) {
                    this.protocol = match[1] || '';
                    this.host = match[2] || '';
                    this.pathname = match[3] || '/';
                    this.search = match[4] || '';
                    this.hash = match[5] || '';
                } else {
                    this.protocol = '';
                    this.host = '';
                    this.pathname = url;
                    this.search = '';
                    this.hash = '';
                }

                this.toString = function() { return this.href; };
            };
        }

        // URLSearchParams
        if (typeof URLSearchParams === 'undefined') {
            var URLSearchParams = function(init) {
                this.params = {};

                if (typeof init === 'string') {
                    var pairs = init.replace(/^\?/, '').split('&');
                    for (var i = 0; i < pairs.length; i++) {
                        var pair = pairs[i].split('=');
                        if (pair.length === 2) {
                            this.params[decodeURIComponent(pair[0])] = decodeURIComponent(pair[1]);
                        }
                    }
                }

                this.get = function(name) {
                    return this.params[name] || null;
                };

                this.set = function(name, value) {
                    this.params[name] = value;
                };

                this.has = function(name) {
                    return this.params.hasOwnProperty(name);
                };

                this.delete = function(name) {
                    delete this.params[name];
                };

                this.toString = function() {
                    var pairs = [];
                    for (var key in this.params) {
                        if (this.params.hasOwnProperty(key)) {
                            pairs.push(encodeURIComponent(key) + '=' + encodeURIComponent(this.params[key]));
                        }
                    }
                    return pairs.join('&');
                };
            };
        }

        // Enhanced JSON with better error handling and recursion protection
        if (typeof JSON === 'undefined') {
            var JSON = {};
        }

        // Safe JSON.stringify implementation with recursion protection
        if (!JSON.stringify || typeof JSON.stringify !== 'function') {
            JSON.stringify = function(obj, replacer, space) {
                var seen = [];

                function stringify(value, depth) {
                    depth = depth || 0;
                    if (depth > 100) return '"[Circular]"'; // Recursion protection

                    if (value === null) return 'null';
                    if (typeof value === 'undefined') return undefined;
                    if (typeof value === 'boolean') return value ? 'true' : 'false';
                    if (typeof value === 'number') return isFinite(value) ? String(value) : 'null';
                    if (typeof value === 'string') return '"' + value.replace(/[\\"\x00-\x1F]/g, function(c) {
                        switch (c) {
                            case '\\': return '\\\\';
                            case '"': return '\\"';
                            case '\n': return '\\n';
                            case '\r': return '\\r';
                            case '\t': return '\\t';
                            default: return '\\u' + ('0000' + c.charCodeAt(0).toString(16)).slice(-4);
                        }
                    }) + '"';

                    if (typeof value === 'object') {
                        // Check for circular reference
                        for (var i = 0; i < seen.length; i++) {
                            if (seen[i] === value) return '"[Circular]"';
                        }
                        seen.push(value);

                        if (Array.isArray(value)) {
                            var result = '[';
                            for (var j = 0; j < value.length; j++) {
                                if (j > 0) result += ',';
                                var item = stringify(value[j], depth + 1);
                                result += (item === undefined) ? 'null' : item;
                            }
                            result += ']';
                            seen.pop();
                            return result;
                        } else {
                            var result = '{';
                            var first = true;
                            for (var key in value) {
                                if (value.hasOwnProperty && value.hasOwnProperty(key)) {
                                    var val = stringify(value[key], depth + 1);
                                    if (val !== undefined) {
                                        if (!first) result += ',';
                                        result += '"' + key + '":' + val;
                                        first = false;
                                    }
                                }
                            }
                            result += '}';
                            seen.pop();
                            return result;
                        }
                    }

                    return undefined;
                }

                return stringify(obj);
            };
        }

        // Safe JSON.parse implementation - completely custom to avoid recursion
        if (!JSON.parse || typeof JSON.parse !== 'function') {
            JSON.parse = function(text) {
                if (typeof text !== 'string') {
                    throw new Error('JSON.parse: argument must be a string');
                }

                text = text.replace(/^\s+|\s+$/g, ''); // trim
                var index = 0;

                function parseValue() {
                    skipWhitespace();
                    if (index >= text.length) throw new Error('Unexpected end of input');

                    var char = text[index];
                    if (char === '"') return parseString();
                    if (char === '{') return parseObject();
                    if (char === '[') return parseArray();
                    if (char === 't' || char === 'f') return parseBoolean();
                    if (char === 'n') return parseNull();
                    if (char === '-' || (char >= '0' && char <= '9')) return parseNumber();

                    throw new Error('Unexpected token: ' + char);
                }

                function parseString() {
                    if (text[index] !== '"') throw new Error('Expected "');
                    index++;
                    var result = '';
                    while (index < text.length && text[index] !== '"') {
                        if (text[index] === '\\') {
                            index++;
                            if (index >= text.length) throw new Error('Incomplete escape sequence');
                            var escaped = text[index];
                            switch (escaped) {
                                case '"': result += '"'; break;
                                case '\\': result += '\\'; break;
                                case '/': result += '/'; break;
                                case 'b': result += '\b'; break;
                                case 'f': result += '\f'; break;
                                case 'n': result += '\n'; break;
                                case 'r': result += '\r'; break;
                                case 't': result += '\t'; break;
                                default: result += escaped; break;
                            }
                        } else {
                            result += text[index];
                        }
                        index++;
                    }
                    if (text[index] !== '"') throw new Error('Unterminated string');
                    index++;
                    return result;
                }

                function parseNumber() {
                    var start = index;
                    if (text[index] === '-') index++;
                    while (index < text.length && text[index] >= '0' && text[index] <= '9') index++;
                    if (text[index] === '.') {
                        index++;
                        while (index < text.length && text[index] >= '0' && text[index] <= '9') index++;
                    }
                    return parseFloat(text.substring(start, index));
                }

                function parseBoolean() {
                    if (text.substr(index, 4) === 'true') {
                        index += 4;
                        return true;
                    }
                    if (text.substr(index, 5) === 'false') {
                        index += 5;
                        return false;
                    }
                    throw new Error('Invalid boolean');
                }

                function parseNull() {
                    if (text.substr(index, 4) === 'null') {
                        index += 4;
                        return null;
                    }
                    throw new Error('Invalid null');
                }

                function parseArray() {
                    if (text[index] !== '[') throw new Error('Expected [');
                    index++;
                    var result = [];
                    skipWhitespace();

                    if (text[index] === ']') {
                        index++;
                        return result;
                    }

                    while (true) {
                        result.push(parseValue());
                        skipWhitespace();

                        if (text[index] === ']') {
                            index++;
                            break;
                        }
                        if (text[index] === ',') {
                            index++;
                            skipWhitespace();
                        } else {
                            throw new Error('Expected , or ]');
                        }
                    }
                    return result;
                }

                function parseObject() {
                    if (text[index] !== '{') throw new Error('Expected {');
                    index++;
                    var result = {};
                    skipWhitespace();

                    if (text[index] === '}') {
                        index++;
                        return result;
                    }

                    while (true) {
                        skipWhitespace();
                        if (text[index] !== '"') throw new Error('Expected property name');
                        var key = parseString();
                        skipWhitespace();

                        if (text[index] !== ':') throw new Error('Expected :');
                        index++;

                        result[key] = parseValue();
                        skipWhitespace();

                        if (text[index] === '}') {
                            index++;
                            break;
                        }
                        if (text[index] === ',') {
                            index++;
                        } else {
                            throw new Error('Expected , or }');
                        }
                    }
                    return result;
                }

                function skipWhitespace() {
                    while (index < text.length && /\s/.test(text[index])) {
                        index++;
                    }
                }

                return parseValue();
            };
        }

        // Safe parsing utility
        JSON.safeParse = function(str) {
            try {
                return { success: true, data: JSON.parse(str) };
            } catch (e) {
                return { success: false, error: e.message };
            }
        };

        // Basic fetch implementation (mock)
        if (typeof fetch === 'undefined') {
            var fetch = function(url, options) {
                options = options || {};

                return new Promise(function(resolve, reject) {
                    // Mock successful response
                    setTimeout(function() {
                        var response = {
                            ok: true,
                            status: 200,
                            statusText: 'OK',
                            url: url,
                            headers: {
                                get: function(name) {
                                    return null;
                                }
                            },
                            json: function() {
                                return Promise.resolve({});
                            },
                            text: function() {
                                return Promise.resolve('');
                            },
                            blob: function() {
                                return Promise.resolve(new Blob());
                            },
                            arrayBuffer: function() {
                                return Promise.resolve(new ArrayBuffer(0));
                            }
                        };
                        resolve(response);
                    }, 10);
                });
            };
        }

        // Basic Blob implementation
        if (typeof Blob === 'undefined') {
            var Blob = function(parts, options) {
                parts = parts || [];
                options = options || {};

                this.size = 0;
                this.type = options.type || '';

                this.text = function() {
                    return Promise.resolve(parts.join(''));
                };

                this.arrayBuffer = function() {
                    return Promise.resolve(new ArrayBuffer(this.size));
                };
            };
        }

        // Basic FormData implementation
        if (typeof FormData === 'undefined') {
            var FormData = function() {
                this.data = {};

                this.append = function(name, value) {
                    if (this.data[name]) {
                        if (Array.isArray(this.data[name])) {
                            this.data[name].push(value);
                        } else {
                            this.data[name] = [this.data[name], value];
                        }
                    } else {
                        this.data[name] = value;
                    }
                };

                this.delete = function(name) {
                    delete this.data[name];
                };

                this.get = function(name) {
                    var value = this.data[name];
                    return Array.isArray(value) ? value[0] : value;
                };

                this.getAll = function(name) {
                    var value = this.data[name];
                    return Array.isArray(value) ? value : [value];
                };

                this.has = function(name) {
                    return this.data.hasOwnProperty(name);
                };

                this.set = function(name, value) {
                    this.data[name] = value;
                };
            };
        }

        // Chrome 126: GamepadHapticActuator constructor
        if (typeof GamepadHapticActuator === 'undefined') {
            var GamepadHapticActuator = function() {
                this.type = 'dual-rumble';
                this.canPlay = function(effects) { return true; };
                this.playEffect = function(type, params) {
                    return Promise.resolve();
                };
                this.reset = function() {
                    return Promise.resolve();
                };
            };
        }

        // Chrome 126: WebGLObject constructor
        if (typeof WebGLObject === 'undefined') {
            var WebGLObject = function() {
                // Base WebGL object constructor
            };
        }

        // Chrome 127: MediaMetadata with chapter support
        if (typeof MediaMetadata === 'undefined') {
            var MediaMetadata = function(metadata) {
                if (metadata) {
                    this.title = metadata.title || '';
                    this.artist = metadata.artist || '';
                    this.album = metadata.album || '';
                    this.artwork = metadata.artwork || [];
                    // Chrome 127: Chapter information support
                    this.chapterInfo = metadata.chapterInfo || [];
                }
            };
        }

        // Chrome 127: User Activation API
        if (typeof navigator !== 'undefined' && typeof navigator.userActivation === 'undefined') {
            navigator.userActivation = {
                hasBeenActive: true,
                isActive: true
            };
        }

        // Chrome 127: View Transitions API
        if (typeof document !== 'undefined' && typeof document.startViewTransition === 'undefined') {
            document.startViewTransition = function(callback) {
                // Mock implementation - execute callback immediately
                if (typeof callback === 'function') {
                    try {
                        callback();
                    } catch (e) {
                        console.warn('View transition callback error:', e);
                    }
                }

                // Return a ViewTransition-like object
                return {
                    finished: Promise.resolve(),
                    ready: Promise.resolve(),
                    updateCallbackDone: Promise.resolve(),
                    skipTransition: function() {}
                };
            };
        }

        // Chrome 127: Document Picture-in-Picture API
        if (typeof documentPictureInPicture === 'undefined') {
            var documentPictureInPicture = {
                requestWindow: function(options) {
                    // Mock implementation - return rejected promise
                    return Promise.reject(new Error('Document Picture-in-Picture not supported in headless mode'));
                },
                window: null
            };
        }

        // Chrome 128: document.caretPositionFromPoint
        if (typeof document !== 'undefined' && typeof document.caretPositionFromPoint === 'undefined') {
            document.caretPositionFromPoint = function(x, y) {
                // Mock implementation - return a CaretPosition-like object
                return {
                    offsetNode: document.body,
                    offset: 0,
                    getClientRect: function() {
                        return {
                            left: x,
                            top: y,
                            right: x,
                            bottom: y,
                            width: 0,
                            height: 0
                        };
                    }
                };
            };
        }

        // Chrome 128: PointerEvent constructor with deviceProperties
        if (typeof PointerEvent === 'undefined') {
            var PointerEvent = function(type, options) {
                options = options || {};

                // Create a basic event object
                var event = {
                    type: type,
                    bubbles: options.bubbles || false,
                    cancelable: options.cancelable || false,
                    pointerId: options.pointerId || 0,
                    width: options.width || 1,
                    height: options.height || 1,
                    pressure: options.pressure || 0,
                    tangentialPressure: options.tangentialPressure || 0,
                    tiltX: options.tiltX || 0,
                    tiltY: options.tiltY || 0,
                    twist: options.twist || 0,
                    pointerType: options.pointerType || '',
                    isPrimary: options.isPrimary || false,
                    // Chrome 128: deviceProperties with uniqueId
                    deviceProperties: {
                        uniqueId: 'mock-device-' + Math.random().toString(36).substr(2, 9)
                    }
                };

                return event;
            };
        } else {
            // Extend existing PointerEvent with deviceProperties if it doesn't exist
            var originalPointerEvent = PointerEvent;
            PointerEvent = function(type, options) {
                var event = new originalPointerEvent(type, options);
                if (!event.deviceProperties) {
                    event.deviceProperties = {
                        uniqueId: 'mock-device-' + Math.random().toString(36).substr(2, 9)
                    };
                }
                return event;
            };
        }

        // Chrome 129: scheduler.yield API
        if (typeof scheduler === 'undefined') {
            var scheduler = {
                yield: function() {
                    // Return a promise that resolves immediately
                    // In real implementation, this would yield to the browser
                    return Promise.resolve();
                }
            };
        } else if (typeof scheduler.yield === 'undefined') {
            scheduler.yield = function() {
                return Promise.resolve();
            };
        }

        // Chrome 129: Intl.DurationFormat
        if (typeof Intl !== 'undefined' && typeof Intl.DurationFormat === 'undefined') {
            Intl.DurationFormat = function(locale, options) {
                this.locale = locale || 'en-US';
                this.options = options || {};

                this.format = function(duration) {
                    var parts = [];
                    if (duration.hours) {
                        parts.push(duration.hours + ' hr');
                    }
                    if (duration.minutes) {
                        parts.push(duration.minutes + ' min');
                    }
                    if (duration.seconds) {
                        parts.push(duration.seconds + ' sec');
                    }
                    return parts.join(' ');
                };

                this.formatToParts = function(duration) {
                    var parts = [];
                    if (duration.hours) {
                        parts.push({type: 'hours', value: duration.hours});
                        parts.push({type: 'literal', value: ' hr'});
                    }
                    if (duration.minutes) {
                        if (parts.length > 0) parts.push({type: 'literal', value: ' '});
                        parts.push({type: 'minutes', value: duration.minutes});
                        parts.push({type: 'literal', value: ' min'});
                    }
                    if (duration.seconds) {
                        if (parts.length > 0) parts.push({type: 'literal', value: ' '});
                        parts.push({type: 'seconds', value: duration.seconds});
                        parts.push({type: 'literal', value: ' sec'});
                    }
                    return parts;
                };
            };
        }

        // Chrome 129: PublicKeyCredential constructor
        if (typeof PublicKeyCredential === 'undefined') {
            var PublicKeyCredential = function() {
                // Mock PublicKeyCredential constructor
            };

            // Chrome 129: WebAuthn serialization methods
            PublicKeyCredential.prototype.toJSON = function() {
                return {
                    id: this.id || 'mock-credential-id',
                    type: 'public-key',
                    response: {}
                };
            };

            PublicKeyCredential.parseCreationOptionsFromJSON = function(json) {
                return json; // Mock implementation
            };

            PublicKeyCredential.parseRequestOptionsFromJSON = function(json) {
                return json; // Mock implementation
            };

            // Chrome 133: getClientCapabilities method
            PublicKeyCredential.getClientCapabilities = function() {
                return Promise.resolve({
                    rk: true, // Resident key support
                    up: true, // User presence support
                    uv: false, // User verification support (limited in headless)
                    plat: false, // Platform attachment
                    clientPin: false, // Client PIN support
                    largeBlobs: false, // Large blob support
                    credMgmt: false, // Credential management support
                    credProtect: false, // Credential protection support
                    bioEnroll: false, // Biometric enrollment support
                    userVerificationMgmtPreview: false
                });
            };
        } else {
            // Add serialization methods if they don't exist
            if (typeof PublicKeyCredential.prototype.toJSON === 'undefined') {
                PublicKeyCredential.prototype.toJSON = function() {
                    return {
                        id: this.id || 'mock-credential-id',
                        type: 'public-key',
                        response: {}
                    };
                };
            }

            if (typeof PublicKeyCredential.parseCreationOptionsFromJSON === 'undefined') {
                PublicKeyCredential.parseCreationOptionsFromJSON = function(json) {
                    return json;
                };
            }

            if (typeof PublicKeyCredential.parseRequestOptionsFromJSON === 'undefined') {
                PublicKeyCredential.parseRequestOptionsFromJSON = function(json) {
                    return json;
                };
            }

            // Chrome 133: getClientCapabilities method
            if (typeof PublicKeyCredential.getClientCapabilities === 'undefined') {
                PublicKeyCredential.getClientCapabilities = function() {
                    return Promise.resolve({
                        rk: true, // Resident key support
                        up: true, // User presence support
                        uv: false, // User verification support (limited in headless)
                        plat: false, // Platform attachment
                        clientPin: false, // Client PIN support
                        largeBlobs: false, // Large blob support
                        credMgmt: false, // Credential management support
                        credProtect: false, // Credential protection support
                        bioEnroll: false, // Biometric enrollment support
                        userVerificationMgmtPreview: false
                    });
                };
            }
        }

        // Chrome 129: FileSystemObserver (Origin Trial)
        if (typeof FileSystemObserver === 'undefined') {
            var FileSystemObserver = function(callback) {
                this.callback = callback;
            };

            FileSystemObserver.prototype.observe = function(handle, options) {
                // Mock implementation - not functional in headless mode
                console.warn('FileSystemObserver is not functional in headless mode');
            };

            FileSystemObserver.prototype.unobserve = function(handle) {
                // Mock implementation
            };

            FileSystemObserver.prototype.disconnect = function() {
                // Mock implementation
            };
        }

        // Chrome 130: IndexedDB API
        if (typeof indexedDB === 'undefined') {
            var indexedDB = {
                open: function(name, version) {
                    // Mock IDBOpenDBRequest
                    var request = {
                        result: null,
                        error: null,
                        readyState: 'pending',
                        onsuccess: null,
                        onerror: null,
                        onupgradeneeded: null,
                        onblocked: null,
                        addEventListener: function(type, listener) {},
                        removeEventListener: function(type, listener) {}
                    };

                    // Simulate async operation
                    setTimeout(function() {
                        request.readyState = 'done';
                        request.result = {
                            name: name,
                            version: version || 1,
                            objectStoreNames: [],
                            createObjectStore: function(name, options) {
                                return {
                                    name: name,
                                    keyPath: options ? options.keyPath : null,
                                    autoIncrement: options ? options.autoIncrement : false,
                                    add: function(value, key) { return { result: key }; },
                                    get: function(key) { return { result: undefined }; },
                                    put: function(value, key) { return { result: key }; },
                                    delete: function(key) { return { result: undefined }; }
                                };
                            },
                            transaction: function(storeNames, mode) {
                                return {
                                    mode: mode || 'readonly',
                                    objectStore: function(name) {
                                        return {
                                            name: name,
                                            add: function(value, key) { return { result: key }; },
                                            get: function(key) { return { result: undefined }; },
                                            put: function(value, key) { return { result: key }; },
                                            delete: function(key) { return { result: undefined }; }
                                        };
                                    }
                                };
                            },
                            close: function() {}
                        };

                        if (request.onsuccess) {
                            request.onsuccess({ target: request });
                        }
                    }, 10);

                    return request;
                },
                deleteDatabase: function(name) {
                    return {
                        onsuccess: null,
                        onerror: null,
                        addEventListener: function(type, listener) {},
                        removeEventListener: function(type, listener) {}
                    };
                },
                cmp: function(first, second) {
                    if (first < second) return -1;
                    if (first > second) return 1;
                    return 0;
                }
            };
        }

        // Chrome 130: Language Detector API (Origin Trial)
        if (typeof LanguageDetector === 'undefined') {
            var LanguageDetector = function() {
                this.detect = function(text) {
                    // Mock language detection
                    return Promise.resolve([{
                        language: 'en',
                        confidence: 0.95
                    }]);
                };
            };
        }

        // Add Language Detector to navigator.ml if available
        if (typeof navigator !== 'undefined' && typeof navigator.ml === 'undefined') {
            navigator.ml = {
                createLanguageDetector: function(options) {
                    return Promise.resolve(new LanguageDetector());
                }
            };
        }

        // Chrome 132: ToggleEvent constructor
        if (typeof ToggleEvent === 'undefined') {
            var ToggleEvent = function(type, eventInitDict) {
                this.type = type;
                this.bubbles = eventInitDict ? eventInitDict.bubbles : false;
                this.cancelable = eventInitDict ? eventInitDict.cancelable : false;
                this.oldState = eventInitDict ? eventInitDict.oldState : '';
                this.newState = eventInitDict ? eventInitDict.newState : '';
            };
        }

        // Chrome 132: MediaStreamTrack constructor
        if (typeof MediaStreamTrack === 'undefined') {
            var MediaStreamTrack = function() {
                this.kind = 'video';
                this.id = 'track_' + Math.random().toString(36).substr(2, 9);
                this.label = 'Mock Track';
                this.enabled = true;
                this.muted = false;
                this.readonly = false;
                this.readyState = 'live';

                this.clone = function() {
                    return new MediaStreamTrack();
                };

                this.stop = function() {
                    this.readyState = 'ended';
                };

                this.getCapabilities = function() {
                    return {};
                };

                this.getConstraints = function() {
                    return {};
                };

                this.getSettings = function() {
                    return {};
                };

                this.applyConstraints = function(constraints) {
                    return Promise.resolve();
                };

                this.addEventListener = function(event, handler) {};
                this.removeEventListener = function(event, handler) {};
            };
        }

        // Chrome 132: File System Access API - File Pickers
        if (typeof showOpenFilePicker === 'undefined') {
            var showOpenFilePicker = function(options) {
                // Mock implementation - return rejected promise for headless mode
                return Promise.reject(new Error('File System Access not supported in headless mode'));
            };

            var showSaveFilePicker = function(options) {
                return Promise.reject(new Error('File System Access not supported in headless mode'));
            };

            var showDirectoryPicker = function(options) {
                return Promise.reject(new Error('File System Access not supported in headless mode'));
            };

            // Also add to window if it exists
            if (typeof window !== 'undefined') {
                window.showOpenFilePicker = showOpenFilePicker;
                window.showSaveFilePicker = showSaveFilePicker;
                window.showDirectoryPicker = showDirectoryPicker;
            }
        }

        // Chrome 133: Basic Web Animations API
        if (typeof Animation === 'undefined') {
            var Animation = function(effect, timeline) {
                this.effect = effect || null;
                this.timeline = timeline || null;
                this.currentTime = 0;
                this.playState = 'idle';
                this.startTime = null;
                this.playbackRate = 1;
            };

            Animation.prototype.play = function() {
                this.playState = 'running';
                this.startTime = Date.now();
            };

            Animation.prototype.pause = function() {
                this.playState = 'paused';
            };

            Animation.prototype.cancel = function() {
                this.playState = 'idle';
                this.currentTime = 0;
                this.startTime = null;
            };

            Animation.prototype.finish = function() {
                this.playState = 'finished';
            };

            Animation.prototype.reverse = function() {
                this.playbackRate *= -1;
            };

            Animation.prototype.addEventListener = function(event, handler) {};
            Animation.prototype.removeEventListener = function(event, handler) {};
        }

        // Chrome 133: Animation.overallProgress property
        if (typeof Animation !== 'undefined' && !('overallProgress' in Animation.prototype)) {
            Object.defineProperty(Animation.prototype, 'overallProgress', {
                get: function() {
                    // Mock implementation - returns normalized progress across all iterations
                    if (this.currentTime === null || this.effect === null || this.effect.getComputedTiming === null) {
                        return null;
                    }
                    var computedTiming = this.effect.getComputedTiming();
                    if (computedTiming.duration === 'auto' || computedTiming.duration === 0) {
                        return 0;
                    }
                    var totalDuration = computedTiming.duration * (computedTiming.iterations || 1);
                    return Math.min(1, Math.max(0, this.currentTime / totalDuration));
                },
                configurable: true
            });
        }

        // Chrome 133: Atomics.pause() method
        if (typeof Atomics !== 'undefined' && typeof Atomics.pause === 'undefined') {
            Atomics.pause = function() {
                // Mock implementation - in real implementation would hint CPU about spinlock
                // In headless mode, this is essentially a no-op
                return undefined;
            };
        }

        // Chrome 133: ClipboardItem constructor with string support
        if (typeof ClipboardItem === 'undefined') {
            var ClipboardItem = function(data, options) {
                this.types = Object.keys(data);
                this._data = {};

                // Convert string values to appropriate format
                for (var type in data) {
                    if (typeof data[type] === 'string') {
                        this._data[type] = Promise.resolve(new Blob([data[type]], {type: type}));
                    } else if (data[type] && typeof data[type].then === 'function') {
                        this._data[type] = data[type];
                    } else {
                        this._data[type] = Promise.resolve(data[type]);
                    }
                }

                this.presentationStyle = (options && options.presentationStyle) || 'unspecified';
            };

            ClipboardItem.prototype.getType = function(type) {
                return this._data[type] || Promise.reject(new Error('Type not found'));
            };
        }

        // Chrome 133: HTMLScriptElement polyfill
        if (typeof HTMLScriptElement === 'undefined') {
            var HTMLScriptElement = function() {
                this.type = '';
                this.src = '';
                this.async = false;
                this.defer = false;
                this.crossOrigin = null;
                this.integrity = '';
                this.noModule = false;
                this.referrerPolicy = '';
            };

            HTMLScriptElement.prototype.addEventListener = function(event, handler) {};
            HTMLScriptElement.prototype.removeEventListener = function(event, handler) {};
        }

        // Chrome 134: HTMLDialogElement with closedby attribute
        if (typeof HTMLDialogElement === 'undefined') {
            var HTMLDialogElement = function() {
                this.tagName = 'DIALOG';
                this.open = false;
                this.returnValue = '';
                this._closedby = 'any'; // Default Chrome 134 behavior
            };

            HTMLDialogElement.prototype.show = function() {
                this.open = true;
            };

            HTMLDialogElement.prototype.showModal = function() {
                this.open = true;
            };

            HTMLDialogElement.prototype.close = function(returnValue) {
                this.open = false;
                if (returnValue !== undefined) {
                    this.returnValue = returnValue;
                }
            };

            HTMLDialogElement.prototype.setAttribute = function(name, value) {
                if (name === 'closedby') {
                    this._closedby = value;
                }
            };

            HTMLDialogElement.prototype.getAttribute = function(name) {
                if (name === 'closedby') {
                    return this._closedby;
                }
                return null;
            };

            HTMLDialogElement.prototype.addEventListener = function(event, handler) {};
            HTMLDialogElement.prototype.removeEventListener = function(event, handler) {};
        }

        // Chrome 134: Web Locks API
        if (typeof navigator !== 'undefined' && typeof navigator.locks === 'undefined') {
            navigator.locks = {
                request: function(name, optionsOrCallback, callback) {
                    // Mock implementation
                    var actualCallback = typeof optionsOrCallback === 'function' ? optionsOrCallback : callback;
                    var options = typeof optionsOrCallback === 'object' ? optionsOrCallback : {};

                    // Simulate async lock acquisition
                    return new Promise(function(resolve) {
                        setTimeout(function() {
                            var result = actualCallback ? actualCallback() : undefined;
                            resolve(result);
                        }, 1);
                    });
                },

                query: function() {
                    return Promise.resolve({
                        pending: [],
                        held: []
                    });
                }
            };
        }

        // Chrome 134: OffscreenCanvas
        if (typeof OffscreenCanvas === 'undefined') {
            var OffscreenCanvas = function(width, height) {
                this.width = width || 300;
                this.height = height || 150;
                this._contexts = {};
            };

            OffscreenCanvas.prototype.getContext = function(contextType, options) {
                if (contextType === '2d') {
                    if (!this._contexts['2d']) {
                        var ctx = {
                            canvas: this,
                            imageSmoothingEnabled: true,
                            imageSmoothingQuality: 'low', // Chrome 134 feature
                            getContextAttributes: function() { // Chrome 134 feature
                                return {
                                    alpha: true,
                                    colorSpace: 'srgb',
                                    desynchronized: false,
                                    willReadFrequently: false
                                };
                            },
                            fillRect: function(x, y, w, h) {},
                            clearRect: function(x, y, w, h) {},
                            strokeRect: function(x, y, w, h) {},
                            beginPath: function() {},
                            closePath: function() {},
                            moveTo: function(x, y) {},
                            lineTo: function(x, y) {},
                            fill: function() {},
                            stroke: function() {}
                        };
                        this._contexts['2d'] = ctx;
                    }
                    return this._contexts['2d'];
                }
                return null;
            };

            OffscreenCanvas.prototype.transferToImageBitmap = function() {
                // Mock ImageBitmap
                return {
                    width: this.width,
                    height: this.height,
                    close: function() {}
                };
            };
        }

        // Chrome 134: Enhanced console.timeStamp
        if (typeof console !== 'undefined') {
            var originalTimeStamp = console.timeStamp;
            if (typeof originalTimeStamp === 'undefined') {
                console.timeStamp = function(label, options) {
                    // Chrome 134: Enhanced timeStamp with options
                    var timestamp = Date.now();
                    var message = label || 'TimeStamp';

                    if (options && options.detail) {
                        message += ': ' + JSON.stringify(options.detail);
                    }

                    console.log('[TimeStamp] ' + message + ' @ ' + timestamp);
                };
            } else {
                // Enhance existing timeStamp
                console.timeStamp = function(label, options) {
                    if (options) {
                        // Chrome 134 enhanced version
                        var message = label || 'TimeStamp';
                        if (options.detail) {
                            message += ': ' + JSON.stringify(options.detail);
                        }
                        originalTimeStamp.call(console, message);
                    } else {
                        // Fallback to original
                        originalTimeStamp.call(console, label);
                    }
                };
            }
        }

        // Chrome 134: Symbol.dispose for Explicit Resource Management
        if (typeof Symbol !== 'undefined' && typeof Symbol.dispose === 'undefined') {
            Symbol.dispose = Symbol('Symbol.dispose');
        }

        if (typeof Symbol !== 'undefined' && typeof Symbol.asyncDispose === 'undefined') {
            Symbol.asyncDispose = Symbol('Symbol.asyncDispose');
        }

        // Chrome 134: Enhanced Canvas context with imageSmoothingQuality
        if (typeof document !== 'undefined') {
            var originalCreateElement = document.createElement;
            if (typeof originalCreateElement === 'function') {
                document.createElement = function(tagName) {
                    var element = originalCreateElement.call(document, tagName);

                    if (tagName.toLowerCase() === 'canvas') {
                        var originalGetContext = element.getContext;
                        if (originalGetContext) {
                            element.getContext = function(contextType, options) {
                                var ctx = originalGetContext.call(element, contextType, options);

                                if (ctx && contextType === '2d') {
                                    // Chrome 134: Add imageSmoothingQuality if missing
                                    if (typeof ctx.imageSmoothingQuality === 'undefined') {
                                        ctx.imageSmoothingQuality = 'low';

                                        // Add property descriptor to make it settable
                                        Object.defineProperty(ctx, 'imageSmoothingQuality', {
                                            value: 'low',
                                            writable: true,
                                            enumerable: true,
                                            configurable: true
                                        });
                                    }
                                }

                                return ctx;
                            };
                        }
                    }

                    return element;
                };
            } else if (typeof document.createElement === 'undefined') {
                // Basic document.createElement polyfill
                document.createElement = function(tagName) {
                    if (tagName.toLowerCase() === 'canvas') {
                        return {
                            tagName: 'CANVAS',
                            width: 300,
                            height: 150,
                            getContext: function(contextType) {
                                if (contextType === '2d') {
                                    return {
                                        canvas: this,
                                        imageSmoothingEnabled: true,
                                        imageSmoothingQuality: 'low', // Chrome 134 feature
                                        fillRect: function(x, y, w, h) {},
                                        clearRect: function(x, y, w, h) {},
                                        strokeRect: function(x, y, w, h) {},
                                        beginPath: function() {},
                                        closePath: function() {},
                                        moveTo: function(x, y) {},
                                        lineTo: function(x, y) {},
                                        fill: function() {},
                                        stroke: function() {}
                                    };
                                }
                                return null;
                            },
                            toDataURL: function() { return 'data:,'; }
                        };
                    } else if (tagName.toLowerCase() === 'dialog') {
                        return new HTMLDialogElement();
                    } else {
                        return {
                            tagName: tagName.toUpperCase(),
                            setAttribute: function(name, value) {},
                            getAttribute: function(name) { return null; },
                            addEventListener: function(event, handler) {},
                            removeEventListener: function(event, handler) {}
                        };
                    }
                };
            }
        }
    "#))?;

    Ok(())
}