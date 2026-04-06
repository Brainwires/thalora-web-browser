//! Path2D Web API implementation
//!
//! Path2D provides a way to create paths that can be reused.
//! https://developer.mozilla.org/en-US/docs/Web/API/Path2D

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};
use tiny_skia::PathBuilder;

/// Internal path data
#[derive(Debug, Clone)]
pub struct PathData {
    /// The path builder - stores all path operations
    builder: PathBuilder,
    /// Last x coordinate (for arc operations)
    last_x: f32,
    /// Last y coordinate (for arc operations)
    last_y: f32,
}

impl Default for PathData {
    fn default() -> Self {
        Self::new()
    }
}

impl PathData {
    pub fn new() -> Self {
        Self {
            builder: PathBuilder::new(),
            last_x: 0.0,
            last_y: 0.0,
        }
    }

    /// Get a clone of the path builder
    pub fn get_builder(&self) -> PathBuilder {
        self.builder.clone()
    }

    /// Move to a point
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.builder.move_to(x, y);
        self.last_x = x;
        self.last_y = y;
    }

    /// Draw a line to a point
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(x, y);
        self.last_x = x;
        self.last_y = y;
    }

    /// Close the current subpath
    pub fn close_path(&mut self) {
        self.builder.close();
    }

    /// Add a quadratic bezier curve
    pub fn quadratic_curve_to(&mut self, cpx: f32, cpy: f32, x: f32, y: f32) {
        self.builder.quad_to(cpx, cpy, x, y);
        self.last_x = x;
        self.last_y = y;
    }

    /// Add a cubic bezier curve
    pub fn bezier_curve_to(&mut self, cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32, x: f32, y: f32) {
        self.builder.cubic_to(cp1x, cp1y, cp2x, cp2y, x, y);
        self.last_x = x;
        self.last_y = y;
    }

    /// Add a rectangle to the path
    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.builder.move_to(x, y);
        self.builder.line_to(x + width, y);
        self.builder.line_to(x + width, y + height);
        self.builder.line_to(x, y + height);
        self.builder.close();
        self.last_x = x;
        self.last_y = y;
    }

    /// Add a rounded rectangle
    pub fn round_rect(&mut self, x: f32, y: f32, width: f32, height: f32, radii: f32) {
        let r = radii.min(width / 2.0).min(height / 2.0);

        self.builder.move_to(x + r, y);
        self.builder.line_to(x + width - r, y);
        self.arc_corner(x + width - r, y + r, r, -std::f32::consts::FRAC_PI_2, 0.0);
        self.builder.line_to(x + width, y + height - r);
        self.arc_corner(
            x + width - r,
            y + height - r,
            r,
            0.0,
            std::f32::consts::FRAC_PI_2,
        );
        self.builder.line_to(x + r, y + height);
        self.arc_corner(
            x + r,
            y + height - r,
            r,
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
        );
        self.builder.line_to(x, y + r);
        self.arc_corner(
            x + r,
            y + r,
            r,
            std::f32::consts::PI,
            -std::f32::consts::FRAC_PI_2,
        );
        self.builder.close();
    }

    /// Helper for arc corners in rounded rectangles
    fn arc_corner(&mut self, cx: f32, cy: f32, r: f32, start: f32, end: f32) {
        // Approximate arc with line segments
        let steps = 8;
        let step = (end - start) / steps as f32;
        let mut angle = start;

        for _ in 0..steps {
            angle += step;
            let px = cx + r * angle.cos();
            let py = cy + r * angle.sin();
            self.builder.line_to(px, py);
        }
    }

    /// Add an arc to the path
    pub fn arc(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        counterclockwise: bool,
    ) {
        let mut angle = start_angle;
        let end = if counterclockwise {
            if end_angle > start_angle {
                end_angle - std::f32::consts::TAU
            } else {
                end_angle
            }
        } else if end_angle < start_angle {
            end_angle + std::f32::consts::TAU
        } else {
            end_angle
        };

        let steps = 32;
        let step = (end - angle) / steps as f32;

        let first_x = x + radius * angle.cos();
        let first_y = y + radius * angle.sin();

        // If this is not the first operation, draw a line to the start of the arc
        if self.builder.clone().finish().is_some() {
            self.builder.line_to(first_x, first_y);
        } else {
            self.builder.move_to(first_x, first_y);
        }

        for _ in 0..steps {
            angle += step;
            let px = x + radius * angle.cos();
            let py = y + radius * angle.sin();
            self.builder.line_to(px, py);
        }

        self.last_x = x + radius * end.cos();
        self.last_y = y + radius * end.sin();
    }

    /// Add an arc using control points
    pub fn arc_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) {
        // Calculate the arc from current point through (x1, y1) toward (x2, y2)
        let x0 = self.last_x;
        let y0 = self.last_y;

        // Vectors from x1,y1 to x0,y0 and x2,y2
        let d0x = x0 - x1;
        let d0y = y0 - y1;
        let d2x = x2 - x1;
        let d2y = y2 - y1;

        // Lengths
        let l0 = (d0x * d0x + d0y * d0y).sqrt();
        let l2 = (d2x * d2x + d2y * d2y).sqrt();

        if l0 < 0.0001 || l2 < 0.0001 {
            self.line_to(x1, y1);
            return;
        }

        // Unit vectors
        let u0x = d0x / l0;
        let u0y = d0y / l0;
        let u2x = d2x / l2;
        let u2y = d2y / l2;

        // Tangent distance
        let cross = u0x * u2y - u0y * u2x;
        if cross.abs() < 0.0001 {
            self.line_to(x1, y1);
            return;
        }

        let half_angle = ((u0x * u2x + u0y * u2y).acos() / 2.0).abs();
        let tan_half = half_angle.tan();

        if tan_half.abs() < 0.0001 {
            self.line_to(x1, y1);
            return;
        }

        let d = radius / tan_half;

        // Start and end points of the arc
        let start_x = x1 + u0x * d;
        let start_y = y1 + u0y * d;
        let end_x = x1 + u2x * d;
        let end_y = y1 + u2y * d;

        // Line to the start of the arc
        self.line_to(start_x, start_y);

        // Center of the arc
        let cx = start_x + u0y * radius * cross.signum();
        let cy = start_y - u0x * radius * cross.signum();

        // Calculate angles
        let start_angle = (start_y - cy).atan2(start_x - cx);
        let end_angle = (end_y - cy).atan2(end_x - cx);

        self.arc(cx, cy, radius, start_angle, end_angle, cross > 0.0);
    }

    /// Add an ellipse to the path
    pub fn ellipse(
        &mut self,
        x: f32,
        y: f32,
        radius_x: f32,
        radius_y: f32,
        rotation: f32,
        start_angle: f32,
        end_angle: f32,
        counterclockwise: bool,
    ) {
        let mut angle = start_angle;
        let end = if counterclockwise {
            if end_angle > start_angle {
                end_angle - std::f32::consts::TAU
            } else {
                end_angle
            }
        } else if end_angle < start_angle {
            end_angle + std::f32::consts::TAU
        } else {
            end_angle
        };

        let steps = 32;
        let step = (end - angle) / steps as f32;

        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();

        let calc_point = |a: f32| -> (f32, f32) {
            let ex = radius_x * a.cos();
            let ey = radius_y * a.sin();
            let px = x + ex * cos_rot - ey * sin_rot;
            let py = y + ex * sin_rot + ey * cos_rot;
            (px, py)
        };

        let (first_x, first_y) = calc_point(angle);

        if self.builder.clone().finish().is_some() {
            self.builder.line_to(first_x, first_y);
        } else {
            self.builder.move_to(first_x, first_y);
        }

        for _ in 0..steps {
            angle += step;
            let (px, py) = calc_point(angle);
            self.builder.line_to(px, py);
        }

        let (lx, ly) = calc_point(end);
        self.last_x = lx;
        self.last_y = ly;
    }
}

