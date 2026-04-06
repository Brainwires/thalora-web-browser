//! OffscreenCanvas Web API implementation
//!
//! OffscreenCanvas provides a canvas that can be rendered off-screen.
//! It can be used in both window and worker contexts.
//! https://html.spec.whatwg.org/multipage/canvas.html#the-offscreencanvas-interface

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, Source,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};

use super::canvas_state::CanvasState;
use super::rendering_context_2d::CanvasRenderingContext2DData;
use crate::dom::image_bitmap::create_image_bitmap_from_data;

/// Internal data for OffscreenCanvas
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct OffscreenCanvasData {
    /// Canvas width
    #[unsafe_ignore_trace]
    width: Arc<Mutex<u32>>,
    /// Canvas height
    #[unsafe_ignore_trace]
    height: Arc<Mutex<u32>>,
    /// The canvas state (shared with 2D context)
    #[unsafe_ignore_trace]
    state: Arc<Mutex<Option<CanvasState>>>,
    /// The 2D rendering context (cached)
    #[unsafe_ignore_trace]
    context_2d: Arc<Mutex<Option<JsObject>>>,
}

impl OffscreenCanvasData {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width: Arc::new(Mutex::new(width)),
            height: Arc::new(Mutex::new(height)),
            state: Arc::new(Mutex::new(CanvasState::new(width, height))),
            context_2d: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_width(&self) -> u32 {
        *self.width.lock().unwrap()
    }

    pub fn get_height(&self) -> u32 {
        *self.height.lock().unwrap()
    }

    pub fn set_width(&self, width: u32) {
        *self.width.lock().unwrap() = width;
        *self.state.lock().unwrap() = CanvasState::new(width, self.get_height());
        *self.context_2d.lock().unwrap() = None;
    }

    pub fn set_height(&self, height: u32) {
        *self.height.lock().unwrap() = height;
        *self.state.lock().unwrap() = CanvasState::new(self.get_width(), height);
        *self.context_2d.lock().unwrap() = None;
    }

    pub fn get_state(&self) -> Arc<Mutex<Option<CanvasState>>> {
        self.state.clone()
    }

    pub fn set_context_2d(&self, ctx: JsObject) {
        *self.context_2d.lock().unwrap() = Some(ctx);
    }

    pub fn get_context_2d(&self) -> Option<JsObject> {
        self.context_2d.lock().unwrap().clone()
    }

    /// Get the pixel data as RGBA bytes
    pub fn get_image_data(&self) -> Option<(u32, u32, Vec<u8>)> {
        let state = self.state.lock().unwrap();
        state.as_ref().map(|s| {
            let w = s.pixmap.width();
            let h = s.pixmap.height();
            let data = s.get_image_data(0, 0, w, h);
            (w, h, data)
        })
    }
}

/// JavaScript `OffscreenCanvas` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct OffscreenCanvas;

impl IntrinsicObject for OffscreenCanvas {
    fn init(realm: &Realm) {
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

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
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
            .method(get_context, js_string!("getContext"), 1)
            .method(convert_to_blob, js_string!("convertToBlob"), 0)
            .method(
                transfer_to_image_bitmap,
                js_string!("transferToImageBitmap"),
                0,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for OffscreenCanvas {
    const NAME: JsString = StaticJsStrings::OFFSCREEN_CANVAS;
}

impl BuiltInConstructor for OffscreenCanvas {
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const PROTOTYPE_STORAGE_SLOTS: usize = 20;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 20;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::offscreen_canvas;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::offscreen_canvas,
            context,
        )?;

        let width = args
            .get_or_undefined(0)
            .to_number(context)
            .map(|n| n as u32)
            .unwrap_or(300);

        let height = args
            .get_or_undefined(1)
            .to_number(context)
            .map(|n| n as u32)
            .unwrap_or(150);

        let canvas_data = OffscreenCanvasData::new(width, height);

        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            canvas_data,
        );

        Ok(obj.into())
    }
}

// ============== Property Accessors ==============

