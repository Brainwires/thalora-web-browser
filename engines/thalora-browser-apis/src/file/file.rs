//! File Web API implementation for Boa
//!
//! Native implementation of the File interface from the File API
//! https://w3c.github.io/FileAPI/#file-section
//!
//! This implements the File interface which inherits from Blob


use boa_engine::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject, BuiltInConstructor},
    object::JsObject,
    value::JsValue,
    Context, JsResult, js_string, JsNativeError,
    realm::Realm, JsString, JsData,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors}
};
use crate::file::blob::BlobData;
use boa_gc::{Finalize, Trace};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// JavaScript `File` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub struct File;

/// Internal data for File objects (extends BlobData)
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct FileData {
    // Inherits from Blob
    blob_data: BlobData,

    // File-specific properties
    name: String,
    last_modified: u64,
    webkit_relative_path: String,
}

impl FileData {
    /// Create new FileData from blob data and file properties
    pub fn new(blob_data: BlobData, name: String, last_modified: Option<u64>) -> Self {
        let last_modified = last_modified.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        });

        Self {
            blob_data,
            name,
            last_modified,
            webkit_relative_path: String::new(),
        }
    }

    /// Get the underlying blob data
    pub fn blob(&self) -> &BlobData {
        &self.blob_data
    }

    /// Get file name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get last modified timestamp
    pub fn last_modified(&self) -> u64 {
        self.last_modified
    }

    /// Get webkit relative path
    pub fn webkit_relative_path(&self) -> &str {
        &self.webkit_relative_path
    }
}

