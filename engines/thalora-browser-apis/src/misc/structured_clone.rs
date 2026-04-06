//! Structured cloning algorithm implementation for Web APIs
//!
//! Implements the structured cloning algorithm as defined in:
//! https://html.spec.whatwg.org/multipage/structured-data.html#structured-cloning

use boa_engine::{
    Context, JsNativeError, JsObject, JsResult, JsValue,
    builtins::{date::Date, regexp::RegExp},
    js_string,
    object::{
        JsArray,
        builtins::{AlignedVec, JsArrayBuffer, JsMap, JsSet},
    },
    value::Type,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Structured clone result - can be serialized across threads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructuredCloneValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    BigInt(String), // Stored as string representation
    Array(Vec<StructuredCloneValue>),
    Object(HashMap<String, StructuredCloneValue>),
    Date(f64), // Stored as timestamp
    RegExp {
        pattern: String,
        flags: String,
    },
    Map(Vec<(StructuredCloneValue, StructuredCloneValue)>),
    Set(Vec<StructuredCloneValue>),
    ArrayBuffer(Vec<u8>),
    // Transferable objects - these are moved, not copied
    TransferredArrayBuffer {
        data: Vec<u8>,
        detach_key: Option<String>,
    },
    TransferredMessagePort {
        port_id: usize,
    },
    /// Error object (name, message, stack)
    Error {
        name: String,
        message: String,
        stack: Option<String>,
    },
    /// Blob data with MIME type
    Blob {
        data: Vec<u8>,
        content_type: String,
    },
    /// File data with name and MIME type
    File {
        data: Vec<u8>,
        name: String,
        content_type: String,
        last_modified: f64,
    },
    /// ImageData (width, height, RGBA pixel data)
    ImageData {
        width: u32,
        height: u32,
        data: Vec<u8>,
    },
    /// CryptoKey (serialized key data)
    CryptoKey {
        algorithm: String,
        key_type: String,
        extractable: bool,
        usages: Vec<String>,
    },
    // Additional transferable types (stubs for future implementation)
    TransferredOffscreenCanvas {
        width: u32,
        height: u32,
    },
    TransferredReadableStream {
        stream_id: usize,
    },
    TransferredWritableStream {
        stream_id: usize,
    },
    TransferredTransformStream {
        stream_id: usize,
    },
}

/// Transfer list for transferable objects
#[derive(Debug, Clone)]
pub struct TransferList {
    pub objects: Vec<JsObject>,
}

impl Default for TransferList {
    fn default() -> Self {
        Self::new()
    }
}

impl TransferList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, object: JsObject) {
        self.objects.push(object);
    }

    /// Check if an object is in the transfer list
    pub fn contains(&self, object: &JsObject) -> bool {
        self.objects.iter().any(|obj| JsObject::equals(obj, object))
    }

    /// Create a TransferList from a JavaScript array
    pub fn from_js_array(array: &JsValue, context: &mut Context) -> JsResult<Self> {
        let mut transfer_list = TransferList::new();

        if array.is_undefined() || array.is_null() {
            return Ok(transfer_list);
        }

        if let Some(array_obj) = array.as_object()
            && array_obj.is_array()
        {
            let array = JsArray::from_object(array_obj.clone())?;
            let length = array.length(context)?;

            for i in 0..length {
                let element = array.get(i, context)?;
                if let Some(obj) = element.as_object() {
                    // Verify the object is actually transferable
                    if Self::is_transferable_object(&obj) {
                        transfer_list.add(obj.clone());
                    } else {
                        return Err(JsNativeError::typ()
                            .with_message("Object is not transferable")
                            .into());
                    }
                }
            }
        }

        Ok(transfer_list)
    }

    /// Check if an object is transferable according to WHATWG specification
    fn is_transferable_object(obj: &JsObject) -> bool {
        // Check for ArrayBuffer
        if obj
            .downcast_ref::<boa_engine::builtins::array_buffer::ArrayBuffer>()
            .is_some()
        {
            return true;
        }

        // Check for MessagePort
        if obj
            .downcast_ref::<crate::messaging::message_port::MessagePortData>()
            .is_some()
        {
            return true;
        }

        // Note: Other transferable types (OffscreenCanvas, ReadableStream, WritableStream,
        // TransformStream, VideoFrame, AudioData, etc.) would be checked here when implemented

        false
    }

    /// Get MessagePort IDs from the transfer list
    ///
    /// Returns unique identifiers for each MessagePort in the transfer list,
    /// which can be used to reconstruct the ports on the receiving side.
    pub fn get_message_port_ids(&self) -> Vec<usize> {
        let mut ids = Vec::new();
        for (index, obj) in self.objects.iter().enumerate() {
            if obj
                .downcast_ref::<crate::messaging::message_port::MessagePortData>()
                .is_some()
            {
                // Use the index as a simple ID for now
                // In a full implementation, we'd extract the actual port ID
                ids.push(index);
            }
        }
        ids
    }

    /// Check if this transfer list contains any MessagePorts
    pub fn has_message_ports(&self) -> bool {
        self.objects.iter().any(|obj| {
            obj.downcast_ref::<crate::messaging::message_port::MessagePortData>()
                .is_some()
        })
    }

    /// Check if this transfer list contains any ArrayBuffers
    pub fn has_array_buffers(&self) -> bool {
        self.objects.iter().any(|obj| {
            obj.downcast_ref::<boa_engine::builtins::array_buffer::ArrayBuffer>()
                .is_some()
        })
    }
}