fn get_width(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object()
        && let Some(data) = obj.downcast_ref::<OffscreenCanvasData>()
    {
        return Ok(JsValue::from(data.get_width()));
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an OffscreenCanvas")
        .into())
}

fn set_width(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object()
        && let Some(data) = obj.downcast_ref::<OffscreenCanvasData>()
    {
        let width = args.get_or_undefined(0).to_number(context)? as u32;
        data.set_width(width);
        return Ok(JsValue::undefined());
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an OffscreenCanvas")
        .into())
}

fn get_height(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object()
        && let Some(data) = obj.downcast_ref::<OffscreenCanvasData>()
    {
        return Ok(JsValue::from(data.get_height()));
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an OffscreenCanvas")
        .into())
}

fn set_height(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object()
        && let Some(data) = obj.downcast_ref::<OffscreenCanvasData>()
    {
        let height = args.get_or_undefined(0).to_number(context)? as u32;
        data.set_height(height);
        return Ok(JsValue::undefined());
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an OffscreenCanvas")
        .into())
}

// ============== Methods ==============

fn get_context(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

    let canvas_data = this_obj
        .downcast_ref::<OffscreenCanvasData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an OffscreenCanvas"))?;

    let context_type = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    match context_type.as_str() {
        "2d" => {
            // Return cached context if available
            if let Some(ctx) = canvas_data.get_context_2d() {
                return Ok(ctx.into());
            }

            // Create new 2D context
            let ctx_data = CanvasRenderingContext2DData::new(canvas_data.get_state());

            let prototype = context
                .intrinsics()
                .constructors()
                .canvas_rendering_context_2d()
                .prototype();

            let ctx_obj = JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                prototype,
                ctx_data,
            );

            // Cache the context
            canvas_data.set_context_2d(ctx_obj.clone().upcast());

            Ok(ctx_obj.upcast().into())
        }
        "webgl" | "experimental-webgl" | "webgl2" | "experimental-webgl2" => {
            // WebGL context - return null for now
            Ok(JsValue::null())
        }
        "bitmaprenderer" => {
            // ImageBitmapRenderingContext - return null for now
            Ok(JsValue::null())
        }
        _ => Ok(JsValue::null()),
    }
}

fn convert_to_blob(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

    let canvas_data = this_obj
        .downcast_ref::<OffscreenCanvasData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an OffscreenCanvas"))?;

    // Get options
    let _options = args.first();

    // Get the image data and create a promise
    if let Some((_width, _height, _data)) = canvas_data.get_image_data() {
        // Create the pixel data
        let state = canvas_data.state.lock().unwrap();
        if let Some(ref s) = *state
            && let Some(png_data) = s.to_png()
        {
            // Create a Uint8Array from the PNG data
            let typed_array = context.eval(Source::from_bytes(&format!(
                "new Uint8Array([{}])",
                png_data
                    .iter()
                    .map(|b| b.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            )))?;

            // Return a resolved promise with the "blob" (typed array for now)
            let code = "Promise.resolve";
            let promise_resolve = context.eval(Source::from_bytes(code))?;

            if let Some(resolve_fn) = promise_resolve.as_callable() {
                return resolve_fn.call(&JsValue::undefined(), &[typed_array], context);
            }
        }
    }

    // Return a rejected promise if something went wrong
    context.eval(Source::from_bytes(
        "Promise.reject(new Error('Failed to convert canvas to blob'))",
    ))
}

fn transfer_to_image_bitmap(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

    let canvas_data = this_obj
        .downcast_ref::<OffscreenCanvasData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an OffscreenCanvas"))?;

    // Get the current image data
    if let Some((width, height, data)) = canvas_data.get_image_data() {
        // Create an ImageBitmap from the data
        let bitmap = create_image_bitmap_from_data(width, height, data, context)?;

        // Clear the canvas (transfer semantics - canvas becomes transparent black)
        canvas_data.set_width(canvas_data.get_width());

        return Ok(bitmap.into());
    }

    Err(JsNativeError::typ()
        .with_message("Failed to transfer canvas to ImageBitmap")
        .into())
}