/// JsData wrapper for Path2D
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct Path2DData {
    #[unsafe_ignore_trace]
    inner: Arc<Mutex<PathData>>,
}

impl Default for Path2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Path2DData {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(PathData::new())),
        }
    }

    pub fn get_path_data(&self) -> PathData {
        self.inner.lock().unwrap().clone()
    }

    pub fn with_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut PathData) -> R,
    {
        let mut data = self.inner.lock().unwrap();
        f(&mut data)
    }
}

/// JavaScript `Path2D` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct Path2D;

impl IntrinsicObject for Path2D {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(add_path, js_string!("addPath"), 1)
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
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Path2D {
    const NAME: JsString = StaticJsStrings::PATH_2D;
}

impl BuiltInConstructor for Path2D {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 20;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 20;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::path_2d;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::path_2d, context)?;

        let path_data = Path2DData::new();

        // Handle initialization from another path or SVG string
        if let Some(arg) = args.first() {
            if let Some(obj) = arg.as_object() {
                // Copy from another Path2D
                if let Some(other) = obj.downcast_ref::<Path2DData>() {
                    let other_data = other.get_path_data();
                    path_data.with_inner(|data| {
                        *data = other_data;
                    });
                }
            } else if let Some(s) = arg.as_string() {
                // TODO: Parse SVG path string
                // For now, just ignore SVG path strings
                let _ = s;
            }
        }

        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            path_data,
        );

        Ok(obj.into())
    }
}

