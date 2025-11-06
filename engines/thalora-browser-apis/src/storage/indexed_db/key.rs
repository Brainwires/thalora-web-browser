//! IndexedDB Key Type System
//!
//! Implements the IndexedDB key data types and comparison logic.
//!
//! Valid IndexedDB key types:
//! - Number (except NaN)
//! - String
//! - Date
//! - Binary (ArrayBuffer, typed arrays)
//! - Array (of other valid keys)
//!
//! Spec: https://w3c.github.io/IndexedDB/#key-construct

use boa_engine::{
    Context, JsNativeError, JsResult, JsString, JsValue,
    builtins::array::Array,
    js_string,
    object::JsObject,
};
use boa_gc::{Finalize, Trace};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// IndexedDB Key type
///
/// Keys are used to organize and retrieve records in object stores.
/// They must be comparable and serializable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Trace, Finalize)]
pub enum IDBKey {
    /// Number key (IEEE 754 double, excluding NaN)
    Number(f64),

    /// String key (UTF-8)
    String(String),

    /// Date key (milliseconds since Unix epoch)
    Date(f64),

    /// Binary key (bytes)
    Binary(Vec<u8>),

    /// Array key (array of other keys)
    Array(Vec<IDBKey>),
}

impl IDBKey {
    /// Convert a JavaScript value to an IndexedDB key
    ///
    /// Spec: https://w3c.github.io/IndexedDB/#convert-a-value-to-a-key
    pub fn from_js_value(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        // Handle undefined/null
        if value.is_undefined() || value.is_null() {
            return Err(JsNativeError::typ()
                .with_message("Cannot convert undefined or null to IndexedDB key")
                .into());
        }

        // Check for Number
        if let Some(num) = value.as_number() {
            if num.is_nan() {
                return Err(JsNativeError::typ()
                    .with_message("NaN is not a valid IndexedDB key")
                    .into());
            }
            return Ok(IDBKey::Number(num));
        }

        // Check for String
        if value.is_string() {
            let string = value.to_string(context)?;
            return Ok(IDBKey::String(string.to_std_string_escaped()));
        }

        // Check for Object types
        if let Some(obj) = value.as_object() {
            // Check if it's a Date object by trying to call getTime
            if let Ok(time_method) = obj.get(js_string!("getTime"), context) {
                if let Some(callable_obj) = time_method.as_object() {
                    if callable_obj.is_callable() {
                        if let Ok(result) = callable_obj.call(value, &[], context) {
                            if let Some(num) = result.as_number() {
                                if !num.is_nan() && num.is_finite() {
                                    return Ok(IDBKey::Date(num));
                                }
                            }
                        }
                    }
                }
            }

            // Check for Array
            if obj.is_array() {
                // Get array length using the "length" property
                let length_value = obj.get(js_string!("length"), context)?;
                let length = length_value.to_u32(context)? as u64;
                let mut keys = Vec::with_capacity(length as usize);

                for i in 0..length {
                    let element = obj.get(i, context)?;
                    // Recursively convert array elements
                    let key = Self::from_js_value(&element, context)?;
                    keys.push(key);
                }

                return Ok(IDBKey::Array(keys));
            }
        }

        Err(JsNativeError::typ()
            .with_message(format!("Value of type {} is not a valid IndexedDB key", value.type_of()))
            .into())
    }

    // TODO: Add Binary key support when ArrayBuffer APIs are available
    //
    // /// Check if an object is a typed array
    // fn is_typed_array(obj: &JsObject) -> bool {
    //     // Try to get the buffer property - all typed arrays have this
    //     obj.has_property(js_string!("buffer"), context)
    //         .unwrap_or(false)
    // }
    //
    // /// Extract bytes from ArrayBuffer or typed array
    // fn extract_bytes(obj: &JsObject, context: &mut Context) -> JsResult<Vec<u8>> {
    //     // Try to get byte length
    //     if let Ok(byte_length) = obj.get(js_string!("byteLength"), context) {
    //         if let Some(len) = byte_length.as_number() {
    //             let len = len as usize;
    //             let mut bytes = Vec::with_capacity(len);
    //
    //             // Try to access via indexing
    //             for i in 0..len {
    //                 if let Ok(byte_val) = obj.get(i, context) {
    //                     if let Some(byte) = byte_val.as_number() {
    //                         bytes.push(byte as u8);
    //                     }
    //                 }
    //             }
    //
    //             if bytes.len() == len {
    //                 return Ok(bytes);
    //             }
    //         }
    //     }
    //
    //     Ok(Vec::new())
    // }

