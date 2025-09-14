use boa_engine::{Context, JsResult, Source};

/// Setup ES2021 polyfills (String.prototype.replaceAll, Promise.any, etc.)
pub fn setup_es2021_polyfills(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // String.prototype.replaceAll (ES2021)
        if (!String.prototype.replaceAll) {
            String.prototype.replaceAll = function(searchValue, replaceValue) {
                if (searchValue instanceof RegExp) {
                    if (!searchValue.global) {
                        throw new TypeError('String.prototype.replaceAll called with a non-global RegExp argument');
                    }
                    return this.replace(searchValue, replaceValue);
                }

                var str = this;
                var search = String(searchValue);
                var replace = String(replaceValue);

                if (search === '') {
                    return replace + str.split('').join(replace) + replace;
                }

                var result = '';
                var lastIndex = 0;
                var index = str.indexOf(search, lastIndex);

                while (index !== -1) {
                    result += str.slice(lastIndex, index) + replace;
                    lastIndex = index + search.length;
                    index = str.indexOf(search, lastIndex);
                }

                result += str.slice(lastIndex);
                return result;
            };
        }

        // Promise.any (ES2021)
        if (!Promise.any) {
            Promise.any = function(promises) {
                return new Promise(function(resolve, reject) {
                    var errors = [];
                    var rejected = 0;
                    var total = promises.length;

                    if (total === 0) {
                        reject(new AggregateError([], 'All promises were rejected'));
                        return;
                    }

                    promises.forEach(function(promise, index) {
                        Promise.resolve(promise).then(
                            function(value) {
                                resolve(value);
                            },
                            function(error) {
                                errors[index] = error;
                                rejected++;
                                if (rejected === total) {
                                    reject(new AggregateError(errors, 'All promises were rejected'));
                                }
                            }
                        );
                    });
                });
            };
        }

        // AggregateError (ES2021)
        if (typeof AggregateError === 'undefined') {
            var AggregateError = function(errors, message) {
                var error = new Error(message);
                error.name = 'AggregateError';
                error.errors = errors || [];

                if (Error.captureStackTrace) {
                    Error.captureStackTrace(error, AggregateError);
                }

                return error;
            };

            AggregateError.prototype = Object.create(Error.prototype);
            AggregateError.prototype.constructor = AggregateError;
            AggregateError.prototype.name = 'AggregateError';
        }

        // WeakRef (ES2021) - basic polyfill (without actual weak semantics)
        if (typeof WeakRef === 'undefined') {
            var WeakRef = function(target) {
                if (typeof target !== 'object' || target === null) {
                    throw new TypeError('WeakRef target must be an object');
                }
                this._target = target;
            };

            WeakRef.prototype.deref = function() {
                return this._target; // In real WeakRef, this might return undefined if collected
            };
        }

        // FinalizationRegistry (ES2021) - basic mock
        if (typeof FinalizationRegistry === 'undefined') {
            var FinalizationRegistry = function(cleanupCallback) {
                this._cleanupCallback = cleanupCallback;
                this._registry = new Map();
            };

            FinalizationRegistry.prototype.register = function(target, heldValue, unregisterToken) {
                // Mock implementation - in real version, this would set up cleanup when target is GC'd
                if (unregisterToken) {
                    this._registry.set(unregisterToken, { target: target, heldValue: heldValue });
                }
            };

            FinalizationRegistry.prototype.unregister = function(unregisterToken) {
                return this._registry.delete(unregisterToken);
            };
        }

        // Logical assignment operators are syntax transformations handled in preprocessing

        // Numeric separators are handled during parsing

        // Private methods and accessors are syntax transformations
    "#))?;

    Ok(())
}