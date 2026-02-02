//! HTMLImageElement Web API implementation
//!
//! Real implementation with actual image loading and decoding
//! https://html.spec.whatwg.org/multipage/embedded-content.html#htmlimageelement

use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject, FunctionObjectBuilder},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute,
    native_function::NativeFunction,
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};
use image::{DynamicImage, ImageFormat, GenericImageView};

/// Image loading state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageLoadState {
    /// Image not yet started loading
    Empty,
    /// Image is currently loading
    Loading,
    /// Image loaded successfully
    Complete,
    /// Image failed to load
    Broken,
}

/// Decoded image data storage
#[derive(Debug, Clone)]
pub struct DecodedImageData {
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
    /// RGBA pixel data
    pub rgba_data: Vec<u8>,
}

/// Internal data for HTMLImageElement
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct HTMLImageElementData {
    /// The src attribute (URL)
    #[unsafe_ignore_trace]
    src: Arc<Mutex<String>>,
    /// The alt attribute
    #[unsafe_ignore_trace]
    alt: Arc<Mutex<String>>,
    /// Width attribute (layout width)
    #[unsafe_ignore_trace]
    width: Arc<Mutex<u32>>,
    /// Height attribute (layout height)
    #[unsafe_ignore_trace]
    height: Arc<Mutex<u32>>,
    /// Natural width (intrinsic image width)
    #[unsafe_ignore_trace]
    natural_width: Arc<Mutex<u32>>,
    /// Natural height (intrinsic image height)
    #[unsafe_ignore_trace]
    natural_height: Arc<Mutex<u32>>,
    /// Loading state
    #[unsafe_ignore_trace]
    load_state: Arc<Mutex<ImageLoadState>>,
    /// crossOrigin attribute
    #[unsafe_ignore_trace]
    cross_origin: Arc<Mutex<Option<String>>>,
    /// isMap attribute
    #[unsafe_ignore_trace]
    is_map: Arc<Mutex<bool>>,
    /// loading attribute ("eager" or "lazy")
    #[unsafe_ignore_trace]
    loading: Arc<Mutex<String>>,
    /// decoding attribute ("sync", "async", or "auto")
    #[unsafe_ignore_trace]
    decoding: Arc<Mutex<String>>,
    /// Decoded image data (for Canvas drawImage)
    #[unsafe_ignore_trace]
    decoded_data: Arc<Mutex<Option<DecodedImageData>>>,
    /// Error message if loading failed
    #[unsafe_ignore_trace]
    error_message: Arc<Mutex<Option<String>>>,
}

