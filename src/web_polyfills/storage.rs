use anyhow::Result;
use boa_engine::{Context, Source};

/// Setup Storage APIs (localStorage, sessionStorage)
pub fn setup_storage(context: &mut Context) -> Result<()> {
    context.eval(Source::from_bytes(r#"
        // STORAGE APIs (localStorage/sessionStorage)
        if (typeof localStorage === 'undefined') {
            window.localStorage = {
                data: {},
                getItem: function(key) {
                    return this.data[key] || null;
                },
                setItem: function(key, value) {
                    this.data[key] = String(value);
                },
                removeItem: function(key) {
                    delete this.data[key];
                },
                clear: function() {
                    this.data = {};
                },
                get length() {
                    return Object.keys(this.data).length;
                },
                key: function(index) {
                    const keys = Object.keys(this.data);
                    return keys[index] || null;
                }
            };
        }

        if (typeof sessionStorage === 'undefined') {
            window.sessionStorage = {
                data: {},
                getItem: function(key) {
                    return this.data[key] || null;
                },
                setItem: function(key, value) {
                    this.data[key] = String(value);
                },
                removeItem: function(key) {
                    delete this.data[key];
                },
                clear: function() {
                    this.data = {};
                },
                get length() {
                    return Object.keys(this.data).length;
                },
                key: function(index) {
                    const keys = Object.keys(this.data);
                    return keys[index] || null;
                }
            };
        }

        console.log('✅ Storage APIs (localStorage, sessionStorage) initialized');
    "#)).map_err(|e| anyhow::anyhow!("Failed to setup storage: {}", e))?;

    Ok(())
}