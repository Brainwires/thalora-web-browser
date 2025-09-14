use boa_engine::{Context, JsResult, Source};

/// Setup ES2017 polyfills (async/await, Object.entries, Object.values, etc.)
pub fn setup_es2017_polyfills(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Object.getOwnPropertyDescriptors (ES2017)
        if (!Object.getOwnPropertyDescriptors) {
            Object.getOwnPropertyDescriptors = function(obj) {
                var descriptors = {};
                var propertyNames = Object.getOwnPropertyNames(obj);
                for (var i = 0; i < propertyNames.length; i++) {
                    var propertyName = propertyNames[i];
                    descriptors[propertyName] = Object.getOwnPropertyDescriptor(obj, propertyName);
                }
                return descriptors;
            };
        }

        // String.prototype.padStart (ES2017)
        if (!String.prototype.padStart) {
            String.prototype.padStart = function(targetLength, padString) {
                targetLength = targetLength >> 0; // truncate if number, or NaN becomes 0
                padString = String(padString !== undefined ? padString : ' ');
                if (this.length >= targetLength) {
                    return String(this);
                } else {
                    targetLength = targetLength - this.length;
                    if (targetLength > padString.length) {
                        padString += padString.repeat(targetLength / padString.length); // append to original to ensure we are longer than needed
                    }
                    return padString.slice(0, targetLength) + String(this);
                }
            };
        }

        // String.prototype.padEnd (ES2017)
        if (!String.prototype.padEnd) {
            String.prototype.padEnd = function(targetLength, padString) {
                targetLength = targetLength >> 0; // floor if number or convert non-number to 0;
                padString = String(padString !== undefined ? padString : ' ');
                if (this.length > targetLength) {
                    return String(this);
                } else {
                    targetLength = targetLength - this.length;
                    if (targetLength > padString.length) {
                        padString += padString.repeat(targetLength / padString.length);
                    }
                    return String(this) + padString.slice(0, targetLength);
                }
            };
        }

        // SharedArrayBuffer polyfill (basic)
        if (typeof SharedArrayBuffer === 'undefined') {
            var SharedArrayBuffer = function(length) {
                this.byteLength = length;
                this._buffer = new ArrayBuffer(length);

                this.slice = function(begin, end) {
                    return this._buffer.slice(begin, end);
                };
            };
        }

        // Atomics polyfill (basic mock)
        if (typeof Atomics === 'undefined') {
            var Atomics = {
                add: function(typedArray, index, value) {
                    var result = typedArray[index];
                    typedArray[index] += value;
                    return result;
                },
                and: function(typedArray, index, value) {
                    var result = typedArray[index];
                    typedArray[index] &= value;
                    return result;
                },
                compareExchange: function(typedArray, index, expectedValue, replacementValue) {
                    var result = typedArray[index];
                    if (result === expectedValue) {
                        typedArray[index] = replacementValue;
                    }
                    return result;
                },
                exchange: function(typedArray, index, value) {
                    var result = typedArray[index];
                    typedArray[index] = value;
                    return result;
                },
                load: function(typedArray, index) {
                    return typedArray[index];
                },
                or: function(typedArray, index, value) {
                    var result = typedArray[index];
                    typedArray[index] |= value;
                    return result;
                },
                store: function(typedArray, index, value) {
                    typedArray[index] = value;
                    return value;
                },
                sub: function(typedArray, index, value) {
                    var result = typedArray[index];
                    typedArray[index] -= value;
                    return result;
                },
                wait: function(typedArray, index, value, timeout) {
                    return 'not-equal'; // simplified
                },
                wake: function(typedArray, index, count) {
                    return 0; // simplified
                },
                xor: function(typedArray, index, value) {
                    var result = typedArray[index];
                    typedArray[index] ^= value;
                    return result;
                }
            };
        }
    "#))?;

    Ok(())
}