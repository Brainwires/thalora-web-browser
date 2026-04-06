//! Encoding Web APIs: TextEncoder, TextDecoder, atob, btoa
//!
//! Implementation of the Encoding Standard APIs
//! https://encoding.spec.whatwg.org/

use base64::{Engine as _, engine::general_purpose};
use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, NativeFunction,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{
        FunctionObjectBuilder, JsObject, builtins::JsUint8Array,
        internal_methods::get_prototype_from_constructor,
    },
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
};
use boa_gc::{Finalize, Trace};

// ============================================================================
// TextEncoder
// ============================================================================

/// JavaScript `TextEncoder` builtin implementation.
/// Always encodes to UTF-8 per spec.
#[derive(Debug, Copy, Clone)]
pub struct TextEncoder;

impl IntrinsicObject for TextEncoder {
    fn init(realm: &Realm) {
        let encoding_func = BuiltInBuilder::callable(realm, get_encoding)
            .name(js_string!("get encoding"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("encoding"),
                Some(encoding_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(encode, js_string!("encode"), 1)
            .method(encode_into, js_string!("encodeInto"), 2)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for TextEncoder {
    const NAME: JsString = StaticJsStrings::TEXT_ENCODER;
}

impl BuiltInConstructor for TextEncoder {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::text_encoder;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::text_encoder,
            context,
        )?;

        let text_encoder = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            TextEncoderData,
        );

        Ok(text_encoder.into())
    }
}

/// Internal data for TextEncoder objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct TextEncoderData;

/// `TextEncoder.prototype.encoding` getter - always returns "utf-8"
fn get_encoding(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Ok(js_string!("utf-8").into())
}

/// `TextEncoder.prototype.encode(input)`
fn encode(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("TextEncoder.prototype.encode called on non-object")
    })?;

    this_obj.downcast_ref::<TextEncoderData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("TextEncoder.prototype.encode called on non-TextEncoder object")
    })?;

    let input = args.get_or_undefined(0);
    let input_str = input.to_string(context)?;
    let input_string = input_str.to_std_string_escaped();

    let bytes: Vec<u8> = input_string.as_bytes().to_vec();

    // Create a Uint8Array with the encoded bytes
    let uint8_array = JsUint8Array::from_iter(bytes, context)?;

    Ok(uint8_array.into())
}

/// `TextEncoder.prototype.encodeInto(source, destination)`
fn encode_into(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("TextEncoder.prototype.encodeInto called on non-object")
    })?;

    this_obj.downcast_ref::<TextEncoderData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("TextEncoder.prototype.encodeInto called on non-TextEncoder object")
    })?;

    let source = args.get_or_undefined(0);
    let destination = args.get_or_undefined(1);

    let source_str = source.to_string(context)?;
    let source_string = source_str.to_std_string_escaped();

    let dest_obj = destination.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("encodeInto: destination must be a Uint8Array")
    })?;

    // Get the bytes from source
    let bytes = source_string.as_bytes();

    // Get destination length
    let dest_length = dest_obj
        .get(js_string!("length"), context)?
        .to_number(context)? as usize;

    // Calculate how many bytes we can write
    let bytes_to_write = bytes.len().min(dest_length);

    // Write bytes to destination
    for (i, &byte) in bytes.iter().take(bytes_to_write).enumerate() {
        dest_obj.set(i as u32, JsValue::from(byte as i32), false, context)?;
    }

    // Return result object { read: number, written: number }
    let result = JsObject::with_null_proto();
    result.set(
        js_string!("read"),
        source_string.chars().count() as i32,
        false,
        context,
    )?;
    result.set(js_string!("written"), bytes_to_write as i32, false, context)?;

    Ok(result.into())
}

// ============================================================================
// TextDecoder
// ============================================================================

/// JavaScript `TextDecoder` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct TextDecoder;