impl HTMLImageElementData {
    pub fn new() -> Self {
        Self {
            src: Arc::new(Mutex::new(String::new())),
            alt: Arc::new(Mutex::new(String::new())),
            width: Arc::new(Mutex::new(0)),
            height: Arc::new(Mutex::new(0)),
            natural_width: Arc::new(Mutex::new(0)),
            natural_height: Arc::new(Mutex::new(0)),
            load_state: Arc::new(Mutex::new(ImageLoadState::Empty)),
            cross_origin: Arc::new(Mutex::new(None)),
            is_map: Arc::new(Mutex::new(false)),
            loading: Arc::new(Mutex::new("auto".to_string())),
            decoding: Arc::new(Mutex::new("auto".to_string())),
            decoded_data: Arc::new(Mutex::new(None)),
            error_message: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the current src
    pub fn get_src(&self) -> String {
        self.src.lock().unwrap().clone()
    }

    /// Set the src and trigger loading
    pub fn set_src(&self, value: String) {
        *self.src.lock().unwrap() = value;
    }

    /// Get complete status
    pub fn is_complete(&self) -> bool {
        matches!(*self.load_state.lock().unwrap(), ImageLoadState::Complete | ImageLoadState::Broken)
    }

    /// Get natural width
    pub fn get_natural_width(&self) -> u32 {
        *self.natural_width.lock().unwrap()
    }

    /// Get natural height
    pub fn get_natural_height(&self) -> u32 {
        *self.natural_height.lock().unwrap()
    }

    /// Get decoded image data for Canvas operations
    pub fn get_decoded_data(&self) -> Option<DecodedImageData> {
        self.decoded_data.lock().unwrap().clone()
    }

    /// Load and decode an image from bytes
    pub fn load_from_bytes(&self, bytes: &[u8]) -> Result<(), String> {
        *self.load_state.lock().unwrap() = ImageLoadState::Loading;

        match image::load_from_memory(bytes) {
            Ok(img) => {
                let (width, height) = img.dimensions();
                let rgba = img.to_rgba8();

                *self.natural_width.lock().unwrap() = width;
                *self.natural_height.lock().unwrap() = height;

                // Set layout dimensions if not already set
                if *self.width.lock().unwrap() == 0 {
                    *self.width.lock().unwrap() = width;
                }
                if *self.height.lock().unwrap() == 0 {
                    *self.height.lock().unwrap() = height;
                }

                *self.decoded_data.lock().unwrap() = Some(DecodedImageData {
                    width,
                    height,
                    rgba_data: rgba.into_raw(),
                });

                *self.load_state.lock().unwrap() = ImageLoadState::Complete;
                *self.error_message.lock().unwrap() = None;

                Ok(())
            }
            Err(e) => {
                *self.load_state.lock().unwrap() = ImageLoadState::Broken;
                let msg = format!("Failed to decode image: {}", e);
                *self.error_message.lock().unwrap() = Some(msg.clone());
                Err(msg)
            }
        }
    }
}

/// JavaScript `HTMLImageElement` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct HTMLImageElement;

impl IntrinsicObject for HTMLImageElement {
    fn init(realm: &Realm) {
        // Create accessor functions
        let src_getter = BuiltInBuilder::callable(realm, get_src)
            .name(js_string!("get src"))
            .build();
        let src_setter = BuiltInBuilder::callable(realm, set_src)
            .name(js_string!("set src"))
            .build();

        let alt_getter = BuiltInBuilder::callable(realm, get_alt)
            .name(js_string!("get alt"))
            .build();
        let alt_setter = BuiltInBuilder::callable(realm, set_alt)
            .name(js_string!("set alt"))
            .build();

        let width_getter = BuiltInBuilder::callable(realm, get_width)
            .name(js_string!("get width"))
            .build();
        let width_setter = BuiltInBuilder::callable(realm, set_width)
            .name(js_string!("set width"))
            .build();

        let height_getter = BuiltInBuilder::callable(realm, get_height)
            .name(js_string!("get height"))
            .build();
        let height_setter = BuiltInBuilder::callable(realm, set_height)
            .name(js_string!("set height"))
            .build();

        let natural_width_getter = BuiltInBuilder::callable(realm, get_natural_width)
            .name(js_string!("get naturalWidth"))
            .build();

        let natural_height_getter = BuiltInBuilder::callable(realm, get_natural_height)
            .name(js_string!("get naturalHeight"))
            .build();

        let complete_getter = BuiltInBuilder::callable(realm, get_complete)
            .name(js_string!("get complete"))
            .build();

        let current_src_getter = BuiltInBuilder::callable(realm, get_current_src)
            .name(js_string!("get currentSrc"))
            .build();

        let cross_origin_getter = BuiltInBuilder::callable(realm, get_cross_origin)
            .name(js_string!("get crossOrigin"))
            .build();
        let cross_origin_setter = BuiltInBuilder::callable(realm, set_cross_origin)
            .name(js_string!("set crossOrigin"))
            .build();

        let is_map_getter = BuiltInBuilder::callable(realm, get_is_map)
            .name(js_string!("get isMap"))
            .build();
        let is_map_setter = BuiltInBuilder::callable(realm, set_is_map)
            .name(js_string!("set isMap"))
            .build();

        let loading_getter = BuiltInBuilder::callable(realm, get_loading)
            .name(js_string!("get loading"))
            .build();
        let loading_setter = BuiltInBuilder::callable(realm, set_loading)
            .name(js_string!("set loading"))
            .build();

        let decoding_getter = BuiltInBuilder::callable(realm, get_decoding)
            .name(js_string!("get decoding"))
            .build();
        let decoding_setter = BuiltInBuilder::callable(realm, set_decoding)
            .name(js_string!("set decoding"))
            .build();

        let tag_name_getter = BuiltInBuilder::callable(realm, get_tag_name)
            .name(js_string!("get tagName"))
            .build();

        let node_name_getter = BuiltInBuilder::callable(realm, get_tag_name)
            .name(js_string!("get nodeName"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Properties with accessors
            .accessor(
                js_string!("src"),
                Some(src_getter),
                Some(src_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("alt"),
                Some(alt_getter),
                Some(alt_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("width"),
                Some(width_getter),
                Some(width_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("height"),
                Some(height_getter),
                Some(height_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("naturalWidth"),
                Some(natural_width_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("naturalHeight"),
                Some(natural_height_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("complete"),
                Some(complete_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("currentSrc"),
                Some(current_src_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("crossOrigin"),
                Some(cross_origin_getter),
                Some(cross_origin_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("isMap"),
                Some(is_map_getter),
                Some(is_map_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("loading"),
                Some(loading_getter),
                Some(loading_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("decoding"),
                Some(decoding_getter),
                Some(decoding_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("tagName"),
                Some(tag_name_getter.clone()),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("nodeName"),
                Some(node_name_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            // Methods
            .method(decode, js_string!("decode"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLImageElement {
    const NAME: JsString = StaticJsStrings::HTML_IMAGE_ELEMENT;
}

impl BuiltInConstructor for HTMLImageElement {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 50;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 50;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_image_element;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If new_target is undefined then this function was called without new
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling HTMLImageElement constructor without `new` is forbidden")
                .into());
        }

        // Get the prototype
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_image_element,
            context,
        )?;

        // Create internal data
        let data = HTMLImageElementData::new();

        // Handle optional width/height arguments
        if let Some(width_arg) = args.get(0) {
            if let Some(width) = width_arg.as_number() {
                *data.width.lock().unwrap() = width as u32;
            }
        }
        if let Some(height_arg) = args.get(1) {
            if let Some(height) = height_arg.as_number() {
                *data.height.lock().unwrap() = height as u32;
            }
        }

        // Create the object with prototype and data
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            data,
        );

        Ok(obj.into())
    }
}

// ============== Property Accessors ==============

fn get_src(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            return Ok(js_string!(data.src.lock().unwrap().clone()).into());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn set_src(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            let value = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

            // Update the src
            *data.src.lock().unwrap() = value.clone();

            // Start loading if we have a URL
            if !value.is_empty() {
                *data.load_state.lock().unwrap() = ImageLoadState::Loading;

                // Synchronous loading for now (async will be added with proper event loop integration)
                // In a real browser, this would use fetch() asynchronously
                if let Ok(url) = url::Url::parse(&value) {
                    // Use tokio block_on since rquest doesn't have a blocking module
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build();

                    match rt {
                        Ok(runtime) => {
                            let url_clone = url.to_string();
                            let result = runtime.block_on(async {
                                let client = rquest::Client::builder()
                                    .timeout(std::time::Duration::from_secs(30))
                                    .build()?;
                                let response = client.get(&url_clone).send().await?;
                                let status = response.status();
                                let bytes = response.bytes().await?;
                                Ok::<(rquest::StatusCode, bytes::Bytes), rquest::Error>((status, bytes))
                            });

                            match result {
                                Ok((status, bytes)) => {
                                    if status.is_success() {
                                        if let Err(e) = data.load_from_bytes(&bytes) {
                                            eprintln!("Image decode error: {}", e);
                                        }
                                    } else {
                                        *data.load_state.lock().unwrap() = ImageLoadState::Broken;
                                        *data.error_message.lock().unwrap() = Some(format!("HTTP error: {}", status));
                                    }
                                }
                                Err(e) => {
                                    *data.load_state.lock().unwrap() = ImageLoadState::Broken;
                                    *data.error_message.lock().unwrap() = Some(format!("Network error: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            *data.load_state.lock().unwrap() = ImageLoadState::Broken;
                            *data.error_message.lock().unwrap() = Some(format!("Failed to create runtime: {}", e));
                        }
                    }
                } else if value.starts_with("data:") {
                    // Handle data URLs
                    if let Some(base64_start) = value.find("base64,") {
                        let base64_data = &value[base64_start + 7..];
                        match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_data) {
                            Ok(bytes) => {
                                if let Err(e) = data.load_from_bytes(&bytes) {
                                    eprintln!("Image decode error from data URL: {}", e);
                                }
                            }
                            Err(e) => {
                                *data.load_state.lock().unwrap() = ImageLoadState::Broken;
                                *data.error_message.lock().unwrap() = Some(format!("Invalid base64: {}", e));
                            }
                        }
                    }
                }
            }

            return Ok(JsValue::undefined());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn get_alt(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            return Ok(js_string!(data.alt.lock().unwrap().clone()).into());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn set_alt(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            let value = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
            *data.alt.lock().unwrap() = value;
            return Ok(JsValue::undefined());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn get_width(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            return Ok(JsValue::from(*data.width.lock().unwrap()));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn set_width(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
                *data.width.lock().unwrap() = value as u32;
            }
            return Ok(JsValue::undefined());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn get_height(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            return Ok(JsValue::from(*data.height.lock().unwrap()));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn set_height(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            if let Some(value) = args.get(0).and_then(|v| v.as_number()) {
                *data.height.lock().unwrap() = value as u32;
            }
            return Ok(JsValue::undefined());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn get_natural_width(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            return Ok(JsValue::from(*data.natural_width.lock().unwrap()));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn get_natural_height(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            return Ok(JsValue::from(*data.natural_height.lock().unwrap()));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn get_complete(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            let state = *data.load_state.lock().unwrap();
            let complete = matches!(state, ImageLoadState::Complete | ImageLoadState::Broken)
                || data.get_src().is_empty();
            return Ok(JsValue::from(complete));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn get_current_src(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // currentSrc returns the actual resolved URL being used
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            let src = data.src.lock().unwrap().clone();
            // Only return src if image is complete
            if *data.load_state.lock().unwrap() == ImageLoadState::Complete {
                return Ok(js_string!(src).into());
            }
            return Ok(js_string!("").into());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn get_cross_origin(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            if let Some(ref value) = *data.cross_origin.lock().unwrap() {
                return Ok(js_string!(value.clone()).into());
            }
            return Ok(JsValue::null());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn set_cross_origin(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            if args.get_or_undefined(0).is_null() {
                *data.cross_origin.lock().unwrap() = None;
            } else {
                let value = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
                *data.cross_origin.lock().unwrap() = Some(value);
            }
            return Ok(JsValue::undefined());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn get_is_map(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            return Ok(JsValue::from(*data.is_map.lock().unwrap()));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn set_is_map(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            *data.is_map.lock().unwrap() = args.get_or_undefined(0).to_boolean();
            return Ok(JsValue::undefined());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn get_loading(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            return Ok(js_string!(data.loading.lock().unwrap().clone()).into());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn set_loading(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            let value = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
            // Normalize to valid values: "eager" or "lazy"
            let normalized = match value.to_lowercase().as_str() {
                "lazy" => "lazy",
                _ => "eager",
            };
            *data.loading.lock().unwrap() = normalized.to_string();
            return Ok(JsValue::undefined());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn get_decoding(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            return Ok(js_string!(data.decoding.lock().unwrap().clone()).into());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn set_decoding(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            let value = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
            // Normalize to valid values: "sync", "async", or "auto"
            let normalized = match value.to_lowercase().as_str() {
                "sync" => "sync",
                "async" => "async",
                _ => "auto",
            };
            *data.decoding.lock().unwrap() = normalized.to_string();
            return Ok(JsValue::undefined());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLImageElement")
        .into())
}

fn get_tag_name(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Ok(js_string!("IMG").into())
}

// ============== Methods ==============

/// The decode() method returns a Promise that resolves when the image is decoded and ready to draw
fn decode(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLImageElementData>() {
            let load_state = *data.load_state.lock().unwrap();

            match load_state {
                ImageLoadState::Complete | ImageLoadState::Loading => {
                    // Already decoded or loading completes synchronously, return resolved promise
                    // Use JavaScript to create a resolved promise
                    context.eval(boa_engine::Source::from_bytes("Promise.resolve()"))
                }
                ImageLoadState::Broken => {
                    // Failed to decode, return rejected promise
                    let error_msg = data.error_message.lock().unwrap().clone()
                        .unwrap_or_else(|| "Image decoding failed".to_string());
                    let code = format!("Promise.reject(new Error({:?}))", error_msg);
                    context.eval(boa_engine::Source::from_bytes(&code))
                }
                ImageLoadState::Empty => {
                    // No source set, return rejected promise
                    context.eval(boa_engine::Source::from_bytes(
                        "Promise.reject(new Error('No image source specified'))"
                    ))
                }
            }
        } else {
            context.eval(boa_engine::Source::from_bytes(
                "Promise.reject(new TypeError(\"'this' is not an HTMLImageElement\"))"
            ))
        }
    } else {
        context.eval(boa_engine::Source::from_bytes(
            "Promise.reject(new TypeError(\"'this' is not an HTMLImageElement\"))"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::{Context, Source};

    #[test]
    fn test_image_constructor() {
        let mut context = Context::default();
        crate::initialize_browser_apis(&mut context).unwrap();

        // Test basic constructor
        let result = context.eval(Source::from_bytes(r#"
            let img = new HTMLImageElement();
            img.tagName === 'IMG' && img.naturalWidth === 0 && img.complete === true
        "#));
        assert!(result.is_ok());
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_image_constructor_with_dimensions() {
        let mut context = Context::default();
        crate::initialize_browser_apis(&mut context).unwrap();

        let result = context.eval(Source::from_bytes(r#"
            let img = new HTMLImageElement(100, 200);
            img.width === 100 && img.height === 200
        "#));
        assert!(result.is_ok());
        assert!(result.unwrap().to_boolean());
    }

    #[test]
    fn test_image_properties() {
        let mut context = Context::default();
        crate::initialize_browser_apis(&mut context).unwrap();

        let result = context.eval(Source::from_bytes(r#"
            let img = new HTMLImageElement();
            img.alt = 'Test image';
            img.crossOrigin = 'anonymous';
            img.loading = 'lazy';
            img.decoding = 'async';

            img.alt === 'Test image' &&
            img.crossOrigin === 'anonymous' &&
            img.loading === 'lazy' &&
            img.decoding === 'async'
        "#));
        assert!(result.is_ok());
        assert!(result.unwrap().to_boolean());
    }
}
