use anyhow::Result;
use boa_engine::{Context, Source};

/// Setup AbortController API for fetch cancellation
pub fn setup_abort_controller(context: &mut Context) -> Result<()> {
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

        console.log('✅ AbortController API initialized');
    "#)).map_err(|e| anyhow::anyhow!("Failed to setup AbortController: {}", e))?;

    Ok(())
}