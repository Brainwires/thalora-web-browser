use anyhow::Result;
use boa_engine::{Context, Source};

/// Setup global objects (window, navigator, document)
pub fn setup_globals(context: &mut Context) -> Result<()> {
    context.eval(Source::from_bytes(r#"
        // Create proper global objects for polyfills
        if (typeof window === 'undefined') {
            var window = {};
            // Make window available globally
            this.window = window;
        }

        // Ensure global navigator object exists
        if (typeof navigator === 'undefined') {
            var navigator = {};
            window.navigator = navigator;
            this.navigator = navigator;
        }

        // Ensure global document object exists
        if (typeof document === 'undefined') {
            var document = {};
            window.document = document;
            this.document = document;
        }

        // Make globals accessible at the top level
        var global = this;

        console.log('✅ Global objects (window, navigator, document) initialized');
    "#)).map_err(|e| anyhow::anyhow!("Failed to setup globals: {}", e))?;

    Ok(())
}