use boa_engine::{Context, JsResult, Source};

/// Setup ES2018 polyfills (async iteration, object spread, Promise.finally, etc.)
pub fn setup_es2018_polyfills(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Promise.prototype.finally (ES2018)
        if (!Promise.prototype.finally) {
            Promise.prototype.finally = function(callback) {
                var constructor = this.constructor || Promise;
                return this.then(
                    function(value) {
                        return constructor.resolve(callback()).then(function() {
                            return value;
                        });
                    },
                    function(reason) {
                        return constructor.resolve(callback()).then(function() {
                            throw reason;
                        });
                    }
                );
            };
        }

        // Object.fromEntries (ES2019, but commonly polyfilled in ES2018 environments)
        if (!Object.fromEntries) {
            Object.fromEntries = function(iterable) {
                var obj = {};
                if (iterable && typeof iterable[Symbol.iterator] === 'function') {
                    var iterator = iterable[Symbol.iterator]();
                    var step;
                    while (!(step = iterator.next()).done) {
                        var entry = step.value;
                        if (entry && typeof entry === 'object' && entry.length >= 2) {
                            obj[entry[0]] = entry[1];
                        }
                    }
                } else if (iterable && iterable.length !== undefined) {
                    // Handle array-like objects
                    for (var i = 0; i < iterable.length; i++) {
                        var entry = iterable[i];
                        if (entry && typeof entry === 'object' && entry.length >= 2) {
                            obj[entry[0]] = entry[1];
                        }
                    }
                }
                return obj;
            };
        }

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