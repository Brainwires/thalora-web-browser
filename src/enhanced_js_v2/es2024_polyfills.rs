use boa_engine::{Context, JsResult, Source};

/// Setup ES2024 polyfills (Temporal API, RegExp v flag, Promise.withResolvers, etc.)
pub fn setup_es2024_polyfills(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Promise.withResolvers (ES2024)
        if (!Promise.withResolvers) {
            Promise.withResolvers = function() {
                var resolve, reject;
                var promise = new Promise(function(res, rej) {
                    resolve = res;
                    reject = rej;
                });
                return { promise: promise, resolve: resolve, reject: reject };
            };
        }

        // Object.groupBy (ES2024)
        if (!Object.groupBy) {
            Object.groupBy = function(items, keySelector) {
                var groups = {};
                for (var i = 0; i < items.length; i++) {
                    var item = items[i];
                    var key = keySelector(item, i);
                    var keyString = String(key);

                    if (!groups[keyString]) {
                        groups[keyString] = [];
                    }
                    groups[keyString].push(item);
                }
                return groups;
            };
        }

        // Map.groupBy (ES2024)
        if (!Map.groupBy) {
            Map.groupBy = function(items, keySelector) {
                var map = new Map();
                for (var i = 0; i < items.length; i++) {
                    var item = items[i];
                    var key = keySelector(item, i);

                    if (!map.has(key)) {
                        map.set(key, []);
                    }
                    map.get(key).push(item);
                }
                return map;
            };
        }

        // ArrayBuffer.prototype.resize (ES2024)
        if (typeof ArrayBuffer !== 'undefined' && !ArrayBuffer.prototype.resize) {
            ArrayBuffer.prototype.resize = function(newLength) {
                // This is a simplified polyfill - real resizable ArrayBuffers require engine support
                if (newLength < 0 || newLength > this.maxByteLength) {
                    throw new RangeError('Invalid ArrayBuffer length');
                }
                // In a real implementation, this would actually resize the buffer
                return undefined;
            };
        }

        // ArrayBuffer.prototype.transfer and transferToFixedLength (ES2024)
        if (typeof ArrayBuffer !== 'undefined' && !ArrayBuffer.prototype.transfer) {
            ArrayBuffer.prototype.transfer = function(newLength) {
                newLength = newLength === undefined ? this.byteLength : newLength;
                var newBuffer = new ArrayBuffer(newLength);
                var sourceArray = new Uint8Array(this);
                var targetArray = new Uint8Array(newBuffer);
                var copyLength = Math.min(sourceArray.length, targetArray.length);

                for (var i = 0; i < copyLength; i++) {
                    targetArray[i] = sourceArray[i];
                }

                // Detach the original buffer (simplified)
                return newBuffer;
            };

            ArrayBuffer.prototype.transferToFixedLength = function(newLength) {
                return this.transfer(newLength);
            };
        }

        // SharedArrayBuffer methods (ES2024)
        if (typeof SharedArrayBuffer !== 'undefined') {
            if (!SharedArrayBuffer.prototype.grow) {
                SharedArrayBuffer.prototype.grow = function(newLength) {
                    // Simplified polyfill - real growable SharedArrayBuffers require engine support
                    if (newLength < this.byteLength || newLength > this.maxByteLength) {
                        throw new RangeError('Invalid SharedArrayBuffer length');
                    }
                    return undefined;
                };
            }
        }

        // Atomics.waitAsync (ES2024)
        if (typeof Atomics !== 'undefined' && !Atomics.waitAsync) {
            Atomics.waitAsync = function(typedArray, index, value, timeout) {
                // Simplified async version of Atomics.wait
                return Promise.resolve({
                    async: true,
                    value: Promise.resolve('not-equal') // Simplified result
                });
            };
        }

        // String.prototype.isWellFormed and toWellFormed (ES2024)
        if (!String.prototype.isWellFormed) {
            String.prototype.isWellFormed = function() {
                // Check for lone surrogate characters
                for (var i = 0; i < this.length; i++) {
                    var charCode = this.charCodeAt(i);

                    // High surrogate without low surrogate
                    if (charCode >= 0xD800 && charCode <= 0xDBFF) {
                        if (i + 1 >= this.length) {
                            return false;
                        }
                        var nextCharCode = this.charCodeAt(i + 1);
                        if (nextCharCode < 0xDC00 || nextCharCode > 0xDFFF) {
                            return false;
                        }
                        i++; // Skip the low surrogate
                    }
                    // Low surrogate without high surrogate
                    else if (charCode >= 0xDC00 && charCode <= 0xDFFF) {
                        return false;
                    }
                }
                return true;
            };
        }

        if (!String.prototype.toWellFormed) {
            String.prototype.toWellFormed = function() {
                var result = '';
                for (var i = 0; i < this.length; i++) {
                    var charCode = this.charCodeAt(i);

                    // High surrogate
                    if (charCode >= 0xD800 && charCode <= 0xDBFF) {
                        if (i + 1 < this.length) {
                            var nextCharCode = this.charCodeAt(i + 1);
                            if (nextCharCode >= 0xDC00 && nextCharCode <= 0xDFFF) {
                                // Valid surrogate pair
                                result += this.charAt(i) + this.charAt(i + 1);
                                i++; // Skip the low surrogate
                                continue;
                            }
                        }
                        // Lone high surrogate, replace with replacement character
                        result += '\uFFFD';
                    }
                    // Low surrogate without high surrogate
                    else if (charCode >= 0xDC00 && charCode <= 0xDFFF) {
                        result += '\uFFFD';
                    }
                    else {
                        result += this.charAt(i);
                    }
                }
                return result;
            };
        }

        // RegExp v flag support (ES2024) - basic polyfill
        if (typeof RegExp !== 'undefined') {
            var originalRegExp = RegExp;
            RegExp = function(pattern, flags) {
                if (flags && flags.includes('v')) {
                    // Remove 'v' flag and add 'u' for Unicode support as fallback
                    flags = flags.replace('v', 'u');
                }
                return new originalRegExp(pattern, flags);
            };
            RegExp.prototype = originalRegExp.prototype;

            // Add unicodeSets property
            if (!RegExp.prototype.unicodeSets) {
                Object.defineProperty(RegExp.prototype, 'unicodeSets', {
                    get: function() {
                        return (this.flags || '').indexOf('v') !== -1;
                    }
                });
            }
        }

        // Enhanced Temporal API (ES2024)
        if (typeof Temporal !== 'undefined') {
            // Extend the basic Temporal implementation with ES2024 features
            if (!Temporal.PlainDateTime) {
                Temporal.PlainDateTime = function(year, month, day, hour, minute, second, millisecond, microsecond, nanosecond) {
                    this.year = year || 0;
                    this.month = month || 1;
                    this.day = day || 1;
                    this.hour = hour || 0;
                    this.minute = minute || 0;
                    this.second = second || 0;
                    this.millisecond = millisecond || 0;
                    this.microsecond = microsecond || 0;
                    this.nanosecond = nanosecond || 0;

                    this.toString = function() {
                        return this.year + '-' +
                               String(this.month).padStart(2, '0') + '-' +
                               String(this.day).padStart(2, '0') + 'T' +
                               String(this.hour).padStart(2, '0') + ':' +
                               String(this.minute).padStart(2, '0') + ':' +
                               String(this.second).padStart(2, '0');
                    };

                    this.toPlainDate = function() {
                        return new Temporal.PlainDate(this.year, this.month, this.day);
                    };

                    this.toPlainTime = function() {
                        return new Temporal.PlainTime(this.hour, this.minute, this.second, this.millisecond);
                    };
                };
            }

            if (!Temporal.Duration) {
                Temporal.Duration = function(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds) {
                    this.years = years || 0;
                    this.months = months || 0;
                    this.weeks = weeks || 0;
                    this.days = days || 0;
                    this.hours = hours || 0;
                    this.minutes = minutes || 0;
                    this.seconds = seconds || 0;
                    this.milliseconds = milliseconds || 0;
                    this.microseconds = microseconds || 0;
                    this.nanoseconds = nanoseconds || 0;

                    this.toString = function() {
                        var result = 'P';
                        if (this.years) result += this.years + 'Y';
                        if (this.months) result += this.months + 'M';
                        if (this.weeks) result += this.weeks + 'W';
                        if (this.days) result += this.days + 'D';

                        var timePart = '';
                        if (this.hours) timePart += this.hours + 'H';
                        if (this.minutes) timePart += this.minutes + 'M';
                        if (this.seconds || this.milliseconds) timePart += this.seconds + 'S';

                        if (timePart) result += 'T' + timePart;

                        return result === 'P' ? 'PT0S' : result;
                    };
                };
            }
        }

        // Iterator helpers (Stage 3, likely ES2025)
        if (typeof Iterator === 'undefined') {
            var Iterator = function() {};

            Iterator.prototype.map = function(mapper) {
                var iterator = this;
                return {
                    next: function() {
                        var result = iterator.next();
                        if (result.done) {
                            return result;
                        }
                        return {
                            value: mapper(result.value),
                            done: false
                        };
                    },
                    [Symbol.iterator]: function() { return this; }
                };
            };

            Iterator.prototype.filter = function(predicate) {
                var iterator = this;
                return {
                    next: function() {
                        while (true) {
                            var result = iterator.next();
                            if (result.done) {
                                return result;
                            }
                            if (predicate(result.value)) {
                                return result;
                            }
                        }
                    },
                    [Symbol.iterator]: function() { return this; }
                };
            };

            Iterator.prototype.take = function(limit) {
                var iterator = this;
                var count = 0;
                return {
                    next: function() {
                        if (count >= limit) {
                            return { done: true };
                        }
                        count++;
                        return iterator.next();
                    },
                    [Symbol.iterator]: function() { return this; }
                };
            };

            Iterator.prototype.drop = function(limit) {
                var iterator = this;
                var dropped = 0;
                return {
                    next: function() {
                        while (dropped < limit) {
                            var result = iterator.next();
                            if (result.done) {
                                return result;
                            }
                            dropped++;
                        }
                        return iterator.next();
                    },
                    [Symbol.iterator]: function() { return this; }
                };
            };
        }
    "#))?;

    Ok(())
}