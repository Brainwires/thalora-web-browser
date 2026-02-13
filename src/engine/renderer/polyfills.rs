use thalora_browser_apis::boa_engine::{Context, Source, JsResult};

/// Extended polyfills for the renderer engine
pub fn setup_extended_polyfills(context: &mut Context) -> JsResult<()> {
    context.eval(Source::from_bytes(r#"
        // Extended polyfills that are renderer-specific

        if (typeof window !== 'undefined') {
            // V8-compatible URL constructor
            if (typeof window.URL === 'undefined') {
                window.URL = URL;
            }

            // Extend TextEncoder with Chrome features
            if (typeof TextEncoder !== 'undefined') {
                var originalTextEncoder = TextEncoder;
                var TextEncoderExtended = function(encoding) {
                    this._encoder = new originalTextEncoder(encoding);
                    this.encoding = encoding || 'utf-8';
                };

                TextEncoderExtended.prototype.encode = function(string) {
                    return this._encoder.encode(string);
                };

                // Chrome 134: Add stream encoding support
                TextEncoderExtended.prototype.encodeInto = function(source, destination) {
                    var encoded = this.encode(source);
                    var written = Math.min(encoded.length, destination.length);
                    for (var i = 0; i < written; i++) {
                        destination[i] = encoded[i];
                    }
                    return { read: source.length, written: written };
                };

                window.TextEncoder = TextEncoderExtended;
                globalThis.TextEncoder = TextEncoderExtended;
            }

            // Extended TextDecoder
            if (typeof TextDecoder !== 'undefined') {
                var originalTextDecoder = TextDecoder;
                var TextDecoderExtended = function(encoding, options) {
                    this._decoder = new originalTextDecoder(encoding, options);
                    this.encoding = encoding || 'utf-8';
                    this.fatal = (options && options.fatal) || false;
                    this.ignoreBOM = (options && options.ignoreBOM) || false;
                };

                TextDecoderExtended.prototype.decode = function(buffer, options) {
                    return this._decoder.decode(buffer, options);
                };

                window.TextDecoder = TextDecoderExtended;
                globalThis.TextDecoder = TextDecoderExtended;
            }

            // HTMLCanvasElement constructor and canvas fingerprinting protection
            if (typeof HTMLCanvasElement !== 'undefined') {
                var originalGetContext = HTMLCanvasElement.prototype.getContext;
                HTMLCanvasElement.prototype.getContext = function(contextType, contextAttributes) {
                    var context = originalGetContext.call(this, contextType, contextAttributes);

                    if (context && contextType === '2d') {
                        // Add fingerprinting protection
                        var originalGetImageData = context.getImageData;
                        context.getImageData = function(sx, sy, sw, sh) {
                            var imageData = originalGetImageData.call(this, sx, sy, sw, sh);
                            // Add slight noise to prevent fingerprinting
                            for (var i = 0; i < imageData.data.length; i += 4) {
                                if (Math.random() < 0.001) {
                                    imageData.data[i] = Math.min(255, imageData.data[i] + Math.floor(Math.random() * 3) - 1);
                                }
                            }
                            return imageData;
                        };

                        // Canvas text API with language support (Chrome 136)
                        if (typeof context.measureText === 'function') {
                            var originalMeasureText = context.measureText;
                            context.measureText = function(text) {
                                var metrics = originalMeasureText.call(this, text);

                                // Chrome 136: Add language-aware text metrics
                                if (!metrics.hasOwnProperty('direction')) {
                                    metrics.direction = 'ltr'; // Default direction
                                }

                                return metrics;
                            };
                        }
                    }

                    return context;
                };

                // Chrome 140: Highlights from point API
                if (typeof document.elementsFromPoint === 'function') {
                    var originalElementsFromPoint = document.elementsFromPoint;
                    document.elementsFromPoint = function(x, y) {
                        var elements = originalElementsFromPoint.call(this, x, y);

                        // Add custom highlights support
                        if (typeof CSS !== 'undefined' && CSS.highlights) {
                            // Mock implementation - returns empty array of highlights
                            // In real implementation, would return CSS custom highlights at point
                            elements.highlights = [];
                        }

                        return elements;
                    };
                }

                // Chrome 140: Get installed related apps
                if (typeof navigator !== 'undefined' && !navigator.getInstalledRelatedApps) {
                    navigator.getInstalledRelatedApps = function() {
                        // Mock implementation - returns promise resolving to empty array
                        // In real implementation, would check for installed related apps
                        return Promise.resolve([]);
                    };
                }
            }

            // Create a basic Selection constructor
            if (typeof Selection === 'undefined') {
                window.Selection = function() {
                    this.anchorNode = null;
                    this.anchorOffset = 0;
                    this.focusNode = null;
                    this.focusOffset = 0;
                    this.isCollapsed = true;
                    this.rangeCount = 0;
                    this.type = 'None';
                };

                Selection.prototype.toString = function() {
                    return '';
                };

                Selection.prototype.getRangeAt = function(index) {
                    if (index >= this.rangeCount) {
                        throw new Error('Index out of range');
                    }
                    return {
                        startContainer: this.anchorNode,
                        startOffset: this.anchorOffset,
                        endContainer: this.focusNode,
                        endOffset: this.focusOffset,
                        collapsed: this.isCollapsed
                    };
                };

                Selection.prototype.addRange = function(range) {
                    this.rangeCount = 1;
                    this.isCollapsed = false;
                    this.type = 'Range';
                };

                Selection.prototype.removeAllRanges = function() {
                    this.rangeCount = 0;
                    this.isCollapsed = true;
                    this.type = 'None';
                };
            }

            // Enhanced File and Blob APIs with Chrome features
            if (typeof File === 'undefined') {
                window.File = function(fileBits, fileName, options) {
                    this.name = fileName;
                    this.size = 0;
                    this.type = (options && options.type) || '';
                    this.lastModified = (options && options.lastModified) || Date.now();
                    this.lastModifiedDate = new Date(this.lastModified);

                    if (fileBits) {
                        this.size = fileBits.reduce(function(total, bit) {
                            if (typeof bit === 'string') {
                                return total + bit.length;
                            } else if (bit instanceof ArrayBuffer) {
                                return total + bit.byteLength;
                            }
                            return total;
                        }, 0);
                    }
                };

                File.prototype.text = function() {
                    return Promise.resolve('');
                };

                File.prototype.arrayBuffer = function() {
                    return Promise.resolve(new ArrayBuffer(0));
                };

                File.prototype.stream = function() {
                    return new ReadableStream({
                        start: function(controller) {
                            controller.close();
                        }
                    });
                };

                File.prototype.slice = function(start, end, contentType) {
                    return new Blob([], { type: contentType });
                };
            }

            if (typeof Blob === 'undefined') {
                window.Blob = function(blobParts, options) {
                    this.size = 0;
                    this.type = (options && options.type) || '';

                    if (blobParts) {
                        this.size = blobParts.reduce(function(total, part) {
                            if (typeof part === 'string') {
                                return total + part.length;
                            } else if (part instanceof ArrayBuffer) {
                                return total + part.byteLength;
                            }
                            return total;
                        }, 0);
                    }
                };

                Blob.prototype.text = function() {
                    return Promise.resolve('');
                };

                Blob.prototype.arrayBuffer = function() {
                    return Promise.resolve(new ArrayBuffer(0));
                };

                Blob.prototype.stream = function() {
                    return new ReadableStream({
                        start: function(controller) {
                            controller.close();
                        }
                    });
                };

                Blob.prototype.slice = function(start, end, contentType) {
                    return new Blob([], { type: contentType });
                };
            }
        }

        // DISABLED - Error.stack polyfill was causing stack overflow
        // Google 2025 bot detection APIs now handled in Boa window initialization
    "#))?;

    Ok(())
}