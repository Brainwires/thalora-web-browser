//! WebGL Buffer Methods
//!
//! Buffer creation and manipulation operations.

use boa_engine::{
    js_string,
    object::builtins::JsArrayBuffer,
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
};

use super::buffer::WebGLBuffer;
use super::context::{get_object_id, WebGLRenderingContextData};
use super::state::WebGLConstants;
use crate::with_webgl_context;

pub fn add_buffer_methods(obj: &JsObject, context: &mut Context) {
    // createBuffer
    obj.set(
        js_string!("createBuffer"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);

            let buffer = WebGLBuffer::new();
            let id = buffer.id;
            data.buffers.lock().unwrap().insert(id, buffer);

            let buffer_obj = JsObject::with_null_proto();
            buffer_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(buffer_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // bindBuffer
    obj.set(
        js_string!("bindBuffer"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let target = args.get_or_undefined(0).to_u32(ctx)?;
            let buffer_id = if args.get_or_undefined(1).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(1), ctx)?)
            };

            let mut state = data.state.lock().unwrap();
            match target {
                WebGLConstants::ARRAY_BUFFER => {
                    state.bound_array_buffer = buffer_id;
                }
                WebGLConstants::ELEMENT_ARRAY_BUFFER => {
                    state.bound_element_array_buffer = buffer_id;
                }
                _ => {
                    data.set_error(WebGLConstants::INVALID_ENUM);
                }
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // bufferData
    obj.set(
        js_string!("bufferData"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let target = args.get_or_undefined(0).to_u32(ctx)?;
            let usage = args.get_or_undefined(2).to_u32(ctx)?;

            let buffer_id = {
                let state = data.state.lock().unwrap();
                match target {
                    WebGLConstants::ARRAY_BUFFER => state.bound_array_buffer,
                    WebGLConstants::ELEMENT_ARRAY_BUFFER => state.bound_element_array_buffer,
                    _ => None,
                }
            };

            let Some(buffer_id) = buffer_id else {
                data.set_error(WebGLConstants::INVALID_OPERATION);
                return Ok(JsValue::undefined());
            };

            let buf_data_arg = args.get_or_undefined(1);

            // Handle size or data
            if buf_data_arg.is_number() {
                let size = buf_data_arg.to_index(ctx)? as usize;
                if let Some(buffer) = data.buffers.lock().unwrap().get_mut(&buffer_id) {
                    buffer.allocate(size, usage);
                }
            } else if let Some(array_buffer) = buf_data_arg.as_object().and_then(|o| JsArrayBuffer::from_object(o.clone()).ok()) {
                let data_ref = array_buffer.data().expect("ArrayBuffer has no data");
                let buf: Vec<u8> = (*data_ref).to_vec();
                if let Some(buffer) = data.buffers.lock().unwrap().get_mut(&buffer_id) {
                    buffer.set_data(&buf, usage);
                }
            } else if let Some(obj) = buf_data_arg.as_object() {
                // Try to get typed array data
                if let Ok(buffer_prop) = obj.get(js_string!("buffer"), ctx) {
                    if let Some(ab) = buffer_prop.as_object().and_then(|o| JsArrayBuffer::from_object(o.clone()).ok()) {
                        let byte_offset = obj.get(js_string!("byteOffset"), ctx)
                            .ok()
                            .and_then(|v| v.to_index(ctx).ok())
                            .unwrap_or(0) as usize;
                        let byte_length = obj.get(js_string!("byteLength"), ctx)
                            .ok()
                            .and_then(|v| v.to_index(ctx).ok())
                            .unwrap_or(0) as usize;

                        let data_ref = ab.data().expect("ArrayBuffer has no data");
                        let full_buf: Vec<u8> = (*data_ref).to_vec();
                        let slice = &full_buf[byte_offset..byte_offset + byte_length];
                        if let Some(buffer) = data.buffers.lock().unwrap().get_mut(&buffer_id) {
                            buffer.set_data(slice, usage);
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

    // bufferSubData
    obj.set(
        js_string!("bufferSubData"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let target = args.get_or_undefined(0).to_u32(ctx)?;
            let offset = args.get_or_undefined(1).to_index(ctx)? as usize;

            let buffer_id = {
                let state = data.state.lock().unwrap();
                match target {
                    WebGLConstants::ARRAY_BUFFER => state.bound_array_buffer,
                    WebGLConstants::ELEMENT_ARRAY_BUFFER => state.bound_element_array_buffer,
                    _ => None,
                }
            };

            let Some(buffer_id) = buffer_id else {
                return Ok(JsValue::undefined());
            };

            let buf_data_arg = args.get_or_undefined(2);

            if let Some(array_buffer) = buf_data_arg.as_object().and_then(|o| JsArrayBuffer::from_object(o.clone()).ok()) {
                let data_ref = array_buffer.data().expect("ArrayBuffer has no data");
                let buf: Vec<u8> = (*data_ref).to_vec();
                if let Some(buffer) = data.buffers.lock().unwrap().get_mut(&buffer_id) {
                    buffer.set_sub_data(offset, &buf);
                }
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // deleteBuffer
    obj.set(
        js_string!("deleteBuffer"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let buffer_id = get_object_id(args.get_or_undefined(0), ctx)?;

            if let Some(buffer) = data.buffers.lock().unwrap().get_mut(&buffer_id) {
                buffer.delete_pending = true;
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();

    // isBuffer
    obj.set(
        js_string!("isBuffer"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl_context!(this => _ctx_obj, data);
            let buffer_id = get_object_id(args.get_or_undefined(0), ctx)?;

            let exists = data.buffers.lock().unwrap().contains_key(&buffer_id);
            Ok(JsValue::from(exists))
        })
        .to_js_function(context.realm()),
        false,
        context,
    ).unwrap();
}
