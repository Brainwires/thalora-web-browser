//! CryptoKey - Represents a cryptographic key
//!
//! The CryptoKey interface represents a cryptographic key obtained from one of the
//! SubtleCrypto methods generateKey(), deriveKey(), importKey(), or unwrapKey().
//!
//! More information:
//! - [W3C Web Crypto API Specification](https://w3c.github.io/webcrypto/#cryptokey-interface)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/CryptoKey)

use std::sync::Arc;

use boa_engine::{
    Context, JsData, JsNativeError, JsObject, JsResult, JsValue,
    js_string, object::ObjectInitializer, property::Attribute,
    NativeFunction,
};
use boa_gc::{Finalize, Trace};
use zeroize::Zeroizing;

/// Key type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    /// Symmetric key (for AES, HMAC, etc.)
    Secret,
    /// Public key of an asymmetric key pair
    Public,
    /// Private key of an asymmetric key pair
    Private,
}

impl KeyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            KeyType::Secret => "secret",
            KeyType::Public => "public",
            KeyType::Private => "private",
        }
    }
}

/// Key usage enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyUsage {
    Encrypt,
    Decrypt,
    Sign,
    Verify,
    DeriveKey,
    DeriveBits,
    WrapKey,
    UnwrapKey,
}

impl KeyUsage {
    pub fn as_str(&self) -> &'static str {
        match self {
            KeyUsage::Encrypt => "encrypt",
            KeyUsage::Decrypt => "decrypt",
            KeyUsage::Sign => "sign",
            KeyUsage::Verify => "verify",
            KeyUsage::DeriveKey => "deriveKey",
            KeyUsage::DeriveBits => "deriveBits",
            KeyUsage::WrapKey => "wrapKey",
            KeyUsage::UnwrapKey => "unwrapKey",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "encrypt" => Some(KeyUsage::Encrypt),
            "decrypt" => Some(KeyUsage::Decrypt),
            "sign" => Some(KeyUsage::Sign),
            "verify" => Some(KeyUsage::Verify),
            "deriveKey" => Some(KeyUsage::DeriveKey),
            "deriveBits" => Some(KeyUsage::DeriveBits),
            "wrapKey" => Some(KeyUsage::WrapKey),
            "unwrapKey" => Some(KeyUsage::UnwrapKey),
            _ => None,
        }
    }
}

/// Algorithm identifier with parameters
#[derive(Debug, Clone)]
pub enum Algorithm {
    // Digest algorithms
    Sha1,
    Sha256,
    Sha384,
    Sha512,

    // Symmetric encryption
    AesGcm { length: u16 },
    AesCbc { length: u16 },
    AesCtr { length: u16 },

    // Asymmetric encryption
    RsaOaep {
        modulus_length: u32,
        public_exponent: Vec<u8>,
        hash: Box<Algorithm>,
    },
    RsaPss {
        modulus_length: u32,
        public_exponent: Vec<u8>,
        hash: Box<Algorithm>,
    },
    RsassaPkcs1v15 {
        modulus_length: u32,
        public_exponent: Vec<u8>,
        hash: Box<Algorithm>,
    },

    // Elliptic curve
    Ecdsa { named_curve: String },
    Ecdh { named_curve: String },

    // Key derivation
    Pbkdf2,
    Hkdf,

    // Message authentication
    Hmac { hash: Box<Algorithm>, length: Option<u32> },
}

