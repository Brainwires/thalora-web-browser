//! CanvasRenderingContext2D Web API implementation
//!
//! The 2D rendering context for the canvas element.
//! https://html.spec.whatwg.org/multipage/canvas.html#canvasrenderingcontext2d

use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute, Source,
};
use boa_gc::{Finalize, Trace};
use tiny_skia::{LineCap, LineJoin};
use std::sync::{Arc, Mutex};

use super::canvas_state::{CanvasState, CanvasStyle};
use super::path::Path2DData;
use crate::dom::image_bitmap::ImageBitmapData;
use crate::dom::html_image_element::HTMLImageElementData;

/// Internal data for CanvasRenderingContext2D
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct CanvasRenderingContext2DData {
    /// The canvas state (shared with HTMLCanvasElement)
    #[unsafe_ignore_trace]
    state: Arc<Mutex<Option<CanvasState>>>,
}

impl CanvasRenderingContext2DData {
    pub fn new(state: Arc<Mutex<Option<CanvasState>>>) -> Self {
        Self { state }
    }

    pub fn with_state<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut CanvasState) -> R,
    {
        let mut guard = self.state.lock().unwrap();
        guard.as_mut().map(f)
    }
}

/// JavaScript `CanvasRenderingContext2D` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct CanvasRenderingContext2D;

