use boa_engine::{Context, JsResult, Source};

/// Setup DOM-specific events
pub fn setup_dom_events(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // DOMContentLoaded event simulation
        if (typeof document !== 'undefined') {
            // Add common DOM event properties
            if (!document.addEventListener) {
                document.addEventListener = EventTarget.prototype.addEventListener;
            }
            if (!document.removeEventListener) {
                document.removeEventListener = EventTarget.prototype.removeEventListener;
            }
            if (!document.dispatchEvent) {
                document.dispatchEvent = EventTarget.prototype.dispatchEvent;
            }

            // Initialize listeners object
            if (!document._listeners) {
                document._listeners = {};
            }

            // Simulate DOMContentLoaded after a short delay
            setTimeout(function() {
                var event = new Event('DOMContentLoaded', {
                    bubbles: true,
                    cancelable: false
                });
                document.dispatchEvent(event);
            }, 10);

            // Simulate load event
            setTimeout(function() {
                var event = new Event('load', {
                    bubbles: false,
                    cancelable: false
                });
                document.dispatchEvent(event);

                if (typeof window !== 'undefined') {
                    window.dispatchEvent(event);
                }
            }, 50);
        }

        // Window events
        if (typeof window !== 'undefined') {
            if (!window.addEventListener) {
                window.addEventListener = EventTarget.prototype.addEventListener;
            }
            if (!window.removeEventListener) {
                window.removeEventListener = EventTarget.prototype.removeEventListener;
            }
            if (!window.dispatchEvent) {
                window.dispatchEvent = EventTarget.prototype.dispatchEvent;
            }

            if (!window._listeners) {
                window._listeners = {};
            }

            // beforeunload event
            window.onbeforeunload = null;

            // resize event simulation
            setTimeout(function() {
                var event = new Event('resize', {
                    bubbles: false,
                    cancelable: false
                });
                window.dispatchEvent(event);
            }, 100);
        }
    "#))?;

    Ok(())
}

/// Setup focus/blur events
pub fn setup_focus_events(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // FocusEvent constructor
        if (typeof FocusEvent === 'undefined') {
            var FocusEvent = function(type, eventInitDict) {
                Event.call(this, type, eventInitDict);
                var init = eventInitDict || {};
                this.relatedTarget = init.relatedTarget || null;
            };
            FocusEvent.prototype = Object.create(Event.prototype);
            FocusEvent.prototype.constructor = FocusEvent;
        }
    "#))?;

    Ok(())
}