impl Algorithm {
    pub fn name(&self) -> &'static str {
        match self {
            Algorithm::Sha1 => "SHA-1",
            Algorithm::Sha256 => "SHA-256",
            Algorithm::Sha384 => "SHA-384",
            Algorithm::Sha512 => "SHA-512",
            Algorithm::AesGcm { .. } => "AES-GCM",
            Algorithm::AesCbc { .. } => "AES-CBC",
            Algorithm::AesCtr { .. } => "AES-CTR",
            Algorithm::RsaOaep { .. } => "RSA-OAEP",
            Algorithm::RsaPss { .. } => "RSA-PSS",
            Algorithm::RsassaPkcs1v15 { .. } => "RSASSA-PKCS1-v1_5",
            Algorithm::Ecdsa { .. } => "ECDSA",
            Algorithm::Ecdh { .. } => "ECDH",
            Algorithm::Pbkdf2 => "PBKDF2",
            Algorithm::Hkdf => "HKDF",
            Algorithm::Hmac { .. } => "HMAC",
        }
    }

    /// Parse algorithm from string or object
    pub fn from_js_value(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        // If it's a string, it's just the algorithm name
        if let Some(s) = value.as_string() {
            let name = s.to_std_string_escaped();
            return Self::from_name(&name);
        }

        // If it's an object, extract the name and parameters
        let obj = value.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Algorithm must be a string or object")
        })?;

        let name_val = obj.get(js_string!("name"), context)?;
        let name = name_val
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("Algorithm name must be a string"))?
            .to_std_string_escaped();

        match name.to_uppercase().as_str() {
            "SHA-1" => Ok(Algorithm::Sha1),
            "SHA-256" => Ok(Algorithm::Sha256),
            "SHA-384" => Ok(Algorithm::Sha384),
            "SHA-512" => Ok(Algorithm::Sha512),

            "AES-GCM" => {
                let length = obj
                    .get(js_string!("length"), context)?
                    .to_u32(context)? as u16;
                Ok(Algorithm::AesGcm { length })
            }

            "AES-CBC" => {
                let length = obj
                    .get(js_string!("length"), context)?
                    .to_u32(context)? as u16;
                Ok(Algorithm::AesCbc { length })
            }

            "AES-CTR" => {
                let length = obj
                    .get(js_string!("length"), context)?
                    .to_u32(context)? as u16;
                Ok(Algorithm::AesCtr { length })
            }

            "RSA-OAEP" => {
                let modulus_length = obj
                    .get(js_string!("modulusLength"), context)?
                    .to_u32(context)?;
                let public_exponent = Self::get_public_exponent(&obj, context)?;
                let hash = Self::get_hash_algorithm(&obj, context)?;
                Ok(Algorithm::RsaOaep {
                    modulus_length,
                    public_exponent,
                    hash: Box::new(hash),
                })
            }

            "RSA-PSS" => {
                let modulus_length = obj
                    .get(js_string!("modulusLength"), context)?
                    .to_u32(context)?;
                let public_exponent = Self::get_public_exponent(&obj, context)?;
                let hash = Self::get_hash_algorithm(&obj, context)?;
                Ok(Algorithm::RsaPss {
                    modulus_length,
                    public_exponent,
                    hash: Box::new(hash),
                })
            }

            "RSASSA-PKCS1-V1_5" | "RSASSA-PKCS1-V1.5" => {
                let modulus_length = obj
                    .get(js_string!("modulusLength"), context)?
                    .to_u32(context)?;
                let public_exponent = Self::get_public_exponent(&obj, context)?;
                let hash = Self::get_hash_algorithm(&obj, context)?;
                Ok(Algorithm::RsassaPkcs1v15 {
                    modulus_length,
                    public_exponent,
                    hash: Box::new(hash),
                })
            }

            "ECDSA" => {
                let named_curve = obj
                    .get(js_string!("namedCurve"), context)?
                    .as_string()
                    .ok_or_else(|| {
                        JsNativeError::typ().with_message("namedCurve must be a string")
                    })?
                    .to_std_string_escaped();
                Ok(Algorithm::Ecdsa { named_curve })
            }

            "ECDH" => {
                let named_curve = obj
                    .get(js_string!("namedCurve"), context)?
                    .as_string()
                    .ok_or_else(|| {
                        JsNativeError::typ().with_message("namedCurve must be a string")
                    })?
                    .to_std_string_escaped();
                Ok(Algorithm::Ecdh { named_curve })
            }

            "PBKDF2" => Ok(Algorithm::Pbkdf2),
            "HKDF" => Ok(Algorithm::Hkdf),

            "HMAC" => {
                let hash = Self::get_hash_algorithm(&obj, context)?;
                let length = obj.get(js_string!("length"), context)?;
                let length = if length.is_undefined() {
                    None
                } else {
                    Some(length.to_u32(context)?)
                };
                Ok(Algorithm::Hmac {
                    hash: Box::new(hash),
                    length,
                })
            }

            _ => Err(JsNativeError::typ()
                .with_message(format!("Unsupported algorithm: {}", name))
                .into()),
        }
    }

    fn from_name(name: &str) -> JsResult<Self> {
        match name.to_uppercase().as_str() {
            "SHA-1" => Ok(Algorithm::Sha1),
            "SHA-256" => Ok(Algorithm::Sha256),
            "SHA-384" => Ok(Algorithm::Sha384),
            "SHA-512" => Ok(Algorithm::Sha512),
            _ => Err(JsNativeError::typ()
                .with_message(format!(
                    "Algorithm '{}' requires additional parameters",
                    name
                ))
                .into()),
        }
    }

    fn get_public_exponent(obj: &JsObject, context: &mut Context) -> JsResult<Vec<u8>> {
        let exp_val = obj.get(js_string!("publicExponent"), context)?;
        if exp_val.is_undefined() {
            // Default to 65537 (0x010001)
            return Ok(vec![0x01, 0x00, 0x01]);
        }

        // Try to get as Uint8Array
        let exp_obj = exp_val.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("publicExponent must be a Uint8Array")
        })?;

        let length = exp_obj.get(js_string!("length"), context)?.to_u32(context)?;
        let mut bytes = Vec::with_capacity(length as usize);
        for i in 0..length {
            let byte = exp_obj.get(i, context)?.to_u32(context)? as u8;
            bytes.push(byte);
        }
        Ok(bytes)
    }

    fn get_hash_algorithm(obj: &JsObject, context: &mut Context) -> JsResult<Algorithm> {
        let hash_val = obj.get(js_string!("hash"), context)?;
        if hash_val.is_undefined() {
            return Ok(Algorithm::Sha256); // Default
        }
        Self::from_js_value(&hash_val, context)
    }

    /// Convert algorithm to JS object representation
    pub fn to_js_object(&self, context: &mut Context) -> JsResult<JsObject> {
        let obj = ObjectInitializer::new(context).build();

        obj.set(js_string!("name"), js_string!(self.name()), false, context)?;

        match self {
            Algorithm::AesGcm { length }
            | Algorithm::AesCbc { length }
            | Algorithm::AesCtr { length } => {
                obj.set(js_string!("length"), JsValue::from(*length as u32), false, context)?;
            }

            Algorithm::RsaOaep { modulus_length, hash, .. }
            | Algorithm::RsaPss { modulus_length, hash, .. }
            | Algorithm::RsassaPkcs1v15 { modulus_length, hash, .. } => {
                obj.set(
                    js_string!("modulusLength"),
                    JsValue::from(*modulus_length),
                    false,
                    context,
                )?;
                obj.set(
                    js_string!("hash"),
                    hash.to_js_object(context)?,
                    false,
                    context,
                )?;
            }

            Algorithm::Ecdsa { named_curve } | Algorithm::Ecdh { named_curve } => {
                obj.set(
                    js_string!("namedCurve"),
                    js_string!(named_curve.as_str()),
                    false,
                    context,
                )?;
            }

            Algorithm::Hmac { hash, length } => {
                obj.set(
                    js_string!("hash"),
                    hash.to_js_object(context)?,
                    false,
                    context,
                )?;
                if let Some(len) = length {
                    obj.set(js_string!("length"), JsValue::from(*len), false, context)?;
                }
            }

            _ => {}
        }

        Ok(obj)
    }
}

