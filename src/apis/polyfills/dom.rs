use boa_engine::{Context, JsResult, Source};

/// DOM API polyfills and basic constructors
///
/// ⚠️ WARNING: These are MOCK implementations for compatibility testing!
/// They provide API shape compatibility but NOT real DOM functionality.
pub fn setup_dom_apis(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // History API - Real implementation will be provided by browser engine
        // The actual History API with navigation is set up by the browser renderer
        // This polyfill only provides minimal fallbacks if the real API isn't available
        if (typeof window !== 'undefined') {
            if (typeof window.history === 'undefined') {
                window.history = {
                    length: 1,
                    state: null,
                    scrollRestoration: 'auto',
                    back: function() {
                        // MOCK - Real implementation would navigate back
                        console.log('history.back() called');
                    },
                    forward: function() {
                        // MOCK - Real implementation would navigate forward
                        console.log('history.forward() called');
                    },
                    go: function(delta) {
                        // MOCK - Real implementation would navigate by delta
                        console.log('history.go(' + delta + ') called');
                    },
                    pushState: function(state, title, url) {
                        // MOCK - Real implementation would add history entry
                        this.state = state;
                        console.log('history.pushState() called with:', state, title, url);
                    },
                    replaceState: function(state, title, url) {
                        // MOCK - Real implementation would replace current history entry
                        this.state = state;
                        console.log('history.replaceState() called with:', state, title, url);
                    }
                };
            }
        }

        // Basic DOM API - Element constructor
        if (typeof Element === 'undefined') {
            var Element = function() {
                // Basic element properties
                this.tagName = '';
                this.id = '';
                this.className = '';
                this.innerHTML = '';
                this.textContent = '';
                this.children = [];
                this.parentNode = null;
                this.style = {};
            };
        }

        // Basic DOM API - Document constructor
        if (typeof Document === 'undefined') {
            var Document = function() {
                this.documentElement = null;
                this.body = null;
                this.head = null;
                this.title = '';
                this.URL = 'about:blank';
            };
        }

        // Minimal document object if not available
        if (typeof document === 'undefined') {
            var document = {
                documentElement: null,
                body: null,
                head: null,
                title: 'Mock Document',
                URL: 'about:blank',
                visibilityState: 'visible',
                hidden: false,
                createElement: function(tagName) {
                    return {
                        tagName: tagName.toUpperCase(),
                        id: '',
                        className: '',
                        innerHTML: '',
                        textContent: '',
                        children: [],
                        parentNode: null,
                        style: {},
                        setAttribute: function(name, value) {
                            this[name] = value;
                        },
                        getAttribute: function(name) {
                            return this[name];
                        },
                        appendChild: function(child) {
                            child.parentNode = this;
                            this.children.push(child);
                            return child;
                        }
                    };
                }
            };
        }

        // Selection API (Chrome 137)
        if (typeof Selection === 'undefined') {
            var Selection = function() {
                this.anchorNode = null;
                this.anchorOffset = 0;
                this.focusNode = null;
                this.focusOffset = 0;
                this.isCollapsed = true;
                this.rangeCount = 0;
                this.type = 'None';
            };

            Selection.prototype.addRange = function(range) {
                // MOCK - Real implementation would add range to selection
                this.rangeCount = 1;
                this.isCollapsed = false;
                this.type = 'Range';
            };

            Selection.prototype.removeAllRanges = function() {
                this.rangeCount = 0;
                this.isCollapsed = true;
                this.type = 'None';
                this.anchorNode = null;
                this.focusNode = null;
            };

            Selection.prototype.getRangeAt = function(index) {
                if (index >= this.rangeCount) {
                    throw new Error('Index out of range');
                }
                // Return mock range
                return {
                    startContainer: this.anchorNode,
                    startOffset: this.anchorOffset,
                    endContainer: this.focusNode,
                    endOffset: this.focusOffset,
                    collapsed: this.isCollapsed
                };
            };

            Selection.prototype.getComposedRanges = function(shadowRoots) {
                // Chrome 137 feature - MOCK implementation
                return [];
            };

            Selection.prototype.setBaseAndExtent = function(anchorNode, anchorOffset, focusNode, focusOffset) {
                this.anchorNode = anchorNode;
                this.anchorOffset = anchorOffset;
                this.focusNode = focusNode;
                this.focusOffset = focusOffset;
                this.isCollapsed = (anchorNode === focusNode && anchorOffset === focusOffset);
                this.rangeCount = this.isCollapsed ? 0 : 1;
                this.type = this.isCollapsed ? 'Caret' : 'Range';
            };

            // Add direction property (Chrome 137)
            Object.defineProperty(Selection.prototype, 'direction', {
                get: function() {
                    if (this.isCollapsed) return 'none';
                    return 'forward'; // MOCK - always forward
                },
                enumerable: true,
                configurable: false
            });
        }

        if (typeof window !== 'undefined' && typeof window.getSelection === 'undefined') {
            window.getSelection = function() {
                if (!window._selection) {
                    window._selection = new Selection();
                }
                return window._selection;
            };
        }

        if (typeof document !== 'undefined' && typeof document.getSelection === 'undefined') {
            document.getSelection = function() {
                return window.getSelection();
            };
        }

        // HTMLScriptElement polyfill (Chrome 133)
        if (typeof HTMLScriptElement !== 'undefined') {
            // supports attribute (Chrome 133)
            if (!HTMLScriptElement.prototype.hasOwnProperty('supports')) {
                Object.defineProperty(HTMLScriptElement.prototype, 'supports', {
                    get: function() {
                        return this.getAttribute('supports') || '';
                    },
                    set: function(value) {
                        this.setAttribute('supports', value);
                    },
                    enumerable: true,
                    configurable: true
                });
            }

            // Add static supports method
            if (typeof HTMLScriptElement.supports === 'undefined') {
                HTMLScriptElement.supports = function(type) {
                    // MOCK - Basic support detection
                    var supportedTypes = [
                        'classic',
                        'module',
                        'importmap',
                        'speculationrules'
                    ];
                    return supportedTypes.includes(type);
                };
            }
        }
    "#))?;

    Ok(())
}