impl IntrinsicObject for CanvasRenderingContext2D {
    fn init(realm: &Realm) {
        // Getters and setters for style properties
        let fill_style_getter = BuiltInBuilder::callable(realm, get_fill_style)
            .name(js_string!("get fillStyle"))
            .build();
        let fill_style_setter = BuiltInBuilder::callable(realm, set_fill_style)
            .name(js_string!("set fillStyle"))
            .build();

        let stroke_style_getter = BuiltInBuilder::callable(realm, get_stroke_style)
            .name(js_string!("get strokeStyle"))
            .build();
        let stroke_style_setter = BuiltInBuilder::callable(realm, set_stroke_style)
            .name(js_string!("set strokeStyle"))
            .build();

        let line_width_getter = BuiltInBuilder::callable(realm, get_line_width)
            .name(js_string!("get lineWidth"))
            .build();
        let line_width_setter = BuiltInBuilder::callable(realm, set_line_width)
            .name(js_string!("set lineWidth"))
            .build();

        let line_cap_getter = BuiltInBuilder::callable(realm, get_line_cap)
            .name(js_string!("get lineCap"))
            .build();
        let line_cap_setter = BuiltInBuilder::callable(realm, set_line_cap)
            .name(js_string!("set lineCap"))
            .build();

        let line_join_getter = BuiltInBuilder::callable(realm, get_line_join)
            .name(js_string!("get lineJoin"))
            .build();
        let line_join_setter = BuiltInBuilder::callable(realm, set_line_join)
            .name(js_string!("set lineJoin"))
            .build();

        let miter_limit_getter = BuiltInBuilder::callable(realm, get_miter_limit)
            .name(js_string!("get miterLimit"))
            .build();
        let miter_limit_setter = BuiltInBuilder::callable(realm, set_miter_limit)
            .name(js_string!("set miterLimit"))
            .build();

        let global_alpha_getter = BuiltInBuilder::callable(realm, get_global_alpha)
            .name(js_string!("get globalAlpha"))
            .build();
        let global_alpha_setter = BuiltInBuilder::callable(realm, set_global_alpha)
            .name(js_string!("set globalAlpha"))
            .build();

        let font_getter = BuiltInBuilder::callable(realm, get_font)
            .name(js_string!("get font"))
            .build();
        let font_setter = BuiltInBuilder::callable(realm, set_font)
            .name(js_string!("set font"))
            .build();

        let text_align_getter = BuiltInBuilder::callable(realm, get_text_align)
            .name(js_string!("get textAlign"))
            .build();
        let text_align_setter = BuiltInBuilder::callable(realm, set_text_align)
            .name(js_string!("set textAlign"))
            .build();

        let text_baseline_getter = BuiltInBuilder::callable(realm, get_text_baseline)
            .name(js_string!("get textBaseline"))
            .build();
        let text_baseline_setter = BuiltInBuilder::callable(realm, set_text_baseline)
            .name(js_string!("set textBaseline"))
            .build();

        let shadow_blur_getter = BuiltInBuilder::callable(realm, get_shadow_blur)
            .name(js_string!("get shadowBlur"))
            .build();
        let shadow_blur_setter = BuiltInBuilder::callable(realm, set_shadow_blur)
            .name(js_string!("set shadowBlur"))
            .build();

        let shadow_color_getter = BuiltInBuilder::callable(realm, get_shadow_color)
            .name(js_string!("get shadowColor"))
            .build();
        let shadow_color_setter = BuiltInBuilder::callable(realm, set_shadow_color)
            .name(js_string!("set shadowColor"))
            .build();

        let shadow_offset_x_getter = BuiltInBuilder::callable(realm, get_shadow_offset_x)
            .name(js_string!("get shadowOffsetX"))
            .build();
        let shadow_offset_x_setter = BuiltInBuilder::callable(realm, set_shadow_offset_x)
            .name(js_string!("set shadowOffsetX"))
            .build();

        let shadow_offset_y_getter = BuiltInBuilder::callable(realm, get_shadow_offset_y)
            .name(js_string!("get shadowOffsetY"))
            .build();
        let shadow_offset_y_setter = BuiltInBuilder::callable(realm, set_shadow_offset_y)
            .name(js_string!("set shadowOffsetY"))
            .build();

        let line_dash_offset_getter = BuiltInBuilder::callable(realm, get_line_dash_offset)
            .name(js_string!("get lineDashOffset"))
            .build();
        let line_dash_offset_setter = BuiltInBuilder::callable(realm, set_line_dash_offset)
            .name(js_string!("set lineDashOffset"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Style properties
            .accessor(
                js_string!("fillStyle"),
                Some(fill_style_getter),
                Some(fill_style_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("strokeStyle"),
                Some(stroke_style_getter),
                Some(stroke_style_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("lineWidth"),
                Some(line_width_getter),
                Some(line_width_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("lineCap"),
                Some(line_cap_getter),
                Some(line_cap_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("lineJoin"),
                Some(line_join_getter),
                Some(line_join_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("miterLimit"),
                Some(miter_limit_getter),
                Some(miter_limit_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("globalAlpha"),
                Some(global_alpha_getter),
                Some(global_alpha_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("font"),
                Some(font_getter),
                Some(font_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("textAlign"),
                Some(text_align_getter),
                Some(text_align_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("textBaseline"),
                Some(text_baseline_getter),
                Some(text_baseline_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("shadowBlur"),
                Some(shadow_blur_getter),
                Some(shadow_blur_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("shadowColor"),
                Some(shadow_color_getter),
                Some(shadow_color_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("shadowOffsetX"),
                Some(shadow_offset_x_getter),
                Some(shadow_offset_x_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("shadowOffsetY"),
                Some(shadow_offset_y_getter),
                Some(shadow_offset_y_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("lineDashOffset"),
                Some(line_dash_offset_getter),
                Some(line_dash_offset_setter),
                Attribute::CONFIGURABLE,
            )
            // State methods
            .method(save, js_string!("save"), 0)
            .method(restore, js_string!("restore"), 0)
            // Path methods
            .method(begin_path, js_string!("beginPath"), 0)
            .method(close_path, js_string!("closePath"), 0)
            .method(move_to, js_string!("moveTo"), 2)
            .method(line_to, js_string!("lineTo"), 2)
            .method(bezier_curve_to, js_string!("bezierCurveTo"), 6)
            .method(quadratic_curve_to, js_string!("quadraticCurveTo"), 4)
            .method(arc, js_string!("arc"), 5)
            .method(arc_to, js_string!("arcTo"), 5)
            .method(ellipse, js_string!("ellipse"), 7)
            .method(rect, js_string!("rect"), 4)
            .method(round_rect, js_string!("roundRect"), 5)
            // Drawing methods
            .method(fill, js_string!("fill"), 0)
            .method(stroke, js_string!("stroke"), 0)
            .method(fill_rect, js_string!("fillRect"), 4)
            .method(stroke_rect, js_string!("strokeRect"), 4)
            .method(clear_rect, js_string!("clearRect"), 4)
            // Transform methods
            .method(scale, js_string!("scale"), 2)
            .method(rotate, js_string!("rotate"), 1)
            .method(translate, js_string!("translate"), 2)
            .method(transform, js_string!("transform"), 6)
            .method(set_transform, js_string!("setTransform"), 6)
            .method(reset_transform, js_string!("resetTransform"), 0)
            .method(get_transform, js_string!("getTransform"), 0)
            // Line methods
            .method(set_line_dash, js_string!("setLineDash"), 1)
            .method(get_line_dash, js_string!("getLineDash"), 0)
            // Image methods
            .method(draw_image, js_string!("drawImage"), 3)
            // Pixel manipulation
            .method(get_image_data, js_string!("getImageData"), 4)
            .method(put_image_data, js_string!("putImageData"), 3)
            .method(create_image_data, js_string!("createImageData"), 2)
            // Text methods
            .method(fill_text, js_string!("fillText"), 3)
            .method(stroke_text, js_string!("strokeText"), 3)
            .method(measure_text, js_string!("measureText"), 1)
            // Clipping
            .method(clip, js_string!("clip"), 0)
            // Hit testing
            .method(is_point_in_path, js_string!("isPointInPath"), 2)
            .method(is_point_in_stroke, js_string!("isPointInStroke"), 2)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for CanvasRenderingContext2D {
    const NAME: JsString = StaticJsStrings::CANVAS_RENDERING_CONTEXT_2D;
}

impl BuiltInConstructor for CanvasRenderingContext2D {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 50;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 50;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::canvas_rendering_context_2d;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // CanvasRenderingContext2D cannot be directly constructed
        Err(JsNativeError::typ()
            .with_message("CanvasRenderingContext2D cannot be directly constructed; use canvas.getContext('2d')")
            .into())
    }
}

// ============== Style Property Accessors ==============

fn get_fill_style(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let color_str = ctx_data.with_state(|state| {
        match &state.current.fill_style {
            CanvasStyle::Color(c) => {
                format!("rgba({},{},{},{})",
                    (c.red() * 255.0) as u8,
                    (c.green() * 255.0) as u8,
                    (c.blue() * 255.0) as u8,
                    c.alpha()
                )
            }
            _ => "#000000".to_string(),
        }
    }).unwrap_or_else(|| "#000000".to_string());

    Ok(JsValue::from(js_string!(color_str)))
}

fn set_fill_style(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let color_str = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(style) = CanvasStyle::from_css_color(&color_str) {
        ctx_data.with_state(|state| {
            state.current.fill_style = style;
        });
    }

    Ok(JsValue::undefined())
}

fn get_stroke_style(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let color_str = ctx_data.with_state(|state| {
        match &state.current.stroke_style {
            CanvasStyle::Color(c) => {
                format!("rgba({},{},{},{})",
                    (c.red() * 255.0) as u8,
                    (c.green() * 255.0) as u8,
                    (c.blue() * 255.0) as u8,
                    c.alpha()
                )
            }
            _ => "#000000".to_string(),
        }
    }).unwrap_or_else(|| "#000000".to_string());

    Ok(JsValue::from(js_string!(color_str)))
}

fn set_stroke_style(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let color_str = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(style) = CanvasStyle::from_css_color(&color_str) {
        ctx_data.with_state(|state| {
            state.current.stroke_style = style;
        });
    }

    Ok(JsValue::undefined())
}

fn get_line_width(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let width = ctx_data.with_state(|state| state.current.line_width).unwrap_or(1.0);
    Ok(JsValue::from(width as f64))
}

fn set_line_width(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let width = args.get_or_undefined(0).to_number(context)? as f32;
    if width > 0.0 {
        ctx_data.with_state(|state| {
            state.current.line_width = width;
        });
    }

    Ok(JsValue::undefined())
}

fn get_line_cap(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let cap = ctx_data.with_state(|state| {
        match state.current.line_cap {
            LineCap::Butt => "butt",
            LineCap::Round => "round",
            LineCap::Square => "square",
        }
    }).unwrap_or("butt");

    Ok(JsValue::from(js_string!(cap)))
}

fn set_line_cap(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let cap_str = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    let cap = match cap_str.as_str() {
        "butt" => LineCap::Butt,
        "round" => LineCap::Round,
        "square" => LineCap::Square,
        _ => return Ok(JsValue::undefined()),
    };

    ctx_data.with_state(|state| {
        state.current.line_cap = cap;
    });

    Ok(JsValue::undefined())
}

fn get_line_join(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let join = ctx_data.with_state(|state| {
        match state.current.line_join {
            LineJoin::Miter => "miter",
            LineJoin::MiterClip => "miter",
            LineJoin::Round => "round",
            LineJoin::Bevel => "bevel",
        }
    }).unwrap_or("miter");

    Ok(JsValue::from(js_string!(join)))
}

fn set_line_join(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let join_str = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    let join = match join_str.as_str() {
        "miter" => LineJoin::Miter,
        "round" => LineJoin::Round,
        "bevel" => LineJoin::Bevel,
        _ => return Ok(JsValue::undefined()),
    };

    ctx_data.with_state(|state| {
        state.current.line_join = join;
    });

    Ok(JsValue::undefined())
}

fn get_miter_limit(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let limit = ctx_data.with_state(|state| state.current.miter_limit).unwrap_or(10.0);
    Ok(JsValue::from(limit as f64))
}

fn set_miter_limit(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let limit = args.get_or_undefined(0).to_number(context)? as f32;
    if limit > 0.0 {
        ctx_data.with_state(|state| {
            state.current.miter_limit = limit;
        });
    }

    Ok(JsValue::undefined())
}

fn get_global_alpha(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let alpha = ctx_data.with_state(|state| state.current.global_alpha).unwrap_or(1.0);
    Ok(JsValue::from(alpha as f64))
}

fn set_global_alpha(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let alpha = args.get_or_undefined(0).to_number(context)? as f32;
    if (0.0..=1.0).contains(&alpha) {
        ctx_data.with_state(|state| {
            state.current.global_alpha = alpha;
        });
    }

    Ok(JsValue::undefined())
}

fn get_font(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let font = ctx_data.with_state(|state| state.current.font.clone())
        .unwrap_or_else(|| "10px sans-serif".to_string());
    Ok(JsValue::from(js_string!(font)))
}

fn set_font(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let font = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    ctx_data.with_state(|state| {
        state.current.font = font;
    });

    Ok(JsValue::undefined())
}

fn get_text_align(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let align = ctx_data.with_state(|state| state.current.text_align.clone())
        .unwrap_or_else(|| "start".to_string());
    Ok(JsValue::from(js_string!(align)))
}

fn set_text_align(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let align = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    if ["start", "end", "left", "right", "center"].contains(&align.as_str()) {
        ctx_data.with_state(|state| {
            state.current.text_align = align;
        });
    }

    Ok(JsValue::undefined())
}

fn get_text_baseline(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let baseline = ctx_data.with_state(|state| state.current.text_baseline.clone())
        .unwrap_or_else(|| "alphabetic".to_string());
    Ok(JsValue::from(js_string!(baseline)))
}

fn set_text_baseline(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let baseline = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    if ["top", "hanging", "middle", "alphabetic", "ideographic", "bottom"].contains(&baseline.as_str()) {
        ctx_data.with_state(|state| {
            state.current.text_baseline = baseline;
        });
    }

    Ok(JsValue::undefined())
}

fn get_shadow_blur(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let blur = ctx_data.with_state(|state| state.current.shadow_blur).unwrap_or(0.0);
    Ok(JsValue::from(blur as f64))
}

fn set_shadow_blur(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let blur = args.get_or_undefined(0).to_number(context)? as f32;
    if blur >= 0.0 {
        ctx_data.with_state(|state| {
            state.current.shadow_blur = blur;
        });
    }

    Ok(JsValue::undefined())
}

fn get_shadow_color(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let color_str = ctx_data.with_state(|state| {
        let c = &state.current.shadow_color;
        format!("rgba({},{},{},{})",
            (c.red() * 255.0) as u8,
            (c.green() * 255.0) as u8,
            (c.blue() * 255.0) as u8,
            c.alpha()
        )
    }).unwrap_or_else(|| "rgba(0,0,0,0)".to_string());

    Ok(JsValue::from(js_string!(color_str)))
}

fn set_shadow_color(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let color_str = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    if let Some(CanvasStyle::Color(color)) = CanvasStyle::from_css_color(&color_str) {
        ctx_data.with_state(|state| {
            state.current.shadow_color = color;
        });
    }

    Ok(JsValue::undefined())
}

fn get_shadow_offset_x(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let offset = ctx_data.with_state(|state| state.current.shadow_offset_x).unwrap_or(0.0);
    Ok(JsValue::from(offset as f64))
}

fn set_shadow_offset_x(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let offset = args.get_or_undefined(0).to_number(context)? as f32;
    ctx_data.with_state(|state| {
        state.current.shadow_offset_x = offset;
    });

    Ok(JsValue::undefined())
}

fn get_shadow_offset_y(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let offset = ctx_data.with_state(|state| state.current.shadow_offset_y).unwrap_or(0.0);
    Ok(JsValue::from(offset as f64))
}

fn set_shadow_offset_y(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let offset = args.get_or_undefined(0).to_number(context)? as f32;
    ctx_data.with_state(|state| {
        state.current.shadow_offset_y = offset;
    });

    Ok(JsValue::undefined())
}

fn get_line_dash_offset(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let offset = ctx_data.with_state(|state| state.current.line_dash_offset).unwrap_or(0.0);
    Ok(JsValue::from(offset as f64))
}

fn set_line_dash_offset(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let offset = args.get_or_undefined(0).to_number(context)? as f32;
    ctx_data.with_state(|state| {
        state.current.line_dash_offset = offset;
    });

    Ok(JsValue::undefined())
}

// ============== State Methods ==============

fn save(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    ctx_data.with_state(|state| state.save());
    Ok(JsValue::undefined())
}

fn restore(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    ctx_data.with_state(|state| state.restore());
    Ok(JsValue::undefined())
}

// ============== Path Methods ==============

fn begin_path(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    ctx_data.with_state(|state| state.begin_path());
    Ok(JsValue::undefined())
}

fn close_path(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    ctx_data.with_state(|state| state.close_path());
    Ok(JsValue::undefined())
}

fn move_to(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let x = args.get_or_undefined(0).to_number(context)? as f32;
    let y = args.get_or_undefined(1).to_number(context)? as f32;

    ctx_data.with_state(|state| state.move_to(x, y));
    Ok(JsValue::undefined())
}

fn line_to(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let x = args.get_or_undefined(0).to_number(context)? as f32;
    let y = args.get_or_undefined(1).to_number(context)? as f32;

    ctx_data.with_state(|state| state.line_to(x, y));
    Ok(JsValue::undefined())
}

fn bezier_curve_to(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let cp1x = args.get_or_undefined(0).to_number(context)? as f32;
    let cp1y = args.get_or_undefined(1).to_number(context)? as f32;
    let cp2x = args.get_or_undefined(2).to_number(context)? as f32;
    let cp2y = args.get_or_undefined(3).to_number(context)? as f32;
    let x = args.get_or_undefined(4).to_number(context)? as f32;
    let y = args.get_or_undefined(5).to_number(context)? as f32;

    ctx_data.with_state(|state| state.bezier_curve_to(cp1x, cp1y, cp2x, cp2y, x, y));
    Ok(JsValue::undefined())
}

fn quadratic_curve_to(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let cpx = args.get_or_undefined(0).to_number(context)? as f32;
    let cpy = args.get_or_undefined(1).to_number(context)? as f32;
    let x = args.get_or_undefined(2).to_number(context)? as f32;
    let y = args.get_or_undefined(3).to_number(context)? as f32;

    ctx_data.with_state(|state| state.quadratic_curve_to(cpx, cpy, x, y));
    Ok(JsValue::undefined())
}

fn arc(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let x = args.get_or_undefined(0).to_number(context)? as f32;
    let y = args.get_or_undefined(1).to_number(context)? as f32;
    let radius = args.get_or_undefined(2).to_number(context)? as f32;
    let start_angle = args.get_or_undefined(3).to_number(context)? as f32;
    let end_angle = args.get_or_undefined(4).to_number(context)? as f32;
    let counterclockwise = args.get_or_undefined(5).to_boolean();

    ctx_data.with_state(|state| state.arc(x, y, radius, start_angle, end_angle, counterclockwise));
    Ok(JsValue::undefined())
}

fn arc_to(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let _ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    // arcTo is more complex - needs the path module
    // For now, just consume the arguments
    let _x1 = args.get_or_undefined(0).to_number(context)? as f32;
    let _y1 = args.get_or_undefined(1).to_number(context)? as f32;
    let _x2 = args.get_or_undefined(2).to_number(context)? as f32;
    let _y2 = args.get_or_undefined(3).to_number(context)? as f32;
    let _radius = args.get_or_undefined(4).to_number(context)? as f32;

    // TODO: Implement arcTo properly
    Ok(JsValue::undefined())
}

fn ellipse(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let _ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    // For now, approximate with an arc
    let _x = args.get_or_undefined(0).to_number(context)? as f32;
    let _y = args.get_or_undefined(1).to_number(context)? as f32;
    let _rx = args.get_or_undefined(2).to_number(context)? as f32;
    let _ry = args.get_or_undefined(3).to_number(context)? as f32;
    let _rotation = args.get_or_undefined(4).to_number(context)? as f32;
    let _start = args.get_or_undefined(5).to_number(context)? as f32;
    let _end = args.get_or_undefined(6).to_number(context)? as f32;
    let _ccw = args.get_or_undefined(7).to_boolean();

    // TODO: Implement proper ellipse
    Ok(JsValue::undefined())
}

fn rect(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let x = args.get_or_undefined(0).to_number(context)? as f32;
    let y = args.get_or_undefined(1).to_number(context)? as f32;
    let width = args.get_or_undefined(2).to_number(context)? as f32;
    let height = args.get_or_undefined(3).to_number(context)? as f32;

    ctx_data.with_state(|state| state.rect(x, y, width, height));
    Ok(JsValue::undefined())
}

fn round_rect(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let x = args.get_or_undefined(0).to_number(context)? as f32;
    let y = args.get_or_undefined(1).to_number(context)? as f32;
    let width = args.get_or_undefined(2).to_number(context)? as f32;
    let height = args.get_or_undefined(3).to_number(context)? as f32;
    // For simplicity, just use rect for now (roundRect needs more work)
    let _radii = args.get_or_undefined(4);

    ctx_data.with_state(|state| state.rect(x, y, width, height));
    Ok(JsValue::undefined())
}

// ============== Drawing Methods ==============

fn fill(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    // Check if first arg is a Path2D
    if let Some(path_obj) = args.get(0).and_then(|v| v.as_object()) {
        if let Some(_path_data) = path_obj.downcast_ref::<Path2DData>() {
            // TODO: Fill the provided path
        }
    }

    ctx_data.with_state(|state| state.fill());
    Ok(JsValue::undefined())
}

fn stroke(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    // Check if first arg is a Path2D
    if let Some(path_obj) = args.get(0).and_then(|v| v.as_object()) {
        if let Some(_path_data) = path_obj.downcast_ref::<Path2DData>() {
            // TODO: Stroke the provided path
        }
    }

    ctx_data.with_state(|state| state.stroke());
    Ok(JsValue::undefined())
}

fn fill_rect(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let x = args.get_or_undefined(0).to_number(context)? as f32;
    let y = args.get_or_undefined(1).to_number(context)? as f32;
    let width = args.get_or_undefined(2).to_number(context)? as f32;
    let height = args.get_or_undefined(3).to_number(context)? as f32;

    ctx_data.with_state(|state| state.fill_rect(x, y, width, height));
    Ok(JsValue::undefined())
}

fn stroke_rect(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let x = args.get_or_undefined(0).to_number(context)? as f32;
    let y = args.get_or_undefined(1).to_number(context)? as f32;
    let width = args.get_or_undefined(2).to_number(context)? as f32;
    let height = args.get_or_undefined(3).to_number(context)? as f32;

    ctx_data.with_state(|state| state.stroke_rect(x, y, width, height));
    Ok(JsValue::undefined())
}

fn clear_rect(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let x = args.get_or_undefined(0).to_number(context)? as f32;
    let y = args.get_or_undefined(1).to_number(context)? as f32;
    let width = args.get_or_undefined(2).to_number(context)? as f32;
    let height = args.get_or_undefined(3).to_number(context)? as f32;

    ctx_data.with_state(|state| state.clear_rect(x, y, width, height));
    Ok(JsValue::undefined())
}

// ============== Transform Methods ==============

fn scale(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let x = args.get_or_undefined(0).to_number(context)? as f32;
    let y = args.get_or_undefined(1).to_number(context)? as f32;

    ctx_data.with_state(|state| state.scale(x, y));
    Ok(JsValue::undefined())
}

fn rotate(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let angle = args.get_or_undefined(0).to_number(context)? as f32;

    ctx_data.with_state(|state| state.rotate(angle));
    Ok(JsValue::undefined())
}

fn translate(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let x = args.get_or_undefined(0).to_number(context)? as f32;
    let y = args.get_or_undefined(1).to_number(context)? as f32;

    ctx_data.with_state(|state| state.translate(x, y));
    Ok(JsValue::undefined())
}

fn transform(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let a = args.get_or_undefined(0).to_number(context)? as f32;
    let b = args.get_or_undefined(1).to_number(context)? as f32;
    let c = args.get_or_undefined(2).to_number(context)? as f32;
    let d = args.get_or_undefined(3).to_number(context)? as f32;
    let e = args.get_or_undefined(4).to_number(context)? as f32;
    let f = args.get_or_undefined(5).to_number(context)? as f32;

    ctx_data.with_state(|state| state.transform(a, b, c, d, e, f));
    Ok(JsValue::undefined())
}

fn set_transform(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let a = args.get_or_undefined(0).to_number(context)? as f32;
    let b = args.get_or_undefined(1).to_number(context)? as f32;
    let c = args.get_or_undefined(2).to_number(context)? as f32;
    let d = args.get_or_undefined(3).to_number(context)? as f32;
    let e = args.get_or_undefined(4).to_number(context)? as f32;
    let f = args.get_or_undefined(5).to_number(context)? as f32;

    ctx_data.with_state(|state| state.set_transform(a, b, c, d, e, f));
    Ok(JsValue::undefined())
}

fn reset_transform(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    ctx_data.with_state(|state| state.reset_transform());
    Ok(JsValue::undefined())
}

fn get_transform(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let (a, b, c, d, e, f) = ctx_data.with_state(|state| {
        let t = state.current.transform;
        (t.sx, t.ky, t.kx, t.sy, t.tx, t.ty)
    }).unwrap_or((1.0, 0.0, 0.0, 1.0, 0.0, 0.0));

    // Create a DOMMatrix-like object
    context.eval(Source::from_bytes(&format!(
        "{{ a: {}, b: {}, c: {}, d: {}, e: {}, f: {} }}",
        a, b, c, d, e, f
    )))
}

// ============== Line Dash Methods ==============

fn set_line_dash(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let segments_arg = args.get_or_undefined(0);

    if let Some(arr) = segments_arg.as_object() {
        let length = arr.get(js_string!("length"), context)?
            .to_number(context)? as usize;

        let mut segments = Vec::with_capacity(length);
        for i in 0..length {
            let val = arr.get(i, context)?.to_number(context)? as f32;
            if val >= 0.0 {
                segments.push(val);
            }
        }

        ctx_data.with_state(|state| {
            state.current.line_dash = segments;
        });
    }

    Ok(JsValue::undefined())
}

fn get_line_dash(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let segments = ctx_data.with_state(|state| state.current.line_dash.clone())
        .unwrap_or_default();

    let arr_str = format!("[{}]",
        segments.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(","));
    context.eval(Source::from_bytes(&arr_str))
}

// ============== Image Methods ==============

fn draw_image(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let source = args.get_or_undefined(0);

    // Get image data based on source type
    let (src_width, src_height, src_data) = if let Some(obj) = source.as_object() {
        if let Some(img_data) = obj.downcast_ref::<HTMLImageElementData>() {
            // HTMLImageElement source
            if let Some(decoded) = img_data.get_decoded_data() {
                (decoded.width, decoded.height, decoded.rgba_data)
            } else {
                return Ok(JsValue::undefined()); // Image not loaded
            }
        } else if let Some(bitmap_data) = obj.downcast_ref::<ImageBitmapData>() {
            // ImageBitmap source
            if let Some(data) = bitmap_data.get_rgba_data() {
                (bitmap_data.get_width(), bitmap_data.get_height(), data)
            } else {
                return Ok(JsValue::undefined()); // Bitmap closed
            }
        } else if let Some(canvas_data) = obj.downcast_ref::<super::html_canvas_element::HTMLCanvasElementData>() {
            // Another canvas source
            let state_arc = canvas_data.get_state();
            let guard = state_arc.lock().unwrap();
            if let Some(state) = guard.as_ref() {
                let w = state.pixmap.width();
                let h = state.pixmap.height();
                let data = state.get_image_data(0, 0, w, h);
                (w, h, data)
            } else {
                return Ok(JsValue::undefined());
            }
        } else {
            return Err(JsNativeError::typ()
                .with_message("Invalid image source")
                .into());
        }
    } else {
        return Err(JsNativeError::typ()
            .with_message("Invalid image source")
            .into());
    };

    // Parse arguments based on count (3, 5, or 9 arguments)
    let arg_count = args.len();

    match arg_count {
        n if n >= 9 => {
            // drawImage(image, sx, sy, sWidth, sHeight, dx, dy, dWidth, dHeight)
            let _sx = args.get_or_undefined(1).to_number(context)? as i32;
            let _sy = args.get_or_undefined(2).to_number(context)? as i32;
            let _s_width = args.get_or_undefined(3).to_number(context)? as u32;
            let _s_height = args.get_or_undefined(4).to_number(context)? as u32;
            let dx = args.get_or_undefined(5).to_number(context)? as i32;
            let dy = args.get_or_undefined(6).to_number(context)? as i32;
            let _d_width = args.get_or_undefined(7).to_number(context)? as u32;
            let _d_height = args.get_or_undefined(8).to_number(context)? as u32;

            // TODO: Implement proper source rect and scaling
            ctx_data.with_state(|state| {
                state.put_image_data(&src_data, dx, dy, src_width, src_height);
            });
        }
        n if n >= 5 => {
            // drawImage(image, dx, dy, dWidth, dHeight)
            let dx = args.get_or_undefined(1).to_number(context)? as i32;
            let dy = args.get_or_undefined(2).to_number(context)? as i32;
            let _d_width = args.get_or_undefined(3).to_number(context)? as u32;
            let _d_height = args.get_or_undefined(4).to_number(context)? as u32;

            // TODO: Implement proper scaling
            ctx_data.with_state(|state| {
                state.put_image_data(&src_data, dx, dy, src_width, src_height);
            });
        }
        _ => {
            // drawImage(image, dx, dy)
            let dx = args.get_or_undefined(1).to_number(context)? as i32;
            let dy = args.get_or_undefined(2).to_number(context)? as i32;

            ctx_data.with_state(|state| {
                state.put_image_data(&src_data, dx, dy, src_width, src_height);
            });
        }
    }

    Ok(JsValue::undefined())
}

// ============== Pixel Manipulation ==============

fn get_image_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let x = args.get_or_undefined(0).to_number(context)? as u32;
    let y = args.get_or_undefined(1).to_number(context)? as u32;
    let width = args.get_or_undefined(2).to_number(context)? as u32;
    let height = args.get_or_undefined(3).to_number(context)? as u32;

    let data = ctx_data.with_state(|state| state.get_image_data(x, y, width, height))
        .unwrap_or_default();

    // Create ImageData object
    let data_str = data.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(",");
    context.eval(Source::from_bytes(&format!(
        "{{ width: {}, height: {}, data: new Uint8ClampedArray([{}]) }}",
        width, height, data_str
    )))
}

fn put_image_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let ctx_data = this_obj.downcast_ref::<CanvasRenderingContext2DData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not a CanvasRenderingContext2D")
    })?;

    let image_data = args.get_or_undefined(0);
    let dx = args.get_or_undefined(1).to_number(context)? as i32;
    let dy = args.get_or_undefined(2).to_number(context)? as i32;

    if let Some(obj) = image_data.as_object() {
        let width = obj.get(js_string!("width"), context)?.to_number(context)? as u32;
        let height = obj.get(js_string!("height"), context)?.to_number(context)? as u32;
        let data_obj = obj.get(js_string!("data"), context)?;

        if let Some(data_arr) = data_obj.as_object() {
            let length = data_arr.get(js_string!("length"), context)?
                .to_number(context)? as usize;

            let mut data = Vec::with_capacity(length);
            for i in 0..length {
                let val = data_arr.get(i, context)?.to_number(context)? as u8;
                data.push(val);
            }

            ctx_data.with_state(|state| {
                state.put_image_data(&data, dx, dy, width, height);
            });
        }
    }

    Ok(JsValue::undefined())
}

fn create_image_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let width = args.get_or_undefined(0).to_number(context)? as u32;
    let height = args.get_or_undefined(1).to_number(context)? as u32;

    let size = (width * height * 4) as usize;
    let zeros = vec![0u8; size];
    let data_str = zeros.iter().map(|_| "0").collect::<Vec<_>>().join(",");

    context.eval(Source::from_bytes(&format!(
        "{{ width: {}, height: {}, data: new Uint8ClampedArray([{}]) }}",
        width, height, data_str
    )))
}

// ============== Text Methods ==============

fn fill_text(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    // Text rendering is complex and requires font shaping
    // For now, just consume the arguments
    let _text = args.get_or_undefined(0).to_string(context)?;
    let _x = args.get_or_undefined(1).to_number(context)?;
    let _y = args.get_or_undefined(2).to_number(context)?;
    let _max_width = args.get(3).map(|v| v.to_number(context)).transpose()?;

    // TODO: Implement text rendering with fontdb/rustybuzz
    Ok(JsValue::undefined())
}

fn stroke_text(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let _text = args.get_or_undefined(0).to_string(context)?;
    let _x = args.get_or_undefined(1).to_number(context)?;
    let _y = args.get_or_undefined(2).to_number(context)?;
    let _max_width = args.get(3).map(|v| v.to_number(context)).transpose()?;

    // TODO: Implement text stroke rendering
    Ok(JsValue::undefined())
}

fn measure_text(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let text = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    // Very rough approximation - proper implementation needs font metrics
    let width = text.len() as f64 * 8.0; // ~8px per character

    context.eval(Source::from_bytes(&format!(
        "{{ width: {}, actualBoundingBoxLeft: 0, actualBoundingBoxRight: {}, fontBoundingBoxAscent: 10, fontBoundingBoxDescent: 2 }}",
        width, width
    )))
}

// ============== Clipping ==============

fn clip(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let _this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    // TODO: Implement clipping
    Ok(JsValue::undefined())
}

// ============== Hit Testing ==============

fn is_point_in_path(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;

    // TODO: Implement hit testing
    Ok(JsValue::from(false))
}

fn is_point_in_stroke(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let _x = args.get_or_undefined(0).to_number(context)?;
    let _y = args.get_or_undefined(1).to_number(context)?;

    // TODO: Implement stroke hit testing
    Ok(JsValue::from(false))
}
