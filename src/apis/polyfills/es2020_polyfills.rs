use boa_engine::{Context, JsResult, Source};

/// Setup ES2020 polyfills (BigInt, nullish coalescing, optional chaining, etc.)
pub fn setup_es2020_polyfills(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Create global object if it doesn't exist
        if (typeof global === 'undefined') {
            var global = globalThis;
        }
        // Pure JavaScript language features now handled natively by Boa:
        // - String.prototype.matchAll (ES2020) - implemented in Boa engine
        // - Promise.allSettled (ES2020) - implemented in Boa engine

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
        if (!globalThis.import && typeof global !== 'undefined') {
            globalThis.import = function(specifier) {
                return Promise.reject(new Error('Dynamic import not supported in this environment'));
            };
        }

        // Nullish coalescing and optional chaining are syntax transformations
        // handled in the preprocessing phase

        // for-in enumeration order improvements (spec compliance)
        // This is handled by the engine implementation

        // import.meta (basic support)
        if (typeof global !== 'undefined' && !globalThis.import) {
            globalThis.import = function() {};
            globalThis.import['meta'] = {
                url: 'file://unknown'
            };
        }
    "#))?;

    Ok(())
}