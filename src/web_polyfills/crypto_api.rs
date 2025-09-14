use anyhow::Result;
use boa_engine::{Context, Source};

/// Setup Crypto API and base64 encoding
pub fn setup_crypto(context: &mut Context) -> Result<()> {
    context.eval(Source::from_bytes(r#"
        // BASE64 ENCODING/DECODING
        if (typeof atob === 'undefined') {
            const base64Chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';

            window.atob = function(input) {
                let str = String(input).replace(/=+$/, '');
                let output = '';

                if (str.length % 4 === 1) {
                    throw new Error('Invalid base64 string');
                }

                for (let i = 0; i < str.length; i += 4) {
                    let encoded = 0;
                    for (let j = 0; j < 4; j++) {
                        const char = str[i + j];
                        if (char) {
                            const index = base64Chars.indexOf(char);
                            if (index === -1) throw new Error('Invalid base64 character');
                            encoded = (encoded << 6) | index;
                        }
                    }

                    output += String.fromCharCode((encoded >> 16) & 255);
                    if (i + 2 < str.length) output += String.fromCharCode((encoded >> 8) & 255);
                    if (i + 3 < str.length) output += String.fromCharCode(encoded & 255);
                }

                return output;
            };

            window.btoa = function(input) {
                let str = String(input);
                let output = '';

                for (let i = 0; i < str.length; i += 3) {
                    let a = str.charCodeAt(i);
                    let b = i + 1 < str.length ? str.charCodeAt(i + 1) : 0;
                    let c = i + 2 < str.length ? str.charCodeAt(i + 2) : 0;

                    let bitmap = (a << 16) | (b << 8) | c;

                    output += base64Chars.charAt((bitmap >> 18) & 63);
                    output += base64Chars.charAt((bitmap >> 12) & 63);
                    output += i + 1 < str.length ? base64Chars.charAt((bitmap >> 6) & 63) : '=';
                    output += i + 2 < str.length ? base64Chars.charAt(bitmap & 63) : '=';
                }

                return output;
            };
        }

        // CRYPTO API (Basic)
        if (typeof crypto === 'undefined') {
            window.crypto = {
                getRandomValues: function(array) {
                    for (let i = 0; i < array.length; i++) {
                        array[i] = Math.floor(Math.random() * 256);
                    }
                    return array;
                },

                randomUUID: function() {
                    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
                        const r = Math.random() * 16 | 0;
                        const v = c === 'x' ? r : (r & 0x3 | 0x8);
                        return v.toString(16);
                    });
                },

                subtle: {
                    digest: function(algorithm, data) {
                        // Simplified digest - in real implementation would use proper crypto
                        return Promise.resolve(new ArrayBuffer(32));
                    }
                }
            };
        }

        // TEXT ENCODER/DECODER
        if (typeof TextEncoder === 'undefined') {
            window.TextEncoder = function() {
                this.encoding = 'utf-8';
                this.encode = function(input) {
                    const str = String(input);
                    const result = new Uint8Array(str.length);
                    for (let i = 0; i < str.length; i++) {
                        result[i] = str.charCodeAt(i) & 0xFF;
                    }
                    return result;
                };
                return this;
            };
        }

        if (typeof TextDecoder === 'undefined') {
            window.TextDecoder = function(encoding) {
                this.encoding = encoding || 'utf-8';
                this.decode = function(input) {
                    if (!input) return '';
                    let result = '';
                    const array = new Uint8Array(input);
                    for (let i = 0; i < array.length; i++) {
                        result += String.fromCharCode(array[i]);
                    }
                    return result;
                };
                return this;
            };
        }

        console.log('✅ Crypto API (crypto, atob, btoa, TextEncoder, TextDecoder) initialized');
    "#)).map_err(|e| anyhow::anyhow!("Failed to setup crypto API: {}", e))?;

    Ok(())
}