impl IntrinsicObject for TextDecoder {
    fn init(realm: &Realm) {
        let encoding_func = BuiltInBuilder::callable(realm, get_decoder_encoding)
            .name(js_string!("get encoding"))
            .build();

        let fatal_func = BuiltInBuilder::callable(realm, get_fatal)
            .name(js_string!("get fatal"))
            .build();

        let ignore_bom_func = BuiltInBuilder::callable(realm, get_ignore_bom)
            .name(js_string!("get ignoreBOM"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("encoding"),
                Some(encoding_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("fatal"),
                Some(fatal_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("ignoreBOM"),
                Some(ignore_bom_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(decode, js_string!("decode"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for TextDecoder {
    const NAME: JsString = StaticJsStrings::TEXT_DECODER;
}

impl BuiltInConstructor for TextDecoder {
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::text_decoder;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::text_decoder,
            context,
        )?;

        // Get encoding label (default: "utf-8")
        let encoding = if let Some(label) = args.first() {
            if !label.is_undefined() {
                let label_str = label.to_string(context)?;
                let encoding = label_str.to_std_string_escaped().to_lowercase();
                // Validate encoding - we only support utf-8 variants
                match encoding.as_str() {
                    "utf-8" | "utf8" | "unicode-1-1-utf-8" => "utf-8".to_string(),
                    _ => {
                        return Err(JsNativeError::range()
                            .with_message(format!(
                                "The encoding label '{}' is not supported",
                                encoding
                            ))
                            .into());
                    }
                }
            } else {
                "utf-8".to_string()
            }
        } else {
            "utf-8".to_string()
        };

        // Get options
        let (fatal, ignore_bom) = if let Some(options) = args.get(1) {
            if let Some(options_obj) = options.as_object() {
                let fatal = options_obj.get(js_string!("fatal"), context)?.to_boolean();
                let ignore_bom = options_obj
                    .get(js_string!("ignoreBOM"), context)?
                    .to_boolean();
                (fatal, ignore_bom)
            } else {
                (false, false)
            }
        } else {
            (false, false)
        };

        let text_decoder = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            TextDecoderData {
                encoding,
                fatal,
                ignore_bom,
            },
        );

        Ok(text_decoder.into())
    }
}

/// Internal data for TextDecoder objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct TextDecoderData {
    #[unsafe_ignore_trace]
    encoding: String,
    fatal: bool,
    ignore_bom: bool,
}

/// `TextDecoder.prototype.encoding` getter
fn get_decoder_encoding(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("TextDecoder.prototype.encoding called on non-object")
    })?;

    let decoder = this_obj.downcast_ref::<TextDecoderData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("TextDecoder.prototype.encoding called on non-TextDecoder object")
    })?;

    Ok(JsString::from(decoder.encoding.as_str()).into())
}

/// `TextDecoder.prototype.fatal` getter
fn get_fatal(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("TextDecoder.prototype.fatal called on non-object")
    })?;

    let decoder = this_obj.downcast_ref::<TextDecoderData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("TextDecoder.prototype.fatal called on non-TextDecoder object")
    })?;

    Ok(decoder.fatal.into())
}

/// `TextDecoder.prototype.ignoreBOM` getter
fn get_ignore_bom(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("TextDecoder.prototype.ignoreBOM called on non-object")
    })?;

    let decoder = this_obj.downcast_ref::<TextDecoderData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("TextDecoder.prototype.ignoreBOM called on non-TextDecoder object")
    })?;

    Ok(decoder.ignore_bom.into())
}

/// `TextDecoder.prototype.decode(input, options)`
fn decode(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("TextDecoder.prototype.decode called on non-object")
    })?;

    let decoder = this_obj.downcast_ref::<TextDecoderData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("TextDecoder.prototype.decode called on non-TextDecoder object")
    })?;

    let input = args.get_or_undefined(0);

    // If no input, return empty string
    if input.is_undefined() {
        return Ok(js_string!("").into());
    }

    // Get bytes from ArrayBuffer, TypedArray, or DataView
    let bytes = extract_bytes(input, context)?;

    // Handle BOM if not ignoring
    let bytes_to_decode = if !decoder.ignore_bom && bytes.len() >= 3 {
        // Check for UTF-8 BOM (EF BB BF)
        if bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
            &bytes[3..]
        } else {
            &bytes[..]
        }
    } else {
        &bytes[..]
    };

    // Decode UTF-8
    match String::from_utf8(bytes_to_decode.to_vec()) {
        Ok(s) => Ok(JsString::from(s.as_str()).into()),
        Err(e) => {
            if decoder.fatal {
                Err(JsNativeError::typ()
                    .with_message(format!("Failed to decode: {}", e))
                    .into())
            } else {
                // Replace invalid sequences with replacement character
                let s = String::from_utf8_lossy(bytes_to_decode);
                Ok(JsString::from(s.as_ref()).into())
            }
        }
    }
}

/// Extract bytes from ArrayBuffer, TypedArray, or DataView
fn extract_bytes(value: &JsValue, context: &mut Context) -> JsResult<Vec<u8>> {
    let obj = value.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Input must be an ArrayBuffer, TypedArray, or DataView")
    })?;

    // Try to get byteLength to check if it's a buffer-like object
    if let Ok(byte_length) = obj.get(js_string!("byteLength"), context)
        && let Some(len) = byte_length.as_number()
    {
        let len = len as usize;
        let mut bytes = Vec::with_capacity(len);

        // Check if it's a TypedArray by checking for length property
        if let Ok(typed_length) = obj.get(js_string!("length"), context)
            && typed_length.is_number()
        {
            // It's a TypedArray - read indexed values
            for i in 0..len {
                let val = obj.get(i as u32, context)?;
                if let Some(num) = val.as_number() {
                    bytes.push(num as u8);
                }
            }
            return Ok(bytes);
        }

        // It's an ArrayBuffer or DataView
        // For now, return empty as we can't easily read raw bytes
        return Ok(bytes);
    }

    Err(JsNativeError::typ()
        .with_message("Input must be an ArrayBuffer, TypedArray, or DataView")
        .into())
}

// ============================================================================
// atob and btoa global functions
// ============================================================================