/// Internal key material storage
#[derive(Clone)]
pub enum KeyMaterial {
    /// Raw symmetric key bytes (zeroized on drop)
    Symmetric(Arc<Zeroizing<Vec<u8>>>),
    /// RSA key pair components
    RsaPublic {
        n: Vec<u8>,
        e: Vec<u8>,
    },
    RsaPrivate {
        n: Vec<u8>,
        e: Vec<u8>,
        d: Arc<Zeroizing<Vec<u8>>>,
        p: Arc<Zeroizing<Vec<u8>>>,
        q: Arc<Zeroizing<Vec<u8>>>,
    },
    /// EC key components
    EcPublic {
        x: Vec<u8>,
        y: Vec<u8>,
    },
    EcPrivate {
        x: Vec<u8>,
        y: Vec<u8>,
        d: Arc<Zeroizing<Vec<u8>>>,
    },
}

impl std::fmt::Debug for KeyMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyMaterial::Symmetric(_) => write!(f, "KeyMaterial::Symmetric([REDACTED])"),
            KeyMaterial::RsaPublic { .. } => write!(f, "KeyMaterial::RsaPublic {{ ... }}"),
            KeyMaterial::RsaPrivate { .. } => write!(f, "KeyMaterial::RsaPrivate {{ [REDACTED] }}"),
            KeyMaterial::EcPublic { .. } => write!(f, "KeyMaterial::EcPublic {{ ... }}"),
            KeyMaterial::EcPrivate { .. } => write!(f, "KeyMaterial::EcPrivate {{ [REDACTED] }}"),
        }
    }
}

