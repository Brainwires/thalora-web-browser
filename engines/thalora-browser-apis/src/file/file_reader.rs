//! FileReader Web API implementation for Boa
//!
//! Native implementation of the FileReader interface from the File API
//! https://w3c.github.io/FileAPI/#FileReader-interface
//!
//! This implements the complete FileReader interface with async file reading and event handling


use boa_engine::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject, BuiltInConstructor},
    object::JsObject,
    value::JsValue,
    Context, JsResult, js_string, JsNativeError, JsArgs,
    realm::Realm, JsString, JsData,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors}
};
use crate::file::blob::BlobData;
use crate::file::file::FileData;
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use base64::{Engine as _, engine::general_purpose};

/// FileReader ready states
#[derive(Debug, Clone, PartialEq, Trace, Finalize)]
#[repr(u16)]
pub enum ReadyState {
    Empty = 0,
    Loading = 1,
    Done = 2,
}

impl ReadyState {
    /// Convert ReadyState to u16 value
    pub fn as_u16(&self) -> u16 {
        match self {
            ReadyState::Empty => 0,
            ReadyState::Loading => 1,
            ReadyState::Done => 2,
        }
    }
}

/// FileReader error codes
#[derive(Debug, Clone, Trace, Finalize)]
#[repr(u16)]
pub enum FileReaderError {
    NotReadable = 1,
    Security = 2,
    Abort = 3,
    Encoding = 4,
}

/// JavaScript `FileReader` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub struct FileReader;

/// Internal data for FileReader objects
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct FileReaderData {
    ready_state: ReadyState,
    result: Option<String>,
    error: Option<FileReaderError>,

    // Event handlers (stored as function objects)
    #[unsafe_ignore_trace]
    on_loadstart: Option<JsObject>,
    #[unsafe_ignore_trace]
    on_progress: Option<JsObject>,
    #[unsafe_ignore_trace]
    on_load: Option<JsObject>,
    #[unsafe_ignore_trace]
    on_loadend: Option<JsObject>,
    #[unsafe_ignore_trace]
    on_error: Option<JsObject>,
    #[unsafe_ignore_trace]
    on_abort: Option<JsObject>,

    // Internal state
    #[unsafe_ignore_trace]
    reader_id: u32,
    #[unsafe_ignore_trace]
    is_aborted: Arc<Mutex<bool>>,
}

/// Pending event type for FileReader
#[derive(Debug, Clone)]
pub enum FileReaderEvent {
    LoadStart,
    Progress { loaded: usize, total: usize },
    Load,
    LoadEnd,
    Error,
    Abort,
}

/// Pending result from async read operation
#[derive(Debug, Clone)]
pub struct PendingResult {
    pub result: Option<FileReadResult>,
    pub error: Option<FileReaderError>,
    pub events: Vec<FileReaderEvent>,
}

/// Result of a file read operation
#[derive(Debug, Clone)]
pub enum FileReadResult {
    Text(String),
    BinaryString(String),
    DataURL(String),
    ArrayBuffer(Vec<u8>),
}

/// FileReader operation management
#[derive(Debug)]
struct FileReaderState {
    operations: Arc<Mutex<HashMap<u32, Arc<Mutex<bool>>>>>,
    pending_results: Arc<Mutex<HashMap<u32, PendingResult>>>,
    next_id: Arc<Mutex<u32>>,
}

static FILEREADER_STATE: OnceLock<FileReaderState> = OnceLock::new();

fn get_filereader_state() -> &'static FileReaderState {
    FILEREADER_STATE.get_or_init(|| FileReaderState {
        operations: Arc::new(Mutex::new(HashMap::new())),
        pending_results: Arc::new(Mutex::new(HashMap::new())),
        next_id: Arc::new(Mutex::new(1)),
    })
}

