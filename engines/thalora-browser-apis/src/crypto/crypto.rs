//! Implementation of the Web Crypto API
//!
//! The Web Crypto API provides cryptographic functionality including:
//! - crypto.getRandomValues() for generating cryptographically strong random values
//! - crypto.randomUUID() for generating random UUIDs
//! - crypto.subtle for advanced cryptographic operations
//!
//! More information:
//! - [W3C Web Crypto API Specification](https://w3c.github.io/webcrypto/)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/Web_Crypto_API)

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction,
    object::ObjectInitializer, js_string,
};

use super::subtle_crypto::SubtleCrypto;

/// Crypto implementation
pub struct Crypto;

impl Crypto {
    /// Initialize the crypto object in the global scope
    pub fn init(context: &mut Context) {
        // Create the SubtleCrypto object
        let subtle_obj = SubtleCrypto::create(context);

        let crypto_obj = ObjectInitializer::new(context)
            .function(NativeFunction::from_fn_ptr(Self::get_random_values), js_string!("getRandomValues"), 1)
            .function(NativeFunction::from_fn_ptr(Self::random_uuid), js_string!("randomUUID"), 0)
            .property(js_string!("subtle"), subtle_obj, boa_engine::property::Attribute::READONLY | boa_engine::property::Attribute::NON_ENUMERABLE)
            .build();

        context
            .register_global_property(js_string!("crypto"), crypto_obj, boa_engine::property::Attribute::all())
            .expect("Failed to register crypto");
    }

    /// `crypto.getRandomValues(array)`
    ///
    /// Fills the provided TypedArray with cryptographically strong random values.
    ///
    /// More information:
    /// - [MDN documentation][mdn]
    /// - [W3C specification][spec]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Crypto/getRandomValues
    /// [spec]: https://w3c.github.io/webcrypto/#Crypto-method-getRandomValues
    fn get_random_values(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Check if argument is provided
        let array_arg = args.get_or_undefined(0);
        if array_arg.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Failed to execute 'getRandomValues' on 'Crypto': 1 argument required, but only 0 present.")
                .into());
        }

        if array_arg.is_null() {
            return Err(JsNativeError::typ()
                .with_message("Failed to execute 'getRandomValues' on 'Crypto': parameter 1 is not of type 'ArrayBufferView'.")
                .into());
        }

        let array_obj = array_arg.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Failed to execute 'getRandomValues' on 'Crypto': parameter 1 is not of type 'ArrayBufferView'.")
        })?;

        // Check if it's a TypedArray by looking for byteLength property
        let byte_length_val = array_obj.get(js_string!("byteLength"), context)
            .map_err(|_| JsNativeError::typ()
                .with_message("Failed to execute 'getRandomValues' on 'Crypto': parameter 1 is not of type 'ArrayBufferView'."))?;

        let byte_length = byte_length_val.to_u32(context)
            .map_err(|_| JsNativeError::typ()
                .with_message("Failed to execute 'getRandomValues' on 'Crypto': parameter 1 is not of type 'ArrayBufferView'."))?;

        // Check maximum quota (65536 bytes as per spec)
        if byte_length > 65536 {
            let message = format!("Failed to execute 'getRandomValues' on 'Crypto': The ArrayBufferView's byte length ({}) exceeds the maximum allowed length (65536).", byte_length);
            return Err(JsNativeError::range()
                .with_message(message)
                .into());
        }

        // Generate random bytes
        let mut random_bytes = vec![0u8; byte_length as usize];

        #[cfg(all(feature = "js", target_family = "wasm", not(any(target_os = "emscripten", target_os = "wasi"))))]
        {
            getrandom::getrandom(&mut random_bytes)
                .map_err(|_| JsNativeError::error()
                    .with_message("Failed to execute 'getRandomValues' on 'Crypto': Unable to generate random values."))?;
        }
        #[cfg(not(all(feature = "js", target_family = "wasm", not(any(target_os = "emscripten", target_os = "wasi")))))]
        {
            // Fallback using thread_rng for non-WASM targets
            use rand::Rng;
            let mut rng = rand::thread_rng();
            rng.fill(&mut random_bytes[..]);
        }

        // Fill the array with random bytes
        // For TypedArray, we need to set each element
        let length = array_obj.get(js_string!("length"), context)?
            .to_u32(context)?;

        // Determine element size (bytes per element)
        let element_size = if byte_length == 0 {
            1
        } else {
            (byte_length / length.max(1)) as usize
        };

        // Set random values in the array
        for i in 0..length {
            let index = (i as usize) * element_size;
            if index >= random_bytes.len() {
                break;
            }

            // Read the appropriate number of bytes based on element size
            let value = match element_size {
                1 => random_bytes[index] as u32,
                2 => {
                    if index + 1 < random_bytes.len() {
                        u16::from_le_bytes([random_bytes[index], random_bytes[index + 1]]) as u32
                    } else {
                        random_bytes[index] as u32
                    }
                }
                4 => {
                    if index + 3 < random_bytes.len() {
                        u32::from_le_bytes([
                            random_bytes[index],
                            random_bytes[index + 1],
                            random_bytes[index + 2],
                            random_bytes[index + 3],
                        ])
                    } else {
                        random_bytes[index] as u32
                    }
                }
                _ => random_bytes[index] as u32,
            };

            array_obj.set(i, JsValue::from(value), false, context)?;
        }

        // Return the filled array
        Ok(array_arg.clone())
    }

    /// `crypto.randomUUID()`
    ///
    /// Generates a new random UUID (Universally Unique Identifier).
    ///
    /// More information:
    /// - [MDN documentation][mdn]
    /// - [W3C specification][spec]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Crypto/randomUUID
    /// [spec]: https://w3c.github.io/webcrypto/#Crypto-method-randomUUID
    fn random_uuid(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // Generate 16 random bytes
        let mut bytes = [0u8; 16];

        #[cfg(all(feature = "js", target_family = "wasm", not(any(target_os = "emscripten", target_os = "wasi"))))]
        {
            getrandom::getrandom(&mut bytes)
                .map_err(|_| JsNativeError::error()
                    .with_message("Failed to execute 'randomUUID' on 'Crypto': Unable to generate random UUID."))?;
        }
        #[cfg(not(all(feature = "js", target_family = "wasm", not(any(target_os = "emscripten", target_os = "wasi")))))]
        {
            // Fallback using thread_rng for non-WASM targets
            use rand::Rng;
            let mut rng = rand::thread_rng();
            rng.fill(&mut bytes);
        }

        // Set version (4) and variant bits according to RFC 4122
        bytes[6] = (bytes[6] & 0x0f) | 0x40; // Version 4
        bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 10

        // Format as UUID string: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
        let uuid = format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5],
            bytes[6], bytes[7],
            bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        );

        Ok(JsValue::from(js_string!(uuid)))
    }
}
