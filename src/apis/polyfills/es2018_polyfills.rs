use boa_engine::{Context, JsResult, Source};

/// Setup ES2018 polyfills (async iteration, object spread, Promise.finally, etc.)
/// NOTE: Major ES2018 features (Promise.finally, Object.fromEntries, String.trimStart/End)
/// are now natively implemented in the Boa JavaScript engine.
pub fn setup_es2018_polyfills(context: &mut Context) -> JsResult<()> {
    // Pure JavaScript language features now handled natively by Boa:
    // - Promise.prototype.finally
    // - Object.fromEntries
    // - String.prototype.trimStart, trimEnd

    context.eval(Source::from_bytes(r#"

        // RegExp features (dotAll flag, named capture groups - basic support)
        if (!RegExp.prototype.dotAll) {
            Object.defineProperty(RegExp.prototype, 'dotAll', {
                get: function() {
                    return (this.flags || '').indexOf('s') !== -1;
                }
            });
        }

        // Symbol.asyncIterator
        if (typeof Symbol !== 'undefined' && !Symbol.asyncIterator) {
            Symbol.asyncIterator = Symbol('Symbol.asyncIterator');
        }

        // Basic async iterator support
        if (typeof Symbol !== 'undefined' && Symbol.asyncIterator) {
            // Add basic async iterator support to arrays
            if (!Array.prototype[Symbol.asyncIterator]) {
                Array.prototype[Symbol.asyncIterator] = function() {
                    var index = 0;
                    var arr = this;
                    return {
                        next: function() {
                            return Promise.resolve({
                                value: arr[index++],
                                done: index > arr.length
                            });
                        }
                    };
                };
            }
        }

        // String.prototype.trimStart and trimEnd (ES2019, but commonly expected in ES2018)
        if (!String.prototype.trimStart && String.prototype.trimLeft) {
            String.prototype.trimStart = String.prototype.trimLeft;
        }
        if (!String.prototype.trimEnd && String.prototype.trimRight) {
            String.prototype.trimEnd = String.prototype.trimRight;
        }

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
    "#))?;

    Ok(())
}