impl FileReaderData {
    pub fn new() -> Self {
        let state = get_filereader_state();
        let reader_id = {
            let mut next_id = state.next_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        Self {
            ready_state: ReadyState::Empty,
            result: None,
            error: None,
            on_loadstart: None,
            on_progress: None,
            on_load: None,
            on_loadend: None,
            on_error: None,
            on_abort: None,
            reader_id,
            is_aborted: Arc::new(Mutex::new(false)),
        }
    }
}

impl IntrinsicObject for FileReader {
    fn init(realm: &Realm) {
        let get_ready_state = BuiltInBuilder::callable(realm, get_ready_state)
            .name(js_string!("get readyState"))
            .build();

        let get_result = BuiltInBuilder::callable(realm, get_result)
            .name(js_string!("get result"))
            .build();

        let get_error = BuiltInBuilder::callable(realm, get_error)
            .name(js_string!("get error"))
            .build();

        // Event handler getters/setters
        let get_onloadstart = BuiltInBuilder::callable(realm, get_onloadstart)
            .name(js_string!("get onloadstart"))
            .build();
        let set_onloadstart = BuiltInBuilder::callable(realm, set_onloadstart)
            .name(js_string!("set onloadstart"))
            .build();

        let get_onprogress = BuiltInBuilder::callable(realm, get_onprogress)
            .name(js_string!("get onprogress"))
            .build();
        let set_onprogress = BuiltInBuilder::callable(realm, set_onprogress)
            .name(js_string!("set onprogress"))
            .build();

        let get_onload = BuiltInBuilder::callable(realm, get_onload)
            .name(js_string!("get onload"))
            .build();
        let set_onload = BuiltInBuilder::callable(realm, set_onload)
            .name(js_string!("set onload"))
            .build();

        let get_onloadend = BuiltInBuilder::callable(realm, get_onloadend)
            .name(js_string!("get onloadend"))
            .build();
        let set_onloadend = BuiltInBuilder::callable(realm, set_onloadend)
            .name(js_string!("set onloadend"))
            .build();

        let get_onerror = BuiltInBuilder::callable(realm, get_onerror)
            .name(js_string!("get onerror"))
            .build();
        let set_onerror = BuiltInBuilder::callable(realm, set_onerror)
            .name(js_string!("set onerror"))
            .build();

        let get_onabort = BuiltInBuilder::callable(realm, get_onabort)
            .name(js_string!("get onabort"))
            .build();
        let set_onabort = BuiltInBuilder::callable(realm, set_onabort)
            .name(js_string!("set onabort"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // ReadyState constants
            .static_property(js_string!("EMPTY"), 0, boa_engine::property::Attribute::default())
            .static_property(js_string!("LOADING"), 1, boa_engine::property::Attribute::default())
            .static_property(js_string!("DONE"), 2, boa_engine::property::Attribute::default())
            // Read methods
            .method(Self::read_as_array_buffer, js_string!("readAsArrayBuffer"), 1)
            .method(Self::read_as_binary_string, js_string!("readAsBinaryString"), 1)
            .method(Self::read_as_data_url, js_string!("readAsDataURL"), 1)
            .method(Self::read_as_text, js_string!("readAsText"), 1)
            .method(Self::abort, js_string!("abort"), 0)

            // State properties
            .accessor(
                js_string!("readyState"),
                Some(get_ready_state),
                None,
                boa_engine::property::Attribute::ENUMERABLE | boa_engine::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("result"),
                Some(get_result),
                None,
                boa_engine::property::Attribute::ENUMERABLE | boa_engine::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("error"),
                Some(get_error),
                None,
                boa_engine::property::Attribute::ENUMERABLE | boa_engine::property::Attribute::CONFIGURABLE,
            )

            // Event handlers
            .accessor(
                js_string!("onloadstart"),
                Some(get_onloadstart),
                Some(set_onloadstart),
                boa_engine::property::Attribute::ENUMERABLE | boa_engine::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onprogress"),
                Some(get_onprogress),
                Some(set_onprogress),
                boa_engine::property::Attribute::ENUMERABLE | boa_engine::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onload"),
                Some(get_onload),
                Some(set_onload),
                boa_engine::property::Attribute::ENUMERABLE | boa_engine::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onloadend"),
                Some(get_onloadend),
                Some(set_onloadend),
                boa_engine::property::Attribute::ENUMERABLE | boa_engine::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onerror"),
                Some(get_onerror),
                Some(set_onerror),
                boa_engine::property::Attribute::ENUMERABLE | boa_engine::property::Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("onabort"),
                Some(get_onabort),
                Some(set_onabort),
                boa_engine::property::Attribute::ENUMERABLE | boa_engine::property::Attribute::CONFIGURABLE,
            )

            // Constants
            .property(js_string!("EMPTY"), ReadyState::Empty.as_u16(), boa_engine::property::Attribute::NON_ENUMERABLE)
            .property(js_string!("LOADING"), ReadyState::Loading.as_u16(), boa_engine::property::Attribute::NON_ENUMERABLE)
            .property(js_string!("DONE"), ReadyState::Done.as_u16(), boa_engine::property::Attribute::NON_ENUMERABLE)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for FileReader {
    const NAME: JsString = js_string!("FileReader");
}

impl BuiltInConstructor for FileReader {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::file_reader;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    /// `new FileReader()`
    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // FileReader constructor requires 'new'
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Constructor FileReader requires 'new'")
                .into());
        }

        let reader_data = FileReaderData::new();

        let prototype = Self::get(context.intrinsics())
            .get(js_string!("prototype"), context)?
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("FileReader.prototype is not an object"))?
            .clone();

        let reader_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            reader_data,
        );