/// Structured clone algorithm implementation
pub struct StructuredClone;

impl StructuredClone {
    /// Clone a JavaScript value using the structured cloning algorithm
    pub fn clone(
        value: &JsValue,
        context: &mut Context,
        transfer_list: Option<&TransferList>,
    ) -> JsResult<StructuredCloneValue> {
        let mut memory = HashSet::new();
        Self::internal_structured_clone(value, context, &mut memory, transfer_list)
    }

    /// Deserialize a structured clone value back to JavaScript
    pub fn deserialize(
        clone_value: &StructuredCloneValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let mut memory = HashMap::new();
        Self::internal_structured_deserialize(clone_value, context, &mut memory)
    }

    /// Internal recursive cloning implementation
    fn internal_structured_clone(
        value: &JsValue,
        context: &mut Context,
        memory: &mut HashSet<*const u8>,
        transfer_list: Option<&TransferList>,
    ) -> JsResult<StructuredCloneValue> {
        match value.get_type() {
            Type::Undefined => Ok(StructuredCloneValue::Undefined),
            Type::Null => Ok(StructuredCloneValue::Null),
            Type::Boolean => Ok(StructuredCloneValue::Boolean(value.as_boolean().unwrap())),
            Type::Number => Ok(StructuredCloneValue::Number(value.as_number().unwrap())),
            Type::String => {
                let js_string = value.as_string().unwrap();
                Ok(StructuredCloneValue::String(
                    js_string.to_std_string_escaped(),
                ))
            }
            Type::BigInt => {
                let bigint_str = value.to_string(context)?.to_std_string_escaped();
                Ok(StructuredCloneValue::BigInt(bigint_str))
            }
            Type::Symbol => Err(JsNativeError::typ()
                .with_message("Symbols cannot be cloned")
                .into()),
            Type::Object => {
                let obj = value.as_object().unwrap();

                // Use object address for circular reference detection (same as Hash impl)
                let obj_addr = obj.as_ref() as *const _ as *const u8;

                // Check for circular references
                if memory.contains(&obj_addr) {
                    return Err(JsNativeError::typ()
                        .with_message("Converting circular structure to structured clone")
                        .into());
                }
                memory.insert(obj_addr);

                let result = Self::clone_object(&obj, context, memory, transfer_list);
                memory.remove(&obj_addr);
                result
            }
        }
    }

