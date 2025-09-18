use anyhow::Result;
use boa_engine::{Context, Source};

/// Setup Fetch API with related objects (Blob, FormData, Headers, Request, Response)
pub fn setup_fetch(context: &mut Context) -> Result<()> {
    context.eval(Source::from_bytes(r#"
        // ABORT CONTROLLER
        if (typeof AbortController === 'undefined') {
            window.AbortController = function() {
                this.signal = {
                    aborted: false,
                    addEventListener: function(type, listener) {},
                    removeEventListener: function(type, listener) {},
                    dispatchEvent: function(event) { return true; }
                };

                this.abort = function() {
                    this.signal.aborted = true;
                };

                return this;
            };

            // Make AbortController available at global level
            this.AbortController = window.AbortController;
        }

        // BLOB API
        if (typeof Blob === 'undefined') {
            window.Blob = function(parts, options) {
                this.size = 0;
                this.type = options && options.type || '';
                this.parts = parts || [];

                // Calculate size
                for (const part of this.parts) {
                    if (typeof part === 'string') {
                        this.size += part.length;
                    } else if (part.length) {
                        this.size += part.length;
                    }
                }

                this.slice = function(start, end, contentType) {
                    return new Blob([this.parts[0] && this.parts[0].slice(start, end) || ''],
                                  { type: contentType || this.type });
                };

                this.text = function() {
                    return Promise.resolve(this.parts.join(''));
                };

                this.arrayBuffer = function() {
                    const text = this.parts.join('');
                    const buffer = new ArrayBuffer(text.length);
                    const view = new Uint8Array(buffer);
                    for (let i = 0; i < text.length; i++) {
                        view[i] = text.charCodeAt(i);
                    }
                    return Promise.resolve(buffer);
                };

                return this;
            };

            // Make Blob available at global level
            this.Blob = window.Blob;
        }

        // FORMDATA API
        if (typeof FormData === 'undefined') {
            window.FormData = function() {
                this.data = new Map();

                this.append = function(name, value, filename) {
                    const existing = this.data.get(name);
                    const entry = { value, filename };

                    if (existing) {
                        if (Array.isArray(existing)) {
                            existing.push(entry);
                        } else {
                            this.data.set(name, [existing, entry]);
                        }
                    } else {
                        this.data.set(name, entry);
                    }
                };

                this.delete = function(name) {
                    this.data.delete(name);
                };

                this.get = function(name) {
                    const entry = this.data.get(name);
                    if (Array.isArray(entry)) {
                        return entry[0].value;
                    }
                    return entry ? entry.value : null;
                };

                this.getAll = function(name) {
                    const entry = this.data.get(name);
                    if (Array.isArray(entry)) {
                        return entry.map(e => e.value);
                    }
                    return entry ? [entry.value] : [];
                };

                this.has = function(name) {
                    return this.data.has(name);
                };

                this.set = function(name, value, filename) {
                    this.data.set(name, { value, filename });
                };

                return this;
            };

            // Make FormData available at global level
            this.FormData = window.FormData;
        }

        // HEADERS API
        if (typeof Headers === 'undefined') {
            window.Headers = function(init) {
                this.map = new Map();

                if (init) {
                    if (init instanceof Headers) {
                        for (const [key, value] of init.map) {
                            this.map.set(key.toLowerCase(), value);
                        }
                    } else if (Array.isArray(init)) {
                        for (const [key, value] of init) {
                            this.map.set(key.toLowerCase(), value);
                        }
                    } else {
                        for (const [key, value] of Object.entries(init)) {
                            this.map.set(key.toLowerCase(), value);
                        }
                    }
                }

                this.append = function(name, value) {
                    const lowerName = name.toLowerCase();
                    const existing = this.map.get(lowerName);
                    this.map.set(lowerName, existing ? existing + ', ' + value : value);
                };

                this.delete = function(name) {
                    this.map.delete(name.toLowerCase());
                };

                this.get = function(name) {
                    return this.map.get(name.toLowerCase()) || null;
                };

                this.has = function(name) {
                    return this.map.has(name.toLowerCase());
                };

                this.set = function(name, value) {
                    this.map.set(name.toLowerCase(), value);
                };

                this.forEach = function(callback) {
                    for (const [key, value] of this.map) {
                        callback(value, key, this);
                    }
                };

                return this;
            };

            // Make Headers available at global level
            this.Headers = window.Headers;
        }

        // REQUEST API
        if (typeof Request === 'undefined') {
            window.Request = function(input, init) {
                this.url = typeof input === 'string' ? input : input.url;
                this.method = (init && init.method) || 'GET';
                this.headers = new Headers(init && init.headers);
                this.body = (init && init.body) || null;
                this.mode = (init && init.mode) || 'cors';
                this.credentials = (init && init.credentials) || 'same-origin';
                this.cache = (init && init.cache) || 'default';
                this.redirect = (init && init.redirect) || 'follow';
                this.referrer = (init && init.referrer) || '';
                this.bodyUsed = false;

                this.clone = function() {
                    return new Request(this.url, {
                        method: this.method,
                        headers: this.headers,
                        body: this.body,
                        mode: this.mode,
                        credentials: this.credentials,
                        cache: this.cache,
                        redirect: this.redirect,
                        referrer: this.referrer
                    });
                };

                this.text = function() {
                    this.bodyUsed = true;
                    return Promise.resolve(this.body || '');
                };

                this.json = function() {
                    return this.text().then(text => JSON.parse(text));
                };

                return this;
            };

            // Make Request available at global level
            this.Request = window.Request;
        }

        // RESPONSE API
        if (typeof Response === 'undefined') {
            window.Response = function(body, init) {
                this.body = body || null;
                this.status = (init && init.status) || 200;
                this.statusText = (init && init.statusText) || 'OK';
                this.headers = new Headers(init && init.headers);
                this.ok = this.status >= 200 && this.status < 300;
                this.redirected = false;
                this.type = 'basic';
                this.url = '';
                this.bodyUsed = false;

                this.clone = function() {
                    return new Response(this.body, {
                        status: this.status,
                        statusText: this.statusText,
                        headers: this.headers
                    });
                };

                this.text = function() {
                    this.bodyUsed = true;
                    return Promise.resolve(typeof this.body === 'string' ? this.body : JSON.stringify(this.body));
                };

                this.json = function() {
                    return this.text().then(text => JSON.parse(text));
                };

                this.blob = function() {
                    return Promise.resolve(new Blob([this.body || '']));
                };

                this.arrayBuffer = function() {
                    const text = this.body || '';
                    const buffer = new ArrayBuffer(text.length);
                    const view = new Uint8Array(buffer);
                    for (let i = 0; i < text.length; i++) {
                        view[i] = text.charCodeAt(i);
                    }
                    return Promise.resolve(buffer);
                };

                return this;
            };

            window.Response.error = function() {
                return new window.Response(null, { status: 0, statusText: '' });
            };

            window.Response.redirect = function(url, status) {
                return new window.Response(null, {
                    status: status || 302,
                    headers: { 'Location': url }
                });
            };

            // Make Response available at global level
            this.Response = window.Response;
        }

        // FETCH API: Prefer the environment/Boa-provided `fetch` if present, but ensure it's
        // available on `window` and the global `this`. If not present, install a simple
        // fallback implementation (used by some tests/environments).
        if (typeof fetch !== 'undefined') {
            // If fetch already exists (e.g. provided by Boa), ensure it's reachable as
            // `window.fetch` and the global `this.fetch` for older code expecting it there.
            try {
                window.fetch = fetch;
            } catch (e) {
                // ignore assignment errors in constrained contexts
            }
            try {
                this.fetch = fetch;
            } catch (e) {}
        } else {
            // Fallback simple fetch implementation
            window.fetch = function(input, init) {
                // Fetch called with input and init

                // Return a basic response for now
                // In a real implementation, this would call into Rust or an HTTP stack
                return Promise.resolve(new window.Response('{}', {
                    status: 200,
                    statusText: 'OK',
                    headers: { 'Content-Type': 'application/json' }
                }));
            };

            // Make fetch available at global level
            this.fetch = window.fetch;
        }
    "#)).map_err(|e| anyhow::anyhow!("Failed to setup fetch API: {}", e))?;

    Ok(())
}