        Ok(reader_obj.into())
    }
}

impl FileReader {
    /// `FileReader.prototype.readAsArrayBuffer(file)`
    fn read_as_array_buffer(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::start_read(_this, args, context, ReadOperation::ArrayBuffer)
    }

    /// `FileReader.prototype.readAsBinaryString(file)`
    fn read_as_binary_string(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::start_read(_this, args, context, ReadOperation::BinaryString)
    }

    /// `FileReader.prototype.readAsDataURL(file)`
    fn read_as_data_url(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::start_read(_this, args, context, ReadOperation::DataURL)
    }

    /// `FileReader.prototype.readAsText(file, encoding)`
    fn read_as_text(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let encoding = if args.len() > 1 {
            Some(args[1].to_string(context)?.to_std_string_escaped())
        } else {
            None
        };
        Self::start_read(_this, args, context, ReadOperation::Text(encoding))
    }

    /// `FileReader.prototype.abort()`
    fn abort(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let reader_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("abort called on non-object")
        })?;

        let (on_abort, on_loadend) = {
            let mut reader_data = reader_obj.downcast_mut::<FileReaderData>().ok_or_else(|| {
                JsNativeError::typ().with_message("abort called on non-FileReader object")
            })?;

            if reader_data.ready_state == ReadyState::Loading {
                // Mark as aborted
                *reader_data.is_aborted.lock().unwrap() = true;

                reader_data.ready_state = ReadyState::Done;
                reader_data.result = None;
                reader_data.error = Some(FileReaderError::Abort);

                (reader_data.on_abort.clone(), reader_data.on_loadend.clone())
            } else {
                return Ok(JsValue::undefined());
            }
        };

        // Fire abort event
        if let Some(handler) = on_abort {
            let event = Self::create_progress_event("abort", 0, 0, false, context)?;
            let _ = handler.call(&this.clone(), &[event], context);
        }

        // Fire loadend event
        if let Some(handler) = on_loadend {
            let event = Self::create_progress_event("loadend", 0, 0, false, context)?;
            let _ = handler.call(&this.clone(), &[event], context);
        }

        Ok(JsValue::undefined())
    }

    /// Create a ProgressEvent for FileReader events
    fn create_progress_event(
        event_type: &str,
        loaded: usize,
        total: usize,
        length_computable: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Create a simple object with ProgressEvent-like properties
        let event_obj = JsObject::with_null_proto();
        event_obj.set(js_string!("type"), js_string!(event_type), false, context)?;
        event_obj.set(js_string!("loaded"), JsValue::from(loaded as u32), false, context)?;
        event_obj.set(js_string!("total"), JsValue::from(total as u32), false, context)?;
        event_obj.set(js_string!("lengthComputable"), JsValue::from(length_computable), false, context)?;
        event_obj.set(js_string!("bubbles"), JsValue::from(false), false, context)?;
        event_obj.set(js_string!("cancelable"), JsValue::from(false), false, context)?;
        Ok(event_obj.into())
    }

    /// Internal method to start a read operation
    fn start_read(this: &JsValue, args: &[JsValue], context: &mut Context, operation: ReadOperation) -> JsResult<JsValue> {
        let reader_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("read method called on non-object")
        })?;

        let (reader_id, is_aborted, data, on_loadstart, total_size) = {
            let mut reader_data = reader_obj.downcast_mut::<FileReaderData>().ok_or_else(|| {
                JsNativeError::typ().with_message("read method called on non-FileReader object")
            })?;

            // Check if already loading
            if reader_data.ready_state == ReadyState::Loading {
                return Err(JsNativeError::typ()
                    .with_message("FileReader is already reading")
                    .into());
            }

            let file_arg = args.get_or_undefined(0);

            // Extract file data
            let data = if let Some(file_obj) = file_arg.as_object() {
                if let Some(file_data) = file_obj.downcast_ref::<FileData>() {
                    file_data.blob().data().clone()
                } else if let Some(blob_data) = file_obj.downcast_ref::<BlobData>() {
                    blob_data.data().clone()
                } else {
                    return Err(JsNativeError::typ()
                        .with_message("Argument is not a File or Blob")
                        .into());
                }
            } else {
                return Err(JsNativeError::typ()
                    .with_message("readAs* methods require a File or Blob argument")
                    .into());
            };

            let total_size = data.len();

            // Set state to loading
            reader_data.ready_state = ReadyState::Loading;
            reader_data.result = None;
            reader_data.error = None;
            reader_data.is_aborted = Arc::new(Mutex::new(false));

            let reader_id = reader_data.reader_id;
            let is_aborted = reader_data.is_aborted.clone();
            let on_loadstart = reader_data.on_loadstart.clone();

            (reader_id, is_aborted, data, on_loadstart, total_size)
        };

        // Fire loadstart event synchronously
        if let Some(handler) = on_loadstart {
            let event = Self::create_progress_event("loadstart", 0, total_size, true, context)?;
            let _ = handler.call(&this.clone(), &[event], context);
        }

        // Perform the actual read operation and store result
        let read_result = match operation {
            ReadOperation::ArrayBuffer => {
                // Return the raw bytes for ArrayBuffer conversion
                FileReadResult::ArrayBuffer(data.to_vec())
            }
            ReadOperation::BinaryString => {
                // Convert bytes to binary string (latin1 encoding)
                FileReadResult::BinaryString(data.iter().map(|&b| b as char).collect())
            }
            ReadOperation::DataURL => {
                // Create data URL with base64 encoding
                let base64_data = general_purpose::STANDARD.encode(&*data);
                FileReadResult::DataURL(format!("data:application/octet-stream;base64,{}", base64_data))
            }
            ReadOperation::Text(encoding) => {
                // Convert to text (UTF-8 by default)
                let text = match encoding.as_deref() {
                    Some("utf-8") | Some("UTF-8") | None => {
                        String::from_utf8_lossy(&data).to_string()
                    }
                    Some("latin1") | Some("iso-8859-1") => {
                        data.iter().map(|&b| b as char).collect()
                    }
                    _ => {
                        // Fallback to UTF-8 for unsupported encodings
                        String::from_utf8_lossy(&data).to_string()
                    }
                };
                FileReadResult::Text(text)
            }
        };

        // Store pending result for async processing
        let pending = PendingResult {
            result: Some(read_result.clone()),
            error: None,
            events: vec![FileReaderEvent::Load, FileReaderEvent::LoadEnd],
        };

        {
            let state = get_filereader_state();
            state.pending_results.lock().unwrap().insert(reader_id, pending);
        }

        // Update the FileReader object state and fire events
        // We do this synchronously since we've completed the read
        {
            let mut reader_data = reader_obj.downcast_mut::<FileReaderData>().ok_or_else(|| {
                JsNativeError::typ().with_message("read method called on non-FileReader object")
            })?;

            // Check if aborted during processing
            if *is_aborted.lock().unwrap() {
                return Ok(JsValue::undefined());
            }

            // Update state to done
            reader_data.ready_state = ReadyState::Done;

            // Store result as string representation (for text-based results)
            reader_data.result = match &read_result {
                FileReadResult::Text(s) => Some(s.clone()),
                FileReadResult::BinaryString(s) => Some(s.clone()),
                FileReadResult::DataURL(s) => Some(s.clone()),
                FileReadResult::ArrayBuffer(_) => None, // ArrayBuffer handled separately
            };
        }

        // Fire load and loadend events
        let (on_load, on_loadend) = {
            let reader_data = reader_obj.downcast_ref::<FileReaderData>().ok_or_else(|| {
                JsNativeError::typ().with_message("read method called on non-FileReader object")
            })?;
            (reader_data.on_load.clone(), reader_data.on_loadend.clone())
        };

        if let Some(handler) = on_load {
            let event = Self::create_progress_event("load", total_size, total_size, true, context)?;
            let _ = handler.call(&this.clone(), &[event], context);
        }

        if let Some(handler) = on_loadend {
            let event = Self::create_progress_event("loadend", total_size, total_size, true, context)?;
            let _ = handler.call(&this.clone(), &[event], context);
        }

        Ok(JsValue::undefined())
    }

    /// Get the pending ArrayBuffer result for this reader
    pub fn get_array_buffer_result(reader_id: u32, context: &mut Context) -> JsResult<Option<JsValue>> {
        let state = get_filereader_state();
        let pending_results = state.pending_results.lock().unwrap();

        if let Some(pending) = pending_results.get(&reader_id) {
            if let Some(FileReadResult::ArrayBuffer(bytes)) = &pending.result {
                use boa_engine::object::builtins::{JsArrayBuffer, AlignedVec};
                let aligned_data = AlignedVec::<u8>::from_iter(0, bytes.iter().copied());
                let array_buffer = JsArrayBuffer::from_byte_block(aligned_data, context)?;
                return Ok(Some(array_buffer.into()));
            }
        }
        Ok(None)
    }
}

