//! ImageBitmap Web API implementation
//!
//! ImageBitmap represents a bitmap image which can be drawn to canvas without undue latency.
//! https://html.spec.whatwg.org/multipage/imagebitmap-and-animations.html#imagebitmap

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};

/// Internal data for ImageBitmap
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct ImageBitmapData {
    /// Image width
    #[unsafe_ignore_trace]
    width: Arc<Mutex<u32>>,
    /// Image height
    #[unsafe_ignore_trace]
    height: Arc<Mutex<u32>>,
    /// RGBA pixel data
    #[unsafe_ignore_trace]
    rgba_data: Arc<Mutex<Option<Vec<u8>>>>,
    /// Whether the bitmap has been closed
    #[unsafe_ignore_trace]
    closed: Arc<Mutex<bool>>,
}

impl Default for ImageBitmapData {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageBitmapData {
    pub fn new() -> Self {
        Self {
            width: Arc::new(Mutex::new(0)),
            height: Arc::new(Mutex::new(0)),
            rgba_data: Arc::new(Mutex::new(None)),
            closed: Arc::new(Mutex::new(false)),
        }
    }

    /// Create an ImageBitmap from RGBA pixel data
    pub fn from_rgba(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width: Arc::new(Mutex::new(width)),
            height: Arc::new(Mutex::new(height)),
            rgba_data: Arc::new(Mutex::new(Some(data))),
            closed: Arc::new(Mutex::new(false)),
        }
    }

    /// Get the width
    pub fn get_width(&self) -> u32 {
        if *self.closed.lock().unwrap() {
            0
        } else {
            *self.width.lock().unwrap()
        }
    }

    /// Get the height
    pub fn get_height(&self) -> u32 {
        if *self.closed.lock().unwrap() {
            0
        } else {
            *self.height.lock().unwrap()
        }
    }

    /// Get the pixel data (for Canvas operations)
    pub fn get_rgba_data(&self) -> Option<Vec<u8>> {
        if *self.closed.lock().unwrap() {
            None
        } else {
            self.rgba_data.lock().unwrap().clone()
        }
    }

    /// Check if the bitmap has been closed
    pub fn is_closed(&self) -> bool {
        *self.closed.lock().unwrap()
    }

    /// Close the bitmap and release resources
    pub fn close(&self) {
        *self.closed.lock().unwrap() = true;
        *self.rgba_data.lock().unwrap() = None;
        *self.width.lock().unwrap() = 0;
        *self.height.lock().unwrap() = 0;
    }
}

/// JavaScript `ImageBitmap` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct ImageBitmap;

impl IntrinsicObject for ImageBitmap {
    fn init(realm: &Realm) {
        let width_getter = BuiltInBuilder::callable(realm, get_width)
            .name(js_string!("get width"))
            .build();

        let height_getter = BuiltInBuilder::callable(realm, get_height)
            .name(js_string!("get height"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("width"),
                Some(width_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("height"),
                Some(height_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(close, js_string!("close"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for ImageBitmap {
    const NAME: JsString = StaticJsStrings::IMAGE_BITMAP;
}

impl BuiltInConstructor for ImageBitmap {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 20;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 20;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::image_bitmap;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // ImageBitmap objects cannot be directly constructed
        // They are created via createImageBitmap()
        Err(JsNativeError::typ()
            .with_message("ImageBitmap objects cannot be directly constructed; use createImageBitmap() instead")
            .into())
    }
}

/// Create an ImageBitmap object from data (used internally by createImageBitmap)
pub fn create_image_bitmap_from_data(
    width: u32,
    height: u32,
    data: Vec<u8>,
    context: &mut Context,
) -> JsResult<JsObject> {
    let prototype = context
        .intrinsics()
        .constructors()
        .image_bitmap()
        .prototype();

    let bitmap_data = ImageBitmapData::from_rgba(width, height, data);

    let obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        prototype,
        bitmap_data,
    );

    Ok(obj.upcast())
}

// ============== Property Accessors ==============

fn get_width(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object()
        && let Some(data) = obj.downcast_ref::<ImageBitmapData>()
    {
        return Ok(JsValue::from(data.get_width()));
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an ImageBitmap")
        .into())
}

fn get_height(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object()
        && let Some(data) = obj.downcast_ref::<ImageBitmapData>()
    {
        return Ok(JsValue::from(data.get_height()));
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an ImageBitmap")
        .into())
}

// ============== Methods ==============

fn close(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object()
        && let Some(data) = obj.downcast_ref::<ImageBitmapData>()
    {
        data.close();
        return Ok(JsValue::undefined());
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an ImageBitmap")
        .into())
}

/// The createImageBitmap global function
/// Creates an ImageBitmap from various sources
pub fn create_image_bitmap(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    use super::html_image_element::HTMLImageElementData;

    let source = args.get_or_undefined(0);

    // Check if source is an HTMLImageElement
    if let Some(obj) = source.as_object() {
        if let Some(img_data) = obj.downcast_ref::<HTMLImageElementData>() {
            // Get the decoded image data
            if let Some(decoded) = img_data.get_decoded_data() {
                let bitmap = create_image_bitmap_from_data(
                    decoded.width,
                    decoded.height,
                    decoded.rgba_data,
                    context,
                )?;

                // Return a resolved promise with the bitmap
                let code = "Promise.resolve";
                let promise_resolve = context.eval(boa_engine::Source::from_bytes(code))?;

                if let Some(resolve_fn) = promise_resolve.as_callable() {
                    return resolve_fn.call(&JsValue::undefined(), &[bitmap.into()], context);
                }
            } else {
                // Image not loaded yet or failed
                return context.eval(boa_engine::Source::from_bytes(
                    "Promise.reject(new Error('Image is not loaded'))",
                ));
            }
        }

        // Check if source is an ImageBitmap
        if let Some(bitmap_data) = obj.downcast_ref::<ImageBitmapData>() {
            if bitmap_data.is_closed() {
                return context.eval(boa_engine::Source::from_bytes(
                    "Promise.reject(new Error('ImageBitmap is closed'))",
                ));
            }

            // Clone the bitmap data
            if let Some(data) = bitmap_data.get_rgba_data() {
                let width = bitmap_data.get_width();
                let height = bitmap_data.get_height();
                let new_bitmap = create_image_bitmap_from_data(width, height, data, context)?;

                let code = "Promise.resolve";
                let promise_resolve = context.eval(boa_engine::Source::from_bytes(code))?;

                if let Some(resolve_fn) = promise_resolve.as_callable() {
                    return resolve_fn.call(&JsValue::undefined(), &[new_bitmap.into()], context);
                }
            }
        }
    }

    // Unsupported source type
    context.eval(boa_engine::Source::from_bytes(
        "Promise.reject(new TypeError('createImageBitmap: unsupported source type'))",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::{Context, Source};

    #[test]
    fn test_image_bitmap_cannot_construct_directly() {
        let mut context = Context::default();
        crate::initialize_browser_apis(&mut context).unwrap();

        // Direct construction should fail
        let result = context.eval(Source::from_bytes(
            r#"
            try {
                new ImageBitmap();
                false;
            } catch (e) {
                e instanceof TypeError;
            }
        "#,
        ));
        assert!(result.is_ok());
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_image_bitmap_properties() {
        let mut context = Context::default();
        crate::initialize_browser_apis(&mut context).unwrap();

        // ImageBitmap should exist as a constructor
        let result = context.eval(Source::from_bytes("typeof ImageBitmap === 'function'"));
        assert!(result.is_ok());
        assert!(result.unwrap().to_boolean());
    }
}
