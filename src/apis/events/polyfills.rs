use boa_engine::{Context, JsResult, Source};

/// Setup basic event polyfills for compatibility
pub fn setup_event_polyfills(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Basic Event constructor
        if (typeof Event === 'undefined') {
            var Event = function(type, eventInitDict) {
                this.type = type;
                this.bubbles = eventInitDict && eventInitDict.bubbles || false;
                this.cancelable = eventInitDict && eventInitDict.cancelable || false;
                this.composed = eventInitDict && eventInitDict.composed || false;
                this.currentTarget = null;
                this.target = null;
                this.timeStamp = Date.now();
                this.defaultPrevented = false;
                this.eventPhase = 0; // NONE
            };

            Event.prototype.preventDefault = function() {
                if (this.cancelable) {
                    this.defaultPrevented = true;
                }
            };

            Event.prototype.stopPropagation = function() {
                // In a real implementation, would stop event propagation
            };

            Event.prototype.stopImmediatePropagation = function() {
                // In a real implementation, would stop immediate propagation
            };

            Event.NONE = 0;
            Event.CAPTURING_PHASE = 1;
            Event.AT_TARGET = 2;
            Event.BUBBLING_PHASE = 3;
        }

        // CustomEvent constructor
        if (typeof CustomEvent === 'undefined') {
            var CustomEvent = function(type, eventInitDict) {
                Event.call(this, type, eventInitDict);
                this.detail = eventInitDict && eventInitDict.detail || null;
            };
            CustomEvent.prototype = Object.create(Event.prototype);
            CustomEvent.prototype.constructor = CustomEvent;
        }

        // MouseEvent constructor
        if (typeof MouseEvent === 'undefined') {
            var MouseEvent = function(type, eventInitDict) {
                Event.call(this, type, eventInitDict);
                var init = eventInitDict || {};
                this.clientX = init.clientX || 0;
                this.clientY = init.clientY || 0;
                this.screenX = init.screenX || 0;
                this.screenY = init.screenY || 0;
                this.button = init.button || 0;
                this.buttons = init.buttons || 0;
                this.ctrlKey = init.ctrlKey || false;
                this.shiftKey = init.shiftKey || false;
                this.altKey = init.altKey || false;
                this.metaKey = init.metaKey || false;
            };
            MouseEvent.prototype = Object.create(Event.prototype);
            MouseEvent.prototype.constructor = MouseEvent;
        }

        // KeyboardEvent constructor
        if (typeof KeyboardEvent === 'undefined') {
            var KeyboardEvent = function(type, eventInitDict) {
                Event.call(this, type, eventInitDict);
                var init = eventInitDict || {};
                this.key = init.key || '';
                this.code = init.code || '';
                this.keyCode = init.keyCode || 0;
                this.which = init.which || this.keyCode;
                this.ctrlKey = init.ctrlKey || false;
                this.shiftKey = init.shiftKey || false;
                this.altKey = init.altKey || false;
                this.metaKey = init.metaKey || false;
                this.repeat = init.repeat || false;
            };
            KeyboardEvent.prototype = Object.create(Event.prototype);
            KeyboardEvent.prototype.constructor = KeyboardEvent;
        }

        // EventTarget interface
        if (typeof EventTarget === 'undefined') {
            var EventTarget = function() {
                this._listeners = {};
            };

            EventTarget.prototype.addEventListener = function(type, listener, options) {
                if (!this._listeners) {
                    this._listeners = {};
                }
                if (!this._listeners[type]) {
                    this._listeners[type] = [];
                }
                this._listeners[type].push(listener);
            };

            EventTarget.prototype.removeEventListener = function(type, listener, options) {
                if (!this._listeners || !this._listeners[type]) {
                    return;
                }
                var index = this._listeners[type].indexOf(listener);
                if (index > -1) {
                    this._listeners[type].splice(index, 1);
                }
            };

            EventTarget.prototype.dispatchEvent = function(event) {
                if (!this._listeners || !this._listeners[event.type]) {
                    return true;
                }

                event.currentTarget = this;
                if (!event.target) {
                    event.target = this;
                }

                var listeners = this._listeners[event.type].slice();
                for (var i = 0; i < listeners.length; i++) {
                    try {
                        listeners[i].call(this, event);
                    } catch (e) {
                        console.error('Event listener error:', e);
                    }
                }

                return !event.defaultPrevented;
            };
        }
    "#))?;

    Ok(())
}