use boa_engine::{Context, JsResult, Source};

/// Setup ES6+ polyfills for compatibility
pub fn setup_es6_polyfills(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // ES6+ Array methods if not available
        if (!Array.prototype.find) {
            Array.prototype.find = function(predicate, thisArg) {
                for (var i = 0; i < this.length; i++) {
                    if (predicate.call(thisArg, this[i], i, this)) {
                        return this[i];
                    }
                }
                return undefined;
            };
        }

        if (!Array.prototype.includes) {
            Array.prototype.includes = function(searchElement, fromIndex) {
                return this.indexOf(searchElement, fromIndex || 0) !== -1;
            };
        }

        if (!Array.prototype.findIndex) {
            Array.prototype.findIndex = function(predicate, thisArg) {
                for (var i = 0; i < this.length; i++) {
                    if (predicate.call(thisArg, this[i], i, this)) {
                        return i;
                    }
                }
                return -1;
            };
        }

        if (!Array.prototype.filter) {
            Array.prototype.filter = function(callback, thisArg) {
                var filtered = [];
                for (var i = 0; i < this.length; i++) {
                    if (callback.call(thisArg, this[i], i, this)) {
                        filtered.push(this[i]);
                    }
                }
                return filtered;
            };
        }

        if (!Array.prototype.map) {
            Array.prototype.map = function(callback, thisArg) {
                var mapped = [];
                for (var i = 0; i < this.length; i++) {
                    mapped.push(callback.call(thisArg, this[i], i, this));
                }
                return mapped;
            };
        }

        if (!Array.prototype.reduce) {
            Array.prototype.reduce = function(callback, initialValue) {
                var accumulator = arguments.length > 1 ? initialValue : this[0];
                var startIndex = arguments.length > 1 ? 0 : 1;

                for (var i = startIndex; i < this.length; i++) {
                    accumulator = callback(accumulator, this[i], i, this);
                }
                return accumulator;
            };
        }

        if (!Array.prototype.forEach) {
            Array.prototype.forEach = function(callback, thisArg) {
                for (var i = 0; i < this.length; i++) {
                    callback.call(thisArg, this[i], i, this);
                }
            };
        }

        if (!Array.isArray) {
            Array.isArray = function(obj) {
                return Object.prototype.toString.call(obj) === '[object Array]';
            };
        }

        // ES6+ Object methods
        if (!Object.assign) {
            Object.assign = function(target) {
                var sources = Array.prototype.slice.call(arguments, 1);
                for (var i = 0; i < sources.length; i++) {
                    var source = sources[i];
                    if (source) {
                        for (var key in source) {
                            if (source.hasOwnProperty(key)) {
                                target[key] = source[key];
                            }
                        }
                    }
                }
                return target;
            };
        }

        if (!Object.keys) {
            Object.keys = function(obj) {
                var keys = [];
                for (var key in obj) {
                    if (obj.hasOwnProperty(key)) {
                        keys.push(key);
                    }
                }
                return keys;
            };
        }

        if (!Object.values) {
            Object.values = function(obj) {
                var values = [];
                for (var key in obj) {
                    if (obj.hasOwnProperty(key)) {
                        values.push(obj[key]);
                    }
                }
                return values;
            };
        }

        if (!Object.entries) {
            Object.entries = function(obj) {
                var entries = [];
                for (var key in obj) {
                    if (obj.hasOwnProperty(key)) {
                        entries.push([key, obj[key]]);
                    }
                }
                return entries;
            };
        }

        // ES6+ String methods
        if (!String.prototype.includes) {
            String.prototype.includes = function(search, start) {
                return this.indexOf(search, start || 0) !== -1;
            };
        }

        if (!String.prototype.startsWith) {
            String.prototype.startsWith = function(search, pos) {
                return this.substr(!pos || pos < 0 ? 0 : +pos, search.length) === search;
            };
        }

        if (!String.prototype.endsWith) {
            String.prototype.endsWith = function(search, this_len) {
                if (this_len === undefined || this_len > this.length) {
                    this_len = this.length;
                }
                return this.substring(this_len - search.length, this_len) === search;
            };
        }

        if (!String.prototype.repeat) {
            String.prototype.repeat = function(count) {
                if (count < 0) {
                    throw new RangeError('repeat count must be non-negative');
                }
                if (count === Infinity) {
                    throw new RangeError('repeat count must be less than infinity');
                }
                count = Math.floor(count);
                if (this.length === 0 || count === 0) {
                    return '';
                }
                var result = '';
                for (var i = 0; i < count; i++) {
                    result += this;
                }
                return result;
            };
        }

        // Number methods
        if (!Number.isNaN) {
            Number.isNaN = function(value) {
                return typeof value === 'number' && isNaN(value);
            };
        }

        if (!Number.isFinite) {
            Number.isFinite = function(value) {
                return typeof value === 'number' && isFinite(value);
            };
        }

        if (!Number.isInteger) {
            Number.isInteger = function(value) {
                return typeof value === 'number' &&
                       isFinite(value) &&
                       Math.floor(value) === value;
            };
        }
    "#))?;

    Ok(())
}