use boa_engine::{Context, JsResult, Source};

/// Setup ES2019 polyfills (Array.flat, Array.flatMap, Object.fromEntries, etc.)
/// NOTE: Major ES2019 features (Array.flat/flatMap, String.trimStart/End, Symbol.description)
/// are now natively implemented in the Boa JavaScript engine.
pub fn setup_es2019_polyfills(context: &mut Context) -> JsResult<()> {
    // Pure JavaScript language features now handled natively by Boa:
    // - Array.prototype.flat, flatMap
    // - String.prototype.trimStart, trimEnd
    // - Symbol.prototype.description

    context.eval(Source::from_bytes(r#"

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