//! SubtleCrypto - Low-level cryptographic operations
//!
//! The SubtleCrypto interface provides a number of low-level cryptographic functions.
//! Access to the features of SubtleCrypto is obtained through the subtle property of
//! the Crypto object you get from crypto.subtle.
//!
//! More information:
//! - [W3C Web Crypto API Specification](https://w3c.github.io/webcrypto/#subtlecrypto-interface)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto)

use std::sync::Arc;

use boa_engine::{
    js_string,
    object::{builtins::JsPromise, ObjectInitializer},
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
};
use zeroize::Zeroizing;

use super::crypto_key::{
    get_crypto_key_data, parse_key_usages, Algorithm, CryptoKeyData, KeyMaterial, KeyType, KeyUsage,
};

/// SubtleCrypto implementation
pub struct SubtleCrypto;

impl SubtleCrypto {
    /// Create the SubtleCrypto object
    pub fn create(context: &mut Context) -> JsObject {
        ObjectInitializer::new(context)
            .function(
                NativeFunction::from_fn_ptr(Self::digest),
                js_string!("digest"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::generate_key),
                js_string!("generateKey"),
                3,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::import_key),
                js_string!("importKey"),
                5,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::export_key),
                js_string!("exportKey"),
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::encrypt),
                js_string!("encrypt"),
                3,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::decrypt),
                js_string!("decrypt"),
                3,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::sign),
                js_string!("sign"),
                3,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::verify),
                js_string!("verify"),
                4,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::derive_key),
                js_string!("deriveKey"),
                5,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::derive_bits),
                js_string!("deriveBits"),
                3,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::wrap_key),
                js_string!("wrapKey"),
                4,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::unwrap_key),
                js_string!("unwrapKey"),
                7,
            )
            .build()
    }

    // ========================================================================
    // digest()
    // ========================================================================

    /// `subtle.digest(algorithm, data)`
    ///
    /// Generates a digest of the given data using the specified algorithm.
    fn digest(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let algorithm = Algorithm::from_js_value(args.get_or_undefined(0), context)?;
        let data = Self::get_buffer_source(args.get_or_undefined(1), context)?;

        // Perform digest synchronously then wrap in resolved promise
        let result = Self::digest_sync(&algorithm, &data)?;

        // Create ArrayBuffer with result
        let array_buffer = Self::create_array_buffer(&result, context)?;

        let promise = JsPromise::resolve(array_buffer, context);
        Ok(promise.into())
    }

    fn digest_sync(algorithm: &Algorithm, data: &[u8]) -> JsResult<Vec<u8>> {
        use sha1::Sha1;
        use sha2::{Digest, Sha256, Sha384, Sha512};

        match algorithm {
            Algorithm::Sha1 => {
                let mut hasher = Sha1::new();
                hasher.update(data);
                Ok(hasher.finalize().to_vec())
            }
            Algorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                Ok(hasher.finalize().to_vec())
            }
            Algorithm::Sha384 => {
                let mut hasher = Sha384::new();
                hasher.update(data);
                Ok(hasher.finalize().to_vec())
            }
            Algorithm::Sha512 => {
                let mut hasher = Sha512::new();
                hasher.update(data);
                Ok(hasher.finalize().to_vec())
            }
            _ => Err(JsNativeError::typ()
                .with_message(format!(
                    "Algorithm {} is not supported for digest",
                    algorithm.name()
                ))
                .into()),
        }
    }

    // ========================================================================
    // generateKey()
    // ========================================================================

    /// `subtle.generateKey(algorithm, extractable, keyUsages)`
    ///
    /// Generates a new key or key pair.
    fn generate_key(
        _this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let algorithm = Algorithm::from_js_value(args.get_or_undefined(0), context)?;
        let extractable = args.get_or_undefined(1).to_boolean();
        let usages = parse_key_usages(args.get_or_undefined(2), context)?;

        let result = Self::generate_key_sync(algorithm, extractable, usages, context)?;

        let promise = JsPromise::resolve(result, context);
        Ok(promise.into())
    }

    fn generate_key_sync(
        algorithm: Algorithm,
        extractable: bool,
        usages: Vec<KeyUsage>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        use rand::RngCore;

        match algorithm {
            Algorithm::AesGcm { length } | Algorithm::AesCbc { length } | Algorithm::AesCtr { length } => {
                // Validate key length
                if length != 128 && length != 192 && length != 256 {
                    return Err(JsNativeError::typ()
                        .with_message("AES key length must be 128, 192, or 256 bits")
                        .into());
                }

                // Generate random key
                let mut key_bytes = Zeroizing::new(vec![0u8; (length / 8) as usize]);
                rand::thread_rng().fill_bytes(&mut key_bytes);

                // Reconstruct the algorithm for storage
                let alg = Algorithm::AesGcm { length };

                let key_data = CryptoKeyData::new(
                    KeyType::Secret,
                    extractable,
                    alg,
                    usages,
                    KeyMaterial::Symmetric(Arc::new(key_bytes)),
                );

                Ok(key_data.to_js_object(context)?.into())
            }

            Algorithm::Hmac { ref hash, length } => {
                // Determine key length from hash if not specified
                let key_len = length.unwrap_or_else(|| match hash.as_ref() {
                    Algorithm::Sha1 => 512,
                    Algorithm::Sha256 => 512,
                    Algorithm::Sha384 => 1024,
                    Algorithm::Sha512 => 1024,
                    _ => 512,
                }) / 8;

                let mut key_bytes = Zeroizing::new(vec![0u8; key_len as usize]);
                rand::thread_rng().fill_bytes(&mut key_bytes);

                // Reconstruct the algorithm
                let alg = Algorithm::Hmac { hash: hash.clone(), length };

                let key_data = CryptoKeyData::new(
                    KeyType::Secret,
                    extractable,
                    alg,
                    usages,
                    KeyMaterial::Symmetric(Arc::new(key_bytes)),
                );

                Ok(key_data.to_js_object(context)?.into())
            }

            Algorithm::Ecdsa { ref named_curve } | Algorithm::Ecdh { ref named_curve } => {
                let curve = named_curve.clone();
                let alg = Algorithm::Ecdsa { named_curve: curve.clone() };
                Self::generate_ec_key_pair(&curve, extractable, alg, usages, context)
            }

            Algorithm::RsaOaep { modulus_length, ref public_exponent, ref hash }
            | Algorithm::RsaPss { modulus_length, ref public_exponent, ref hash }
            | Algorithm::RsassaPkcs1v15 { modulus_length, ref public_exponent, ref hash } => {
                let alg = Algorithm::RsaOaep {
                    modulus_length,
                    public_exponent: public_exponent.clone(),
                    hash: hash.clone(),
                };
                Self::generate_rsa_key_pair(modulus_length, extractable, alg, usages, context)
            }

            ref alg => Err(JsNativeError::typ()
                .with_message(format!(
                    "Algorithm {} is not supported for key generation",
                    alg.name()
                ))
                .into()),
        }
    }

    fn generate_ec_key_pair(
        named_curve: &str,
        extractable: bool,
        algorithm: Algorithm,
        usages: Vec<KeyUsage>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        match named_curve {
            "P-256" => {
                use p256::ecdsa::SigningKey;
                use p256::elliptic_curve::sec1::ToEncodedPoint;

                let signing_key = SigningKey::random(&mut rand::thread_rng());
                let verifying_key = signing_key.verifying_key();

                // Get public key point
                let public_point = verifying_key.to_encoded_point(false);
                let x = public_point.x().map(|x| x.to_vec()).unwrap_or_default();
                let y = public_point.y().map(|y| y.to_vec()).unwrap_or_default();
                let d = Zeroizing::new(signing_key.to_bytes().to_vec());

                // Create key pair object
                Self::create_ec_key_pair(x, y, d, extractable, algorithm, usages, context)
            }
            "P-384" => {
                use p384::ecdsa::SigningKey;
                use p384::elliptic_curve::sec1::ToEncodedPoint;

                let signing_key = SigningKey::random(&mut rand::thread_rng());
                let verifying_key = signing_key.verifying_key();

                let public_point = verifying_key.to_encoded_point(false);
                let x = public_point.x().map(|x| x.to_vec()).unwrap_or_default();
                let y = public_point.y().map(|y| y.to_vec()).unwrap_or_default();
                let d = Zeroizing::new(signing_key.to_bytes().to_vec());

                Self::create_ec_key_pair(x, y, d, extractable, algorithm, usages, context)
            }
            _ => Err(JsNativeError::typ()
                .with_message(format!("Unsupported curve: {}", named_curve))
                .into()),
        }
    }

    fn create_ec_key_pair(
        x: Vec<u8>,
        y: Vec<u8>,
        d: Zeroizing<Vec<u8>>,
        extractable: bool,
        algorithm: Algorithm,
        usages: Vec<KeyUsage>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Determine which usages go to which key
        let (public_usages, private_usages): (Vec<_>, Vec<_>) = usages
            .into_iter()
            .partition(|u| matches!(u, KeyUsage::Verify | KeyUsage::Encrypt));

        // Create public key
        let public_key = CryptoKeyData::new(
            KeyType::Public,
            true, // Public keys are always extractable
            algorithm.clone(),
            public_usages,
            KeyMaterial::EcPublic {
                x: x.clone(),
                y: y.clone(),
            },
        );

        // Create private key
        let private_key = CryptoKeyData::new(
            KeyType::Private,
            extractable,
            algorithm,
            private_usages,
            KeyMaterial::EcPrivate {
                x,
                y,
                d: Arc::new(d),
            },
        );

        // Create CryptoKeyPair object
        let key_pair = ObjectInitializer::new(context).build();
        key_pair.set(
            js_string!("publicKey"),
            public_key.to_js_object(context)?,
            false,
            context,
        )?;
        key_pair.set(
            js_string!("privateKey"),
            private_key.to_js_object(context)?,
            false,
            context,
        )?;

        Ok(key_pair.into())
    }

    fn generate_rsa_key_pair(
        modulus_length: u32,
        extractable: bool,
        algorithm: Algorithm,
        usages: Vec<KeyUsage>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        use rsa::{RsaPrivateKey, traits::PublicKeyParts, pkcs8::EncodePrivateKey};

        // Validate modulus length
        if modulus_length < 1024 || modulus_length > 4096 {
            return Err(JsNativeError::typ()
                .with_message("RSA modulus length must be between 1024 and 4096 bits")
                .into());
        }

        // Generate RSA key pair
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, modulus_length as usize)
            .map_err(|e| JsNativeError::error().with_message(format!("RSA key generation failed: {}", e)))?;

        let public_key = private_key.to_public_key();

        // Extract key components from public key
        let n = public_key.n().to_bytes_be();
        let e = public_key.e().to_bytes_be();

        // For private key material, we serialize to PKCS8 and store as raw bytes
        // This is a simplified approach - in a full implementation you'd extract individual components
        let private_key_der = private_key.to_pkcs8_der()
            .map_err(|e| JsNativeError::error().with_message(format!("Failed to encode private key: {}", e)))?;
        let d = Zeroizing::new(private_key_der.as_bytes().to_vec());
        let p = Zeroizing::new(Vec::new()); // Stored in PKCS8 blob
        let q = Zeroizing::new(Vec::new()); // Stored in PKCS8 blob

        // Determine which usages go to which key
        let (public_usages, private_usages): (Vec<_>, Vec<_>) = usages
            .into_iter()
            .partition(|u| matches!(u, KeyUsage::Verify | KeyUsage::Encrypt | KeyUsage::WrapKey));

        // Create public key
        let public_key_data = CryptoKeyData::new(
            KeyType::Public,
            true,
            algorithm.clone(),
            public_usages,
            KeyMaterial::RsaPublic {
                n: n.clone(),
                e: e.clone(),
            },
        );

        // Create private key
        let private_key_data = CryptoKeyData::new(
            KeyType::Private,
            extractable,
            algorithm,
            private_usages,
            KeyMaterial::RsaPrivate {
                n,
                e,
                d: Arc::new(d),
                p: Arc::new(p),
                q: Arc::new(q),
            },
        );

        // Create CryptoKeyPair object
        let key_pair = ObjectInitializer::new(context).build();
        key_pair.set(
            js_string!("publicKey"),
            public_key_data.to_js_object(context)?,
            false,
            context,
        )?;
        key_pair.set(
            js_string!("privateKey"),
            private_key_data.to_js_object(context)?,
            false,
            context,
        )?;

        Ok(key_pair.into())
    }

    // ========================================================================
    // importKey() / exportKey()
    // ========================================================================

    /// `subtle.importKey(format, keyData, algorithm, extractable, keyUsages)`
    fn import_key(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let format = args
            .get_or_undefined(0)
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("format must be a string"))?
            .to_std_string_escaped();

        let key_data = args.get_or_undefined(1);
        let algorithm = Algorithm::from_js_value(args.get_or_undefined(2), context)?;
        let extractable = args.get_or_undefined(3).to_boolean();
        let usages = parse_key_usages(args.get_or_undefined(4), context)?;

        let result =
            Self::import_key_sync(&format, key_data, algorithm, extractable, usages, context)?;

        let promise = JsPromise::resolve(result, context);
        Ok(promise.into())
    }

    fn import_key_sync(
        format: &str,
        key_data: &JsValue,
        algorithm: Algorithm,
        extractable: bool,
        usages: Vec<KeyUsage>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        match format {
            "raw" => {
                let raw_bytes = Self::get_buffer_source(key_data, context)?;
                Self::import_raw_key(raw_bytes, algorithm, extractable, usages, context)
            }
            "jwk" => {
                Self::import_jwk_key(key_data, algorithm, extractable, usages, context)
            }
            "spki" | "pkcs8" => {
                Err(JsNativeError::typ()
                    .with_message(format!("Key format '{}' is not yet implemented", format))
                    .into())
            }
            _ => Err(JsNativeError::typ()
                .with_message(format!("Unknown key format: {}", format))
                .into()),
        }
    }

    fn import_raw_key(
        raw_bytes: Vec<u8>,
        algorithm: Algorithm,
        extractable: bool,
        usages: Vec<KeyUsage>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        match &algorithm {
            Algorithm::AesGcm { .. } | Algorithm::AesCbc { .. } | Algorithm::AesCtr { .. } => {
                let key_len = raw_bytes.len();
                if key_len != 16 && key_len != 24 && key_len != 32 {
                    return Err(JsNativeError::typ()
                        .with_message("Invalid AES key length")
                        .into());
                }

                let key_data = CryptoKeyData::new(
                    KeyType::Secret,
                    extractable,
                    algorithm,
                    usages,
                    KeyMaterial::Symmetric(Arc::new(Zeroizing::new(raw_bytes))),
                );

                Ok(key_data.to_js_object(context)?.into())
            }
            Algorithm::Hmac { .. } => {
                let key_data = CryptoKeyData::new(
                    KeyType::Secret,
                    extractable,
                    algorithm,
                    usages,
                    KeyMaterial::Symmetric(Arc::new(Zeroizing::new(raw_bytes))),
                );

                Ok(key_data.to_js_object(context)?.into())
            }
            _ => Err(JsNativeError::typ()
                .with_message(format!(
                    "Algorithm {} does not support raw key import",
                    algorithm.name()
                ))
                .into()),
        }
    }

    fn import_jwk_key(
        _key_data: &JsValue,
        _algorithm: Algorithm,
        _extractable: bool,
        _usages: Vec<KeyUsage>,
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // JWK import is complex - implement basic version
        Err(JsNativeError::typ()
            .with_message("JWK import is not yet fully implemented")
            .into())
    }

    /// `subtle.exportKey(format, key)`
    fn export_key(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let format = args
            .get_or_undefined(0)
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("format must be a string"))?
            .to_std_string_escaped();

        let key = get_crypto_key_data(args.get_or_undefined(1))?;

        if !key.extractable {
            return Err(JsNativeError::typ()
                .with_message("Key is not extractable")
                .into());
        }

        let result = Self::export_key_sync(&format, &key, context)?;

        let promise = JsPromise::resolve(result, context);
        Ok(promise.into())
    }

    fn export_key_sync(
        format: &str,
        key: &CryptoKeyData,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        match format {
            "raw" => {
                match &key.material {
                    KeyMaterial::Symmetric(bytes) => {
                        let array_buffer = Self::create_array_buffer(bytes.as_ref(), context)?;
                        Ok(array_buffer.into())
                    }
                    _ => Err(JsNativeError::typ()
                        .with_message("Only symmetric keys can be exported in raw format")
                        .into()),
                }
            }
            "jwk" => {
                // Create JWK object
                let jwk = Self::export_as_jwk(key, context)?;
                Ok(jwk.into())
            }
            "spki" | "pkcs8" => {
                Err(JsNativeError::typ()
                    .with_message(format!("Export format '{}' is not yet implemented", format))
                    .into())
            }
            _ => Err(JsNativeError::typ()
                .with_message(format!("Unknown export format: {}", format))
                .into()),
        }
    }

    fn export_as_jwk(key: &CryptoKeyData, context: &mut Context) -> JsResult<JsObject> {
        let jwk = ObjectInitializer::new(context).build();

        // Set common JWK properties
        jwk.set(
            js_string!("kty"),
            js_string!(match &key.material {
                KeyMaterial::Symmetric(_) => "oct",
                KeyMaterial::RsaPublic { .. } | KeyMaterial::RsaPrivate { .. } => "RSA",
                KeyMaterial::EcPublic { .. } | KeyMaterial::EcPrivate { .. } => "EC",
            }),
            false,
            context,
        )?;

        jwk.set(
            js_string!("alg"),
            js_string!(key.algorithm.name()),
            false,
            context,
        )?;

        jwk.set(
            js_string!("ext"),
            JsValue::from(key.extractable),
            false,
            context,
        )?;

        // Set key_ops
        let key_ops: Vec<JsValue> = key
            .usages
            .iter()
            .map(|u| JsValue::from(js_string!(u.as_str())))
            .collect();
        let key_ops_array = boa_engine::object::builtins::JsArray::from_iter(key_ops, context);
        jwk.set(js_string!("key_ops"), key_ops_array, false, context)?;

        // Set key-specific properties
        match &key.material {
            KeyMaterial::Symmetric(bytes) => {
                let k = base64::Engine::encode(
                    &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                    bytes.as_ref(),
                );
                jwk.set(js_string!("k"), js_string!(k), false, context)?;
            }
            KeyMaterial::EcPublic { x, y } | KeyMaterial::EcPrivate { x, y, .. } => {
                let x_b64 = base64::Engine::encode(
                    &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                    x,
                );
                let y_b64 = base64::Engine::encode(
                    &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                    y,
                );
                jwk.set(js_string!("x"), js_string!(x_b64), false, context)?;
                jwk.set(js_string!("y"), js_string!(y_b64), false, context)?;

                if let KeyMaterial::EcPrivate { d, .. } = &key.material {
                    let d_b64 = base64::Engine::encode(
                        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                        d.as_ref(),
                    );
                    jwk.set(js_string!("d"), js_string!(d_b64), false, context)?;
                }

                // Set curve
                if let Algorithm::Ecdsa { named_curve } | Algorithm::Ecdh { named_curve } =
                    &key.algorithm
                {
                    jwk.set(
                        js_string!("crv"),
                        js_string!(named_curve.as_str()),
                        false,
                        context,
                    )?;
                }
            }
            _ => {}
        }

        Ok(jwk)
    }

    // ========================================================================
    // encrypt() / decrypt()
    // ========================================================================

    /// `subtle.encrypt(algorithm, key, data)`
    fn encrypt(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let algorithm_val = args.get_or_undefined(0);
        let key = get_crypto_key_data(args.get_or_undefined(1))?;
        let data = Self::get_buffer_source(args.get_or_undefined(2), context)?;

        if !key.can_use(KeyUsage::Encrypt) {
            return Err(JsNativeError::typ()
                .with_message("Key does not support encryption")
                .into());
        }

        let result = Self::encrypt_sync(algorithm_val, &key, &data, context)?;

        let array_buffer = Self::create_array_buffer(&result, context)?;
        let promise = JsPromise::resolve(array_buffer, context);
        Ok(promise.into())
    }

    fn encrypt_sync(
        algorithm_val: &JsValue,
        key: &CryptoKeyData,
        data: &[u8],
        context: &mut Context,
    ) -> JsResult<Vec<u8>> {
        // Parse algorithm with encryption-specific parameters
        let algorithm_obj = algorithm_val.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Algorithm must be an object")
        })?;

        let name = algorithm_obj
            .get(js_string!("name"), context)?
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("Algorithm name required"))?
            .to_std_string_escaped();

        match name.to_uppercase().as_str() {
            "AES-GCM" => {
                let iv = Self::get_algorithm_param_buffer(&algorithm_obj, "iv", context)?;
                Self::aes_gcm_encrypt(key, &iv, data)
            }
            "AES-CBC" => {
                let iv = Self::get_algorithm_param_buffer(&algorithm_obj, "iv", context)?;
                Self::aes_cbc_encrypt(key, &iv, data)
            }
            _ => Err(JsNativeError::typ()
                .with_message(format!("Encryption algorithm {} not supported", name))
                .into()),
        }
    }

    fn aes_gcm_encrypt(key: &CryptoKeyData, iv: &[u8], data: &[u8]) -> JsResult<Vec<u8>> {
        use aes_gcm::{Aes128Gcm, Aes256Gcm, KeyInit, aead::Aead};
        use aes_gcm::Nonce;

        let key_bytes = match &key.material {
            KeyMaterial::Symmetric(k) => k.as_ref().as_slice(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("Invalid key type for AES-GCM")
                    .into())
            }
        };

        let nonce = Nonce::from_slice(iv);

        match key_bytes.len() {
            16 => {
                let cipher = Aes128Gcm::new_from_slice(key_bytes)
                    .map_err(|e| JsNativeError::error().with_message(format!("AES-GCM error: {}", e)))?;
                cipher
                    .encrypt(nonce, data)
                    .map_err(|e| JsNativeError::error().with_message(format!("Encryption failed: {}", e)).into())
            }
            32 => {
                let cipher = Aes256Gcm::new_from_slice(key_bytes)
                    .map_err(|e| JsNativeError::error().with_message(format!("AES-GCM error: {}", e)))?;
                cipher
                    .encrypt(nonce, data)
                    .map_err(|e| JsNativeError::error().with_message(format!("Encryption failed: {}", e)).into())
            }
            _ => Err(JsNativeError::typ()
                .with_message("Invalid AES key length")
                .into()),
        }
    }

    fn aes_cbc_encrypt(key: &CryptoKeyData, iv: &[u8], data: &[u8]) -> JsResult<Vec<u8>> {
        use aes::Aes128;
        use cbc::{Encryptor, cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7}};

        let key_bytes = match &key.material {
            KeyMaterial::Symmetric(k) => k.as_ref().as_slice(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("Invalid key type for AES-CBC")
                    .into())
            }
        };

        if key_bytes.len() != 16 {
            return Err(JsNativeError::typ()
                .with_message("AES-CBC currently only supports 128-bit keys")
                .into());
        }

        type Aes128CbcEnc = Encryptor<Aes128>;
        let cipher = Aes128CbcEnc::new_from_slices(key_bytes, iv)
            .map_err(|e| JsNativeError::error().with_message(format!("AES-CBC error: {}", e)))?;

        Ok(cipher.encrypt_padded_vec_mut::<Pkcs7>(data))
    }

    /// `subtle.decrypt(algorithm, key, data)`
    fn decrypt(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let algorithm_val = args.get_or_undefined(0);
        let key = get_crypto_key_data(args.get_or_undefined(1))?;
        let data = Self::get_buffer_source(args.get_or_undefined(2), context)?;

        if !key.can_use(KeyUsage::Decrypt) {
            return Err(JsNativeError::typ()
                .with_message("Key does not support decryption")
                .into());
        }

        let result = Self::decrypt_sync(algorithm_val, &key, &data, context)?;

        let array_buffer = Self::create_array_buffer(&result, context)?;
        let promise = JsPromise::resolve(array_buffer, context);
        Ok(promise.into())
    }

    fn decrypt_sync(
        algorithm_val: &JsValue,
        key: &CryptoKeyData,
        data: &[u8],
        context: &mut Context,
    ) -> JsResult<Vec<u8>> {
        let algorithm_obj = algorithm_val.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Algorithm must be an object")
        })?;

        let name = algorithm_obj
            .get(js_string!("name"), context)?
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("Algorithm name required"))?
            .to_std_string_escaped();

        match name.to_uppercase().as_str() {
            "AES-GCM" => {
                let iv = Self::get_algorithm_param_buffer(&algorithm_obj, "iv", context)?;
                Self::aes_gcm_decrypt(key, &iv, data)
            }
            "AES-CBC" => {
                let iv = Self::get_algorithm_param_buffer(&algorithm_obj, "iv", context)?;
                Self::aes_cbc_decrypt(key, &iv, data)
            }
            _ => Err(JsNativeError::typ()
                .with_message(format!("Decryption algorithm {} not supported", name))
                .into()),
        }
    }

    fn aes_gcm_decrypt(key: &CryptoKeyData, iv: &[u8], data: &[u8]) -> JsResult<Vec<u8>> {
        use aes_gcm::{Aes128Gcm, Aes256Gcm, KeyInit, aead::Aead};
        use aes_gcm::Nonce;

        let key_bytes = match &key.material {
            KeyMaterial::Symmetric(k) => k.as_ref().as_slice(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("Invalid key type for AES-GCM")
                    .into())
            }
        };

        let nonce = Nonce::from_slice(iv);

        match key_bytes.len() {
            16 => {
                let cipher = Aes128Gcm::new_from_slice(key_bytes)
                    .map_err(|e| JsNativeError::error().with_message(format!("AES-GCM error: {}", e)))?;
                cipher
                    .decrypt(nonce, data)
                    .map_err(|e| JsNativeError::error().with_message(format!("Decryption failed: {}", e)).into())
            }
            32 => {
                let cipher = Aes256Gcm::new_from_slice(key_bytes)
                    .map_err(|e| JsNativeError::error().with_message(format!("AES-GCM error: {}", e)))?;
                cipher
                    .decrypt(nonce, data)
                    .map_err(|e| JsNativeError::error().with_message(format!("Decryption failed: {}", e)).into())
            }
            _ => Err(JsNativeError::typ()
                .with_message("Invalid AES key length")
                .into()),
        }
    }

    fn aes_cbc_decrypt(key: &CryptoKeyData, iv: &[u8], data: &[u8]) -> JsResult<Vec<u8>> {
        use aes::Aes128;
        use cbc::{Decryptor, cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7}};

        let key_bytes = match &key.material {
            KeyMaterial::Symmetric(k) => k.as_ref().as_slice(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("Invalid key type for AES-CBC")
                    .into())
            }
        };

        if key_bytes.len() != 16 {
            return Err(JsNativeError::typ()
                .with_message("AES-CBC currently only supports 128-bit keys")
                .into());
        }

        type Aes128CbcDec = Decryptor<Aes128>;
        let cipher = Aes128CbcDec::new_from_slices(key_bytes, iv)
            .map_err(|e| JsNativeError::error().with_message(format!("AES-CBC error: {}", e)))?;

        cipher
            .decrypt_padded_vec_mut::<Pkcs7>(data)
            .map_err(|e| JsNativeError::error().with_message(format!("Decryption failed: {}", e)).into())
    }

    // ========================================================================
    // sign() / verify()
    // ========================================================================

    /// `subtle.sign(algorithm, key, data)`
    fn sign(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let algorithm_val = args.get_or_undefined(0);
        let key = get_crypto_key_data(args.get_or_undefined(1))?;
        let data = Self::get_buffer_source(args.get_or_undefined(2), context)?;

        if !key.can_use(KeyUsage::Sign) {
            return Err(JsNativeError::typ()
                .with_message("Key does not support signing")
                .into());
        }

        let result = Self::sign_sync(algorithm_val, &key, &data, context)?;

        let array_buffer = Self::create_array_buffer(&result, context)?;
        let promise = JsPromise::resolve(array_buffer, context);
        Ok(promise.into())
    }

    fn sign_sync(
        algorithm_val: &JsValue,
        key: &CryptoKeyData,
        data: &[u8],
        context: &mut Context,
    ) -> JsResult<Vec<u8>> {
        let algorithm = Algorithm::from_js_value(algorithm_val, context)?;

        match (&algorithm, &key.algorithm) {
            (Algorithm::Hmac { .. }, Algorithm::Hmac { hash, .. }) => {
                Self::hmac_sign(key, hash, data)
            }
            (Algorithm::Ecdsa { named_curve }, Algorithm::Ecdsa { named_curve: key_curve })
                if named_curve == key_curve =>
            {
                Self::ecdsa_sign(key, named_curve, data)
            }
            _ => Err(JsNativeError::typ()
                .with_message("Algorithm mismatch for signing")
                .into()),
        }
    }

    fn hmac_sign(key: &CryptoKeyData, hash: &Algorithm, data: &[u8]) -> JsResult<Vec<u8>> {
        use hmac::{Hmac, Mac};
        use sha1::Sha1;
        use sha2::{Sha256, Sha384, Sha512};

        let key_bytes = match &key.material {
            KeyMaterial::Symmetric(k) => k.as_ref().as_slice(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("Invalid key type for HMAC")
                    .into())
            }
        };

        match hash {
            Algorithm::Sha1 => {
                let mut mac = Hmac::<Sha1>::new_from_slice(key_bytes)
                    .map_err(|e| JsNativeError::error().with_message(format!("HMAC error: {}", e)))?;
                mac.update(data);
                Ok(mac.finalize().into_bytes().to_vec())
            }
            Algorithm::Sha256 => {
                let mut mac = Hmac::<Sha256>::new_from_slice(key_bytes)
                    .map_err(|e| JsNativeError::error().with_message(format!("HMAC error: {}", e)))?;
                mac.update(data);
                Ok(mac.finalize().into_bytes().to_vec())
            }
            Algorithm::Sha384 => {
                let mut mac = Hmac::<Sha384>::new_from_slice(key_bytes)
                    .map_err(|e| JsNativeError::error().with_message(format!("HMAC error: {}", e)))?;
                mac.update(data);
                Ok(mac.finalize().into_bytes().to_vec())
            }
            Algorithm::Sha512 => {
                let mut mac = Hmac::<Sha512>::new_from_slice(key_bytes)
                    .map_err(|e| JsNativeError::error().with_message(format!("HMAC error: {}", e)))?;
                mac.update(data);
                Ok(mac.finalize().into_bytes().to_vec())
            }
            _ => Err(JsNativeError::typ()
                .with_message("Unsupported hash algorithm for HMAC")
                .into()),
        }
    }

    fn ecdsa_sign(key: &CryptoKeyData, curve: &str, data: &[u8]) -> JsResult<Vec<u8>> {
        use sha2::{Digest, Sha256, Sha384};

        let d = match &key.material {
            KeyMaterial::EcPrivate { d, .. } => d.as_ref().as_slice(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("Invalid key type for ECDSA signing")
                    .into())
            }
        };

        match curve {
            "P-256" => {
                use p256::ecdsa::{SigningKey, signature::Signer};

                let signing_key = SigningKey::from_slice(d)
                    .map_err(|e| JsNativeError::error().with_message(format!("Invalid key: {}", e)))?;

                let signature: p256::ecdsa::Signature = signing_key.sign(data);
                Ok(signature.to_vec())
            }
            "P-384" => {
                use p384::ecdsa::{SigningKey, signature::Signer};

                let signing_key = SigningKey::from_slice(d)
                    .map_err(|e| JsNativeError::error().with_message(format!("Invalid key: {}", e)))?;

                let signature: p384::ecdsa::Signature = signing_key.sign(data);
                Ok(signature.to_vec())
            }
            _ => Err(JsNativeError::typ()
                .with_message(format!("Unsupported curve: {}", curve))
                .into()),
        }
    }

    /// `subtle.verify(algorithm, key, signature, data)`
    fn verify(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let algorithm_val = args.get_or_undefined(0);
        let key = get_crypto_key_data(args.get_or_undefined(1))?;
        let signature = Self::get_buffer_source(args.get_or_undefined(2), context)?;
        let data = Self::get_buffer_source(args.get_or_undefined(3), context)?;

        if !key.can_use(KeyUsage::Verify) {
            return Err(JsNativeError::typ()
                .with_message("Key does not support verification")
                .into());
        }

        let result = Self::verify_sync(algorithm_val, &key, &signature, &data, context)?;

        let promise = JsPromise::resolve(JsValue::from(result), context);
        Ok(promise.into())
    }

    fn verify_sync(
        algorithm_val: &JsValue,
        key: &CryptoKeyData,
        signature: &[u8],
        data: &[u8],
        context: &mut Context,
    ) -> JsResult<bool> {
        let algorithm = Algorithm::from_js_value(algorithm_val, context)?;

        match (&algorithm, &key.algorithm) {
            (Algorithm::Hmac { .. }, Algorithm::Hmac { hash, .. }) => {
                Self::hmac_verify(key, hash, signature, data)
            }
            (Algorithm::Ecdsa { named_curve }, Algorithm::Ecdsa { named_curve: key_curve })
                if named_curve == key_curve =>
            {
                Self::ecdsa_verify(key, named_curve, signature, data)
            }
            _ => Err(JsNativeError::typ()
                .with_message("Algorithm mismatch for verification")
                .into()),
        }
    }

    fn hmac_verify(
        key: &CryptoKeyData,
        hash: &Algorithm,
        signature: &[u8],
        data: &[u8],
    ) -> JsResult<bool> {
        let computed = Self::hmac_sign(key, hash, data)?;
        // Constant-time comparison
        Ok(computed.len() == signature.len()
            && computed
                .iter()
                .zip(signature.iter())
                .fold(0u8, |acc, (a, b)| acc | (a ^ b))
                == 0)
    }

    fn ecdsa_verify(
        key: &CryptoKeyData,
        curve: &str,
        signature: &[u8],
        data: &[u8],
    ) -> JsResult<bool> {
        let (x, y) = match &key.material {
            KeyMaterial::EcPublic { x, y } => (x.as_slice(), y.as_slice()),
            KeyMaterial::EcPrivate { x, y, .. } => (x.as_slice(), y.as_slice()),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("Invalid key type for ECDSA verification")
                    .into())
            }
        };

        match curve {
            "P-256" => {
                use p256::ecdsa::{VerifyingKey, signature::Verifier};
                use p256::EncodedPoint;

                let point = EncodedPoint::from_affine_coordinates(
                    x.into(),
                    y.into(),
                    false,
                );
                let verifying_key = VerifyingKey::from_encoded_point(&point)
                    .map_err(|e| JsNativeError::error().with_message(format!("Invalid key: {}", e)))?;

                let sig = p256::ecdsa::Signature::from_slice(signature)
                    .map_err(|e| JsNativeError::error().with_message(format!("Invalid signature: {}", e)))?;

                Ok(verifying_key.verify(data, &sig).is_ok())
            }
            "P-384" => {
                use p384::ecdsa::{VerifyingKey, signature::Verifier};
                use p384::EncodedPoint;

                let point = EncodedPoint::from_affine_coordinates(
                    x.into(),
                    y.into(),
                    false,
                );
                let verifying_key = VerifyingKey::from_encoded_point(&point)
                    .map_err(|e| JsNativeError::error().with_message(format!("Invalid key: {}", e)))?;

                let sig = p384::ecdsa::Signature::from_slice(signature)
                    .map_err(|e| JsNativeError::error().with_message(format!("Invalid signature: {}", e)))?;

                Ok(verifying_key.verify(data, &sig).is_ok())
            }
            _ => Err(JsNativeError::typ()
                .with_message(format!("Unsupported curve: {}", curve))
                .into()),
        }
    }

    // ========================================================================
    // deriveKey() / deriveBits()
    // ========================================================================

    /// `subtle.deriveKey(algorithm, baseKey, derivedKeyAlgorithm, extractable, keyUsages)`
    fn derive_key(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let algorithm_val = args.get_or_undefined(0);
        let base_key = get_crypto_key_data(args.get_or_undefined(1))?;
        let derived_algorithm = Algorithm::from_js_value(args.get_or_undefined(2), context)?;
        let extractable = args.get_or_undefined(3).to_boolean();
        let usages = parse_key_usages(args.get_or_undefined(4), context)?;

        if !base_key.can_use(KeyUsage::DeriveKey) {
            return Err(JsNativeError::typ()
                .with_message("Key does not support key derivation")
                .into());
        }

        // Determine required key length from derived algorithm
        let length = match &derived_algorithm {
            Algorithm::AesGcm { length } | Algorithm::AesCbc { length } | Algorithm::AesCtr { length } => {
                *length as usize
            }
            Algorithm::Hmac { length, hash } => {
                length.unwrap_or_else(|| match hash.as_ref() {
                    Algorithm::Sha256 => 256,
                    Algorithm::Sha384 => 384,
                    Algorithm::Sha512 => 512,
                    _ => 256,
                }) as usize
            }
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("Cannot determine key length for derived algorithm")
                    .into())
            }
        };

        let derived_bits = Self::derive_bits_sync(algorithm_val, &base_key, length, context)?;

        // Create key from derived bits
        let key_data = CryptoKeyData::new(
            KeyType::Secret,
            extractable,
            derived_algorithm,
            usages,
            KeyMaterial::Symmetric(Arc::new(Zeroizing::new(derived_bits))),
        );

        let promise = JsPromise::resolve(key_data.to_js_object(context)?, context);
        Ok(promise.into())
    }

    /// `subtle.deriveBits(algorithm, baseKey, length)`
    fn derive_bits(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let algorithm_val = args.get_or_undefined(0);
        let base_key = get_crypto_key_data(args.get_or_undefined(1))?;
        let length = args.get_or_undefined(2).to_u32(context)? as usize;

        if !base_key.can_use(KeyUsage::DeriveBits) {
            return Err(JsNativeError::typ()
                .with_message("Key does not support bits derivation")
                .into());
        }

        let result = Self::derive_bits_sync(algorithm_val, &base_key, length, context)?;

        let array_buffer = Self::create_array_buffer(&result, context)?;
        let promise = JsPromise::resolve(array_buffer, context);
        Ok(promise.into())
    }

    fn derive_bits_sync(
        algorithm_val: &JsValue,
        base_key: &CryptoKeyData,
        length: usize,
        context: &mut Context,
    ) -> JsResult<Vec<u8>> {
        let algorithm_obj = algorithm_val.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Algorithm must be an object")
        })?;

        let name = algorithm_obj
            .get(js_string!("name"), context)?
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("Algorithm name required"))?
            .to_std_string_escaped();

        match name.to_uppercase().as_str() {
            "PBKDF2" => {
                let salt = Self::get_algorithm_param_buffer(&algorithm_obj, "salt", context)?;
                let iterations = algorithm_obj
                    .get(js_string!("iterations"), context)?
                    .to_u32(context)?;
                let hash = Algorithm::from_js_value(
                    &algorithm_obj.get(js_string!("hash"), context)?,
                    context,
                )?;

                Self::pbkdf2_derive(base_key, &salt, iterations, &hash, length)
            }
            "HKDF" => {
                let salt = Self::get_algorithm_param_buffer(&algorithm_obj, "salt", context)?;
                let info = Self::get_algorithm_param_buffer(&algorithm_obj, "info", context)?;
                let hash = Algorithm::from_js_value(
                    &algorithm_obj.get(js_string!("hash"), context)?,
                    context,
                )?;

                Self::hkdf_derive(base_key, &salt, &info, &hash, length)
            }
            _ => Err(JsNativeError::typ()
                .with_message(format!("Key derivation algorithm {} not supported", name))
                .into()),
        }
    }

    fn pbkdf2_derive(
        key: &CryptoKeyData,
        salt: &[u8],
        iterations: u32,
        hash: &Algorithm,
        length: usize,
    ) -> JsResult<Vec<u8>> {
        use pbkdf2::pbkdf2_hmac;
        use sha1::Sha1;
        use sha2::{Sha256, Sha384, Sha512};

        let password = match &key.material {
            KeyMaterial::Symmetric(k) => k.as_ref().as_slice(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("Invalid key type for PBKDF2")
                    .into())
            }
        };

        let mut output = vec![0u8; length / 8];

        match hash {
            Algorithm::Sha1 => {
                pbkdf2_hmac::<Sha1>(password, salt, iterations, &mut output);
            }
            Algorithm::Sha256 => {
                pbkdf2_hmac::<Sha256>(password, salt, iterations, &mut output);
            }
            Algorithm::Sha384 => {
                pbkdf2_hmac::<Sha384>(password, salt, iterations, &mut output);
            }
            Algorithm::Sha512 => {
                pbkdf2_hmac::<Sha512>(password, salt, iterations, &mut output);
            }
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("Unsupported hash for PBKDF2")
                    .into())
            }
        }

        Ok(output)
    }

    fn hkdf_derive(
        key: &CryptoKeyData,
        salt: &[u8],
        info: &[u8],
        hash: &Algorithm,
        length: usize,
    ) -> JsResult<Vec<u8>> {
        use hkdf::Hkdf;
        use sha1::Sha1;
        use sha2::{Sha256, Sha384, Sha512};

        let ikm = match &key.material {
            KeyMaterial::Symmetric(k) => k.as_ref().as_slice(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("Invalid key type for HKDF")
                    .into())
            }
        };

        let mut output = vec![0u8; length / 8];

        match hash {
            Algorithm::Sha1 => {
                let hk = Hkdf::<Sha1>::new(Some(salt), ikm);
                hk.expand(info, &mut output)
                    .map_err(|e| JsNativeError::error().with_message(format!("HKDF error: {}", e)))?;
            }
            Algorithm::Sha256 => {
                let hk = Hkdf::<Sha256>::new(Some(salt), ikm);
                hk.expand(info, &mut output)
                    .map_err(|e| JsNativeError::error().with_message(format!("HKDF error: {}", e)))?;
            }
            Algorithm::Sha384 => {
                let hk = Hkdf::<Sha384>::new(Some(salt), ikm);
                hk.expand(info, &mut output)
                    .map_err(|e| JsNativeError::error().with_message(format!("HKDF error: {}", e)))?;
            }
            Algorithm::Sha512 => {
                let hk = Hkdf::<Sha512>::new(Some(salt), ikm);
                hk.expand(info, &mut output)
                    .map_err(|e| JsNativeError::error().with_message(format!("HKDF error: {}", e)))?;
            }
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("Unsupported hash for HKDF")
                    .into())
            }
        }

        Ok(output)
    }

    // ========================================================================
    // wrapKey() / unwrapKey()
    // ========================================================================

    /// `subtle.wrapKey(format, key, wrappingKey, wrapAlgorithm)`
    fn wrap_key(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let format = args
            .get_or_undefined(0)
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("format must be a string"))?
            .to_std_string_escaped();

        let key = get_crypto_key_data(args.get_or_undefined(1))?;
        let wrapping_key = get_crypto_key_data(args.get_or_undefined(2))?;
        let wrap_algorithm = args.get_or_undefined(3);

        if !key.extractable {
            return Err(JsNativeError::typ()
                .with_message("Key is not extractable")
                .into());
        }

        if !wrapping_key.can_use(KeyUsage::WrapKey) {
            return Err(JsNativeError::typ()
                .with_message("Wrapping key does not support key wrapping")
                .into());
        }

        // Export key first
        let exported = Self::export_key_sync(&format, &key, context)?;
        let exported_bytes = if let Some(exported_obj) = exported.as_object() {
            // If JWK, serialize to JSON
            if format == "jwk" {
                let json = serde_json::to_vec(&Self::js_object_to_json(&exported_obj, context)?)
                    .map_err(|e| JsNativeError::error().with_message(format!("JSON error: {}", e)))?;
                json
            } else {
                // ArrayBuffer
                Self::get_buffer_source(&exported, context)?
            }
        } else {
            return Err(JsNativeError::typ()
                .with_message("Export failed")
                .into());
        };

        // Encrypt the exported key
        let wrapped = Self::encrypt_sync(wrap_algorithm, &wrapping_key, &exported_bytes, context)?;

        let array_buffer = Self::create_array_buffer(&wrapped, context)?;
        let promise = JsPromise::resolve(array_buffer, context);
        Ok(promise.into())
    }

    /// `subtle.unwrapKey(format, wrappedKey, unwrappingKey, unwrapAlgorithm, unwrappedKeyAlgorithm, extractable, keyUsages)`
    fn unwrap_key(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let format = args
            .get_or_undefined(0)
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("format must be a string"))?
            .to_std_string_escaped();

        let wrapped_key = Self::get_buffer_source(args.get_or_undefined(1), context)?;
        let unwrapping_key = get_crypto_key_data(args.get_or_undefined(2))?;
        let unwrap_algorithm = args.get_or_undefined(3);
        let unwrapped_algorithm = Algorithm::from_js_value(args.get_or_undefined(4), context)?;
        let extractable = args.get_or_undefined(5).to_boolean();
        let usages = parse_key_usages(args.get_or_undefined(6), context)?;

        if !unwrapping_key.can_use(KeyUsage::UnwrapKey) {
            return Err(JsNativeError::typ()
                .with_message("Key does not support key unwrapping")
                .into());
        }

        // Decrypt the wrapped key
        let unwrapped_bytes =
            Self::decrypt_sync(unwrap_algorithm, &unwrapping_key, &wrapped_key, context)?;

        // Import the unwrapped key
        let key_data_val = Self::create_array_buffer(&unwrapped_bytes, context)?;
        let result = Self::import_key_sync(
            &format,
            &key_data_val.into(),
            unwrapped_algorithm,
            extractable,
            usages,
            context,
        )?;

        let promise = JsPromise::resolve(result, context);
        Ok(promise.into())
    }

    // ========================================================================
    // Helper functions
    // ========================================================================

    /// Extract bytes from ArrayBuffer, TypedArray, or DataView
    fn get_buffer_source(value: &JsValue, context: &mut Context) -> JsResult<Vec<u8>> {
        if value.is_undefined() || value.is_null() {
            return Err(JsNativeError::typ()
                .with_message("Data must be provided")
                .into());
        }

        let obj = value.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Data must be an ArrayBuffer or TypedArray")
        })?;

        // Try to get byteLength to determine size
        let byte_length = obj
            .get(js_string!("byteLength"), context)?
            .to_u32(context)?;

        // Try to read bytes
        let mut bytes = Vec::with_capacity(byte_length as usize);

        // Check if it has a buffer property (TypedArray/DataView)
        let buffer_val = obj.get(js_string!("buffer"), context)?;
        let _source_obj = if buffer_val.is_undefined() {
            obj.clone()
        } else {
            buffer_val.as_object().map(|o| o.clone()).unwrap_or_else(|| obj.clone())
        };

        // Get byteOffset if present
        let byte_offset = obj
            .get(js_string!("byteOffset"), context)
            .unwrap_or(JsValue::from(0))
            .to_u32(context)
            .unwrap_or(0);

        // Try to access as Uint8Array view
        let length = obj
            .get(js_string!("length"), context)?
            .to_u32(context)?;

        for i in 0..length {
            let val = obj.get(i, context)?.to_u32(context)? as u8;
            bytes.push(val);
        }

        Ok(bytes)
    }

    /// Get algorithm parameter as buffer
    fn get_algorithm_param_buffer(
        obj: &JsObject,
        name: &str,
        context: &mut Context,
    ) -> JsResult<Vec<u8>> {
        let val = obj.get(js_string!(name), context)?;
        Self::get_buffer_source(&val, context)
    }

    /// Create ArrayBuffer from bytes using Uint8Array approach
    fn create_array_buffer(bytes: &[u8], context: &mut Context) -> JsResult<JsObject> {
        // Create Uint8Array first, then extract its buffer
        let byte_values: Vec<JsValue> = bytes.iter().map(|&b| JsValue::from(b)).collect();

        let uint8array_constructor = context.intrinsics().constructors().typed_uint8_array().constructor();

        // Create a JS array with the byte values
        let js_array = boa_engine::object::builtins::JsArray::new(context);
        for (i, val) in byte_values.into_iter().enumerate() {
            js_array.set(i as u32, val, false, context)?;
        }

        // Construct Uint8Array from the array - construct returns JsResult<JsObject>
        let uint8array_obj = uint8array_constructor.construct(
            &[js_array.into()],
            Some(&uint8array_constructor),
            context
        )?;

        // Get the buffer property from the Uint8Array
        let buffer = uint8array_obj.get(js_string!("buffer"), context)?;
        if let Some(buf_obj) = buffer.as_object() {
            return Ok(buf_obj);
        }

        // Fallback: create ArrayBuffer directly
        let array_buffer_constructor = context.intrinsics().constructors().array_buffer().constructor();
        let length = JsValue::from(bytes.len());
        let array_buffer_obj = array_buffer_constructor.construct(
            &[length],
            Some(&array_buffer_constructor),
            context
        )?;

        Ok(array_buffer_obj)
    }

    /// Convert JsObject to serde_json::Value
    fn js_object_to_json(
        obj: &JsObject,
        context: &mut Context,
    ) -> JsResult<serde_json::Value> {
        // Simple conversion for JWK export
        let mut map = serde_json::Map::new();

        // Get enumerable own properties using standard property enumeration
        let keys = obj.own_property_keys(context)?;
        for key in keys {
            // Convert PropertyKey to string
            let key_str = key.to_string();
            let val = obj.get(key.clone(), context)?;
            let json_val = Self::js_value_to_json(&val, context)?;
            map.insert(key_str, json_val);
        }

        Ok(serde_json::Value::Object(map))
    }

    fn js_value_to_json(
        val: &JsValue,
        context: &mut Context,
    ) -> JsResult<serde_json::Value> {
        if val.is_undefined() || val.is_null() {
            Ok(serde_json::Value::Null)
        } else if let Some(b) = val.as_boolean() {
            Ok(serde_json::Value::Bool(b))
        } else if let Some(n) = val.as_number() {
            Ok(serde_json::json!(n))
        } else if let Some(s) = val.as_string() {
            Ok(serde_json::Value::String(s.to_std_string_escaped()))
        } else if let Some(obj) = val.as_object() {
            // Check if array
            let length_val = obj.get(js_string!("length"), context)?;
            if !length_val.is_undefined() {
                let length = length_val.to_u32(context)?;
                let mut arr = Vec::with_capacity(length as usize);
                for i in 0..length {
                    let item = obj.get(i, context)?;
                    arr.push(Self::js_value_to_json(&item, context)?);
                }
                Ok(serde_json::Value::Array(arr))
            } else {
                Self::js_object_to_json(&obj, context)
            }
        } else {
            Ok(serde_json::Value::Null)
        }
    }
}
