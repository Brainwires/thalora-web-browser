use anyhow::Result;
use thalora_browser_apis::boa_engine::{Context, Source};
use thalora_constants::USER_AGENT;

/// Setup URL and URLSearchParams API
pub fn setup_url_api(context: &mut Context) -> Result<()> {
    // First inject the USER_AGENT as a global constant
    context.eval(Source::from_bytes(&format!("const THALORA_USER_AGENT = '{}';", USER_AGENT)))
        .map_err(|e| anyhow::anyhow!("Failed to set USER_AGENT constant: {}", e))?;

    // Then run the polyfill which uses that constant
    context.eval(Source::from_bytes(r#"
        // Create window object if it doesn't exist
        if (typeof window === 'undefined') {
            var window = globalThis;
        }

        // Ensure window has necessary browser properties
        if (!window.location) {
            window.location = {
                href: 'about:blank',
                protocol: 'about:',
                host: '',
                hostname: '',
                port: '',
                pathname: 'blank',
                search: '',
                hash: '',
                origin: 'null'
            };
        }

        // REMOVED: document polyfill override - use native Boa Document instead
        // The polyfill was overriding the real Boa Document object which broke querySelector and dispatchEvent
        // if (!window.document) {
        //     window.document = { ... fake implementation ... };
        // }

        if (!window.navigator) {
            window.navigator = {
                userAgent: THALORA_USER_AGENT,
                platform: 'Win32',
                language: 'en-US',
                languages: ['en-US', 'en'],
                cookieEnabled: true,
                onLine: true
            };
        }
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

            window.URL.createObjectURL = function(blob) {
                return 'blob:' + Date.now() + Math.random();
            };

            window.URL.revokeObjectURL = function(url) {
                // Revoked object URL (console.log removed for testing)
            };

            // Make URL available at global level
            this.URL = window.URL;
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

            // Make URLSearchParams available at global level
            this.URLSearchParams = window.URLSearchParams;
        }

        // URL and URLSearchParams APIs initialized
    "#)).map_err(|e| anyhow::anyhow!("Failed to setup URL API: {}", e))?;

    Ok(())
}