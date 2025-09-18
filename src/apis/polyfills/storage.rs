use boa_engine::{Context, JsResult, Source};

/// Storage and IndexedDB API polyfills
///
/// ⚠️ WARNING: These are MOCK implementations for compatibility testing!
/// They provide API shape compatibility but NOT real storage functionality.
pub fn setup_storage_apis(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Web Locks API (Chrome 134)
        if (typeof navigator !== 'undefined' && typeof navigator.locks === 'undefined') {
            Object.defineProperty(navigator, 'locks', {
                value: {
                    request: function(name, options, callback) {
                        // Handle both (name, callback) and (name, options, callback) signatures
                        if (typeof options === 'function') {
                            callback = options;
                            options = {};
                        }

                        // MOCK - Real implementation would acquire lock
                        console.log('Web Lock requested:', name, 'options:', options);

                        var mockLock = {
                            name: name,
                            mode: options.mode || 'exclusive' // 'exclusive' or 'shared'
                        };

                        return new Promise(function(resolve, reject) {
                            setTimeout(function() {
                                try {
                                    var result = callback(mockLock);
                                    if (result && typeof result.then === 'function') {
                                        result.then(resolve).catch(reject);
                                    } else {
                                        resolve(result);
                                    }
                                } catch (error) {
                                    reject(error);
                                }
                            }, 10);
                        });
                    },
                    query: function() {
                        // MOCK - Real implementation would return current locks state
                        return Promise.resolve({
                            held: [],
                            pending: []
                        });
                    }
                },
                writable: false,
                enumerable: true,
                configurable: false
            });
        }

        // IndexedDB API (Chrome 130)
        if (typeof indexedDB === 'undefined') {
            var indexedDB = {
                open: function(name, version) {
                    // MOCK - Real implementation would open database
                    console.log('IndexedDB open:', name, 'version:', version);

                    var request = {
                        result: null,
                        error: null,
                        readyState: 'pending',
                        onsuccess: null,
                        onerror: null,
                        onupgradeneeded: null,
                        onblocked: null,

                        addEventListener: function(type, listener) {
                            this['on' + type] = listener;
                        },

                        removeEventListener: function(type, listener) {
                            if (this['on' + type] === listener) {
                                this['on' + type] = null;
                            }
                        }
                    };

                    // Simulate async database opening
                    setTimeout(function() {
                        var mockDB = {
                            name: name,
                            version: version || 1,
                            objectStoreNames: [],

                            createObjectStore: function(name, options) {
                                console.log('Creating object store:', name, options);
                                this.objectStoreNames.push(name);
                                return {
                                    name: name,
                                    keyPath: (options && options.keyPath) || null,
                                    autoIncrement: (options && options.autoIncrement) || false,

                                    createIndex: function(name, keyPath, options) {
                                        console.log('Creating index:', name, keyPath, options);
                                        return { name: name, keyPath: keyPath };
                                    }
                                };
                            },

                            deleteObjectStore: function(name) {
                                console.log('Deleting object store:', name);
                                var index = this.objectStoreNames.indexOf(name);
                                if (index > -1) {
                                    this.objectStoreNames.splice(index, 1);
                                }
                            },

                            transaction: function(storeNames, mode) {
                                mode = mode || 'readonly';
                                console.log('Creating transaction:', storeNames, mode);

                                return {
                                    mode: mode,
                                    objectStoreNames: Array.isArray(storeNames) ? storeNames : [storeNames],

                                    objectStore: function(name) {
                                        return {
                                            name: name,

                                            add: function(value, key) {
                                                console.log('ObjectStore add:', value, key);
                                                return this._createRequest(key || Date.now());
                                            },

                                            put: function(value, key) {
                                                console.log('ObjectStore put:', value, key);
                                                return this._createRequest(key || Date.now());
                                            },

                                            get: function(key) {
                                                console.log('ObjectStore get:', key);
                                                return this._createRequest(null);
                                            },

                                            delete: function(key) {
                                                console.log('ObjectStore delete:', key);
                                                return this._createRequest(undefined);
                                            },

                                            clear: function() {
                                                console.log('ObjectStore clear');
                                                return this._createRequest(undefined);
                                            },

                                            count: function(key) {
                                                console.log('ObjectStore count:', key);
                                                return this._createRequest(0);
                                            },

                                            getAll: function(query, count) {
                                                console.log('ObjectStore getAll:', query, count);
                                                return this._createRequest([]);
                                            },

                                            getAllKeys: function(query, count) {
                                                console.log('ObjectStore getAllKeys:', query, count);
                                                return this._createRequest([]);
                                            },

                                            _createRequest: function(result) {
                                                var req = {
                                                    result: result,
                                                    error: null,
                                                    readyState: 'pending',
                                                    onsuccess: null,
                                                    onerror: null
                                                };

                                                setTimeout(function() {
                                                    req.readyState = 'done';
                                                    if (req.onsuccess) {
                                                        req.onsuccess({ target: req, type: 'success' });
                                                    }
                                                }, 5);

                                                return req;
                                            }
                                        };
                                    }
                                };
                            },

                            close: function() {
                                console.log('Database closed');
                            }
                        };

                        request.readyState = 'done';
                        request.result = mockDB;

                        if (request.onsuccess) {
                            request.onsuccess({ target: request, type: 'success' });
                        }
                    }, 50);

                    return request;
                },

                deleteDatabase: function(name) {
                    console.log('IndexedDB deleteDatabase:', name);
                    var request = {
                        result: null,
                        error: null,
                        readyState: 'pending',
                        onsuccess: null,
                        onerror: null,
                        onblocked: null
                    };

                    setTimeout(function() {
                        request.readyState = 'done';
                        if (request.onsuccess) {
                            request.onsuccess({ target: request, type: 'success' });
                        }
                    }, 10);

                    return request;
                },

                databases: function() {
                    console.log('IndexedDB databases()');
                    return Promise.resolve([]);
                },

                cmp: function(first, second) {
                    // MOCK - Basic comparison
                    if (first < second) return -1;
                    if (first > second) return 1;
                    return 0;
                }
            };
        }

        // IDBKeyRange (part of IndexedDB)
        if (typeof IDBKeyRange === 'undefined') {
            var IDBKeyRange = {
                bound: function(lower, upper, lowerOpen, upperOpen) {
                    return {
                        lower: lower,
                        upper: upper,
                        lowerOpen: !!lowerOpen,
                        upperOpen: !!upperOpen,
                        includes: function(key) {
                            var lowerValid = this.lowerOpen ? key > this.lower : key >= this.lower;
                            var upperValid = this.upperOpen ? key < this.upper : key <= this.upper;
                            return lowerValid && upperValid;
                        }
                    };
                },

                only: function(value) {
                    return {
                        lower: value,
                        upper: value,
                        lowerOpen: false,
                        upperOpen: false,
                        includes: function(key) {
                            return key === value;
                        }
                    };
                },

                lowerBound: function(bound, open) {
                    return {
                        lower: bound,
                        upper: undefined,
                        lowerOpen: !!open,
                        upperOpen: false,
                        includes: function(key) {
                            return this.lowerOpen ? key > this.lower : key >= this.lower;
                        }
                    };
                },

                upperBound: function(bound, open) {
                    return {
                        lower: undefined,
                        upper: bound,
                        lowerOpen: false,
                        upperOpen: !!open,
                        includes: function(key) {
                            return this.upperOpen ? key < this.upper : key <= this.upper;
                        }
                    };
                }
            };
        }
    "#))?;

    Ok(())
}