use boa_engine::{Context, JsResult, Source};

/// Performance API polyfills
///
/// ⚠️ WARNING: These are MOCK implementations that return fake timing data!
/// They provide API shape compatibility but NOT real functionality.
pub fn setup_performance_apis(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // MOCK Performance API - Returns fake timing data, not real measurements
        if (typeof performance === 'undefined') {
            var performance = {
                now: function() {
                    return Date.now(); // Fallback to Date.now()
                },
                timeOrigin: Date.now() - 1000, // Mock time origin
                timing: {
                    navigationStart: Date.now() - 1000,
                    unloadEventStart: Date.now() - 950,
                    unloadEventEnd: Date.now() - 945,
                    redirectStart: 0,
                    redirectEnd: 0,
                    fetchStart: Date.now() - 940,
                    domainLookupStart: Date.now() - 935,
                    domainLookupEnd: Date.now() - 930,
                    connectStart: Date.now() - 925,
                    connectEnd: Date.now() - 920,
                    secureConnectionStart: Date.now() - 915,
                    requestStart: Date.now() - 910,
                    responseStart: Date.now() - 905,
                    responseEnd: Date.now() - 900,
                    domLoading: Date.now() - 895,
                    domInteractive: Date.now() - 890,
                    domContentLoadedEventStart: Date.now() - 885,
                    domContentLoadedEventEnd: Date.now() - 880,
                    domComplete: Date.now() - 875,
                    loadEventStart: Date.now() - 870,
                    loadEventEnd: Date.now() - 865
                },
                mark: function(name) {
                    // MOCK - Does nothing, real implementation would create performance marks
                    return undefined;
                },
                measure: function(name, startMark, endMark) {
                    // MOCK - Does nothing, real implementation would measure performance
                    return undefined;
                },
                clearMarks: function(name) {
                    // MOCK - Does nothing, real implementation would clear marks
                    return undefined;
                },
                clearMeasures: function(name) {
                    // MOCK - Does nothing, real implementation would clear measures
                    return undefined;
                },
                getEntries: function() {
                    // MOCK - Always returns empty array, real implementation would return performance entries
                    return [];
                },
                getEntriesByType: function(type) {
                    // MOCK - Always returns empty array, real implementation would filter by type
                    return [];
                },
                getEntriesByName: function(name, type) {
                    // MOCK - Always returns empty array, real implementation would filter by name
                    return [];
                }
            };
        }

        // MOCK PerformanceObserver API - Does not actually observe performance
        if (typeof PerformanceObserver === 'undefined') {
            var PerformanceObserver = function(callback) {
                this.callback = callback;
                this.observing = false;
            };

            PerformanceObserver.prototype.observe = function(options) {
                this.observing = true;
                // MOCK - Does nothing, real implementation would start observing performance entries
            };

            PerformanceObserver.prototype.disconnect = function() {
                this.observing = false;
                // MOCK - Does nothing, real implementation would stop observing
            };

            PerformanceObserver.prototype.takeRecords = function() {
                // MOCK - Always returns empty array, real implementation would return buffered entries
                return [];
            };
        }

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