use boa_engine::{Context, JsResult, Source};

/// Setup ES2022 polyfills (Array.at, Object.hasOwn, Error.cause, etc.)
pub fn setup_es2022_polyfills(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Array.prototype.at (ES2022)
        if (!Array.prototype.at) {
            Array.prototype.at = function(index) {
                var len = this.length;
                var relativeIndex = Math.floor(index) || 0;
                var actualIndex = relativeIndex >= 0 ? relativeIndex : len + relativeIndex;

                if (actualIndex >= 0 && actualIndex < len) {
                    return this[actualIndex];
                }
                return undefined;
            };
        }

        // String.prototype.at (ES2022)
        if (!String.prototype.at) {
            String.prototype.at = function(index) {
                var len = this.length;
                var relativeIndex = Math.floor(index) || 0;
                var actualIndex = relativeIndex >= 0 ? relativeIndex : len + relativeIndex;

                if (actualIndex >= 0 && actualIndex < len) {
                    return this[actualIndex];
                }
                return undefined;
            };
        }

        // Object.hasOwn (ES2022)
        if (!Object.hasOwn) {
            Object.hasOwn = function(obj, prop) {
                return Object.prototype.hasOwnProperty.call(obj, prop);
            };
        }

        // Error.cause support (ES2022)
        if (typeof Error !== 'undefined') {
            var OriginalError = Error;
            Error = function(message, options) {
                var error = new OriginalError(message);

                if (options && options.cause !== undefined) {
                    error.cause = options.cause;
                }

                return error;
            };

            // Copy static properties
            Object.setPrototypeOf(Error, OriginalError);
            Error.prototype = OriginalError.prototype;

            // Apply to other Error types
            ['TypeError', 'ReferenceError', 'SyntaxError', 'RangeError', 'EvalError', 'URIError'].forEach(function(ErrorType) {
                if (typeof global[ErrorType] !== 'undefined') {
                    var OriginalErrorType = global[ErrorType];
                    global[ErrorType] = function(message, options) {
                        var error = new OriginalErrorType(message);

                        if (options && options.cause !== undefined) {
                            error.cause = options.cause;
                        }

                        return error;
                    };

                    Object.setPrototypeOf(global[ErrorType], OriginalErrorType);
                    global[ErrorType].prototype = OriginalErrorType.prototype;
                }
            });
        }

        // RegExp match indices (ES2022) - basic support
        if (typeof RegExp !== 'undefined') {
            var originalExec = RegExp.prototype.exec;
            RegExp.prototype.exec = function(string) {
                var result = originalExec.call(this, string);
                if (result && this.hasIndices) {
                    result.indices = [];
                    for (var i = 0; i < result.length; i++) {
                        if (result[i] !== undefined) {
                            var start = string.indexOf(result[i], result.index);
                            result.indices[i] = [start, start + result[i].length];
                        } else {
                            result.indices[i] = undefined;
                        }
                    }
                }
                return result;
            };
        }

        // Top-level await support (requires syntax transformation)
        // This is handled in the preprocessing phase

        // Class fields and private methods (requires syntax transformation)
        // This is handled in the preprocessing phase

        // Static class blocks (requires syntax transformation)
        // This is handled in the preprocessing phase

        // Ergonomic brand checks for private fields (requires syntax transformation)
        // This is handled in the preprocessing phase

        // Array.prototype.findLast and findLastIndex (ES2023, but commonly expected)
        if (!Array.prototype.findLast) {
            Array.prototype.findLast = function(callback, thisArg) {
                for (var i = this.length - 1; i >= 0; i--) {
                    if (callback.call(thisArg, this[i], i, this)) {
                        return this[i];
                    }
                }
                return undefined;
            };
        }

        if (!Array.prototype.findLastIndex) {
            Array.prototype.findLastIndex = function(callback, thisArg) {
                for (var i = this.length - 1; i >= 0; i--) {
                    if (callback.call(thisArg, this[i], i, this)) {
                        return i;
                    }
                }
                return -1;
            };
        }

        // Temporal API placeholder (ES2024 proposal, but worth mentioning)
        if (typeof Temporal === 'undefined') {
            var Temporal = {
                Now: {
                    instant: function() {
                        return {
                            epochNanoseconds: BigInt(Date.now() * 1000000),
                            toString: function() {
                                return new Date().toISOString();
                            }
                        };
                    },
                    timeZone: function() {
                        return Intl.DateTimeFormat().resolvedOptions().timeZone;
                    }
                },
                PlainDate: function(year, month, day) {
                    this.year = year;
                    this.month = month;
                    this.day = day;
                    this.toString = function() {
                        return year + '-' + String(month).padStart(2, '0') + '-' + String(day).padStart(2, '0');
                    };
                },
                PlainTime: function(hour, minute, second, millisecond, microsecond, nanosecond) {
                    this.hour = hour || 0;
                    this.minute = minute || 0;
                    this.second = second || 0;
                    this.millisecond = millisecond || 0;
                    this.toString = function() {
                        return String(this.hour).padStart(2, '0') + ':' +
                               String(this.minute).padStart(2, '0') + ':' +
                               String(this.second).padStart(2, '0');
                    };
                }
            };
        }
    "#))?;

    Ok(())
}