/// `atob(data)` - Decodes a base64 encoded string
pub fn atob(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let data = args.get_or_undefined(0);
    let data_str = data.to_string(context)?;
    let input = data_str.to_std_string_escaped();

    // Per spec, atob only accepts characters in the base64 alphabet plus whitespace
    // First, remove all ASCII whitespace
    let cleaned: String = input
        .chars()
        .filter(|c| !matches!(c, ' ' | '\t' | '\n' | '\x0C' | '\r'))
        .collect();

    // Validate characters and check for invalid characters
    for c in cleaned.chars() {
        if !c.is_ascii() {
            return Err(JsNativeError::error()
                .with_message(
                    "The string to be decoded contains characters outside of the Latin1 range",
                )
                .into());
        }
    }

    match general_purpose::STANDARD.decode(&cleaned) {
        Ok(bytes) => {
            // Convert bytes to string (Latin1 encoding per spec)
            let result: String = bytes.iter().map(|&b| b as char).collect();
            Ok(JsString::from(result.as_str()).into())
        }
        Err(_) => Err(JsNativeError::error()
            .with_message("The string to be decoded is not correctly encoded")
            .into()),
    }
}

/// `btoa(data)` - Encodes a string to base64
pub fn btoa(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let data = args.get_or_undefined(0);
    let data_str = data.to_string(context)?;
    let input = data_str.to_std_string_escaped();

    // Per spec, btoa only accepts characters with code points <= 255 (Latin1)
    let mut bytes = Vec::with_capacity(input.len());
    for c in input.chars() {
        let code = c as u32;
        if code > 255 {
            return Err(JsNativeError::error()
                .with_message(
                    "The string to be encoded contains characters outside of the Latin1 range",
                )
                .into());
        }
        bytes.push(code as u8);
    }

    let encoded = general_purpose::STANDARD.encode(&bytes);
    Ok(JsString::from(encoded.as_str()).into())
}

/// Create the atob function object
pub fn create_atob_function(context: &mut Context) -> JsResult<JsValue> {
    let func = FunctionObjectBuilder::new(context.realm(), NativeFunction::from_fn_ptr(atob))
        .name(js_string!("atob"))
        .length(1)
        .build();

    Ok(func.into())
}

/// Create the btoa function object
pub fn create_btoa_function(context: &mut Context) -> JsResult<JsValue> {
    let func = FunctionObjectBuilder::new(context.realm(), NativeFunction::from_fn_ptr(btoa))
        .name(js_string!("btoa"))
        .length(1)
        .build();

    Ok(func.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::{Context, Source};

    fn create_test_context() -> Context {
        let mut context = Context::default();
        crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
        context
    }

    #[test]
    fn test_text_encoder_exists() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes("typeof TextEncoder"))
            .unwrap();
        assert_eq!(result, JsValue::from(js_string!("function")));
    }

    #[test]
    fn test_text_encoder_encoding() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            let encoder = new TextEncoder();
            encoder.encoding;
        "#,
            ))
            .unwrap();
        assert_eq!(result, JsValue::from(js_string!("utf-8")));
    }

    #[test]
    fn test_text_encoder_encode() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            let encoder = new TextEncoder();
            let encoded = encoder.encode('hello');
            encoded instanceof Uint8Array && encoded.length === 5;
        "#,
            ))
            .unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_text_decoder_exists() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes("typeof TextDecoder"))
            .unwrap();
        assert_eq!(result, JsValue::from(js_string!("function")));
    }

    #[test]
    fn test_text_decoder_encoding() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            let decoder = new TextDecoder();
            decoder.encoding;
        "#,
            ))
            .unwrap();
        assert_eq!(result, JsValue::from(js_string!("utf-8")));
    }

    #[test]
    fn test_text_decoder_decode() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes(
                r#"
            let decoder = new TextDecoder();
            let encoder = new TextEncoder();
            let encoded = encoder.encode('hello');
            decoder.decode(encoded);
        "#,
            ))
            .unwrap();
        assert_eq!(result, JsValue::from(js_string!("hello")));
    }

    #[test]
    fn test_atob_exists() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes("typeof atob")).unwrap();
        assert_eq!(result, JsValue::from(js_string!("function")));
    }

    #[test]
    fn test_btoa_exists() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes("typeof btoa")).unwrap();
        assert_eq!(result, JsValue::from(js_string!("function")));
    }

    #[test]
    fn test_btoa_encode() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes("btoa('hello')")).unwrap();
        assert_eq!(result, JsValue::from(js_string!("aGVsbG8=")));
    }

    #[test]
    fn test_atob_decode() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes("atob('aGVsbG8=')"))
            .unwrap();
        assert_eq!(result, JsValue::from(js_string!("hello")));
    }

    #[test]
    fn test_btoa_atob_roundtrip() {
        let mut context = create_test_context();
        let result = context
            .eval(Source::from_bytes("atob(btoa('test string'))"))
            .unwrap();
        assert_eq!(result, JsValue::from(js_string!("test string")));
    }
}
