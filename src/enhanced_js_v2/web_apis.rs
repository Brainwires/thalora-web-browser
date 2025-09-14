use boa_engine::{Context, JsResult, Source};

/// Setup Web APIs (fetch, URL, etc.)
pub fn setup_web_apis(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Performance API
        if (typeof performance === 'undefined') {
            var performance = {
                now: function() {
                    return Date.now(); // Fallback to Date.now()
                },
                mark: function(name) {
                    // Mock implementation
                    return undefined;
                },
                measure: function(name, startMark, endMark) {
                    // Mock implementation
                    return undefined;
                }
            };
        }

        // Enhanced Date.now() for consistency
        Date.now = Date.now || function() {
            return new Date().getTime();
        };

        // URL Constructor (basic)
        if (typeof URL === 'undefined') {
            var URL = function(url, base) {
                this.href = url;

                // Basic URL parsing
                var match = url.match(/^(https?:)\/\/([^\/]+)(\/[^?#]*)?(\?[^#]*)?(#.*)?$/);
                if (match) {
                    this.protocol = match[1] || '';
                    this.host = match[2] || '';
                    this.pathname = match[3] || '/';
                    this.search = match[4] || '';
                    this.hash = match[5] || '';
                } else {
                    this.protocol = '';
                    this.host = '';
                    this.pathname = url;
                    this.search = '';
                    this.hash = '';
                }

                this.toString = function() { return this.href; };
            };
        }

        // URLSearchParams
        if (typeof URLSearchParams === 'undefined') {
            var URLSearchParams = function(init) {
                this.params = {};

                if (typeof init === 'string') {
                    var pairs = init.replace(/^\?/, '').split('&');
                    for (var i = 0; i < pairs.length; i++) {
                        var pair = pairs[i].split('=');
                        if (pair.length === 2) {
                            this.params[decodeURIComponent(pair[0])] = decodeURIComponent(pair[1]);
                        }
                    }
                }

                this.get = function(name) {
                    return this.params[name] || null;
                };

                this.set = function(name, value) {
                    this.params[name] = value;
                };

                this.has = function(name) {
                    return this.params.hasOwnProperty(name);
                };

                this.delete = function(name) {
                    delete this.params[name];
                };

                this.toString = function() {
                    var pairs = [];
                    for (var key in this.params) {
                        if (this.params.hasOwnProperty(key)) {
                            pairs.push(encodeURIComponent(key) + '=' + encodeURIComponent(this.params[key]));
                        }
                    }
                    return pairs.join('&');
                };
            };
        }

        // Enhanced JSON with better error handling and recursion protection
        if (typeof JSON === 'undefined') {
            var JSON = {};
        }

        // Safe JSON.stringify implementation with recursion protection
        if (!JSON.stringify || typeof JSON.stringify !== 'function') {
            JSON.stringify = function(obj, replacer, space) {
                var seen = [];

                function stringify(value, depth) {
                    depth = depth || 0;
                    if (depth > 100) return '"[Circular]"'; // Recursion protection

                    if (value === null) return 'null';
                    if (typeof value === 'undefined') return undefined;
                    if (typeof value === 'boolean') return value ? 'true' : 'false';
                    if (typeof value === 'number') return isFinite(value) ? String(value) : 'null';
                    if (typeof value === 'string') return '"' + value.replace(/[\\"\x00-\x1F]/g, function(c) {
                        switch (c) {
                            case '\\': return '\\\\';
                            case '"': return '\\"';
                            case '\n': return '\\n';
                            case '\r': return '\\r';
                            case '\t': return '\\t';
                            default: return '\\u' + ('0000' + c.charCodeAt(0).toString(16)).slice(-4);
                        }
                    }) + '"';

                    if (typeof value === 'object') {
                        // Check for circular reference
                        for (var i = 0; i < seen.length; i++) {
                            if (seen[i] === value) return '"[Circular]"';
                        }
                        seen.push(value);

                        if (Array.isArray(value)) {
                            var result = '[';
                            for (var j = 0; j < value.length; j++) {
                                if (j > 0) result += ',';
                                var item = stringify(value[j], depth + 1);
                                result += (item === undefined) ? 'null' : item;
                            }
                            result += ']';
                            seen.pop();
                            return result;
                        } else {
                            var result = '{';
                            var first = true;
                            for (var key in value) {
                                if (value.hasOwnProperty && value.hasOwnProperty(key)) {
                                    var val = stringify(value[key], depth + 1);
                                    if (val !== undefined) {
                                        if (!first) result += ',';
                                        result += '"' + key + '":' + val;
                                        first = false;
                                    }
                                }
                            }
                            result += '}';
                            seen.pop();
                            return result;
                        }
                    }

                    return undefined;
                }

                return stringify(obj);
            };
        }

        // Safe JSON.parse implementation - completely custom to avoid recursion
        if (!JSON.parse || typeof JSON.parse !== 'function') {
            JSON.parse = function(text) {
                if (typeof text !== 'string') {
                    throw new Error('JSON.parse: argument must be a string');
                }

                text = text.replace(/^\s+|\s+$/g, ''); // trim
                var index = 0;

                function parseValue() {
                    skipWhitespace();
                    if (index >= text.length) throw new Error('Unexpected end of input');

                    var char = text[index];
                    if (char === '"') return parseString();
                    if (char === '{') return parseObject();
                    if (char === '[') return parseArray();
                    if (char === 't' || char === 'f') return parseBoolean();
                    if (char === 'n') return parseNull();
                    if (char === '-' || (char >= '0' && char <= '9')) return parseNumber();

                    throw new Error('Unexpected token: ' + char);
                }

                function parseString() {
                    if (text[index] !== '"') throw new Error('Expected "');
                    index++;
                    var result = '';
                    while (index < text.length && text[index] !== '"') {
                        if (text[index] === '\\') {
                            index++;
                            if (index >= text.length) throw new Error('Incomplete escape sequence');
                            var escaped = text[index];
                            switch (escaped) {
                                case '"': result += '"'; break;
                                case '\\': result += '\\'; break;
                                case '/': result += '/'; break;
                                case 'b': result += '\b'; break;
                                case 'f': result += '\f'; break;
                                case 'n': result += '\n'; break;
                                case 'r': result += '\r'; break;
                                case 't': result += '\t'; break;
                                default: result += escaped; break;
                            }
                        } else {
                            result += text[index];
                        }
                        index++;
                    }
                    if (text[index] !== '"') throw new Error('Unterminated string');
                    index++;
                    return result;
                }

                function parseNumber() {
                    var start = index;
                    if (text[index] === '-') index++;
                    while (index < text.length && text[index] >= '0' && text[index] <= '9') index++;
                    if (text[index] === '.') {
                        index++;
                        while (index < text.length && text[index] >= '0' && text[index] <= '9') index++;
                    }
                    return parseFloat(text.substring(start, index));
                }

                function parseBoolean() {
                    if (text.substr(index, 4) === 'true') {
                        index += 4;
                        return true;
                    }
                    if (text.substr(index, 5) === 'false') {
                        index += 5;
                        return false;
                    }
                    throw new Error('Invalid boolean');
                }

                function parseNull() {
                    if (text.substr(index, 4) === 'null') {
                        index += 4;
                        return null;
                    }
                    throw new Error('Invalid null');
                }

                function parseArray() {
                    if (text[index] !== '[') throw new Error('Expected [');
                    index++;
                    var result = [];
                    skipWhitespace();

                    if (text[index] === ']') {
                        index++;
                        return result;
                    }

                    while (true) {
                        result.push(parseValue());
                        skipWhitespace();

                        if (text[index] === ']') {
                            index++;
                            break;
                        }
                        if (text[index] === ',') {
                            index++;
                            skipWhitespace();
                        } else {
                            throw new Error('Expected , or ]');
                        }
                    }
                    return result;
                }

                function parseObject() {
                    if (text[index] !== '{') throw new Error('Expected {');
                    index++;
                    var result = {};
                    skipWhitespace();

                    if (text[index] === '}') {
                        index++;
                        return result;
                    }

                    while (true) {
                        skipWhitespace();
                        if (text[index] !== '"') throw new Error('Expected property name');
                        var key = parseString();
                        skipWhitespace();

                        if (text[index] !== ':') throw new Error('Expected :');
                        index++;

                        result[key] = parseValue();
                        skipWhitespace();

                        if (text[index] === '}') {
                            index++;
                            break;
                        }
                        if (text[index] === ',') {
                            index++;
                        } else {
                            throw new Error('Expected , or }');
                        }
                    }
                    return result;
                }

                function skipWhitespace() {
                    while (index < text.length && /\s/.test(text[index])) {
                        index++;
                    }
                }

                return parseValue();
            };
        }

        // Safe parsing utility
        JSON.safeParse = function(str) {
            try {
                return { success: true, data: JSON.parse(str) };
            } catch (e) {
                return { success: false, error: e.message };
            }
        };

        // Basic fetch implementation (mock)
        if (typeof fetch === 'undefined') {
            var fetch = function(url, options) {
                options = options || {};

                return new Promise(function(resolve, reject) {
                    // Mock successful response
                    setTimeout(function() {
                        var response = {
                            ok: true,
                            status: 200,
                            statusText: 'OK',
                            url: url,
                            headers: {
                                get: function(name) {
                                    return null;
                                }
                            },
                            json: function() {
                                return Promise.resolve({});
                            },
                            text: function() {
                                return Promise.resolve('');
                            },
                            blob: function() {
                                return Promise.resolve(new Blob());
                            },
                            arrayBuffer: function() {
                                return Promise.resolve(new ArrayBuffer(0));
                            }
                        };
                        resolve(response);
                    }, 10);
                });
            };
        }

        // Basic Blob implementation
        if (typeof Blob === 'undefined') {
            var Blob = function(parts, options) {
                parts = parts || [];
                options = options || {};

                this.size = 0;
                this.type = options.type || '';

                this.text = function() {
                    return Promise.resolve(parts.join(''));
                };

                this.arrayBuffer = function() {
                    return Promise.resolve(new ArrayBuffer(this.size));
                };
            };
        }

        // Basic FormData implementation
        if (typeof FormData === 'undefined') {
            var FormData = function() {
                this.data = {};

                this.append = function(name, value) {
                    if (this.data[name]) {
                        if (Array.isArray(this.data[name])) {
                            this.data[name].push(value);
                        } else {
                            this.data[name] = [this.data[name], value];
                        }
                    } else {
                        this.data[name] = value;
                    }
                };

                this.delete = function(name) {
                    delete this.data[name];
                };

                this.get = function(name) {
                    var value = this.data[name];
                    return Array.isArray(value) ? value[0] : value;
                };

                this.getAll = function(name) {
                    var value = this.data[name];
                    return Array.isArray(value) ? value : [value];
                };

                this.has = function(name) {
                    return this.data.hasOwnProperty(name);
                };

                this.set = function(name, value) {
                    this.data[name] = value;
                };
            };
        }
    "#))?;

    Ok(())
}