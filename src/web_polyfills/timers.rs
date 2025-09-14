use anyhow::Result;
use boa_engine::{Context, Source};

/// Setup timers API (setTimeout, setInterval, etc.)
pub fn setup_timers(context: &mut Context) -> Result<()> {
    context.eval(Source::from_bytes(r#"
        // TIMERS API
        if (typeof setTimeout === 'undefined') {
            let timerIdCounter = 1;
            const timers = new Map();

            window.setTimeout = function(callback, delay) {
                const id = timerIdCounter++;
                const startTime = Date.now();
                timers.set(id, { callback, delay, startTime, type: 'timeout' });
                console.log('Timer', id, 'scheduled for', delay, 'ms');
                return id;
            };

            window.setInterval = function(callback, delay) {
                const id = timerIdCounter++;
                const startTime = Date.now();
                timers.set(id, { callback, delay, startTime, type: 'interval' });
                console.log('Interval', id, 'scheduled for', delay, 'ms');
                return id;
            };

            window.clearTimeout = function(id) {
                timers.delete(id);
                console.log('Timer', id, 'cleared');
            };

            window.clearInterval = function(id) {
                timers.delete(id);
                console.log('Interval', id, 'cleared');
            };
        }

        console.log('✅ Timers API (setTimeout, setInterval) initialized');
    "#)).map_err(|e| anyhow::anyhow!("Failed to setup timers: {}", e))?;

    Ok(())
}