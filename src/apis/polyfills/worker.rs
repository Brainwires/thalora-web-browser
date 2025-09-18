use boa_engine::{Context, JsResult, Source};

/// Worker API polyfills
///
/// ⚠️ WARNING: These are MOCK implementations for compatibility testing!
/// They provide API shape compatibility but NOT real worker functionality.
pub fn setup_worker_apis(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Worker API (basic implementation)
        if (typeof Worker === 'undefined') {
            var Worker = function(scriptURL, options) {
                this.scriptURL = scriptURL;
                this.options = options || {};
                this.terminated = false;
                this.onmessage = null;
                this.onerror = null;
                this.onmessageerror = null;

                // MOCK - Real implementation would load and execute script in separate context
                console.log('Worker created with script:', scriptURL);
            };

            Worker.prototype.postMessage = function(message, transfer) {
                if (this.terminated) {
                    throw new Error('Worker has been terminated');
                }
                // MOCK - Real implementation would send message to worker context
                console.log('Message posted to worker:', message);

                // Simulate response after delay
                var self = this;
                setTimeout(function() {
                    if (self.onmessage && !self.terminated) {
                        var event = {
                            data: 'Echo: ' + JSON.stringify(message),
                            type: 'message',
                            target: self
                        };
                        self.onmessage(event);
                    }
                }, 10);
            };

            Worker.prototype.terminate = function() {
                this.terminated = true;
                // MOCK - Real implementation would terminate worker thread
                console.log('Worker terminated');
            };

            Worker.prototype.addEventListener = function(type, listener, options) {
                if (type === 'message') {
                    this.onmessage = listener;
                } else if (type === 'error') {
                    this.onerror = listener;
                } else if (type === 'messageerror') {
                    this.onmessageerror = listener;
                }
            };

            Worker.prototype.removeEventListener = function(type, listener, options) {
                if (type === 'message' && this.onmessage === listener) {
                    this.onmessage = null;
                } else if (type === 'error' && this.onerror === listener) {
                    this.onerror = null;
                } else if (type === 'messageerror' && this.onmessageerror === listener) {
                    this.onmessageerror = null;
                }
            };
        }

        // SharedWorker API with Extended Lifetime support (Chrome 139)
        if (typeof SharedWorker === 'undefined') {
            var SharedWorker = function(scriptURL, options) {
                this.scriptURL = scriptURL;
                this.options = options || {};

                // Chrome 139: Extended Lifetime support
                if (typeof options === 'object' && options.sameSiteCrossOriginWorkerInfo) {
                    this.sameSiteCrossOriginWorkerInfo = options.sameSiteCrossOriginWorkerInfo;
                }

                // Create port for communication
                this.port = {
                    onmessage: null,
                    onmessageerror: null,
                    postMessage: function(message, transfer) {
                        console.log('Message posted to SharedWorker:', message);

                        // Simulate response
                        var port = this;
                        setTimeout(function() {
                            if (port.onmessage) {
                                var event = {
                                    data: 'SharedWorker Echo: ' + JSON.stringify(message),
                                    type: 'message',
                                    target: port
                                };
                                port.onmessage(event);
                            }
                        }, 10);
                    },
                    start: function() {
                        console.log('SharedWorker port started');
                    },
                    close: function() {
                        console.log('SharedWorker port closed');
                        this.onmessage = null;
                        this.onmessageerror = null;
                    },
                    addEventListener: function(type, listener, options) {
                        if (type === 'message') {
                            this.onmessage = listener;
                        } else if (type === 'messageerror') {
                            this.onmessageerror = listener;
                        }
                    },
                    removeEventListener: function(type, listener, options) {
                        if (type === 'message' && this.onmessage === listener) {
                            this.onmessage = null;
                        } else if (type === 'messageerror' && this.onmessageerror === listener) {
                            this.onmessageerror = null;
                        }
                    }
                };

                this.onerror = null;
                console.log('SharedWorker created with script:', scriptURL, 'options:', options);
            };

            SharedWorker.prototype.addEventListener = function(type, listener, options) {
                if (type === 'error') {
                    this.onerror = listener;
                }
            };

            SharedWorker.prototype.removeEventListener = function(type, listener, options) {
                if (type === 'error' && this.onerror === listener) {
                    this.onerror = null;
                }
            };
        }

        // ServiceWorker basic implementation
        if (typeof ServiceWorker === 'undefined') {
            var ServiceWorker = function() {
                this.scriptURL = '';
                this.state = 'installing'; // installing, installed, activating, activated, redundant
                this.onstatechange = null;
                this.onerror = null;
            };

            ServiceWorker.prototype.postMessage = function(message, transfer) {
                console.log('Message posted to ServiceWorker:', message);
            };

            ServiceWorker.prototype.addEventListener = function(type, listener, options) {
                if (type === 'statechange') {
                    this.onstatechange = listener;
                } else if (type === 'error') {
                    this.onerror = listener;
                }
            };

            ServiceWorker.prototype.removeEventListener = function(type, listener, options) {
                if (type === 'statechange' && this.onstatechange === listener) {
                    this.onstatechange = null;
                } else if (type === 'error' && this.onerror === listener) {
                    this.onerror = null;
                }
            };
        }

        // scheduler.yield API (Chrome 129)
        if (typeof scheduler === 'undefined') {
            var scheduler = {
                yield: function(options) {
                    // MOCK - Real implementation would yield to browser for other tasks
                    return new Promise(function(resolve) {
                        setTimeout(resolve, 0);
                    });
                },
                postTask: function(callback, options) {
                    // Basic task scheduling - real implementation would use browser's scheduler
                    var priority = (options && options.priority) || 'user-visible';
                    var delay = 0;

                    // Simple priority mapping
                    if (priority === 'user-blocking') {
                        delay = 0;
                    } else if (priority === 'user-visible') {
                        delay = 5;
                    } else if (priority === 'background') {
                        delay = 10;
                    }

                    return new Promise(function(resolve, reject) {
                        setTimeout(function() {
                            try {
                                var result = callback();
                                resolve(result);
                            } catch (error) {
                                reject(error);
                            }
                        }, delay);
                    });
                }
            };
        }
    "#))?;

    Ok(())
}