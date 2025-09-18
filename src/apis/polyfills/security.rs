use boa_engine::{Context, JsResult, Source};

/// Security and Privacy API polyfills
///
/// ⚠️ WARNING: These are MOCK implementations for compatibility testing!
/// They provide API shape compatibility but NOT real security functionality.
pub fn setup_security_apis(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Chrome-specific APIs (should be undefined in non-Chrome browsers)
        if (typeof chrome === 'undefined') {
            // Chrome APIs should be undefined, not throw errors
            var chrome = undefined;
        }

        // Security Context APIs - detect actual context
        if (typeof window !== 'undefined') {
            if (typeof window.isSecureContext === 'undefined') {
                var currentLocation = typeof location !== 'undefined' ? location.protocol : 'https:';
                Object.defineProperty(window, 'isSecureContext', {
                    value: currentLocation === 'https:' || currentLocation === 'file:',
                    writable: false,
                    enumerable: true,
                    configurable: false
                });
            }

            if (typeof window.origin === 'undefined') {
                if (typeof location !== 'undefined' && location.protocol && location.host) {
                    Object.defineProperty(window, 'origin', {
                        value: location.protocol + '//' + location.host,
                        writable: false,
                        enumerable: true,
                        configurable: false
                    });
                }
            }
        }

        if (typeof document !== 'undefined') {
            if (typeof document.visibilityState === 'undefined') {
                document.visibilityState = 'visible';
                document.hidden = false;
            }
        }

        // Security and Privacy APIs
        if (typeof TrustedHTML === 'undefined') {
            var TrustedHTML = function(value) {
                this.toString = function() { return value; };
            };
        }

        if (typeof SecurityPolicyViolationEvent === 'undefined') {
            var SecurityPolicyViolationEvent = function(type, eventInitDict) {
                this.type = type;
                this.blockedURI = eventInitDict ? eventInitDict.blockedURI : '';
                this.violatedDirective = eventInitDict ? eventInitDict.violatedDirective : '';
                this.effectiveDirective = eventInitDict ? eventInitDict.effectiveDirective : '';
                this.originalPolicy = eventInitDict ? eventInitDict.originalPolicy : '';
            };
        }

        if (typeof crossOriginIsolated === 'undefined') {
            var crossOriginIsolated = false;
        }

        // Storage Access API (Chrome 125) - add to document using Object.defineProperty
        if (typeof document !== 'undefined' && typeof document.requestStorageAccess === 'undefined') {
            Object.defineProperty(document, 'requestStorageAccess', {
                value: function(types) {
                    // MOCK - Always resolves, real implementation would prompt user
                    return new Promise(function(resolve, reject) {
                        setTimeout(function() {
                            resolve({
                                all: function() {
                                    return Promise.resolve({
                                        sessionStorage: true,
                                        localStorage: true,
                                        indexedDB: true,
                                        locks: true,
                                        caches: true,
                                        getDirectory: function() {
                                            return Promise.resolve(null);
                                        },
                                        estimate: function() {
                                            return Promise.resolve({ quota: 1000000, usage: 0 });
                                        }
                                    });
                                },
                                sessionStorage: true,
                                localStorage: true,
                                indexedDB: true,
                                locks: true,
                                caches: true
                            });
                        }, 100);
                    });
                },
                writable: false,
                enumerable: true,
                configurable: true
            });
        }

        if (typeof document !== 'undefined' && typeof document.hasStorageAccess === 'undefined') {
            Object.defineProperty(document, 'hasStorageAccess', {
                value: function(types) {
                    // MOCK - Always resolves to true, real implementation would check actual access
                    return Promise.resolve(true);
                },
                writable: false,
                enumerable: true,
                configurable: true
            });
        }

        // User Activation API (Chrome 127)
        if (typeof navigator !== 'undefined' && typeof navigator.userActivation === 'undefined') {
            Object.defineProperty(navigator, 'userActivation', {
                value: {
                    hasBeenActive: true, // MOCK - always true
                    isActive: false // MOCK - usually false unless during user interaction
                },
                writable: false,
                enumerable: true,
                configurable: false
            });
        }
    "#))?;

    Ok(())
}