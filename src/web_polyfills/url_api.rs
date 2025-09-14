use anyhow::Result;
use boa_engine::{Context, Source};

/// Setup URL and URLSearchParams API
pub fn setup_url_api(context: &mut Context) -> Result<()> {
    context.eval(Source::from_bytes(r#"
        // URL API
        if (typeof URL === 'undefined') {
            window.URL = function(url, base) {
                this.href = url;
                this.protocol = 'https:';
                this.host = 'example.com';
                this.hostname = 'example.com';
                this.port = '';
                this.pathname = '/';
                this.search = '';
                this.hash = '';
                this.origin = 'https://example.com';

                // Basic URL parsing
                if (url) {
                    const match = url.match(/^(https?:)\/\/([^\/]+)(\/[^?#]*)?(\?[^#]*)?(#.*)?$/);
                    if (match) {
                        this.protocol = match[1] || 'https:';
                        this.host = match[2] || 'example.com';
                        this.hostname = this.host.split(':')[0];
                        this.port = this.host.includes(':') ? this.host.split(':')[1] : '';
                        this.pathname = match[3] || '/';
                        this.search = match[4] || '';
                        this.hash = match[5] || '';
                        this.origin = this.protocol + '//' + this.host;
                        this.href = url;
                    }
                }

                this.toString = function() { return this.href; };
                return this;
            };

            URL.createObjectURL = function(blob) {
                return 'blob:' + Date.now() + Math.random();
            };

            URL.revokeObjectURL = function(url) {
                console.log('Revoked object URL:', url);
            };
        }

        // URLSearchParams API
        if (typeof URLSearchParams === 'undefined') {
            window.URLSearchParams = function(init) {
                this.params = new Map();

                if (typeof init === 'string') {
                    const pairs = init.replace(/^\?/, '').split('&');
                    for (const pair of pairs) {
                        const [key, value] = pair.split('=');
                        if (key) {
                            this.params.set(decodeURIComponent(key), decodeURIComponent(value || ''));
                        }
                    }
                }

                this.append = function(name, value) {
                    const existing = this.params.get(name);
                    if (existing) {
                        this.params.set(name, existing + ',' + value);
                    } else {
                        this.params.set(name, value);
                    }
                };
                this.delete = function(name) { this.params.delete(name); };
                this.get = function(name) { return this.params.get(name) || null; };
                this.getAll = function(name) {
                    const value = this.params.get(name);
                    return value ? value.split(',') : [];
                };
                this.has = function(name) { return this.params.has(name); };
                this.set = function(name, value) { this.params.set(name, value); };
                this.toString = function() {
                    const pairs = [];
                    for (const [key, value] of this.params) {
                        pairs.push(encodeURIComponent(key) + '=' + encodeURIComponent(value));
                    }
                    return pairs.join('&');
                };

                return this;
            };
        }

        console.log('✅ URL and URLSearchParams APIs initialized');
    "#)).map_err(|e| anyhow::anyhow!("Failed to setup URL API: {}", e))?;

    Ok(())
}