impl IntrinsicObject for File {
    fn init(realm: &Realm) {
        let get_name = BuiltInBuilder::callable(realm, get_name)
            .name(js_string!("get name"))
            .build();

        let get_last_modified = BuiltInBuilder::callable(realm, get_last_modified)
            .name(js_string!("get lastModified"))
            .build();

        let get_webkit_relative_path = BuiltInBuilder::callable(realm, get_webkit_relative_path)
            .name(js_string!("get webkitRelativePath"))
            .build();

        let get_size = BuiltInBuilder::callable(realm, get_size)
            .name(js_string!("get size"))
            .build();

        let get_type = BuiltInBuilder::callable(realm, get_type)
            .name(js_string!("get type"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Set up prototype chain: File -> Blob
            .inherits(Some(realm.intrinsics().constructors().blob().prototype()))
            // Inherit Blob methods
            .method(Self::slice, js_string!("slice"), 0)
            .method(Self::stream, js_string!("stream"), 0)
            .method(Self::text, js_string!("text"), 0)
            .method(Self::array_buffer, js_string!("arrayBuffer"), 0)
            .method(Self::bytes, js_string!("bytes"), 0)

            // File-specific properties
            .accessor(
                js_string!("name"),
                Some(get_name),
                None,
                boa_engine::property::Attribute::ENUMERABLE | boa_engine::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("lastModified"),
                Some(get_last_modified),
                None,
                boa_engine::property::Attribute::ENUMERABLE | boa_engine::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("webkitRelativePath"),
                Some(get_webkit_relative_path),
                None,
                boa_engine::property::Attribute::ENUMERABLE | boa_engine::property::Attribute::CONFIGURABLE,
            )

            // Inherited Blob properties
            .accessor(
                js_string!("size"),
                Some(get_size),
                None,
                boa_engine::property::Attribute::ENUMERABLE | boa_engine::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("type"),
                Some(get_type),
                None,
                boa_engine::property::Attribute::ENUMERABLE | boa_engine::property::Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for File {
    const NAME: JsString = js_string!("File");
}

impl BuiltInConstructor for File {
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::file;
    const PROTOTYPE_STORAGE_SLOTS: usize = 2;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    /// `new File(fileBits, fileName, options)`
    fn constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // File constructor requires at least fileName parameter
        if args.is_empty() {
            return Err(JsNativeError::typ()
                .with_message("File constructor requires at least 1 argument")
                .into());
        }

        // Handle file bits array (same as Blob constructor)
        let mut data = Vec::new();
        if let Some(parts) = args.get(0) {
            if let Some(array) = parts.as_object() {
                // Handle array-like object
                let length_prop = array.get(js_string!("length"), context)?;
                let length = length_prop.to_length(context)?;

                for i in 0..length {
                    let part = array.get(i, context)?;

                    if let Some(part_str) = part.as_string() {
                        // String part
                        data.extend_from_slice(part_str.to_std_string_escaped().as_bytes());
                    } else if let Some(part_obj) = part.as_object() {
                        // Check if it's a Blob or File
                        if let Some(blob_data) = part_obj.downcast_ref::<BlobData>() {
                            data.extend_from_slice(blob_data.data());
                        } else if let Some(file_data) = part_obj.downcast_ref::<FileData>() {
                            data.extend_from_slice(file_data.blob_data.data());
                        } else {
                            // Convert to string
                            let part_str = part.to_string(context)?;
                            data.extend_from_slice(part_str.to_std_string_escaped().as_bytes());
                        }
                    } else {
                        // Convert to string
                        let part_str = part.to_string(context)?;
                        data.extend_from_slice(part_str.to_std_string_escaped().as_bytes());
                    }
                }
            } else if !parts.is_undefined() && !parts.is_null() {
                // Single item, convert to string
                let part_str = parts.to_string(context)?;
                data.extend_from_slice(part_str.to_std_string_escaped().as_bytes());
            }
        }

        // Extract fileName (required parameter)
        let file_name = if let Some(name_arg) = args.get(1) {
            name_arg.to_string(context)?.to_std_string_escaped()
        } else {
            return Err(JsNativeError::typ()
                .with_message("File constructor requires fileName parameter")
                .into());
        };

        // Handle options
        let mut mime_type = String::new();
        let mut last_modified = None;
        if let Some(options) = args.get(2) {
            if let Some(options_obj) = options.as_object() {
                // Extract type
                let type_prop = options_obj.get(js_string!("type"), context)?;
                if !type_prop.is_undefined() {
                    mime_type = type_prop.to_string(context)?.to_std_string_escaped();
                }

                // Extract lastModified
                let last_modified_prop = options_obj.get(js_string!("lastModified"), context)?;
                if !last_modified_prop.is_undefined() {
                    last_modified = Some(last_modified_prop.to_number(context)? as u64);
                }

                // TODO: Handle endings option (normalize line endings)
            }
        }

        // Create blob data
        let blob_data = BlobData::new(data, mime_type);

        // Create file data
        let file_data = FileData::new(blob_data, file_name, last_modified);

        let prototype = Self::get(context.intrinsics())
            .get(js_string!("prototype"), context)?
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("File.prototype is not an object"))?
            .clone();

        let file_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            file_data,
        );

        Ok(file_obj.into())
    }
}

impl File {
    /// `File.prototype.slice(start, end, contentType)` - inherits from Blob
    fn slice(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // Delegate to blob slice implementation but return a new File
        let file_obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("slice called on non-object")
        })?;

        let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
            JsNativeError::typ().with_message("slice called on non-File object")
        })?;

        // Use blob slice logic from the blob module
        // For now, implement simplified version
        let data_len = file_data.blob_data.size();

        // Parse start parameter
        let start = if let Some(start_val) = args.get(0) {
            if start_val.is_undefined() {
                0
            } else {
                let start_int = start_val.to_integer_or_infinity(context)?;
                match start_int {
                    boa_engine::value::IntegerOrInfinity::Integer(i) if i < 0 => {
                        (data_len as i64 + i).max(0) as usize
                    }
                    boa_engine::value::IntegerOrInfinity::Integer(i) => (i as usize).min(data_len),
                    boa_engine::value::IntegerOrInfinity::NegativeInfinity => 0,
                    boa_engine::value::IntegerOrInfinity::PositiveInfinity => data_len,
                }
            }
        } else {
            0
        };

        // Parse end parameter
        let end = if let Some(end_val) = args.get(1) {
            if end_val.is_undefined() {
                data_len
            } else {
                let end_int = end_val.to_integer_or_infinity(context)?;
                match end_int {
                    boa_engine::value::IntegerOrInfinity::Integer(i) if i < 0 => {
                        (data_len as i64 + i).max(0) as usize
                    }
                    boa_engine::value::IntegerOrInfinity::Integer(i) => (i as usize).min(data_len),
                    boa_engine::value::IntegerOrInfinity::NegativeInfinity => 0,
                    boa_engine::value::IntegerOrInfinity::PositiveInfinity => data_len,
                }
            }
        } else {
            data_len
        };

        // Parse contentType parameter
        let content_type = if let Some(type_val) = args.get(2) {
            if type_val.is_undefined() {
                file_data.blob_data.mime_type().to_string()
            } else {
                type_val.to_string(context)?.to_std_string_escaped()
            }
        } else {
            file_data.blob_data.mime_type().to_string()
        };

        // Extract slice data
        let slice_data = if start < end {
            file_data.blob_data.data()[start..end].to_vec()
        } else {
            Vec::new()
        };

        let new_blob_data = BlobData::new(slice_data, content_type);

        // Return a new File object with the same name and lastModified
        let new_file_data = FileData::new(
            new_blob_data,
            file_data.name.clone(),
            Some(file_data.last_modified),
        );

        let prototype = Self::get(context.intrinsics())
            .get(js_string!("prototype"), context)?
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("File.prototype is not an object"))?
            .clone();

        let new_file = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            new_file_data,
        );

        Ok(new_file.into())
    }

    /// `File.prototype.stream()` - inherits from Blob
    ///
    /// Returns a ReadableStream that can be used to read the file's contents.
    fn stream(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let file_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("stream called on non-object")
        })?;

        let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
            JsNativeError::typ().with_message("stream called on non-File object")
        })?;

        // Create a ReadableStream from the file data
        use crate::streams::readable_stream::ReadableStreamData;

        let bytes = file_data.blob_data.data().to_vec();
        let mut stream_data = ReadableStreamData::new(JsValue::undefined(), JsValue::undefined());

        // Convert bytes to a Uint8Array-like chunk
        // For simplicity, we'll enqueue the entire file as a single chunk
        // In a more complete implementation, we'd chunk large files
        let chunk = create_uint8array_from_bytes(&bytes, context)?;
        stream_data.enqueue_chunk(chunk);
        stream_data.close();

        let stream = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().readable_stream().prototype(),
            stream_data,
        );

        eprintln!("File.stream(): Created ReadableStream with {} bytes", bytes.len());
        Ok(stream.into())
    }

    /// `File.prototype.text()` - inherits from Blob
    ///
    /// Returns a Promise that resolves to the file's contents as a UTF-8 string.
    fn text(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let file_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("text called on non-object")
        })?;

        let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
            JsNativeError::typ().with_message("text called on non-File object")
        })?;

        // Convert bytes to UTF-8 string
        let text = String::from_utf8_lossy(file_data.blob_data.data());
        let text_value = JsValue::from(js_string!(text.as_ref()));

        // Return a Promise that resolves to the text
        let promise_constructor = context.intrinsics().constructors().promise().constructor();
        boa_engine::builtins::promise::Promise::resolve(
            &promise_constructor.into(),
            &[text_value],
            context
        )
    }

    /// `File.prototype.arrayBuffer()` - inherits from Blob
    ///
    /// Returns a Promise that resolves to the file's contents as an ArrayBuffer.
    fn array_buffer(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let file_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("arrayBuffer called on non-object")
        })?;

        let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
            JsNativeError::typ().with_message("arrayBuffer called on non-File object")
        })?;

        // Create an ArrayBuffer from the file data
        let bytes = file_data.blob_data.data();
        let array_buffer = create_array_buffer_from_bytes(bytes, context)?;

        // Return a Promise that resolves to the ArrayBuffer
        let promise_constructor = context.intrinsics().constructors().promise().constructor();
        boa_engine::builtins::promise::Promise::resolve(
            &promise_constructor.into(),
            &[array_buffer],
            context
        )
    }

    /// `File.prototype.bytes()` - returns contents as a Uint8Array
    ///
    /// Returns a Promise that resolves to the file's contents as a Uint8Array.
    fn bytes(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let file_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("bytes called on non-object")
        })?;

        let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
            JsNativeError::typ().with_message("bytes called on non-File object")
        })?;

        // Create a Uint8Array from the file data
        let bytes = file_data.blob_data.data();
        let uint8array = create_uint8array_from_bytes(bytes, context)?;

        // Return a Promise that resolves to the Uint8Array
        let promise_constructor = context.intrinsics().constructors().promise().constructor();
        boa_engine::builtins::promise::Promise::resolve(
            &promise_constructor.into(),
            &[uint8array],
            context
        )
    }
}