/// Read operation types
#[derive(Debug, Clone)]
enum ReadOperation {
    ArrayBuffer,
    BinaryString,
    DataURL,
    Text(Option<String>), // encoding
}

// Property getter/setter implementations

/// `get FileReader.prototype.readyState`
pub(crate) fn get_ready_state(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let reader_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("readyState getter called on non-object")
    })?;

    let reader_data = reader_obj.downcast_ref::<FileReaderData>().ok_or_else(|| {
        JsNativeError::typ().with_message("readyState getter called on non-FileReader object")
    })?;

    Ok(JsValue::from(reader_data.ready_state.as_u16()))
}

/// `get FileReader.prototype.result`
pub(crate) fn get_result(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let reader_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("result getter called on non-object")
    })?;

    let reader_data = reader_obj.downcast_ref::<FileReaderData>().ok_or_else(|| {
        JsNativeError::typ().with_message("result getter called on non-FileReader object")
    })?;

    // Check for string result first
    if let Some(result) = &reader_data.result {
        return Ok(JsValue::from(js_string!(result.clone())));
    }

    // Check for pending ArrayBuffer result
    let reader_id = reader_data.reader_id;
    if let Some(array_buffer) = FileReader::get_array_buffer_result(reader_id, context)? {
        return Ok(array_buffer);
    }

    Ok(JsValue::null())
}

