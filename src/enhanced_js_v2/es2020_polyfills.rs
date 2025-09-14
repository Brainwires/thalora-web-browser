use boa_engine::{Context, JsResult, Source};

/// Setup ES2020 polyfills (BigInt, nullish coalescing, optional chaining, etc.)
pub fn setup_es2020_polyfills(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // String.prototype.matchAll (ES2020)
        if (!String.prototype.matchAll) {
            String.prototype.matchAll = function(regexp) {
                if (!regexp) {
                    throw new TypeError('matchAll requires a RegExp');
                }

                if (!regexp.global) {
                    throw new TypeError('matchAll requires a global RegExp');
                }

                var matches = [];
                var str = this;
                var match;

                // Reset lastIndex for global regexes
                regexp.lastIndex = 0;

                while ((match = regexp.exec(str)) !== null) {
                    matches.push(match);
                    if (match.index === regexp.lastIndex) {
                        regexp.lastIndex++;
                    }
                }

                return {
                    [Symbol.iterator]: function() {
                        var index = 0;
                        return {
                            next: function() {
                                if (index < matches.length) {
                                    return { value: matches[index++], done: false };
                                }
                                return { done: true };
                            }
                        };
                    }
                };
            };
        }

        // Promise.allSettled (ES2020)
        if (!Promise.allSettled) {
            Promise.allSettled = function(promises) {
                return Promise.all(promises.map(function(promise) {
                    return Promise.resolve(promise).then(
                        function(value) {
                            return { status: 'fulfilled', value: value };
                        },
                        function(reason) {
                            return { status: 'rejected', reason: reason };
                        }
                    );
                }));
            };
        }

        // globalThis (ES2020)
        if (typeof globalThis === 'undefined') {
            (function() {
                if (typeof self !== 'undefined') {
                    self.globalThis = self;
                } else if (typeof window !== 'undefined') {
                    window.globalThis = window;
                } else if (typeof global !== 'undefined') {
                    global.globalThis = global;
                } else {
                    // Create a minimal global object
                    this.globalThis = this;
                }
            })();
        }

        // BigInt basic support (simplified - real BigInt requires native support)
        if (typeof BigInt === 'undefined') {
            var BigInt = function(value) {
                if (typeof value === 'number' && value % 1 === 0) {
                    this.value = value;
                } else if (typeof value === 'string') {
                    this.value = parseInt(value, 10);
                } else {
                    throw new TypeError('Cannot convert ' + typeof value + ' to a BigInt');
                }

                this.toString = function() {
                    return this.value + 'n';
                };

                this.valueOf = function() {
                    return this.value;
                };
            };

            BigInt.asIntN = function(bits, bigint) {
                return new BigInt(bigint.value);
            };

            BigInt.asUintN = function(bits, bigint) {
                return new BigInt(Math.abs(bigint.value));
            };
        }

        // Dynamic imports (basic support - actual module loading would need runtime support)
        if (!global.import && typeof global !== 'undefined') {
            global.import = function(specifier) {
                return Promise.reject(new Error('Dynamic import not supported in this environment'));
            };
        }

        // Nullish coalescing and optional chaining are syntax transformations
        // handled in the preprocessing phase

        // for-in enumeration order improvements (spec compliance)
        // This is handled by the engine implementation

        // import.meta (basic support)
        if (typeof global !== 'undefined' && !global.import) {
            global.import = function() {};
            global.import.meta = {
                url: 'file://unknown'
            };
        }
    "#))?;

    Ok(())
}