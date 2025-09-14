use boa_engine::{Context, JsResult, Source};

/// Setup ES2023 polyfills (Array methods, Hashbang, WeakMap methods, etc.)
pub fn setup_es2023_polyfills(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Array.prototype.findLast and findLastIndex (ES2023)
        if (!Array.prototype.findLast) {
            Array.prototype.findLast = function(predicate, thisArg) {
                if (typeof predicate !== 'function') {
                    throw new TypeError('predicate must be a function');
                }

                for (var i = this.length - 1; i >= 0; i--) {
                    if (i in this) {
                        var value = this[i];
                        if (predicate.call(thisArg, value, i, this)) {
                            return value;
                        }
                    }
                }
                return undefined;
            };
        }

        if (!Array.prototype.findLastIndex) {
            Array.prototype.findLastIndex = function(predicate, thisArg) {
                if (typeof predicate !== 'function') {
                    throw new TypeError('predicate must be a function');
                }

                for (var i = this.length - 1; i >= 0; i--) {
                    if (i in this) {
                        var value = this[i];
                        if (predicate.call(thisArg, value, i, this)) {
                            return i;
                        }
                    }
                }
                return -1;
            };
        }

        // Array.prototype.toReversed (ES2023)
        if (!Array.prototype.toReversed) {
            Array.prototype.toReversed = function() {
                return this.slice().reverse();
            };
        }

        // Array.prototype.toSorted (ES2023)
        if (!Array.prototype.toSorted) {
            Array.prototype.toSorted = function(compareFn) {
                return this.slice().sort(compareFn);
            };
        }

        // Array.prototype.toSpliced (ES2023)
        if (!Array.prototype.toSpliced) {
            Array.prototype.toSpliced = function(start, deleteCount) {
                var result = this.slice();
                var args = Array.prototype.slice.call(arguments);
                args.unshift(result);
                Array.prototype.splice.apply(null, args);
                return result;
            };
        }

        // Array.prototype.with (ES2023)
        if (!Array.prototype.with) {
            Array.prototype.with = function(index, value) {
                var len = this.length;
                var relativeIndex = Math.floor(index) || 0;
                var actualIndex = relativeIndex >= 0 ? relativeIndex : len + relativeIndex;

                if (actualIndex < 0 || actualIndex >= len) {
                    throw new RangeError('Invalid index');
                }

                var result = this.slice();
                result[actualIndex] = value;
                return result;
            };
        }

        // TypedArray methods (ES2023) - for regular arrays as fallback
        if (!Array.prototype.toReversed) {
            // Already defined above
        }

        // WeakMap.prototype.emplace (ES2024 proposal, but implementing early)
        if (typeof WeakMap !== 'undefined' && !WeakMap.prototype.emplace) {
            WeakMap.prototype.emplace = function(key, handlers) {
                if (this.has(key)) {
                    var value = this.get(key);
                    if (handlers.update) {
                        value = handlers.update(value, key, this);
                        this.set(key, value);
                    }
                    return value;
                } else {
                    var value = handlers.insert ? handlers.insert(key, this) : undefined;
                    this.set(key, value);
                    return value;
                }
            };
        }

        // Map.prototype.emplace (ES2024 proposal)
        if (typeof Map !== 'undefined' && !Map.prototype.emplace) {
            Map.prototype.emplace = function(key, handlers) {
                if (this.has(key)) {
                    var value = this.get(key);
                    if (handlers.update) {
                        value = handlers.update(value, key, this);
                        this.set(key, value);
                    }
                    return value;
                } else {
                    var value = handlers.insert ? handlers.insert(key, this) : undefined;
                    this.set(key, value);
                    return value;
                }
            };
        }

        // Set methods (ES2023/2024)
        if (typeof Set !== 'undefined') {
            // Set.prototype.intersection
            if (!Set.prototype.intersection) {
                Set.prototype.intersection = function(other) {
                    var result = new Set();
                    var iterable = other;

                    if (typeof other[Symbol.iterator] !== 'function') {
                        throw new TypeError('other must be iterable');
                    }

                    for (var value of this) {
                        if (other.has ? other.has(value) : Array.prototype.includes.call(other, value)) {
                            result.add(value);
                        }
                    }
                    return result;
                };
            }

            // Set.prototype.union
            if (!Set.prototype.union) {
                Set.prototype.union = function(other) {
                    var result = new Set(this);

                    if (typeof other[Symbol.iterator] !== 'function') {
                        throw new TypeError('other must be iterable');
                    }

                    for (var value of other) {
                        result.add(value);
                    }
                    return result;
                };
            }

            // Set.prototype.difference
            if (!Set.prototype.difference) {
                Set.prototype.difference = function(other) {
                    var result = new Set();

                    if (typeof other[Symbol.iterator] !== 'function') {
                        throw new TypeError('other must be iterable');
                    }

                    for (var value of this) {
                        if (!(other.has ? other.has(value) : Array.prototype.includes.call(other, value))) {
                            result.add(value);
                        }
                    }
                    return result;
                };
            }

            // Set.prototype.symmetricDifference
            if (!Set.prototype.symmetricDifference) {
                Set.prototype.symmetricDifference = function(other) {
                    var result = new Set();

                    if (typeof other[Symbol.iterator] !== 'function') {
                        throw new TypeError('other must be iterable');
                    }

                    for (var value of this) {
                        if (!(other.has ? other.has(value) : Array.prototype.includes.call(other, value))) {
                            result.add(value);
                        }
                    }

                    for (var value of other) {
                        if (!this.has(value)) {
                            result.add(value);
                        }
                    }

                    return result;
                };
            }

            // Set.prototype.isSubsetOf
            if (!Set.prototype.isSubsetOf) {
                Set.prototype.isSubsetOf = function(other) {
                    if (typeof other[Symbol.iterator] !== 'function') {
                        throw new TypeError('other must be iterable');
                    }

                    for (var value of this) {
                        if (!(other.has ? other.has(value) : Array.prototype.includes.call(other, value))) {
                            return false;
                        }
                    }
                    return true;
                };
            }

            // Set.prototype.isSupersetOf
            if (!Set.prototype.isSupersetOf) {
                Set.prototype.isSupersetOf = function(other) {
                    if (typeof other[Symbol.iterator] !== 'function') {
                        throw new TypeError('other must be iterable');
                    }

                    for (var value of other) {
                        if (!this.has(value)) {
                            return false;
                        }
                    }
                    return true;
                };
            }

            // Set.prototype.isDisjointFrom
            if (!Set.prototype.isDisjointFrom) {
                Set.prototype.isDisjointFrom = function(other) {
                    if (typeof other[Symbol.iterator] !== 'function') {
                        throw new TypeError('other must be iterable');
                    }

                    for (var value of other) {
                        if (this.has(value)) {
                            return false;
                        }
                    }
                    return true;
                };
            }
        }

        // Hashbang comments support (handled in preprocessing)
        // This is a syntax feature that's handled by the parser

        // Symbols as WeakMap keys (engine-level feature)
        // This is handled by the engine implementation

        // Array grouping (Stage 3, likely ES2024)
        if (!Array.prototype.group) {
            Array.prototype.group = function(keySelector) {
                var groups = {};
                for (var i = 0; i < this.length; i++) {
                    if (i in this) {
                        var element = this[i];
                        var key = keySelector(element, i, this);
                        var keyString = String(key);

                        if (!groups[keyString]) {
                            groups[keyString] = [];
                        }
                        groups[keyString].push(element);
                    }
                }
                return groups;
            };
        }

        if (!Array.prototype.groupToMap) {
            Array.prototype.groupToMap = function(keySelector) {
                var map = new Map();
                for (var i = 0; i < this.length; i++) {
                    if (i in this) {
                        var element = this[i];
                        var key = keySelector(element, i, this);

                        if (!map.has(key)) {
                            map.set(key, []);
                        }
                        map.get(key).push(element);
                    }
                }
                return map;
            };
        }
    "#))?;

    Ok(())
}