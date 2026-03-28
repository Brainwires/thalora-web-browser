//! WebGL Texture and Framebuffer Methods
//!
//! Texture and framebuffer creation and manipulation operations.

use boa_engine::{
    Context, JsArgs, JsObject, JsValue, NativeFunction, js_string,
    object::builtins::JsArrayBuffer,
};

use super::buffer::{WebGLFramebuffer, WebGLRenderbuffer};
use super::context::get_object_id;
use super::state::WebGLConstants;
use super::texture::WebGLTexture;
use crate::with_webgl_context;

pub fn add_texture_methods(obj: &JsObject, context: &mut Context) {
    // createTexture
    obj.set(
        js_string!("createTexture"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);

            let texture = WebGLTexture::new();
            let id = texture.id;
            data.textures.lock().unwrap().insert(id, texture);

            let tex_obj = JsObject::with_null_proto();
            tex_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(tex_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // bindTexture
    obj.set(
        js_string!("bindTexture"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let target = args.get_or_undefined(0).to_u32(ctx)?;
            let texture_id = if args.get_or_undefined(1).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(1), ctx)?)
            };

            let state = data.state.lock().unwrap();
            let unit = state.active_texture - WebGLConstants::TEXTURE0;
            drop(state);

            if let Some(tex_id) = texture_id {
                let mut state = data.state.lock().unwrap();
                match target {
                    WebGLConstants::TEXTURE_2D => {
                        state.texture_bindings_2d.insert(unit, tex_id);
                    }
                    WebGLConstants::TEXTURE_CUBE_MAP => {
                        state.texture_bindings_cube.insert(unit, tex_id);
                    }
                    _ => {}
                }

                if let Some(texture) = data.textures.lock().unwrap().get_mut(&tex_id) {
                    texture.bind(target);
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // activeTexture
    obj.set(
        js_string!("activeTexture"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let texture = args.get_or_undefined(0).to_u32(ctx)?;

            data.state.lock().unwrap().active_texture = texture;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // texImage2D
    obj.set(
        js_string!("texImage2D"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let target = args.get_or_undefined(0).to_u32(ctx)?;
            let level = args.get_or_undefined(1).to_i32(ctx)?;
            let internal_format = args.get_or_undefined(2).to_u32(ctx)?;
            let width = args.get_or_undefined(3).to_u32(ctx)?;
            let height = args.get_or_undefined(4).to_u32(ctx)?;
            let _border = args.get_or_undefined(5).to_i32(ctx)?;
            let format = args.get_or_undefined(6).to_u32(ctx)?;
            let data_type = args.get_or_undefined(7).to_u32(ctx)?;

            // Get pixel data
            let pixels = args.get_or_undefined(8);
            let pixel_data: Option<Vec<u8>> = if pixels.is_null() || pixels.is_undefined() {
                None
            } else if let Some(array_buffer) = pixels
                .as_object()
                .and_then(|o| JsArrayBuffer::from_object(o.clone()).ok())
            {
                let data_ref = array_buffer.data().expect("ArrayBuffer has no data");
                Some((*data_ref).to_vec())
            } else if let Some(obj) = pixels.as_object() {
                // Try typed array
                if let Ok(buffer_prop) = obj.get(js_string!("buffer"), ctx) {
                    if let Some(ab) = buffer_prop
                        .as_object()
                        .and_then(|o| JsArrayBuffer::from_object(o.clone()).ok())
                    {
                        let byte_offset = obj
                            .get(js_string!("byteOffset"), ctx)
                            .ok()
                            .and_then(|v| v.to_index(ctx).ok())
                            .unwrap_or(0) as usize;
                        let byte_length = obj
                            .get(js_string!("byteLength"), ctx)
                            .ok()
                            .and_then(|v| v.to_index(ctx).ok())
                            .unwrap_or(0) as usize;

                        let data_ref = ab.data().expect("ArrayBuffer has no data");
                        let full_buf: Vec<u8> = (*data_ref).to_vec();
                        Some(full_buf[byte_offset..byte_offset + byte_length].to_vec())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Get bound texture
            let texture_id = {
                let state = data.state.lock().unwrap();
                let unit = state.active_texture - WebGLConstants::TEXTURE0;
                if target == WebGLConstants::TEXTURE_2D {
                    state.texture_bindings_2d.get(&unit).copied()
                } else if target >= WebGLConstants::TEXTURE_CUBE_MAP_POSITIVE_X
                    && target <= WebGLConstants::TEXTURE_CUBE_MAP_NEGATIVE_Z
                {
                    state.texture_bindings_cube.get(&unit).copied()
                } else {
                    None
                }
            };

            if let Some(tex_id) = texture_id {
                if let Some(texture) = data.textures.lock().unwrap().get_mut(&tex_id) {
                    if target >= WebGLConstants::TEXTURE_CUBE_MAP_POSITIVE_X
                        && target <= WebGLConstants::TEXTURE_CUBE_MAP_NEGATIVE_Z
                    {
                        texture.tex_image_cube_face(
                            target,
                            level,
                            internal_format,
                            width,
                            height,
                            format,
                            data_type,
                            pixel_data.as_deref(),
                        );
                    } else {
                        texture.tex_image_2d(
                            target,
                            level,
                            internal_format,
                            width,
                            height,
                            format,
                            data_type,
                            pixel_data.as_deref(),
                        );
                    }
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // texParameteri
    obj.set(
        js_string!("texParameteri"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let target = args.get_or_undefined(0).to_u32(ctx)?;
            let pname = args.get_or_undefined(1).to_u32(ctx)?;
            let param = args.get_or_undefined(2).to_u32(ctx)?;

            let texture_id = {
                let state = data.state.lock().unwrap();
                let unit = state.active_texture - WebGLConstants::TEXTURE0;
                if target == WebGLConstants::TEXTURE_2D {
                    state.texture_bindings_2d.get(&unit).copied()
                } else {
                    state.texture_bindings_cube.get(&unit).copied()
                }
            };

            if let Some(tex_id) = texture_id {
                if let Some(texture) = data.textures.lock().unwrap().get_mut(&tex_id) {
                    texture.tex_parameter(pname, param);
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // generateMipmap
    obj.set(
        js_string!("generateMipmap"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let target = args.get_or_undefined(0).to_u32(ctx)?;

            let texture_id = {
                let state = data.state.lock().unwrap();
                let unit = state.active_texture - WebGLConstants::TEXTURE0;
                if target == WebGLConstants::TEXTURE_2D {
                    state.texture_bindings_2d.get(&unit).copied()
                } else {
                    state.texture_bindings_cube.get(&unit).copied()
                }
            };

            if let Some(tex_id) = texture_id {
                if let Some(texture) = data.textures.lock().unwrap().get_mut(&tex_id) {
                    texture.generate_mipmap();
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // deleteTexture
    obj.set(
        js_string!("deleteTexture"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let texture_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(texture) = data.textures.lock().unwrap().get_mut(&texture_id) {
                texture.delete_pending = true;
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();
}

/// Add framebuffer methods
pub fn add_framebuffer_methods(obj: &JsObject, context: &mut Context) {
    // createFramebuffer
    obj.set(
        js_string!("createFramebuffer"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);

            let fb = WebGLFramebuffer::new();
            let id = fb.id;
            data.framebuffers.lock().unwrap().insert(id, fb);

            let fb_obj = JsObject::with_null_proto();
            fb_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(fb_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // bindFramebuffer
    obj.set(
        js_string!("bindFramebuffer"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let _target = args.get_or_undefined(0).to_u32(ctx)?;
            let fb_id = if args.get_or_undefined(1).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(1), ctx)?)
            };

            data.state.lock().unwrap().bound_framebuffer = fb_id;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // checkFramebufferStatus
    obj.set(
        js_string!("checkFramebufferStatus"),
        NativeFunction::from_fn_ptr(|this, _args, _ctx| {
            with_webgl_context!(this => _ctx_obj, data);

            let fb_id = data.state.lock().unwrap().bound_framebuffer;

            if let Some(fb_id) = fb_id {
                if let Some(fb) = data.framebuffers.lock().unwrap().get(&fb_id) {
                    return Ok(JsValue::from(fb.check_status()));
                }
            }

            // Default framebuffer is always complete
            Ok(JsValue::from(WebGLConstants::FRAMEBUFFER_COMPLETE))
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // framebufferTexture2D
    obj.set(
        js_string!("framebufferTexture2D"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let _target = args.get_or_undefined(0).to_u32(ctx)?;
            let attachment = args.get_or_undefined(1).to_u32(ctx)?;
            let tex_target = args.get_or_undefined(2).to_u32(ctx)?;
            let texture_id = get_object_id(args.get_or_undefined(3), ctx)?;
            let level = args.get_or_undefined(4).to_i32(ctx)?;

            let fb_id = data.state.lock().unwrap().bound_framebuffer;

            if let Some(fb_id) = fb_id {
                if let Some(fb) = data.framebuffers.lock().unwrap().get_mut(&fb_id) {
                    let face = if tex_target >= WebGLConstants::TEXTURE_CUBE_MAP_POSITIVE_X
                        && tex_target <= WebGLConstants::TEXTURE_CUBE_MAP_NEGATIVE_Z
                    {
                        Some(tex_target)
                    } else {
                        None
                    };
                    fb.attach_texture(attachment, texture_id, level, face);
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // createRenderbuffer
    obj.set(
        js_string!("createRenderbuffer"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);

            let rb = WebGLRenderbuffer::new();
            let id = rb.id;
            data.renderbuffers.lock().unwrap().insert(id, rb);

            let rb_obj = JsObject::with_null_proto();
            rb_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(rb_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // bindRenderbuffer
    obj.set(
        js_string!("bindRenderbuffer"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let _target = args.get_or_undefined(0).to_u32(ctx)?;
            let rb_id = if args.get_or_undefined(1).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(1), ctx)?)
            };

            data.state.lock().unwrap().bound_renderbuffer = rb_id;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // renderbufferStorage
    obj.set(
        js_string!("renderbufferStorage"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let _target = args.get_or_undefined(0).to_u32(ctx)?;
            let internal_format = args.get_or_undefined(1).to_u32(ctx)?;
            let width = args.get_or_undefined(2).to_u32(ctx)?;
            let height = args.get_or_undefined(3).to_u32(ctx)?;

            let rb_id = data.state.lock().unwrap().bound_renderbuffer;

            if let Some(rb_id) = rb_id {
                if let Some(rb) = data.renderbuffers.lock().unwrap().get_mut(&rb_id) {
                    rb.storage(internal_format, width, height);
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // framebufferRenderbuffer
    obj.set(
        js_string!("framebufferRenderbuffer"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let _target = args.get_or_undefined(0).to_u32(ctx)?;
            let attachment = args.get_or_undefined(1).to_u32(ctx)?;
            let _rb_target = args.get_or_undefined(2).to_u32(ctx)?;
            let rb_id = get_object_id(args.get_or_undefined(3), ctx)?;

            let fb_id = data.state.lock().unwrap().bound_framebuffer;

            if let Some(fb_id) = fb_id {
                if let Some(fb) = data.framebuffers.lock().unwrap().get_mut(&fb_id) {
                    fb.attach_renderbuffer(attachment, rb_id);
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();
}