// ============== Methods ==============

fn add_path(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

    let this_data = this_obj
        .downcast_ref::<Path2DData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Path2D"))?;

    let path = args.get_or_undefined(0);
    if let Some(obj) = path.as_object()
        && let Some(other_data) = obj.downcast_ref::<Path2DData>()
    {
        let other = other_data.get_path_data();
        // TODO: Apply transform matrix if provided in args[1]

        // For now, we'll just copy the path operations
        // A full implementation would merge the path builders
        this_data.with_inner(|_data| {
            // Copy the other path's builder
            let other_builder = other.builder;
            if let Some(path) = other_builder.finish() {
                // We can't easily merge paths in tiny-skia, so we'll skip for now
                let _ = path;
            }
        });
    }

    Ok(JsValue::undefined())
}

fn close_path(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

    let this_data = this_obj
        .downcast_ref::<Path2DData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Path2D"))?;

    this_data.with_inner(|data| data.close_path());
    Ok(JsValue::undefined())
}

fn move_to(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

    let this_data = this_obj
        .downcast_ref::<Path2DData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Path2D"))?;

    let x = args.get_or_undefined(0).to_number(context)? as f32;
    let y = args.get_or_undefined(1).to_number(context)? as f32;

    this_data.with_inner(|data| data.move_to(x, y));
    Ok(JsValue::undefined())
}

fn line_to(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

    let this_data = this_obj
        .downcast_ref::<Path2DData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Path2D"))?;

    let x = args.get_or_undefined(0).to_number(context)? as f32;
    let y = args.get_or_undefined(1).to_number(context)? as f32;

    this_data.with_inner(|data| data.line_to(x, y));
    Ok(JsValue::undefined())
}

fn bezier_curve_to(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

    let this_data = this_obj
        .downcast_ref::<Path2DData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Path2D"))?;

    let cp1x = args.get_or_undefined(0).to_number(context)? as f32;
    let cp1y = args.get_or_undefined(1).to_number(context)? as f32;
    let cp2x = args.get_or_undefined(2).to_number(context)? as f32;
    let cp2y = args.get_or_undefined(3).to_number(context)? as f32;
    let x = args.get_or_undefined(4).to_number(context)? as f32;
    let y = args.get_or_undefined(5).to_number(context)? as f32;

    this_data.with_inner(|data| data.bezier_curve_to(cp1x, cp1y, cp2x, cp2y, x, y));
    Ok(JsValue::undefined())
}