/// Helper function to create a Uint8Array from bytes
fn create_uint8array_from_bytes(bytes: &[u8], context: &mut Context) -> JsResult<JsValue> {
    // Create an array of byte values
    let byte_values: Vec<JsValue> = bytes.iter().map(|&b| JsValue::from(b)).collect();

    // Create a Uint8Array using its constructor with array of values
    let uint8array_constructor = context.intrinsics().constructors().typed_uint8_array().constructor();

    // Create a JS array with the byte values
    let js_array = boa_engine::object::JsArray::new(context);
    for (i, val) in byte_values.into_iter().enumerate() {
        js_array.set(i as u32, val, false, context)?;
    }

    // Construct Uint8Array from the array
    let uint8array = uint8array_constructor.construct(
        &[js_array.into()],
        Some(&uint8array_constructor),
        context
    )?;

    Ok(uint8array.into())
}

/// Helper function to create an ArrayBuffer from bytes
fn create_array_buffer_from_bytes(bytes: &[u8], context: &mut Context) -> JsResult<JsValue> {
    // For ArrayBuffer, we create it via Uint8Array's buffer property
    // This is a workaround since direct ArrayBuffer manipulation is complex

    // Create Uint8Array first
    let uint8array = create_uint8array_from_bytes(bytes, context)?;

    // Get the buffer property from the Uint8Array
    if let Some(uint8array_obj) = uint8array.as_object() {
        let buffer = uint8array_obj.get(js_string!("buffer"), context)?;
        return Ok(buffer);
    }

    // Fallback: create empty ArrayBuffer
    let array_buffer_constructor = context.intrinsics().constructors().array_buffer().constructor();
    let length = JsValue::from(bytes.len());
    let array_buffer = array_buffer_constructor.construct(
        &[length],
        Some(&array_buffer_constructor),
        context
    )?;

    Ok(array_buffer.into())
}