    /// Convert key to bytes for storage
    ///
    /// Uses a sortable binary encoding:
    /// - Type prefix (1 byte): Number=0x01, String=0x02, Date=0x03, Binary=0x04, Array=0x05
    /// - Type-specific encoding
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        match self {
            IDBKey::Number(num) => {
                bytes.push(0x01); // Number prefix
                // Use lexicographically sortable encoding for f64
                let bits = num.to_bits();
                // Flip sign bit for proper ordering
                let encoded = if num.is_sign_negative() {
                    !bits
                } else {
                    bits ^ (1u64 << 63)
                };
                bytes.extend_from_slice(&encoded.to_be_bytes());
            },
            IDBKey::String(s) => {
                bytes.push(0x02); // String prefix
                bytes.extend_from_slice(s.as_bytes());
            },
            IDBKey::Date(ms) => {
                bytes.push(0x03); // Date prefix
                let bits = ms.to_bits();
                let encoded = if ms.is_sign_negative() {
                    !bits
                } else {
                    bits ^ (1u64 << 63)
                };
                bytes.extend_from_slice(&encoded.to_be_bytes());
            },
            IDBKey::Binary(data) => {
                bytes.push(0x04); // Binary prefix
                bytes.extend_from_slice(data);
            },
            IDBKey::Array(keys) => {
                bytes.push(0x05); // Array prefix
                // Encode length
                bytes.extend_from_slice(&(keys.len() as u32).to_be_bytes());
                // Encode each key
                for key in keys {
                    let key_bytes = key.to_bytes();
                    // Store length of each key for proper parsing
                    bytes.extend_from_slice(&(key_bytes.len() as u32).to_be_bytes());
                    bytes.extend_from_slice(&key_bytes);
                }
            },
        }

        bytes
    }

    /// Parse key from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Empty byte array".to_string());
        }

        let type_prefix = bytes[0];
        let data = &bytes[1..];

        match type_prefix {
            0x01 => {
                // Number
                if data.len() < 8 {
                    return Err("Invalid number encoding".to_string());
                }
                let mut bits_bytes = [0u8; 8];
                bits_bytes.copy_from_slice(&data[0..8]);
                let encoded = u64::from_be_bytes(bits_bytes);

                // Decode the sortable encoding
                let bits = if encoded & (1u64 << 63) != 0 {
                    encoded ^ (1u64 << 63)
                } else {
                    !encoded
                };

                Ok(IDBKey::Number(f64::from_bits(bits)))
            },
            0x02 => {
                // String
                let s = String::from_utf8(data.to_vec())
                    .map_err(|e| format!("Invalid UTF-8: {}", e))?;
                Ok(IDBKey::String(s))
            },
            0x03 => {
                // Date
                if data.len() < 8 {
                    return Err("Invalid date encoding".to_string());
                }
                let mut bits_bytes = [0u8; 8];
                bits_bytes.copy_from_slice(&data[0..8]);
                let encoded = u64::from_be_bytes(bits_bytes);

                let bits = if encoded & (1u64 << 63) != 0 {
                    encoded ^ (1u64 << 63)
                } else {
                    !encoded
                };

                Ok(IDBKey::Date(f64::from_bits(bits)))
            },
            0x04 => {
                // Binary
                Ok(IDBKey::Binary(data.to_vec()))
            },
            0x05 => {
                // Array
                if data.len() < 4 {
                    return Err("Invalid array encoding".to_string());
                }

                let mut len_bytes = [0u8; 4];
                len_bytes.copy_from_slice(&data[0..4]);
                let array_len = u32::from_be_bytes(len_bytes) as usize;

                let mut keys = Vec::with_capacity(array_len);
                let mut offset = 4;

                for _ in 0..array_len {
                    if offset + 4 > data.len() {
                        return Err("Invalid array element length".to_string());
                    }

                    let mut key_len_bytes = [0u8; 4];
                    key_len_bytes.copy_from_slice(&data[offset..offset+4]);
                    let key_len = u32::from_be_bytes(key_len_bytes) as usize;
                    offset += 4;

                    if offset + key_len > data.len() {
                        return Err("Invalid array element data".to_string());
                    }

                    let key = Self::from_bytes(&data[offset..offset+key_len])?;
                    keys.push(key);
                    offset += key_len;
                }

                Ok(IDBKey::Array(keys))
            },
            _ => Err(format!("Unknown key type prefix: {}", type_prefix)),
        }
    }

    /// Convert key back to JavaScript value
    pub fn to_js_value(&self, context: &mut Context) -> JsResult<JsValue> {
        match self {
            IDBKey::Number(num) => Ok(JsValue::from(*num)),
            IDBKey::String(s) => Ok(JsValue::from(JsString::from(s.clone()))),
            IDBKey::Date(ms) => {
                // Create a new Date object
                let date_constructor = context.intrinsics().constructors().date().constructor();
                let date = date_constructor.construct(&[JsValue::from(*ms)], None, context)?;
                Ok(date.into())
            },
            IDBKey::Binary(bytes) => {
                // Create Uint8Array
                let array = Array::array_create(bytes.len() as u64, None, context)?;
                for (i, &byte) in bytes.iter().enumerate() {
                    array.set(i, JsValue::from(byte), true, context)?;
                }
                Ok(array.into())
            },
            IDBKey::Array(keys) => {
                // Create JavaScript array
                let array = Array::array_create(keys.len() as u64, None, context)?;
                for (i, key) in keys.iter().enumerate() {
                    let value = key.to_js_value(context)?;
                    array.set(i, value, true, context)?;
                }
                Ok(array.into())
            },
        }
    }
}