fn quadratic_curve_to(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

    let this_data = this_obj
        .downcast_ref::<Path2DData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Path2D"))?;

    let cpx = args.get_or_undefined(0).to_number(context)? as f32;
    let cpy = args.get_or_undefined(1).to_number(context)? as f32;
    let x = args.get_or_undefined(2).to_number(context)? as f32;
    let y = args.get_or_undefined(3).to_number(context)? as f32;

    this_data.with_inner(|data| data.quadratic_curve_to(cpx, cpy, x, y));
    Ok(JsValue::undefined())
}

fn arc(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

    let this_data = this_obj
        .downcast_ref::<Path2DData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Path2D"))?;

    let x = args.get_or_undefined(0).to_number(context)? as f32;
    let y = args.get_or_undefined(1).to_number(context)? as f32;
    let radius = args.get_or_undefined(2).to_number(context)? as f32;
    let start_angle = args.get_or_undefined(3).to_number(context)? as f32;
    let end_angle = args.get_or_undefined(4).to_number(context)? as f32;
    let counterclockwise = args.get_or_undefined(5).to_boolean();

    this_data.with_inner(|data| data.arc(x, y, radius, start_angle, end_angle, counterclockwise));
    Ok(JsValue::undefined())
}

fn arc_to(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

    let this_data = this_obj
        .downcast_ref::<Path2DData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Path2D"))?;

    let x1 = args.get_or_undefined(0).to_number(context)? as f32;
    let y1 = args.get_or_undefined(1).to_number(context)? as f32;
    let x2 = args.get_or_undefined(2).to_number(context)? as f32;
    let y2 = args.get_or_undefined(3).to_number(context)? as f32;
    let radius = args.get_or_undefined(4).to_number(context)? as f32;

    this_data.with_inner(|data| data.arc_to(x1, y1, x2, y2, radius));
    Ok(JsValue::undefined())
}

fn ellipse(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

    let this_data = this_obj
        .downcast_ref::<Path2DData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Path2D"))?;

    let x = args.get_or_undefined(0).to_number(context)? as f32;
    let y = args.get_or_undefined(1).to_number(context)? as f32;
    let radius_x = args.get_or_undefined(2).to_number(context)? as f32;
    let radius_y = args.get_or_undefined(3).to_number(context)? as f32;
    let rotation = args.get_or_undefined(4).to_number(context)? as f32;
    let start_angle = args.get_or_undefined(5).to_number(context)? as f32;
    let end_angle = args.get_or_undefined(6).to_number(context)? as f32;
    let counterclockwise = args.get_or_undefined(7).to_boolean();

    this_data.with_inner(|data| {
        data.ellipse(
            x,
            y,
            radius_x,
            radius_y,
            rotation,
            start_angle,
            end_angle,
            counterclockwise,
        )
    });
    Ok(JsValue::undefined())
}

fn rect(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

    let this_data = this_obj
        .downcast_ref::<Path2DData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Path2D"))?;

    let x = args.get_or_undefined(0).to_number(context)? as f32;
    let y = args.get_or_undefined(1).to_number(context)? as f32;
    let width = args.get_or_undefined(2).to_number(context)? as f32;
    let height = args.get_or_undefined(3).to_number(context)? as f32;

    this_data.with_inner(|data| data.rect(x, y, width, height));
    Ok(JsValue::undefined())
}

fn round_rect(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an object"))?;

    let this_data = this_obj
        .downcast_ref::<Path2DData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Path2D"))?;

    let x = args.get_or_undefined(0).to_number(context)? as f32;
    let y = args.get_or_undefined(1).to_number(context)? as f32;
    let width = args.get_or_undefined(2).to_number(context)? as f32;
    let height = args.get_or_undefined(3).to_number(context)? as f32;

    // radii can be a number or an array, for now just support a single number
    let radii = args.get_or_undefined(4).to_number(context).unwrap_or(0.0) as f32;

    this_data.with_inner(|data| data.round_rect(x, y, width, height, radii));
    Ok(JsValue::undefined())
}
