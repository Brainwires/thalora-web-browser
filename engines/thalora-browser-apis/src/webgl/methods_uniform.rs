//! WebGL Uniform and Vertex Attribute Methods
//!
//! Uniform variable and vertex attribute operations.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsValue, NativeFunction, js_string,
    object::builtins::JsArray,
};

use super::context::{
    MAX_VERTEX_ATTRIBS, WebGLRenderingContextData, get_location_id, get_object_id,
};
use super::state::WebGLConstants;
use crate::with_webgl_context;

pub fn add_uniform_methods(obj: &JsObject, context: &mut Context) {
    // uniform1f
    obj.set(
        js_string!("uniform1f"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let location_id = get_location_id(args.get_or_undefined(0), ctx)?;
            let x = args.get_or_undefined(1).to_number(ctx)? as f32;

            data.uniform_values
                .lock()
                .unwrap()
                .insert(location_id, vec![x]);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // uniform2f
    obj.set(
        js_string!("uniform2f"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let location_id = get_location_id(args.get_or_undefined(0), ctx)?;
            let x = args.get_or_undefined(1).to_number(ctx)? as f32;
            let y = args.get_or_undefined(2).to_number(ctx)? as f32;

            data.uniform_values
                .lock()
                .unwrap()
                .insert(location_id, vec![x, y]);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // uniform3f
    obj.set(
        js_string!("uniform3f"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let location_id = get_location_id(args.get_or_undefined(0), ctx)?;
            let x = args.get_or_undefined(1).to_number(ctx)? as f32;
            let y = args.get_or_undefined(2).to_number(ctx)? as f32;
            let z = args.get_or_undefined(3).to_number(ctx)? as f32;

            data.uniform_values
                .lock()
                .unwrap()
                .insert(location_id, vec![x, y, z]);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // uniform4f
    obj.set(
        js_string!("uniform4f"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let location_id = get_location_id(args.get_or_undefined(0), ctx)?;
            let x = args.get_or_undefined(1).to_number(ctx)? as f32;
            let y = args.get_or_undefined(2).to_number(ctx)? as f32;
            let z = args.get_or_undefined(3).to_number(ctx)? as f32;
            let w = args.get_or_undefined(4).to_number(ctx)? as f32;

            data.uniform_values
                .lock()
                .unwrap()
                .insert(location_id, vec![x, y, z, w]);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // uniform1i
    obj.set(
        js_string!("uniform1i"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let location_id = get_location_id(args.get_or_undefined(0), ctx)?;
            let x = args.get_or_undefined(1).to_i32(ctx)? as f32;

            data.uniform_values
                .lock()
                .unwrap()
                .insert(location_id, vec![x]);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // uniformMatrix4fv
    obj.set(
        js_string!("uniformMatrix4fv"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let location_id = get_location_id(args.get_or_undefined(0), ctx)?;
            let _transpose = args.get_or_undefined(1).to_boolean();

            let value_arg = args.get_or_undefined(2);
            let mut values = Vec::with_capacity(16);

            if let Some(obj) = value_arg.as_object() {
                // Get length property
                if let Ok(len) = obj.get(js_string!("length"), ctx) {
                    let len = len.to_index(ctx).unwrap_or(0) as usize;
                    for i in 0..len.min(16) {
                        if let Ok(val) = obj.get(i as u32, ctx) {
                            values.push(val.to_number(ctx).unwrap_or(0.0) as f32);
                        }
                    }
                }
            }

            data.uniform_values
                .lock()
                .unwrap()
                .insert(location_id, values);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();
}

/// Add vertex attribute methods
pub fn add_vertex_attrib_methods(obj: &JsObject, context: &mut Context) {
    // vertexAttribPointer
    obj.set(
        js_string!("vertexAttribPointer"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let index = args.get_or_undefined(0).to_u32(ctx)? as usize;
            let size = args.get_or_undefined(1).to_i32(ctx)?;
            let data_type = args.get_or_undefined(2).to_u32(ctx)?;
            let normalized = args.get_or_undefined(3).to_boolean();
            let stride = args.get_or_undefined(4).to_i32(ctx)?;
            let offset = args.get_or_undefined(5).to_i32(ctx)?;

            if index >= MAX_VERTEX_ATTRIBS {
                data.set_error(WebGLConstants::INVALID_VALUE);
                return Ok(JsValue::undefined());
            }

            let buffer_id = data.state.lock().unwrap().bound_array_buffer;

            let mut attribs = data.vertex_attribs.lock().unwrap();
            attribs[index].size = size;
            attribs[index].data_type = data_type;
            attribs[index].normalized = normalized;
            attribs[index].stride = stride;
            attribs[index].offset = offset;
            attribs[index].buffer_id = buffer_id;

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // enableVertexAttribArray
    obj.set(
        js_string!("enableVertexAttribArray"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let index = args.get_or_undefined(0).to_u32(ctx)? as usize;

            if index >= MAX_VERTEX_ATTRIBS {
                data.set_error(WebGLConstants::INVALID_VALUE);
                return Ok(JsValue::undefined());
            }

            data.vertex_attribs.lock().unwrap()[index].enabled = true;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // disableVertexAttribArray
    obj.set(
        js_string!("disableVertexAttribArray"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let index = args.get_or_undefined(0).to_u32(ctx)? as usize;

            if index >= MAX_VERTEX_ATTRIBS {
                data.set_error(WebGLConstants::INVALID_VALUE);
                return Ok(JsValue::undefined());
            }

            data.vertex_attribs.lock().unwrap()[index].enabled = false;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();
}
