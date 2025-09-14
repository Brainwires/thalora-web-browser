use boa_engine::{Context, JsResult, Source};

/// Setup ES2019 polyfills (Array.flat, Array.flatMap, Object.fromEntries, etc.)
pub fn setup_es2019_polyfills(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Array.prototype.flat (ES2019)
        if (!Array.prototype.flat) {
            Array.prototype.flat = function(depth) {
                depth = depth === undefined ? 1 : Number(depth);
                var flattenArray = function(arr, currentDepth) {
                    var result = [];
                    for (var i = 0; i < arr.length; i++) {
                        if (Array.isArray(arr[i]) && currentDepth > 0) {
                            result = result.concat(flattenArray(arr[i], currentDepth - 1));
                        } else {
                            result.push(arr[i]);
                        }
                    }
                    return result;
                };

                return flattenArray(this, depth);
            };
        }

        // Array.prototype.flatMap (ES2019)
        if (!Array.prototype.flatMap) {
            Array.prototype.flatMap = function(callback, thisArg) {
                return this.map(callback, thisArg).flat(1);
            };
        }

        // String.prototype.trimStart and trimEnd (ES2019)
        if (!String.prototype.trimStart) {
            String.prototype.trimStart = function() {
                return this.replace(/^\s+/, '');
            };
        }

        if (!String.prototype.trimEnd) {
            String.prototype.trimEnd = function() {
                return this.replace(/\s+$/, '');
            };
        }

        // Optional catch binding support (syntax transformation needed)
        // This is handled in the preprocessing phase

        // JSON.stringify improvements (well-formed JSON.stringify)
        // Override JSON.stringify to handle lone surrogates
        if (typeof JSON !== 'undefined' && JSON.stringify) {
            var originalStringify = JSON.stringify;
            JSON.stringify = function(value, replacer, space) {
                try {
                    var result = originalStringify.call(this, value, replacer, space);
                    // Handle lone surrogates by replacing them with replacement character
                    if (typeof result === 'string') {
                        return result.replace(/[\uD800-\uDFFF]/g, '\uFFFD');
                    }
                    return result;
                } catch (e) {
                    throw e;
                }
            };
        }

        // Symbol.prototype.description (ES2019)
        if (typeof Symbol !== 'undefined' && !Symbol.prototype.hasOwnProperty('description')) {
            Object.defineProperty(Symbol.prototype, 'description', {
                get: function() {
                    var match = this.toString().match(/^Symbol\((.+)\)$/);
                    return match ? match[1] : undefined;
                },
                configurable: true
            });
        }
    "#))?;

    Ok(())
}