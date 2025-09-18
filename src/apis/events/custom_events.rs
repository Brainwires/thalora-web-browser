use boa_engine::{Context, JsResult, Source};

/// Setup custom events handling
pub fn setup_custom_events(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Message event support for window.postMessage
        if (typeof MessageEvent === 'undefined') {
            var MessageEvent = function(type, eventInitDict) {
                Event.call(this, type, eventInitDict);
                var init = eventInitDict || {};
                this.data = init.data || null;
                this.origin = init.origin || '';
                this.source = init.source || null;
                this.ports = init.ports || [];
            };
            MessageEvent.prototype = Object.create(Event.prototype);
            MessageEvent.prototype.constructor = MessageEvent;
        }

        // Error event for window.onerror
        if (typeof ErrorEvent === 'undefined') {
            var ErrorEvent = function(type, eventInitDict) {
                Event.call(this, type, eventInitDict);
                var init = eventInitDict || {};
                this.message = init.message || '';
                this.filename = init.filename || '';
                this.lineno = init.lineno || 0;
                this.colno = init.colno || 0;
                this.error = init.error || null;
            };
            ErrorEvent.prototype = Object.create(Event.prototype);
            ErrorEvent.prototype.constructor = ErrorEvent;
        }

        // PopStateEvent for history API
        if (typeof PopStateEvent === 'undefined') {
            var PopStateEvent = function(type, eventInitDict) {
                Event.call(this, type, eventInitDict);
                var init = eventInitDict || {};
                this.state = init.state || null;
            };
            PopStateEvent.prototype = Object.create(Event.prototype);
            PopStateEvent.prototype.constructor = PopStateEvent;
        }

        // HashChangeEvent for URL hash changes
        if (typeof HashChangeEvent === 'undefined') {
            var HashChangeEvent = function(type, eventInitDict) {
                Event.call(this, type, eventInitDict);
                var init = eventInitDict || {};
                this.oldURL = init.oldURL || '';
                this.newURL = init.newURL || '';
            };
            HashChangeEvent.prototype = Object.create(Event.prototype);
            HashChangeEvent.prototype.constructor = HashChangeEvent;
        }

        // StorageEvent for localStorage/sessionStorage
        if (typeof StorageEvent === 'undefined') {
            var StorageEvent = function(type, eventInitDict) {
                Event.call(this, type, eventInitDict);
                var init = eventInitDict || {};
                this.key = init.key || null;
                this.oldValue = init.oldValue || null;
                this.newValue = init.newValue || null;
                this.url = init.url || '';
                this.storageArea = init.storageArea || null;
            };
            StorageEvent.prototype = Object.create(Event.prototype);
            StorageEvent.prototype.constructor = StorageEvent;
        }
    "#))?;

    Ok(())
}