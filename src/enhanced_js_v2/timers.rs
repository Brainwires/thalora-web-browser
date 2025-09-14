use boa_engine::{Context, JsResult, Source};

/// Setup timer functions (setTimeout, clearTimeout, etc.)
pub fn setup_timers(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Timer implementation - simplified for now
        var __timerId = 1;
        var __timers = {};

        if (typeof setTimeout === 'undefined') {
            var setTimeout = function(callback, delay) {
                var id = __timerId++;
                var actualDelay = delay || 0;

                // In a real implementation, this would be handled by Rust
                // For now, we simulate immediate execution for testing
                if (actualDelay <= 0) {
                    // Execute immediately
                    try {
                        if (typeof callback === 'function') {
                            callback();
                        } else if (typeof callback === 'string') {
                            eval(callback);
                        }
                    } catch (e) {
                        // Ignore errors in timer callbacks
                    }
                } else {
                    // Store timer info for potential cancellation
                    __timers[id] = {
                        callback: callback,
                        delay: actualDelay,
                        type: 'timeout'
                    };
                }

                return id;
            };

            var clearTimeout = function(id) {
                delete __timers[id];
                return undefined;
            };

            var setInterval = function(callback, delay) {
                var id = __timerId++;
                var actualDelay = delay || 0;

                // Store timer info
                __timers[id] = {
                    callback: callback,
                    delay: actualDelay,
                    type: 'interval'
                };

                return id;
            };

            var clearInterval = function(id) {
                delete __timers[id];
                return undefined;
            };

            // requestAnimationFrame polyfill
            var requestAnimationFrame = function(callback) {
                return setTimeout(callback, 16); // ~60fps
            };

            var cancelAnimationFrame = function(id) {
                return clearTimeout(id);
            };

            // Make them globally available
            global.setTimeout = setTimeout;
            global.clearTimeout = clearTimeout;
            global.setInterval = setInterval;
            global.clearInterval = clearInterval;
            global.requestAnimationFrame = requestAnimationFrame;
            global.cancelAnimationFrame = cancelAnimationFrame;
        }
    "#))?;

    Ok(())
}