/// `get File.prototype.name`
pub(crate) fn get_name(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let file_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("name getter called on non-object")
    })?;

    let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
        JsNativeError::typ().with_message("name getter called on non-File object")
    })?;

    Ok(JsValue::from(js_string!(file_data.name.clone())))
}

/// `get File.prototype.lastModified`
pub(crate) fn get_last_modified(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let file_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("lastModified getter called on non-object")
    })?;

    let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
        JsNativeError::typ().with_message("lastModified getter called on non-File object")
    })?;

    Ok(JsValue::from(file_data.last_modified as f64))
}

/// `get File.prototype.webkitRelativePath`
pub(crate) fn get_webkit_relative_path(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let file_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("webkitRelativePath getter called on non-object")
    })?;

    let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
        JsNativeError::typ().with_message("webkitRelativePath getter called on non-File object")
    })?;

    Ok(JsValue::from(js_string!(file_data.webkit_relative_path.clone())))
}

/// `get File.prototype.size` - inherits from Blob
pub(crate) fn get_size(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let file_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("size getter called on non-object")
    })?;

    let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
        JsNativeError::typ().with_message("size getter called on non-File object")
    })?;

    Ok(JsValue::from(file_data.blob_data.size()))
}

/// `get File.prototype.type` - inherits from Blob
pub(crate) fn get_type(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let file_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("type getter called on non-object")
    })?;

    let file_data = file_obj.downcast_ref::<FileData>().ok_or_else(|| {
        JsNativeError::typ().with_message("type getter called on non-File object")
    })?;

    Ok(JsValue::from(js_string!(file_data.blob_data.mime_type())))
}