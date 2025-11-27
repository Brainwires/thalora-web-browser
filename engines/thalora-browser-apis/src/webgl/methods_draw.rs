//! WebGL Draw and State Methods
//!
//! Drawing operations and state management.

use boa_engine::{
    js_string,
    object::builtins::JsArrayBuffer,
    property::Attribute,
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
};

use super::context::{get_object_id, WebGLRenderingContextData};
use super::state::WebGLConstants;
use crate::with_webgl_context;

pub fn add_draw_methods(obj: &JsObject, context: &mut Context) {
    // clear
    obj.set(
        js_string!("clear"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let mask = args.get_or_undefined(0).to_u32(ctx)?;

            if mask & WebGLConstants::COLOR_BUFFER_BIT != 0 {
                let state = data.state.lock().unwrap();
                let clear_color = state.clear_color;
                drop(state);

                let mut render_target = data.render_target.lock().unwrap();
                let r = (clear_color[0] * 255.0) as u8;
                let g = (clear_color[1] * 255.0) as u8;
                let b = (clear_color[2] * 255.0) as u8;
                let a = (clear_color[3] * 255.0) as u8;

                for chunk in render_target.chunks_exact_mut(4) {
                    chunk[0] = r;
                    chunk[1] = g;
                    chunk[2] = b;
                    chunk[3] = a;
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // drawArrays
    obj.set(
        js_string!("drawArrays"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let _mode = args.get_or_undefined(0).to_u32(ctx)?;
            let _first = args.get_or_undefined(1).to_i32(ctx)?;
            let _count = args.get_or_undefined(2).to_i32(ctx)?;

            // Validate state
            if data.state.lock().unwrap().current_program.is_none() {
                data.set_error(WebGLConstants::INVALID_OPERATION);
                return Ok(JsValue::undefined());
            }

            // Drawing implementation would go here
            // For now, this is a no-op as we need full wgpu pipeline setup

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // drawElements
    obj.set(
        js_string!("drawElements"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let _mode = args.get_or_undefined(0).to_u32(ctx)?;
            let _count = args.get_or_undefined(1).to_i32(ctx)?;
            let _type_ = args.get_or_undefined(2).to_u32(ctx)?;
            let _offset = args.get_or_undefined(3).to_i32(ctx)?;

            // Validate state
            if data.state.lock().unwrap().current_program.is_none() {
                data.set_error(WebGLConstants::INVALID_OPERATION);
                return Ok(JsValue::undefined());
            }

            if data.state.lock().unwrap().bound_element_array_buffer.is_none() {
                data.set_error(WebGLConstants::INVALID_OPERATION);
                return Ok(JsValue::undefined());
            }

            // Drawing implementation would go here
            // For now, this is a no-op as we need full wgpu pipeline setup

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // flush
    obj.set(
        js_string!("flush"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // finish
    obj.set(
        js_string!("finish"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // readPixels
    obj.set(
        js_string!("readPixels"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let x = args.get_or_undefined(0).to_i32(ctx)?;
            let y = args.get_or_undefined(1).to_i32(ctx)?;
            let width = args.get_or_undefined(2).to_i32(ctx)?;
            let height = args.get_or_undefined(3).to_i32(ctx)?;
            let _format = args.get_or_undefined(4).to_u32(ctx)?;
            let _type = args.get_or_undefined(5).to_u32(ctx)?;
            let pixels_arg = args.get_or_undefined(6);

            // Read from render target
            let render_target = data.render_target.lock().unwrap();
            let target_width = data.width as i32;
            let target_height = data.height as i32;

            if let Some(obj) = pixels_arg.as_object() {
                if let Ok(buffer_prop) = obj.get(js_string!("buffer"), ctx) {
                    if let Some(ab) = buffer_prop.as_object().and_then(|o| JsArrayBuffer::from_object(o.clone()).ok()) {
                        let byte_offset = obj.get(js_string!("byteOffset"), ctx)
                            .ok()
                            .and_then(|v| v.to_index(ctx).ok())
                            .unwrap_or(0) as usize;

                        // Copy pixel data
                        let data_slice = ab.data();
                        if let Some(data_guard) = data_slice {
                            let mut dst: Vec<u8> = (*data_guard).to_vec();
                            for row in 0..height {
                                let src_y = target_height - 1 - (y + row); // Flip Y
                                if src_y < 0 || src_y >= target_height {
                                    continue;
                                }
                                for col in 0..width {
                                    let src_x = x + col;
                                    if src_x < 0 || src_x >= target_width {
                                        continue;
                                    }
                                    let src_idx = ((src_y * target_width + src_x) * 4) as usize;
                                    let dst_idx = byte_offset + ((row * width + col) * 4) as usize;
                                    if src_idx + 4 <= render_target.len() && dst_idx + 4 <= dst.len() {
                                        dst[dst_idx..dst_idx + 4].copy_from_slice(&render_target[src_idx..src_idx + 4]);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}

/// Add state methods
pub fn add_state_methods(obj: &JsObject, context: &mut Context) {
    // clearColor
    obj.set(
        js_string!("clearColor"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let r = args.get_or_undefined(0).to_number(ctx)? as f32;
            let g = args.get_or_undefined(1).to_number(ctx)? as f32;
            let b = args.get_or_undefined(2).to_number(ctx)? as f32;
            let a = args.get_or_undefined(3).to_number(ctx)? as f32;

            data.state.lock().unwrap().clear_color = [r, g, b, a];
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // clearDepth
    obj.set(
        js_string!("clearDepth"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let depth = args.get_or_undefined(0).to_number(ctx)? as f32;

            data.state.lock().unwrap().clear_depth = depth;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // clearStencil
    obj.set(
        js_string!("clearStencil"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let stencil = args.get_or_undefined(0).to_i32(ctx)?;

            data.state.lock().unwrap().clear_stencil = stencil;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // enable
    obj.set(
        js_string!("enable"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let cap = args.get_or_undefined(0).to_u32(ctx)?;

            let mut state = data.state.lock().unwrap();
            match cap {
                WebGLConstants::BLEND => state.blend = true,
                WebGLConstants::CULL_FACE => state.cull_face = true,
                WebGLConstants::DEPTH_TEST => state.depth_test = true,
                WebGLConstants::DITHER => state.dither = true,
                WebGLConstants::POLYGON_OFFSET_FILL => state.polygon_offset_fill = true,
                WebGLConstants::SAMPLE_ALPHA_TO_COVERAGE => state.sample_alpha_to_coverage = true,
                WebGLConstants::SAMPLE_COVERAGE => state.sample_coverage = true,
                WebGLConstants::SCISSOR_TEST => state.scissor_test = true,
                WebGLConstants::STENCIL_TEST => state.stencil_test = true,
                _ => {
                    drop(state);
                    data.set_error(WebGLConstants::INVALID_ENUM);
                }
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // disable
    obj.set(
        js_string!("disable"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let cap = args.get_or_undefined(0).to_u32(ctx)?;

            let mut state = data.state.lock().unwrap();
            match cap {
                WebGLConstants::BLEND => state.blend = false,
                WebGLConstants::CULL_FACE => state.cull_face = false,
                WebGLConstants::DEPTH_TEST => state.depth_test = false,
                WebGLConstants::DITHER => state.dither = false,
                WebGLConstants::POLYGON_OFFSET_FILL => state.polygon_offset_fill = false,
                WebGLConstants::SAMPLE_ALPHA_TO_COVERAGE => state.sample_alpha_to_coverage = false,
                WebGLConstants::SAMPLE_COVERAGE => state.sample_coverage = false,
                WebGLConstants::SCISSOR_TEST => state.scissor_test = false,
                WebGLConstants::STENCIL_TEST => state.stencil_test = false,
                _ => {
                    drop(state);
                    data.set_error(WebGLConstants::INVALID_ENUM);
                }
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // isEnabled
    obj.set(
        js_string!("isEnabled"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let cap = args.get_or_undefined(0).to_u32(ctx)?;

            let state = data.state.lock().unwrap();
            let enabled = match cap {
                WebGLConstants::BLEND => state.blend,
                WebGLConstants::CULL_FACE => state.cull_face,
                WebGLConstants::DEPTH_TEST => state.depth_test,
                WebGLConstants::DITHER => state.dither,
                WebGLConstants::POLYGON_OFFSET_FILL => state.polygon_offset_fill,
                WebGLConstants::SAMPLE_ALPHA_TO_COVERAGE => state.sample_alpha_to_coverage,
                WebGLConstants::SAMPLE_COVERAGE => state.sample_coverage,
                WebGLConstants::SCISSOR_TEST => state.scissor_test,
                WebGLConstants::STENCIL_TEST => state.stencil_test,
                _ => false,
            };
            Ok(JsValue::from(enabled))
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // viewport
    obj.set(
        js_string!("viewport"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let x = args.get_or_undefined(0).to_i32(ctx)?;
            let y = args.get_or_undefined(1).to_i32(ctx)?;
            let width = args.get_or_undefined(2).to_i32(ctx)?;
            let height = args.get_or_undefined(3).to_i32(ctx)?;

            data.state.lock().unwrap().viewport = [x, y, width, height];
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // scissor
    obj.set(
        js_string!("scissor"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let x = args.get_or_undefined(0).to_i32(ctx)?;
            let y = args.get_or_undefined(1).to_i32(ctx)?;
            let width = args.get_or_undefined(2).to_i32(ctx)?;
            let height = args.get_or_undefined(3).to_i32(ctx)?;

            data.state.lock().unwrap().scissor = [x, y, width, height];
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // blendFunc
    obj.set(
        js_string!("blendFunc"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let sfactor = args.get_or_undefined(0).to_u32(ctx)?;
            let dfactor = args.get_or_undefined(1).to_u32(ctx)?;

            let mut state = data.state.lock().unwrap();
            state.blend_src_rgb = sfactor;
            state.blend_src_alpha = sfactor;
            state.blend_dst_rgb = dfactor;
            state.blend_dst_alpha = dfactor;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // blendFuncSeparate
    obj.set(
        js_string!("blendFuncSeparate"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let src_rgb = args.get_or_undefined(0).to_u32(ctx)?;
            let dst_rgb = args.get_or_undefined(1).to_u32(ctx)?;
            let src_alpha = args.get_or_undefined(2).to_u32(ctx)?;
            let dst_alpha = args.get_or_undefined(3).to_u32(ctx)?;

            let mut state = data.state.lock().unwrap();
            state.blend_src_rgb = src_rgb;
            state.blend_dst_rgb = dst_rgb;
            state.blend_src_alpha = src_alpha;
            state.blend_dst_alpha = dst_alpha;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // blendEquation
    obj.set(
        js_string!("blendEquation"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let mode = args.get_or_undefined(0).to_u32(ctx)?;

            let mut state = data.state.lock().unwrap();
            state.blend_equation_rgb = mode;
            state.blend_equation_alpha = mode;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // depthFunc
    obj.set(
        js_string!("depthFunc"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let func = args.get_or_undefined(0).to_u32(ctx)?;

            data.state.lock().unwrap().depth_func = func;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // depthMask
    obj.set(
        js_string!("depthMask"),
        NativeFunction::from_fn_ptr(|this, args, _ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let flag = args.get_or_undefined(0).to_boolean();

            data.state.lock().unwrap().depth_mask = flag;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // colorMask
    obj.set(
        js_string!("colorMask"),
        NativeFunction::from_fn_ptr(|this, args, _ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let r = args.get_or_undefined(0).to_boolean();
            let g = args.get_or_undefined(1).to_boolean();
            let b = args.get_or_undefined(2).to_boolean();
            let a = args.get_or_undefined(3).to_boolean();

            data.state.lock().unwrap().color_mask = [r, g, b, a];
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // cullFace
    obj.set(
        js_string!("cullFace"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let mode = args.get_or_undefined(0).to_u32(ctx)?;

            data.state.lock().unwrap().cull_face_mode = mode;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // frontFace
    obj.set(
        js_string!("frontFace"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let mode = args.get_or_undefined(0).to_u32(ctx)?;

            data.state.lock().unwrap().front_face = mode;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // lineWidth
    obj.set(
        js_string!("lineWidth"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let width = args.get_or_undefined(0).to_number(ctx)? as f32;

            data.state.lock().unwrap().line_width = width;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // pixelStorei
    obj.set(
        js_string!("pixelStorei"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let pname = args.get_or_undefined(0).to_u32(ctx)?;
            let param = args.get_or_undefined(1);

            let mut state = data.state.lock().unwrap();
            match pname {
                WebGLConstants::UNPACK_FLIP_Y_WEBGL => {
                    state.unpack_flip_y = param.to_boolean();
                }
                WebGLConstants::UNPACK_PREMULTIPLY_ALPHA_WEBGL => {
                    state.unpack_premultiply_alpha = param.to_boolean();
                }
                WebGLConstants::UNPACK_COLORSPACE_CONVERSION_WEBGL => {
                    state.unpack_colorspace_conversion = param.to_u32(ctx).unwrap_or(0);
                }
                WebGLConstants::UNPACK_ALIGNMENT => {
                    state.unpack_alignment = param.to_u32(ctx).unwrap_or(4);
                }
                WebGLConstants::PACK_ALIGNMENT => {
                    state.pack_alignment = param.to_u32(ctx).unwrap_or(4);
                }
                _ => {}
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}