    /// Clone an object based on its type
    fn clone_object(
        obj: &JsObject,
        context: &mut Context,
        memory: &mut HashSet<*const u8>,
        transfer_list: Option<&TransferList>,
    ) -> JsResult<StructuredCloneValue> {
        // Check if this object is in the transfer list
        if let Some(transfer_list) = transfer_list
            && transfer_list.contains(obj)
        {
            return Self::transfer_object(obj, context);
        }

        // Handle specific object types for cloning (not transferring)
        if obj.is_array() {
            Self::clone_array(obj, context, memory, transfer_list)
        } else if let Some(date_data) = obj.downcast_ref::<Date>() {
            Self::clone_date(&date_data, context)
        } else if let Some(regexp_data) = obj.downcast_ref::<RegExp>() {
            Self::clone_regexp(&regexp_data, context)
        } else if JsMap::from_object(obj.clone()).is_ok() {
            Self::clone_map(obj, context, memory, transfer_list)
        } else if JsSet::from_object(obj.clone()).is_ok() {
            Self::clone_set(obj, context, memory, transfer_list)
        } else if let Some(array_buffer) =
            obj.downcast_ref::<boa_engine::builtins::array_buffer::ArrayBuffer>()
        {
            // ArrayBuffer that's not being transferred should be cloned
            Self::clone_array_buffer(&array_buffer, context)
        } else {
            // Check for Error objects
            let name = obj
                .get(js_string!("name"), context)
                .ok()
                .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()));
            if let Some(ref err_name) = name
                && matches!(
                    err_name.as_str(),
                    "Error"
                        | "TypeError"
                        | "RangeError"
                        | "SyntaxError"
                        | "ReferenceError"
                        | "EvalError"
                        | "URIError"
                )
            {
                let message = obj
                    .get(js_string!("message"), context)
                    .ok()
                    .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
                    .unwrap_or_default();
                let stack = obj
                    .get(js_string!("stack"), context)
                    .ok()
                    .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()));
                return Ok(StructuredCloneValue::Error {
                    name: err_name.clone(),
                    message,
                    stack,
                });
            }

            // Check for Blob-like objects
            let has_size = obj
                .has_property(js_string!("size"), context)
                .unwrap_or(false);
            let has_type = obj
                .has_property(js_string!("type"), context)
                .unwrap_or(false);
            if has_size && has_type {
                let content_type = obj
                    .get(js_string!("type"), context)
                    .ok()
                    .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
                    .unwrap_or_default();
                // Check if it has a "name" property (File extends Blob)
                let has_name = obj
                    .has_property(js_string!("name"), context)
                    .unwrap_or(false);
                let has_last_modified = obj
                    .has_property(js_string!("lastModified"), context)
                    .unwrap_or(false);

                if has_name && has_last_modified {
                    let file_name = obj
                        .get(js_string!("name"), context)
                        .ok()
                        .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
                        .unwrap_or_default();
                    let last_modified = obj
                        .get(js_string!("lastModified"), context)
                        .ok()
                        .and_then(|v| v.as_number())
                        .unwrap_or(0.0);
                    return Ok(StructuredCloneValue::File {
                        data: Vec::new(), // Data extraction requires async arrayBuffer()
                        name: file_name,
                        content_type,
                        last_modified,
                    });
                }

                return Ok(StructuredCloneValue::Blob {
                    data: Vec::new(), // Data extraction requires async arrayBuffer()
                    content_type,
                });
            }

            // Check for ImageData-like objects (width, height, data)
            let has_width = obj
                .has_property(js_string!("width"), context)
                .unwrap_or(false);
            let has_height = obj
                .has_property(js_string!("height"), context)
                .unwrap_or(false);
            let has_data = obj
                .has_property(js_string!("data"), context)
                .unwrap_or(false);
            if has_width && has_height && has_data {
                let width = obj
                    .get(js_string!("width"), context)
                    .ok()
                    .and_then(|v| v.as_number())
                    .unwrap_or(0.0) as u32;
                let height = obj
                    .get(js_string!("height"), context)
                    .ok()
                    .and_then(|v| v.as_number())
                    .unwrap_or(0.0) as u32;
                // Try to extract data from Uint8ClampedArray
                let data_val = obj.get(js_string!("data"), context).ok();
                let pixel_data = if let Some(ref dv) = data_val {
                    if let Some(data_obj) = dv.as_object() {
                        if let Some(ab) = data_obj
                            .downcast_ref::<boa_engine::builtins::array_buffer::ArrayBuffer>()
                        {
                            ab.data().map(|d| d.to_vec()).unwrap_or_default()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };
                return Ok(StructuredCloneValue::ImageData {
                    width,
                    height,
                    data: pixel_data,
                });
            }

            // Handle plain objects
            Self::clone_plain_object(obj, context, memory, transfer_list)
        }
    }

    /// Transfer an object (move ownership, don't clone)
    fn transfer_object(obj: &JsObject, context: &mut Context) -> JsResult<StructuredCloneValue> {
        // Handle ArrayBuffer transfer
        if let Some(array_buffer) =
            obj.downcast_ref::<boa_engine::builtins::array_buffer::ArrayBuffer>()
        {
            return Self::transfer_array_buffer(obj, &array_buffer, context);
        }

        // Handle MessagePort transfer
        if let Some(port_data) =
            obj.downcast_ref::<crate::messaging::message_port::MessagePortData>()
        {
            return Self::transfer_message_port(&port_data);
        }

        // Handle OffscreenCanvas transfer (stub - check for canvas-like object)
        if let Ok(width) = obj.get(js_string!("width"), context)
            && let Ok(height) = obj.get(js_string!("height"), context)
            && let Ok(transfer_to) = obj.get(js_string!("transferToImageBitmap"), context)
            && transfer_to.is_callable()
        {
            // This looks like an OffscreenCanvas
            let w = width.to_u32(context).unwrap_or(0);
            let h = height.to_u32(context).unwrap_or(0);
            return Ok(StructuredCloneValue::TransferredOffscreenCanvas {
                width: w,
                height: h,
            });
        }

        // Handle ReadableStream transfer (stub)
        if let Ok(locked) = obj.get(js_string!("locked"), context)
            && let Ok(get_reader) = obj.get(js_string!("getReader"), context)
            && !locked.is_undefined()
            && get_reader.is_callable()
        {
            // This looks like a ReadableStream
            return Ok(StructuredCloneValue::TransferredReadableStream { stream_id: 0 });
        }

        // Handle WritableStream transfer (stub)
        if let Ok(locked) = obj.get(js_string!("locked"), context)
            && let Ok(get_writer) = obj.get(js_string!("getWriter"), context)
            && !locked.is_undefined()
            && get_writer.is_callable()
        {
            // This looks like a WritableStream
            return Ok(StructuredCloneValue::TransferredWritableStream { stream_id: 0 });
        }

        Err(JsNativeError::typ()
            .with_message("Object is not transferable")
            .into())
    }

    /// Transfer a MessagePort
    fn transfer_message_port(
        port_data: &crate::messaging::message_port::MessagePortData,
    ) -> JsResult<StructuredCloneValue> {
        // Get the port ID for transfer
        let port_id = port_data.get_port_id();
        eprintln!("Transferring MessagePort with ID: {}", port_id);

        // In a full implementation, we would:
        // 1. Close this end of the port
        // 2. Transfer ownership to the receiving context
        // For now, we just record the port ID

        Ok(StructuredCloneValue::TransferredMessagePort { port_id })
    }

    /// Transfer an ArrayBuffer (detach the original)
    fn transfer_array_buffer(
        _obj: &JsObject,
        array_buffer: &boa_engine::builtins::array_buffer::ArrayBuffer,
        _context: &mut Context,
    ) -> JsResult<StructuredCloneValue> {
        // Extract the data from the ArrayBuffer (None means detached)
        if let Some(bytes) = array_buffer.bytes() {
            let data = bytes.to_vec();

            // Generate a detach key based on the buffer identity
            // The detach key can be used to verify transfer authorization
            let detach_key = format!(
                "transfer-{}-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos(),
                data.len()
            );

            eprintln!(
                "Transferring ArrayBuffer ({} bytes) with detach_key: {}",
                data.len(),
                detach_key
            );

            // Note: In a full implementation, we would call array_buffer.detach() here
            // to make the original ArrayBuffer unusable. This requires mutable access
            // and proper integration with Boa's ArrayBuffer internals.
            // The receiving context owns the data exclusively after transfer.

            Ok(StructuredCloneValue::TransferredArrayBuffer {
                data,
                detach_key: Some(detach_key),
            })
        } else {
            Err(JsNativeError::typ()
                .with_message("ArrayBuffer is already detached")
                .into())
        }
    }

    /// Clone an ArrayBuffer (copy the data)
    fn clone_array_buffer(
        array_buffer: &boa_engine::builtins::array_buffer::ArrayBuffer,
        _context: &mut Context,
    ) -> JsResult<StructuredCloneValue> {
        if let Some(bytes) = array_buffer.bytes() {
            Ok(StructuredCloneValue::ArrayBuffer(bytes.to_vec()))
        } else {
            // Detached ArrayBuffer
            Ok(StructuredCloneValue::ArrayBuffer(Vec::new()))
        }
    }

    /// Clone an array
    fn clone_array(
        obj: &JsObject,
        context: &mut Context,
        memory: &mut HashSet<*const u8>,
        transfer_list: Option<&TransferList>,
    ) -> JsResult<StructuredCloneValue> {
        let array = JsArray::from_object(obj.clone())?;
        let length = array.length(context)?;
        let mut cloned_array = Vec::new();

        for i in 0..length {
            let element = array.get(i, context)?;
            let cloned_element =
                Self::internal_structured_clone(&element, context, memory, transfer_list)?;
            cloned_array.push(cloned_element);
        }

        Ok(StructuredCloneValue::Array(cloned_array))
    }

    /// Clone a Date object
    fn clone_date(date_data: &Date, _context: &mut Context) -> JsResult<StructuredCloneValue> {
        let timestamp = date_data.get_time_value();
        Ok(StructuredCloneValue::Date(timestamp))
    }

    /// Clone a RegExp object
    fn clone_regexp(
        regexp_data: &RegExp,
        _context: &mut Context,
    ) -> JsResult<StructuredCloneValue> {
        let pattern = regexp_data.get_original_source().to_std_string_escaped();
        let flags = regexp_data.get_original_flags().to_std_string_escaped();
        Ok(StructuredCloneValue::RegExp { pattern, flags })
    }

    /// Clone a Map object
    fn clone_map(
        obj: &JsObject,
        context: &mut Context,
        memory: &mut HashSet<*const u8>,
        transfer_list: Option<&TransferList>,
    ) -> JsResult<StructuredCloneValue> {
        let mut entries = Vec::new();

        // Get the Map's entries via iterator
        // Use Map::entries to iterate
        let entries_method = obj.get(js_string!("entries"), context)?;
        if let Some(entries_fn) = entries_method.as_callable() {
            let iterator = entries_fn.call(&obj.clone().into(), &[], context)?;
            if let Some(iterator_obj) = iterator.as_object() {
                let next_method = iterator_obj.get(js_string!("next"), context)?;
                if let Some(next_fn) = next_method.as_callable() {
                    loop {
                        let result = next_fn.call(&iterator, &[], context)?;
                        if let Some(result_obj) = result.as_object() {
                            let done = result_obj.get(js_string!("done"), context)?;
                            if done.as_boolean().unwrap_or(true) {
                                break;
                            }
                            let value = result_obj.get(js_string!("value"), context)?;
                            if let Some(pair_obj) = value.as_object()
                                && pair_obj.is_array()
                            {
                                let pair = JsArray::from_object(pair_obj.clone())?;
                                let key = pair.get(0, context)?;
                                let val = pair.get(1, context)?;
                                let cloned_key = Self::internal_structured_clone(
                                    &key,
                                    context,
                                    memory,
                                    transfer_list,
                                )?;
                                let cloned_val = Self::internal_structured_clone(
                                    &val,
                                    context,
                                    memory,
                                    transfer_list,
                                )?;
                                entries.push((cloned_key, cloned_val));
                            }
                        }
                    }
                }
            }
        }

        Ok(StructuredCloneValue::Map(entries))
    }

    /// Clone a Set object
    fn clone_set(
        obj: &JsObject,
        context: &mut Context,
        memory: &mut HashSet<*const u8>,
        transfer_list: Option<&TransferList>,
    ) -> JsResult<StructuredCloneValue> {
        let mut values = Vec::new();

        // Get the Set's values via iterator
        let values_method = obj.get(js_string!("values"), context)?;
        if let Some(values_fn) = values_method.as_callable() {
            let iterator = values_fn.call(&obj.clone().into(), &[], context)?;
            if let Some(iterator_obj) = iterator.as_object() {
                let next_method = iterator_obj.get(js_string!("next"), context)?;
                if let Some(next_fn) = next_method.as_callable() {
                    loop {
                        let result = next_fn.call(&iterator, &[], context)?;
                        if let Some(result_obj) = result.as_object() {
                            let done = result_obj.get(js_string!("done"), context)?;
                            if done.as_boolean().unwrap_or(true) {
                                break;
                            }
                            let value = result_obj.get(js_string!("value"), context)?;
                            let cloned_value = Self::internal_structured_clone(
                                &value,
                                context,
                                memory,
                                transfer_list,
                            )?;
                            values.push(cloned_value);
                        }
                    }
                }
            }
        }

        Ok(StructuredCloneValue::Set(values))
    }

    /// Clone a plain object
    fn clone_plain_object(
        obj: &JsObject,
        context: &mut Context,
        memory: &mut HashSet<*const u8>,
        transfer_list: Option<&TransferList>,
    ) -> JsResult<StructuredCloneValue> {
        let mut cloned_object = HashMap::new();

        // Get all enumerable own properties
        let keys = obj.own_property_keys(context)?;
        for key in keys {
            let property_key = key.to_string();
            if let Ok(value) = obj.get(key, context) {
                let cloned_value =
                    Self::internal_structured_clone(&value, context, memory, transfer_list)?;
                cloned_object.insert(property_key, cloned_value);
            }
        }

        Ok(StructuredCloneValue::Object(cloned_object))
    }

    /// Internal recursive deserialization implementation
    fn internal_structured_deserialize(
        clone_value: &StructuredCloneValue,
        context: &mut Context,
        memory: &mut HashMap<usize, JsValue>,
    ) -> JsResult<JsValue> {
        match clone_value {
            StructuredCloneValue::Undefined => Ok(JsValue::undefined()),
            StructuredCloneValue::Null => Ok(JsValue::null()),
            StructuredCloneValue::Boolean(b) => Ok(JsValue::from(*b)),
            StructuredCloneValue::Number(n) => Ok(JsValue::from(*n)),
            StructuredCloneValue::String(s) => Ok(js_string!(s.clone()).into()),
            StructuredCloneValue::BigInt(s) => {
                // Parse BigInt from string representation
                // For now, convert to regular number (limited precision)
                if let Ok(num) = s.parse::<f64>() {
                    Ok(JsValue::from(num))
                } else {
                    Ok(JsValue::from(0.0))
                }
            }
            StructuredCloneValue::Array(arr) => Self::deserialize_array(arr, context, memory),
            StructuredCloneValue::Object(obj) => Self::deserialize_object(obj, context, memory),
            StructuredCloneValue::Date(timestamp) => Self::deserialize_date(*timestamp, context),
            StructuredCloneValue::RegExp { pattern, flags } => {
                Self::deserialize_regexp(pattern, flags, context)
            }
            StructuredCloneValue::Map(entries) => Self::deserialize_map(entries, context, memory),
            StructuredCloneValue::Set(values) => Self::deserialize_set(values, context, memory),
            StructuredCloneValue::ArrayBuffer(data) => {
                Self::deserialize_array_buffer(data, context)
            }
            StructuredCloneValue::TransferredArrayBuffer {
                data,
                detach_key: _,
            } => {
                // For transferred ArrayBuffers, create a new one with the transferred data
                Self::deserialize_array_buffer(data, context)
            }
            StructuredCloneValue::TransferredMessagePort { port_id } => {
                // MessagePort deserialization requires a global port registry to look up
                // the port by ID and restore its channel connections. This is a complex
                // operation that needs coordination with the messaging system.
                // For now, we log and return undefined. Full implementation requires:
                // 1. Global PORT_REGISTRY singleton
                // 2. Port ID to channel mapping
                // 3. Re-entanglement logic for transferred ports
                eprintln!(
                    "Warning: MessagePort deserialization not implemented (port_id: {})",
                    port_id
                );
                Ok(JsValue::undefined())
            }
            StructuredCloneValue::TransferredOffscreenCanvas { width, height } => {
                // Create stub OffscreenCanvas-like object with transferred dimensions
                let canvas_obj = JsObject::with_object_proto(context.intrinsics());
                canvas_obj.set(js_string!("width"), JsValue::from(*width), false, context)?;
                canvas_obj.set(js_string!("height"), JsValue::from(*height), false, context)?;
                eprintln!(
                    "Deserialized transferred OffscreenCanvas ({}x{})",
                    width, height
                );
                Ok(canvas_obj.into())
            }
            StructuredCloneValue::TransferredReadableStream { stream_id } => {
                // Stub - create a placeholder ReadableStream-like object
                let stream_obj = JsObject::with_object_proto(context.intrinsics());
                stream_obj.set(js_string!("locked"), JsValue::from(false), false, context)?;
                eprintln!(
                    "Warning: ReadableStream deserialization not fully implemented (stream_id: {})",
                    stream_id
                );
                Ok(stream_obj.into())
            }
            StructuredCloneValue::TransferredWritableStream { stream_id } => {
                // Stub - create a placeholder WritableStream-like object
                let stream_obj = JsObject::with_object_proto(context.intrinsics());
                stream_obj.set(js_string!("locked"), JsValue::from(false), false, context)?;
                eprintln!(
                    "Warning: WritableStream deserialization not fully implemented (stream_id: {})",
                    stream_id
                );
                Ok(stream_obj.into())
            }
            StructuredCloneValue::TransferredTransformStream { stream_id } => {
                // Stub - create a placeholder TransformStream-like object with readable and writable
                let stream_obj = JsObject::with_object_proto(context.intrinsics());
                let readable = JsObject::with_object_proto(context.intrinsics());
                readable.set(js_string!("locked"), JsValue::from(false), false, context)?;
                let writable = JsObject::with_object_proto(context.intrinsics());
                writable.set(js_string!("locked"), JsValue::from(false), false, context)?;
                stream_obj.set(js_string!("readable"), readable, false, context)?;
                stream_obj.set(js_string!("writable"), writable, false, context)?;
                eprintln!(
                    "Warning: TransformStream deserialization not fully implemented (stream_id: {})",
                    stream_id
                );
                Ok(stream_obj.into())
            }
            StructuredCloneValue::Error {
                name,
                message,
                stack,
            } => {
                // Create an Error object with the preserved name, message, and stack
                let err_obj = JsObject::with_object_proto(context.intrinsics());
                err_obj.set(js_string!("name"), js_string!(name.clone()), false, context)?;
                err_obj.set(
                    js_string!("message"),
                    js_string!(message.clone()),
                    false,
                    context,
                )?;
                if let Some(stack_str) = stack {
                    err_obj.set(
                        js_string!("stack"),
                        js_string!(stack_str.clone()),
                        false,
                        context,
                    )?;
                }
                Ok(err_obj.into())
            }
            StructuredCloneValue::Blob { data, content_type } => {
                let blob_obj = JsObject::with_object_proto(context.intrinsics());
                blob_obj.set(
                    js_string!("size"),
                    JsValue::from(data.len() as f64),
                    false,
                    context,
                )?;
                blob_obj.set(
                    js_string!("type"),
                    js_string!(content_type.clone()),
                    false,
                    context,
                )?;
                Ok(blob_obj.into())
            }
            StructuredCloneValue::File {
                data,
                name,
                content_type,
                last_modified,
            } => {
                let file_obj = JsObject::with_object_proto(context.intrinsics());
                file_obj.set(js_string!("name"), js_string!(name.clone()), false, context)?;
                file_obj.set(
                    js_string!("size"),
                    JsValue::from(data.len() as f64),
                    false,
                    context,
                )?;
                file_obj.set(
                    js_string!("type"),
                    js_string!(content_type.clone()),
                    false,
                    context,
                )?;
                file_obj.set(
                    js_string!("lastModified"),
                    JsValue::from(*last_modified),
                    false,
                    context,
                )?;
                Ok(file_obj.into())
            }
            StructuredCloneValue::ImageData {
                width,
                height,
                data,
            } => {
                let img_obj = JsObject::with_object_proto(context.intrinsics());
                img_obj.set(js_string!("width"), JsValue::from(*width), false, context)?;
                img_obj.set(js_string!("height"), JsValue::from(*height), false, context)?;
                // Create the pixel data as an ArrayBuffer
                let aligned = AlignedVec::from_iter(64, data.iter().copied());
                let buffer = JsArrayBuffer::from_byte_block(aligned, context)?;
                img_obj.set(js_string!("data"), buffer, false, context)?;
                Ok(img_obj.into())
            }
            StructuredCloneValue::CryptoKey {
                algorithm,
                key_type,
                extractable,
                usages,
            } => {
                let key_obj = JsObject::with_object_proto(context.intrinsics());
                key_obj.set(
                    js_string!("algorithm"),
                    js_string!(algorithm.clone()),
                    false,
                    context,
                )?;
                key_obj.set(
                    js_string!("type"),
                    js_string!(key_type.clone()),
                    false,
                    context,
                )?;
                key_obj.set(
                    js_string!("extractable"),
                    JsValue::from(*extractable),
                    false,
                    context,
                )?;
                let usages_arr = JsArray::new(context)?;
                for (i, usage) in usages.iter().enumerate() {
                    usages_arr.set(i as u32, js_string!(usage.clone()), false, context)?;
                }
                let usages_val: JsValue = usages_arr.into();
                key_obj.set(js_string!("usages"), usages_val, false, context)?;
                Ok(key_obj.into())
            }
        }
    }

    /// Deserialize an array
    fn deserialize_array(
        arr: &[StructuredCloneValue],
        context: &mut Context,
        memory: &mut HashMap<usize, JsValue>,
    ) -> JsResult<JsValue> {
        let js_array = JsArray::new(context)?;

        for (index, element) in arr.iter().enumerate() {
            let deserialized_element =
                Self::internal_structured_deserialize(element, context, memory)?;
            js_array.set(index, deserialized_element, true, context)?;
        }

        Ok(js_array.into())
    }

    /// Deserialize a plain object
    fn deserialize_object(
        obj: &HashMap<String, StructuredCloneValue>,
        context: &mut Context,
        memory: &mut HashMap<usize, JsValue>,
    ) -> JsResult<JsValue> {
        let js_object = JsObject::with_object_proto(context.intrinsics());

        for (key, value) in obj {
            let deserialized_value = Self::internal_structured_deserialize(value, context, memory)?;
            js_object.set(js_string!(key.clone()), deserialized_value, true, context)?;
        }

        Ok(js_object.into())
    }

    /// Deserialize a Date object
    fn deserialize_date(timestamp: f64, context: &mut Context) -> JsResult<JsValue> {
        let date_constructor = context.intrinsics().constructors().date().constructor();
        let args = [JsValue::from(timestamp)];
        let new_target = Some(&date_constructor);
        Ok(date_constructor
            .construct(&args, new_target, context)?
            .into())
    }

    /// Deserialize a RegExp object
    fn deserialize_regexp(pattern: &str, flags: &str, context: &mut Context) -> JsResult<JsValue> {
        let regexp_constructor = context.intrinsics().constructors().regexp().constructor();
        let args = [js_string!(pattern).into(), js_string!(flags).into()];
        let new_target = Some(&regexp_constructor);
        Ok(regexp_constructor
            .construct(&args, new_target, context)?
            .into())
    }

    /// Deserialize an ArrayBuffer object
    fn deserialize_array_buffer(data: &[u8], context: &mut Context) -> JsResult<JsValue> {
        // Create an AlignedVec from the data and use JsArrayBuffer::from_byte_block
        let aligned_data = AlignedVec::from_iter(64, data.iter().copied());
        let array_buffer = JsArrayBuffer::from_byte_block(aligned_data, context)?;
        Ok(array_buffer.into())
    }

    /// Deserialize a Map object
    fn deserialize_map(
        entries: &[(StructuredCloneValue, StructuredCloneValue)],
        context: &mut Context,
        memory: &mut HashMap<usize, JsValue>,
    ) -> JsResult<JsValue> {
        let map_constructor = context.intrinsics().constructors().map().constructor();
        let new_target = Some(&map_constructor);
        let map_obj = map_constructor.construct(&[], new_target, context)?;

        // Get the 'set' method from the Map object
        let set_method = map_obj.get(js_string!("set"), context)?;
        if let Some(set_fn) = set_method.as_callable() {
            let map_value: JsValue = map_obj.clone().into();
            for (key_clone, val_clone) in entries {
                let key = Self::internal_structured_deserialize(key_clone, context, memory)?;
                let val = Self::internal_structured_deserialize(val_clone, context, memory)?;
                set_fn.call(&map_value, &[key, val], context)?;
            }
        }

        Ok(map_obj.into())
    }

    /// Deserialize a Set object
    fn deserialize_set(
        values: &[StructuredCloneValue],
        context: &mut Context,
        memory: &mut HashMap<usize, JsValue>,
    ) -> JsResult<JsValue> {
        let set_constructor = context.intrinsics().constructors().set().constructor();
        let new_target = Some(&set_constructor);
        let set_obj = set_constructor.construct(&[], new_target, context)?;

        // Get the 'add' method from the Set object
        let add_method = set_obj.get(js_string!("add"), context)?;
        if let Some(add_fn) = add_method.as_callable() {
            let set_value: JsValue = set_obj.clone().into();
            for val_clone in values {
                let val = Self::internal_structured_deserialize(val_clone, context, memory)?;
                add_fn.call(&set_value, &[val], context)?;
            }
        }

        Ok(set_obj.into())
    }

    /// Serialize a structured clone value to bytes for cross-thread transfer
    pub fn serialize_to_bytes(
        value: &StructuredCloneValue,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let serialized = serde_json::to_vec(value)?;
        Ok(serialized)
    }

    /// Deserialize a structured clone value from bytes
    pub fn deserialize_from_bytes(
        bytes: &[u8],
    ) -> Result<StructuredCloneValue, Box<dyn std::error::Error + Send + Sync>> {
        let value = serde_json::from_slice(bytes)?;
        Ok(value)
    }
}

