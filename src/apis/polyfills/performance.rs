use thalora_browser_apis::boa_engine::{Context, JsResult, Source};

/// Performance API polyfills
///
/// Performance APIs should be implemented natively in Boa engine for real timing data.
/// Mock implementations have been removed to avoid conflicts with native implementations.
pub fn setup_performance_apis(context: &mut Context) -> JsResult<()> {
    // Performance API mock implementation removed - use native Boa implementation when available
    context.eval(Source::from_bytes(r#"
        // Performance APIs removed - waiting for native Boa implementation

        // PressureObserver API (Chrome 125 - Compute Pressure)
        if (typeof PressureObserver === 'undefined') {
            var PressureObserver = function(callback, options) {
                this.callback = callback;
                this.options = options || {};
                this.observing = false;
            };

            PressureObserver.prototype.observe = function(source, options) {
                this.observing = true;
                this.source = source;

                // Mock pressure data - real implementation would read system pressure
                var mockRecord = {
                    source: source || 'cpu',
                    state: 'nominal', // nominal, fair, serious, critical
                    factors: [], // thermal, power-supply
                    time: Date.now()
                };

                // Simulate callback with mock data after a delay
                if (this.callback && typeof this.callback === 'function') {
                    setTimeout(() => {
                        try {
                            this.callback([mockRecord], this);
                        } catch (e) {
                            console.warn('PressureObserver callback error:', e);
                        }
                    }, 100);
                }
            };

            PressureObserver.prototype.unobserve = function(source) {
                this.observing = false;
                this.source = null;
            };

            PressureObserver.prototype.disconnect = function() {
                this.observing = false;
                this.source = null;
            };

            // Static method to check if source is supported
            PressureObserver.knownSources = ['cpu'];
        }
    "#))?;

    Ok(())
}