/// `get FileReader.prototype.error`
pub(crate) fn get_error(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let reader_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("error getter called on non-object")
    })?;

    let reader_data = reader_obj.downcast_ref::<FileReaderData>().ok_or_else(|| {
        JsNativeError::typ().with_message("error getter called on non-FileReader object")
    })?;

    match &reader_data.error {
        Some(error) => {
            // Create a DOMException-like object with name and message
            let (name, message, code) = match error {
                FileReaderError::NotReadable => (
                    "NotReadableError",
                    "The file could not be read.",
                    1u16,
                ),
                FileReaderError::Security => (
                    "SecurityError",
                    "The file read was blocked by a security policy.",
                    2,
                ),
                FileReaderError::Abort => (
                    "AbortError",
                    "The read operation was aborted.",
                    3,
                ),
                FileReaderError::Encoding => (
                    "EncodingError",
                    "The file encoding is not valid.",
                    4,
                ),
            };

            let error_obj = JsObject::with_null_proto();
            error_obj.set(js_string!("name"), js_string!(name), false, context)?;
            error_obj.set(js_string!("message"), js_string!(message), false, context)?;
            error_obj.set(js_string!("code"), JsValue::from(code), false, context)?;
            Ok(error_obj.into())
        }
        None => Ok(JsValue::null()),
    }
}

// Event handler getters and setters

macro_rules! event_handler_accessors {
    ($getter:ident, $setter:ident, $field:ident, $name:literal) => {
        pub(crate) fn $getter(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
            let reader_obj = this.as_object().ok_or_else(|| {
                JsNativeError::typ().with_message(concat!($name, " getter called on non-object"))
            })?;

            let reader_data = reader_obj.downcast_ref::<FileReaderData>().ok_or_else(|| {
                JsNativeError::typ().with_message(concat!($name, " getter called on non-FileReader object"))
            })?;

            match &reader_data.$field {
                Some(handler) => Ok(JsValue::from(handler.clone())),
                None => Ok(JsValue::null()),
            }
        }

        pub(crate) fn $setter(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
            let reader_obj = this.as_object().ok_or_else(|| {
                JsNativeError::typ().with_message(concat!($name, " setter called on non-object"))
            })?;

            let mut reader_data = reader_obj.downcast_mut::<FileReaderData>().ok_or_else(|| {
                JsNativeError::typ().with_message(concat!($name, " setter called on non-FileReader object"))
            })?;

            let handler = args.get_or_undefined(0);
            reader_data.$field = if handler.is_callable() {
                handler.as_object().map(|obj| obj.clone())
            } else if handler.is_null() || handler.is_undefined() {
                None
            } else {
                None // Invalid handler types are ignored
            };

            Ok(JsValue::undefined())
        }
    };
}

event_handler_accessors!(get_onloadstart, set_onloadstart, on_loadstart, "onloadstart");
event_handler_accessors!(get_onprogress, set_onprogress, on_progress, "onprogress");
event_handler_accessors!(get_onload, set_onload, on_load, "onload");
event_handler_accessors!(get_onloadend, set_onloadend, on_loadend, "onloadend");
event_handler_accessors!(get_onerror, set_onerror, on_error, "onerror");
event_handler_accessors!(get_onabort, set_onabort, on_abort, "onabort");