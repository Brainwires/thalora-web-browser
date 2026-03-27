use thalora_browser_apis::boa_engine::{Context, JsResult, Source};

/// Setup ES2025+ experimental features and Stage 3 proposals
pub fn setup_es2025_experimental(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Array.fromAsync (Stage 3)
        if (!Array.fromAsync) {
            Array.fromAsync = async function(asyncIterable, mapfn, thisArg) {
                var result = [];
                var index = 0;

                if (asyncIterable && typeof asyncIterable[Symbol.asyncIterator] === 'function') {
                    var iterator = asyncIterable[Symbol.asyncIterator]();
                    var step;

                    while (!(step = await iterator.next()).done) {
                        var value = step.value;
                        if (mapfn) {
                            value = await mapfn.call(thisArg, value, index);
                        }
                        result.push(value);
                        index++;
                    }
                } else if (asyncIterable && typeof asyncIterable[Symbol.iterator] === 'function') {
                    // Handle sync iterables
                    for (var value of asyncIterable) {
                        if (mapfn) {
                            value = await mapfn.call(thisArg, value, index);
                        }
                        result.push(value);
                        index++;
                    }
                }

                return result;
            };
        }

        // Decorator metadata (Stage 3)
        if (typeof Symbol !== 'undefined' && !Symbol.metadata) {
            Symbol.metadata = Symbol('Symbol.metadata');
        }

        // Records and Tuples (Stage 2) - simplified implementation
        if (typeof Record === 'undefined') {
            var Record = function(obj) {
                if (obj === null || typeof obj !== 'object') {
                    throw new TypeError('Record() requires an object');
                }

                var record = Object.freeze(Object.assign({}, obj));
                record.__proto__ = Record.prototype;
                return record;
            };

            Record.prototype = Object.create(Object.prototype);
            Record.prototype.constructor = Record;

            Record.prototype.toString = function() {
                // Safe toString without potential recursion
                var props = [];
                for (var key in this) {
                    if (this.hasOwnProperty(key)) {
                        var value = this[key];
                        var valueStr = (typeof value === 'string') ? '"' + value + '"' : String(value);
                        props.push('"' + key + '": ' + valueStr);
                    }
                }
                return 'Record({ ' + props.join(', ') + ' })';
            };
        }

        if (typeof Tuple === 'undefined') {
            var Tuple = function() {
                var items = Array.prototype.slice.call(arguments);
                var tuple = Object.freeze(items.slice());
                tuple.__proto__ = Tuple.prototype;
                return tuple;
            };

            Tuple.prototype = Object.create(Array.prototype);
            Tuple.prototype.constructor = Tuple;

            Tuple.prototype.toString = function() {
                return 'Tuple(' + Array.prototype.join.call(this, ', ') + ')';
            };
        }

        // Pattern matching (Stage 1) - basic implementation
        if (typeof match === 'undefined') {
            var match = function(value) {
                return {
                    with: function(pattern, handler) {
                        // Simplified pattern matching with safe object comparison
                        if (typeof pattern === 'function') {
                            if (pattern(value)) {
                                return { result: handler(value), matched: true };
                            }
                        } else if (pattern === value) {
                            return { result: handler(value), matched: true };
                        } else if (typeof pattern === 'object' && pattern !== null && typeof value === 'object' && value !== null) {
                            // Safe shallow comparison instead of JSON.stringify to avoid recursion
                            var matched = true;
                            for (var key in pattern) {
                                if (pattern.hasOwnProperty(key) && pattern[key] !== value[key]) {
                                    matched = false;
                                    break;
                                }
                            }
                            if (matched) {
                                return { result: handler(value), matched: true };
                            }
                        }
                        return { result: undefined, matched: false };
                    },
                    otherwise: function(handler) {
                        return { result: handler(value), matched: true };
                    }
                };
            };
        }

        // Pipeline operator (Stage 2) - function form
        if (typeof pipe === 'undefined') {
            var pipe = function(value) {
                var fns = Array.prototype.slice.call(arguments, 1);
                return fns.reduce(function(acc, fn) {
                    return fn(acc);
                }, value);
            };
        }

        // Partial application (Stage 1)
        if (!Function.prototype.partial) {
            Function.prototype.partial = function() {
                var fn = this;
                var partialArgs = Array.prototype.slice.call(arguments);

                return function() {
                    var remainingArgs = Array.prototype.slice.call(arguments);
                    var allArgs = [];
                    var remainingIndex = 0;

                    for (var i = 0; i < partialArgs.length; i++) {
                        if (partialArgs[i] === undefined) {
                            allArgs.push(remainingArgs[remainingIndex++]);
                        } else {
                            allArgs.push(partialArgs[i]);
                        }
                    }

                    // Add any remaining arguments
                    while (remainingIndex < remainingArgs.length) {
                        allArgs.push(remainingArgs[remainingIndex++]);
                    }

                    return fn.apply(this, allArgs);
                };
            };
        }

        // Import assertions (Stage 3) - syntax handled in preprocessing
        // This is a syntax feature handled by the module system

        // JSON modules (Stage 3) - syntax handled in preprocessing
        // This is a syntax feature handled by the module system

        // Explicit resource management (Stage 3)
        if (typeof Symbol !== 'undefined') {
            if (!Symbol.dispose) {
                Symbol.dispose = Symbol('Symbol.dispose');
            }
            if (!Symbol.asyncDispose) {
                Symbol.asyncDispose = Symbol('Symbol.asyncDispose');
            }
        }

        // using declarations (Stage 3) - now a native keyword in Boa, no polyfill needed

        // String.prototype.dedent (Stage 2)
        if (!String.prototype.dedent) {
            String.prototype.dedent = function() {
                var lines = this.split('\n');
                if (lines.length === 0) return this;

                // Remove leading/trailing empty lines
                while (lines.length > 0 && lines[0].trim() === '') {
                    lines.shift();
                }
                while (lines.length > 0 && lines[lines.length - 1].trim() === '') {
                    lines.pop();
                }

                if (lines.length === 0) return '';

                // Find minimum indentation
                var minIndent = Infinity;
                for (var i = 0; i < lines.length; i++) {
                    var line = lines[i];
                    if (line.trim() !== '') {
                        var indent = line.match(/^(\s*)/)[1].length;
                        minIndent = Math.min(minIndent, indent);
                    }
                }

                // Remove common indentation
                if (minIndent < Infinity && minIndent > 0) {
                    for (var i = 0; i < lines.length; i++) {
                        lines[i] = lines[i].substring(minIndent);
                    }
                }

                return lines.join('\n');
            };
        }

        // Math.sumPrecise (Stage 1)
        if (!Math.sumPrecise) {
            Math.sumPrecise = function(values) {
                // Kahan summation algorithm for better precision
                var sum = 0;
                var c = 0; // Compensation for lost low-order bits

                for (var i = 0; i < values.length; i++) {
                    var y = values[i] - c;
                    var t = sum + y;
                    c = (t - sum) - y;
                    sum = t;
                }

                return sum;
            };
        }

        // Observable (Stage 1) - basic implementation
        if (typeof Observable === 'undefined') {
            var Observable = function(subscriber) {
                this._subscriber = subscriber;
            };

            Observable.prototype.subscribe = function(observer) {
                var subscription = { unsubscribed: false };

                var unsubscribe = this._subscriber({
                    next: function(value) {
                        if (!subscription.unsubscribed && observer.next) {
                            observer.next(value);
                        }
                    },
                    error: function(error) {
                        if (!subscription.unsubscribed && observer.error) {
                            observer.error(error);
                        }
                    },
                    complete: function() {
                        if (!subscription.unsubscribed && observer.complete) {
                            observer.complete();
                        }
                    }
                });

                return {
                    unsubscribe: function() {
                        subscription.unsubscribed = true;
                        if (typeof unsubscribe === 'function') {
                            unsubscribe();
                        }
                    }
                };
            };

            Observable.of = function() {
                var values = Array.prototype.slice.call(arguments);
                return new Observable(function(observer) {
                    for (var i = 0; i < values.length; i++) {
                        observer.next(values[i]);
                    }
                    observer.complete();
                });
            };
        }

        // AsyncContext (Stage 2) - basic implementation
        if (typeof AsyncContext === 'undefined') {
            var AsyncContext = function(name) {
                this.name = name || 'AsyncContext';
                this._storage = new Map();
            };

            AsyncContext.prototype.run = function(value, callback) {
                var previousValue = this._storage.get('current');
                this._storage.set('current', value);

                try {
                    return callback();
                } finally {
                    if (previousValue !== undefined) {
                        this._storage.set('current', previousValue);
                    } else {
                        this._storage.delete('current');
                    }
                }
            };

            AsyncContext.prototype.get = function() {
                return this._storage.get('current');
            };
        }

        // Error.isError (Stage 1)
        if (!Error.isError) {
            Error.isError = function(value) {
                return value instanceof Error ||
                       (typeof value === 'object' && value !== null &&
                        typeof value.name === 'string' &&
                        typeof value.message === 'string');
            };
        }

    "#))?;

    Ok(())
}
