//! Clipboard API implementation for Boa
//!
//! Implements the Async Clipboard API as defined in:
//! https://w3c.github.io/clipboard-apis/
//!
//! This provides an in-memory clipboard for headless browser operation.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, NativeFunction,
    builtins::promise::Promise,
    js_string,
    object::{FunctionObjectBuilder, JsObject, ObjectInitializer},
    property::Attribute,
    value::JsValue,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Global clipboard storage (shared across all Clipboard instances)
static CLIPBOARD_STORAGE: std::sync::LazyLock<Arc<Mutex<ClipboardStorage>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(ClipboardStorage::new())));

/// Internal clipboard storage
#[derive(Debug, Clone)]
struct ClipboardStorage {
    /// Plain text content
    text: String,
    /// MIME type to data mapping for rich content
    items: HashMap<String, Vec<u8>>,
}

impl ClipboardStorage {
    fn new() -> Self {
        Self {
            text: String::new(),
            items: HashMap::new(),
        }
    }
}

/// Create a Clipboard instance for navigator.clipboard
pub fn create_clipboard(context: &mut Context) -> JsResult<JsObject> {
    let clipboard = ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(clipboard_read_text),
            js_string!("readText"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(clipboard_write_text),
            js_string!("writeText"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(clipboard_read),
            js_string!("read"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(clipboard_write),
            js_string!("write"),
            1,
        )
        .build();

    Ok(clipboard)
}

/// `Clipboard.prototype.readText()` - Returns a Promise that resolves with the clipboard's text content
fn clipboard_read_text(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let text = {
        let storage = CLIPBOARD_STORAGE.lock().unwrap();
        storage.text.clone()
    };

    // Return a Promise that resolves with the text
    let promise_constructor = context.intrinsics().constructors().promise().constructor();
    Promise::resolve(
        &promise_constructor.into(),
        &[JsValue::from(js_string!(text))],
        context,
    )
}

/// `Clipboard.prototype.writeText(text)` - Writes text to the clipboard
fn clipboard_write_text(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let text = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    {
        let mut storage = CLIPBOARD_STORAGE.lock().unwrap();
        let text_bytes = text.as_bytes().to_vec();
        storage.text = text;
        // Also store as text/plain MIME type
        storage.items.insert("text/plain".to_string(), text_bytes);
    }

    // Return a Promise that resolves to undefined
    let promise_constructor = context.intrinsics().constructors().promise().constructor();
    Promise::resolve(
        &promise_constructor.into(),
        &[JsValue::undefined()],
        context,
    )
}

/// `Clipboard.prototype.read()` - Returns a Promise that resolves with ClipboardItem array
fn clipboard_read(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let items = {
        let storage = CLIPBOARD_STORAGE.lock().unwrap();
        storage.items.clone()
    };

    // Create ClipboardItem array
    let clipboard_items = boa_engine::object::JsArray::new(context)?;

    if !items.is_empty() {
        // Create a ClipboardItem with all the stored MIME types
        let clipboard_item = create_clipboard_item(&items, context)?;
        clipboard_items.set(0u32, JsValue::from(clipboard_item), false, context)?;
    }

    // Return a Promise that resolves with the items array
    let promise_constructor = context.intrinsics().constructors().promise().constructor();
    Promise::resolve(
        &promise_constructor.into(),
        &[clipboard_items.into()],
        context,
    )
}

/// `Clipboard.prototype.write(data)` - Writes ClipboardItem array to the clipboard
fn clipboard_write(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let data = args.get_or_undefined(0);

    // data should be an array of ClipboardItem objects
    if let Some(array_obj) = data.as_object() {
        let length = array_obj
            .get(js_string!("length"), context)?
            .to_u32(context)
            .unwrap_or(0);

        let mut storage = CLIPBOARD_STORAGE.lock().unwrap();
        storage.items.clear();

        for i in 0..length {
            if let Ok(item_val) = array_obj.get(i, context) {
                if let Some(item_obj) = item_val.as_object() {
                    // Check for _data property (our internal format)
                    if let Ok(data_obj_val) = item_obj.get(js_string!("_data"), context) {
                        if let Some(data_obj) = data_obj_val.as_object() {
                            // Iterate through data object properties
                            if let Ok(keys) = data_obj.own_property_keys(context) {
                                for key in keys {
                                    let key_str = key.to_string();
                                    if let Ok(value) = data_obj.get(key.clone(), context) {
                                        let data_str =
                                            value.to_string(context)?.to_std_string_escaped();

                                        // Update plain text if text/plain
                                        if key_str == "text/plain" {
                                            storage.text = data_str.clone();
                                        }

                                        storage.items.insert(key_str, data_str.into_bytes());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Return a Promise that resolves to undefined
    let promise_constructor = context.intrinsics().constructors().promise().constructor();
    Promise::resolve(
        &promise_constructor.into(),
        &[JsValue::undefined()],
        context,
    )
}

/// Create a ClipboardItem JavaScript object
fn create_clipboard_item(
    items: &HashMap<String, Vec<u8>>,
    context: &mut Context,
) -> JsResult<JsObject> {
    // Create types array
    let types_array = boa_engine::object::JsArray::new(context)?;
    for (i, mime_type) in items.keys().enumerate() {
        types_array.set(
            i as u32,
            JsValue::from(js_string!(mime_type.clone())),
            false,
            context,
        )?;
    }

    // Create internal data storage
    let data_obj = ObjectInitializer::new(context).build();
    for (mime_type, data) in items {
        let data_str = String::from_utf8_lossy(data).to_string();
        data_obj.set(
            js_string!(mime_type.clone()),
            js_string!(data_str),
            false,
            context,
        )?;
    }

    // Create ClipboardItem object with getType method
    let clipboard_item = ObjectInitializer::new(context)
        .property(
            js_string!("types"),
            types_array,
            Attribute::READONLY | Attribute::ENUMERABLE,
        )
        .property(js_string!("_data"), data_obj, Attribute::all())
        .function(
            NativeFunction::from_fn_ptr(clipboard_item_get_type),
            js_string!("getType"),
            1,
        )
        .build();

    Ok(clipboard_item)
}

/// ClipboardItem.getType(type) - Returns a Promise that resolves with a Blob
fn clipboard_item_get_type(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("ClipboardItem.getType called on non-object")
    })?;

    let mime_type = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    // Try to get data from _data property
    let data_val = this_obj.get(js_string!("_data"), context)?;
    let promise_constructor = context.intrinsics().constructors().promise().constructor();

    if let Some(data_obj) = data_val.as_object() {
        if let Ok(value) = data_obj.get(js_string!(mime_type.clone()), context) {
            if !value.is_undefined() {
                let value_str = value.to_string(context)?.to_std_string_escaped();

                // Create a Blob-like object
                let blob = ObjectInitializer::new(context)
                    .property(js_string!("size"), value_str.len(), Attribute::READONLY)
                    .property(
                        js_string!("type"),
                        js_string!(mime_type),
                        Attribute::READONLY,
                    )
                    .property(
                        js_string!("_bytes"),
                        js_string!(value_str.clone()),
                        Attribute::all(),
                    )
                    .function(
                        NativeFunction::from_fn_ptr(blob_text),
                        js_string!("text"),
                        0,
                    )
                    .build();

                return Promise::resolve(
                    &promise_constructor.into(),
                    &[JsValue::from(blob)],
                    context,
                );
            }
        }
    }

    // Type not found
    Err(JsNativeError::typ()
        .with_message(format!("The type '{}' was not found", mime_type))
        .into())
}

/// Blob.text() - Returns a Promise that resolves with the blob's text content
fn blob_text(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Blob.text called on non-object"))?;

    let bytes = this_obj.get(js_string!("_bytes"), context)?;
    let text = bytes.to_string(context)?.to_std_string_escaped();

    let promise_constructor = context.intrinsics().constructors().promise().constructor();
    Promise::resolve(
        &promise_constructor.into(),
        &[JsValue::from(js_string!(text))],
        context,
    )
}

/// ClipboardItem constructor implementation
fn clipboard_item_constructor(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let data_arg = args.get_or_undefined(0);

    // data should be an object with MIME types as keys
    let mut items: HashMap<String, Vec<u8>> = HashMap::new();

    if let Some(data_obj) = data_arg.as_object() {
        if let Ok(keys) = data_obj.own_property_keys(context) {
            for key in keys {
                let key_str = key.to_string();
                if let Ok(value) = data_obj.get(key.clone(), context) {
                    // Convert value to string for storage
                    let value_str = value.to_string(context)?.to_std_string_escaped();
                    items.insert(key_str, value_str.into_bytes());
                }
            }
        }
    }

    create_clipboard_item(&items, context).map(|obj| JsValue::from(obj))
}

/// Create a ClipboardItem constructor function
pub fn create_clipboard_item_constructor(context: &mut Context) -> JsResult<JsObject> {
    // Create constructor function using FunctionObjectBuilder with constructor capability
    let constructor_obj = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(clipboard_item_constructor),
    )
    .name(js_string!("ClipboardItem"))
    .length(1)
    .constructor(true)
    .build();

    Ok(constructor_obj.into())
}