/// CryptoKey internal data
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct CryptoKeyData {
    /// Key type
    #[unsafe_ignore_trace]
    pub key_type: KeyType,
    /// Whether key is extractable
    #[unsafe_ignore_trace]
    pub extractable: bool,
    /// Algorithm associated with the key
    #[unsafe_ignore_trace]
    pub algorithm: Algorithm,
    /// Allowed usages for this key
    #[unsafe_ignore_trace]
    pub usages: Vec<KeyUsage>,
    /// Internal key material (not accessible from JS)
    #[unsafe_ignore_trace]
    pub material: KeyMaterial,
}

impl CryptoKeyData {
    /// Create a new CryptoKey
    pub fn new(
        key_type: KeyType,
        extractable: bool,
        algorithm: Algorithm,
        usages: Vec<KeyUsage>,
        material: KeyMaterial,
    ) -> Self {
        Self {
            key_type,
            extractable,
            algorithm,
            usages,
            material,
        }
    }

    /// Check if this key can be used for a specific operation
    pub fn can_use(&self, usage: KeyUsage) -> bool {
        self.usages.contains(&usage)
    }

    /// Create a JS object representing this CryptoKey
    pub fn to_js_object(&self, context: &mut Context) -> JsResult<JsObject> {
        let obj = JsObject::from_proto_and_data(None, self.clone());

        // Add type property
        obj.set(
            js_string!("type"),
            js_string!(self.key_type.as_str()),
            false,
            context,
        )?;

        // Add extractable property
        obj.set(
            js_string!("extractable"),
            JsValue::from(self.extractable),
            false,
            context,
        )?;

        // Add algorithm property
        obj.set(
            js_string!("algorithm"),
            self.algorithm.to_js_object(context)?,
            false,
            context,
        )?;

        // Add usages array
        let usages: Vec<JsValue> = self
            .usages
            .iter()
            .map(|u| JsValue::from(js_string!(u.as_str())))
            .collect();
        let usages_array =
            boa_engine::object::builtins::JsArray::from_iter(usages, context);
        obj.set(js_string!("usages"), usages_array, false, context)?;

        Ok(obj)
    }
}

/// Parse key usages from JS array
pub fn parse_key_usages(value: &JsValue, context: &mut Context) -> JsResult<Vec<KeyUsage>> {
    let array = value.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("keyUsages must be an array")
    })?;

    let length = array.get(js_string!("length"), context)?.to_u32(context)?;
    let mut usages = Vec::with_capacity(length as usize);

    for i in 0..length {
        let usage_val = array.get(i, context)?;
        let usage_str = usage_val
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("keyUsage must be a string"))?
            .to_std_string_escaped();

        let usage = KeyUsage::from_str(&usage_str).ok_or_else(|| {
            JsNativeError::typ().with_message(format!("Invalid key usage: {}", usage_str))
        })?;

        usages.push(usage);
    }

    Ok(usages)
}

/// Extract CryptoKeyData from a JsValue
pub fn get_crypto_key_data(value: &JsValue) -> JsResult<CryptoKeyData> {
    let obj = value.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Value is not a CryptoKey")
    })?;

    let data = obj.downcast_ref::<CryptoKeyData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Value is not a CryptoKey")
    })?;

    Ok(data.clone())
}