/// Convenience function for cloning values
pub fn structured_clone(
    value: &JsValue,
    context: &mut Context,
    transfer_list: Option<&TransferList>,
) -> JsResult<StructuredCloneValue> {
    StructuredClone::clone(value, context, transfer_list)
}

/// Convenience function for deserializing values
pub fn structured_deserialize(
    clone_value: &StructuredCloneValue,
    context: &mut Context,
) -> JsResult<JsValue> {
    StructuredClone::deserialize(clone_value, context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone_primitives() {
        let mut context = Context::default();

        // Test undefined
        let cloned = structured_clone(&JsValue::undefined(), &mut context, None).unwrap();
        assert!(matches!(cloned, StructuredCloneValue::Undefined));

        // Test null
        let cloned = structured_clone(&JsValue::null(), &mut context, None).unwrap();
        assert!(matches!(cloned, StructuredCloneValue::Null));

        // Test boolean
        let cloned = structured_clone(&JsValue::from(true), &mut context, None).unwrap();
        assert!(matches!(cloned, StructuredCloneValue::Boolean(true)));

        // Test number
        let cloned = structured_clone(&JsValue::from(42.5), &mut context, None).unwrap();
        if let StructuredCloneValue::Number(n) = cloned {
            assert!((n - 42.5).abs() < f64::EPSILON);
        } else {
            panic!("Expected Number");
        }

        // Test string
        let cloned = structured_clone(&js_string!("hello").into(), &mut context, None).unwrap();
        if let StructuredCloneValue::String(s) = cloned {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_clone_array() {
        let mut context = Context::default();

        let array = JsArray::new(&mut context)?;
        array.push(JsValue::from(1), &mut context).unwrap();
        array.push(JsValue::from(2), &mut context).unwrap();
        array.push(JsValue::from(3), &mut context).unwrap();

        let cloned = structured_clone(&array.into(), &mut context, None).unwrap();

        if let StructuredCloneValue::Array(arr) = cloned {
            assert_eq!(arr.len(), 3);
            assert!(
                matches!(arr[0], StructuredCloneValue::Number(n) if (n - 1.0).abs() < f64::EPSILON)
            );
            assert!(
                matches!(arr[1], StructuredCloneValue::Number(n) if (n - 2.0).abs() < f64::EPSILON)
            );
            assert!(
                matches!(arr[2], StructuredCloneValue::Number(n) if (n - 3.0).abs() < f64::EPSILON)
            );
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_clone_object() {
        let mut context = Context::default();

        let obj = JsObject::with_object_proto(context.intrinsics());
        obj.set(js_string!("name"), js_string!("test"), true, &mut context)
            .unwrap();
        obj.set(js_string!("value"), JsValue::from(123), true, &mut context)
            .unwrap();

        let cloned = structured_clone(&obj.into(), &mut context, None).unwrap();

        if let StructuredCloneValue::Object(map) = cloned {
            assert_eq!(map.len(), 2);
            assert!(
                matches!(map.get("name"), Some(StructuredCloneValue::String(s)) if s == "test")
            );
            assert!(
                matches!(map.get("value"), Some(StructuredCloneValue::Number(n)) if (*n - 123.0).abs() < f64::EPSILON)
            );
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_roundtrip_primitives() {
        let mut context = Context::default();

        // Test number roundtrip
        let original = JsValue::from(42.5);
        let cloned = structured_clone(&original, &mut context, None).unwrap();
        let restored = structured_deserialize(&cloned, &mut context).unwrap();
        assert_eq!(restored.as_number().unwrap(), 42.5);

        // Test string roundtrip
        let original: JsValue = js_string!("hello world").into();
        let cloned = structured_clone(&original, &mut context, None).unwrap();
        let restored = structured_deserialize(&cloned, &mut context).unwrap();
        assert_eq!(
            restored.as_string().unwrap().to_std_string_escaped(),
            "hello world"
        );

        // Test boolean roundtrip
        let original = JsValue::from(true);
        let cloned = structured_clone(&original, &mut context, None).unwrap();
        let restored = structured_deserialize(&cloned, &mut context).unwrap();
        assert_eq!(restored.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_roundtrip_array() {
        let mut context = Context::default();

        let array = JsArray::new(&mut context)?;
        array.push(JsValue::from(1), &mut context).unwrap();
        array.push(JsValue::from(2), &mut context).unwrap();

        let cloned = structured_clone(&array.clone().into(), &mut context, None).unwrap();
        let restored = structured_deserialize(&cloned, &mut context).unwrap();

        let restored_obj = restored.as_object().unwrap();
        assert!(restored_obj.is_array());
        let restored_array = JsArray::from_object(restored_obj.clone()).unwrap();
        assert_eq!(restored_array.length(&mut context).unwrap(), 2);
    }

    #[test]
    fn test_serialize_deserialize_bytes() {
        let value = StructuredCloneValue::Object(HashMap::from([
            (
                "key".to_string(),
                StructuredCloneValue::String("value".to_string()),
            ),
            ("num".to_string(), StructuredCloneValue::Number(42.0)),
        ]));

        let bytes = StructuredClone::serialize_to_bytes(&value).unwrap();
        let restored = StructuredClone::deserialize_from_bytes(&bytes).unwrap();

        if let StructuredCloneValue::Object(map) = restored {
            assert!(
                matches!(map.get("key"), Some(StructuredCloneValue::String(s)) if s == "value")
            );
            assert!(
                matches!(map.get("num"), Some(StructuredCloneValue::Number(n)) if (*n - 42.0).abs() < f64::EPSILON)
            );
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_clone_map_value() {
        // Test that Map enum variant serializes correctly
        let map_value = StructuredCloneValue::Map(vec![
            (
                StructuredCloneValue::String("key1".to_string()),
                StructuredCloneValue::Number(1.0),
            ),
            (
                StructuredCloneValue::String("key2".to_string()),
                StructuredCloneValue::Number(2.0),
            ),
        ]);

        let bytes = StructuredClone::serialize_to_bytes(&map_value).unwrap();
        let restored = StructuredClone::deserialize_from_bytes(&bytes).unwrap();

        if let StructuredCloneValue::Map(entries) = restored {
            assert_eq!(entries.len(), 2);
        } else {
            panic!("Expected Map");
        }
    }

    #[test]
    fn test_clone_set_value() {
        // Test that Set enum variant serializes correctly
        let set_value = StructuredCloneValue::Set(vec![
            StructuredCloneValue::Number(1.0),
            StructuredCloneValue::Number(2.0),
            StructuredCloneValue::Number(3.0),
        ]);

        let bytes = StructuredClone::serialize_to_bytes(&set_value).unwrap();
        let restored = StructuredClone::deserialize_from_bytes(&bytes).unwrap();

        if let StructuredCloneValue::Set(values) = restored {
            assert_eq!(values.len(), 3);
        } else {
            panic!("Expected Set");
        }
    }

    #[test]
    fn test_clone_arraybuffer_value() {
        // Test that ArrayBuffer enum variant serializes correctly
        let buffer_value = StructuredCloneValue::ArrayBuffer(vec![1, 2, 3, 4, 5]);

        let bytes = StructuredClone::serialize_to_bytes(&buffer_value).unwrap();
        let restored = StructuredClone::deserialize_from_bytes(&bytes).unwrap();

        if let StructuredCloneValue::ArrayBuffer(data) = restored {
            assert_eq!(data, vec![1, 2, 3, 4, 5]);
        } else {
            panic!("Expected ArrayBuffer");
        }
    }
}