/// Implement ordering for IndexedDB keys
///
/// Spec: https://w3c.github.io/IndexedDB/#key-construct
/// Order: Array < Binary < String < Date < Number
impl Ord for IDBKey {
    fn cmp(&self, other: &Self) -> Ordering {
        use IDBKey::*;

        match (self, other) {
            // Same types - compare values
            (Number(a), Number(b)) => {
                if a < b { Ordering::Less }
                else if a > b { Ordering::Greater }
                else { Ordering::Equal }
            },
            (String(a), String(b)) => a.cmp(b),
            (Date(a), Date(b)) => {
                if a < b { Ordering::Less }
                else if a > b { Ordering::Greater }
                else { Ordering::Equal }
            },
            (Binary(a), Binary(b)) => a.cmp(b),
            (Array(a), Array(b)) => a.cmp(b),

            // Different types - use IndexedDB type ordering: Number < Date < String < Binary < Array
            // Array is highest
            (Array(_), _) => Ordering::Greater,
            (_, Array(_)) => Ordering::Less,

            // Binary is higher than String, Date, Number
            (Binary(_), String(_)) | (Binary(_), Date(_)) | (Binary(_), Number(_)) => Ordering::Greater,
            (String(_), Binary(_)) | (Date(_), Binary(_)) | (Number(_), Binary(_)) => Ordering::Less,

            // String is higher than Date, Number
            (String(_), Date(_)) | (String(_), Number(_)) => Ordering::Greater,
            (Date(_), String(_)) | (Number(_), String(_)) => Ordering::Less,

            // Date is higher than Number
            (Date(_), Number(_)) => Ordering::Greater,
            (Number(_), Date(_)) => Ordering::Less,
        }
    }
}

impl PartialOrd for IDBKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for IDBKey {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_ordering() {
        let num1 = IDBKey::Number(1.0);
        let num2 = IDBKey::Number(2.0);
        let str1 = IDBKey::String("abc".to_string());
        let date1 = IDBKey::Date(1000.0);

        // Within same type
        assert!(num1 < num2);

        // Cross-type ordering per IndexedDB spec: Number < Date < String < Binary < Array
        assert!(num1 < date1);  // Number < Date
        assert!(date1 < str1);  // Date < String
    }

    #[test]
    fn test_key_serialization() {
        let key = IDBKey::String("hello".to_string());
        let bytes = key.to_bytes();
        let restored = IDBKey::from_bytes(&bytes).unwrap();
        assert_eq!(key, restored);
    }

    #[test]
    fn test_array_key() {
        let key = IDBKey::Array(vec![
            IDBKey::Number(1.0),
            IDBKey::String("test".to_string()),
        ]);
        let bytes = key.to_bytes();
        let restored = IDBKey::from_bytes(&bytes).unwrap();
        assert_eq!(key, restored);
    }

    #[test]
    fn test_type_ordering() {
        let array = IDBKey::Array(vec![IDBKey::Number(1.0)]);
        let binary = IDBKey::Binary(vec![1, 2, 3]);
        let string = IDBKey::String("test".to_string());
        let date = IDBKey::Date(1000.0);
        let number = IDBKey::Number(42.0);

        // Verify spec ordering: Number < Date < String < Binary < Array
        assert!(number < date);
        assert!(date < string);
        assert!(string < binary);
        assert!(binary < array);
    }
}
