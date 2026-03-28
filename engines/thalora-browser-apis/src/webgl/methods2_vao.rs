//! WebGL2 VAO, Buffer, and Texture Methods
//!
//! Vertex Array Object, buffer, and texture operations for WebGL2.

use boa_engine::{Context, JsArgs, JsObject, JsValue, NativeFunction, js_string};

use super::buffer::VertexAttribArray;
use super::context2::{MAX_VERTEX_ATTRIBS, WebGLVertexArrayObject, get_object_id};
use crate::with_webgl2_context;

/// Add VAO methods
pub fn add_vao_methods(obj: &JsObject, context: &mut Context) {
    static VAO_ID_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

    // createVertexArray
    obj.set(
        js_string!("createVertexArray"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);

            let vao = WebGLVertexArrayObject {
                id: VAO_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                attribs: std::array::from_fn(|_| VertexAttribArray::default()),
                element_array_buffer: None,
            };
            let id = vao.id;
            data.vertex_array_objects.lock().unwrap().insert(id, vao);

            let vao_obj = JsObject::with_null_proto();
            vao_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(vao_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // bindVertexArray
    obj.set(
        js_string!("bindVertexArray"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);
            let vao_id = if args.get_or_undefined(0).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(0), ctx)?)
            };

            *data.current_vao.lock().unwrap() = vao_id;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // deleteVertexArray
    obj.set(
        js_string!("deleteVertexArray"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);
            let vao_id = get_object_id(args.get_or_undefined(0), ctx)?;

            data.vertex_array_objects.lock().unwrap().remove(&vao_id);

            // Unbind if currently bound
            let mut current = data.current_vao.lock().unwrap();
            if *current == Some(vao_id) {
                *current = None;
            }

            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // isVertexArray
    obj.set(
        js_string!("isVertexArray"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);
            let vao_id = get_object_id(args.get_or_undefined(0), ctx)?;

            let exists = data
                .vertex_array_objects
                .lock()
                .unwrap()
                .contains_key(&vao_id);
            Ok(JsValue::from(exists))
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();
}

/// Add WebGL2 buffer methods
pub fn add_buffer2_methods(obj: &JsObject, context: &mut Context) {
    // copyBufferSubData
    obj.set(
        js_string!("copyBufferSubData"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            // Implementation would copy data between buffers
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // getBufferSubData
    obj.set(
        js_string!("getBufferSubData"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            // Implementation would read buffer data back
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();
}

/// Add WebGL2 texture methods
pub fn add_texture2_methods(obj: &JsObject, context: &mut Context) {
    // texImage3D
    obj.set(
        js_string!("texImage3D"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            // 3D texture implementation
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // texSubImage3D
    obj.set(
        js_string!("texSubImage3D"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // compressedTexImage3D
    obj.set(
        js_string!("compressedTexImage3D"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // texStorage2D
    obj.set(
        js_string!("texStorage2D"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // texStorage3D
    obj.set(
        js_string!("texStorage3D"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();
}
