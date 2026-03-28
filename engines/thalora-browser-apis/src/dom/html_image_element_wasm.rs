//! HTMLImageElement stub for WASM builds
//!
//! In WASM builds, the browser's native HTMLImageElement is used directly.

use boa_engine::{
    Context, JsData, JsNativeError, JsResult, JsString, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    realm::Realm,
    string::StaticJsStrings,
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};

/// Image loading state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageLoadState {
    Empty,
    Loading,
    Complete,
    Broken,
}

/// Decoded image data storage
#[derive(Debug, Clone)]
pub struct DecodedImageData {
    pub width: u32,
    pub height: u32,
    pub rgba_data: Vec<u8>,
}

#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct HTMLImageElementData {
    #[unsafe_ignore_trace]
    src: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    alt: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    width: Arc<Mutex<u32>>,
    #[unsafe_ignore_trace]
    height: Arc<Mutex<u32>>,
    #[unsafe_ignore_trace]
    natural_width: Arc<Mutex<u32>>,
    #[unsafe_ignore_trace]
    natural_height: Arc<Mutex<u32>>,
    #[unsafe_ignore_trace]
    load_state: Arc<Mutex<ImageLoadState>>,
    #[unsafe_ignore_trace]
    cross_origin: Arc<Mutex<Option<String>>>,
    #[unsafe_ignore_trace]
    is_map: Arc<Mutex<bool>>,
    #[unsafe_ignore_trace]
    loading: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    decoding: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    decoded_data: Arc<Mutex<Option<DecodedImageData>>>,
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

    pub fn get_decoded_data(&self) -> Option<DecodedImageData> {
        self.decoded_data.lock().ok()?.clone()
    }

    pub fn get_natural_dimensions(&self) -> (u32, u32) {
        let w = *self.natural_width.lock().unwrap();
        let h = *self.natural_height.lock().unwrap();
        (w, h)
    }

    pub fn is_complete(&self) -> bool {
        *self.load_state.lock().unwrap() == ImageLoadState::Complete
    }
}

#[derive(Debug, Copy, Clone)]
pub struct HTMLImageElement;

impl IntrinsicObject for HTMLImageElement {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm).build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLImageElement {
    const NAME: JsString = StaticJsStrings::HTML_IMAGE_ELEMENT;
}

impl BuiltInConstructor for HTMLImageElement {
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_image_element;

    fn constructor(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message(
                "HTMLImageElement is not available in WASM. Use the browser's native Image().",
            )
            .into())
    }
}
