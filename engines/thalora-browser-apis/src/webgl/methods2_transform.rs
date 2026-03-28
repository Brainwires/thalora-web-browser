//! WebGL2 Transform Feedback and Uniform Buffer Methods
//!
//! Transform feedback and uniform buffer operations for WebGL2.

use boa_engine::{Context, JsArgs, JsObject, JsValue, NativeFunction, js_string};

use super::context2::{WebGLTransformFeedback, get_object_id};
use crate::with_webgl2_context;

/// Add transform feedback methods
pub fn add_transform_feedback_methods(obj: &JsObject, context: &mut Context) {
    static TF_ID_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

    // createTransformFeedback
    obj.set(
        js_string!("createTransformFeedback"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);

            let tf = WebGLTransformFeedback {
                id: TF_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                active: false,
                paused: false,
            };
            let id = tf.id;
            data.transform_feedbacks.lock().unwrap().insert(id, tf);

            let tf_obj = JsObject::with_null_proto();
            tf_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(tf_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // deleteTransformFeedback
    obj.set(
        js_string!("deleteTransformFeedback"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);
            let tf_id = get_object_id(args.get_or_undefined(0), ctx)?;

            data.transform_feedbacks.lock().unwrap().remove(&tf_id);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // bindTransformFeedback
    obj.set(
        js_string!("bindTransformFeedback"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);
            let _target = args.get_or_undefined(0).to_u32(ctx)?;
            let tf_id = if args.get_or_undefined(1).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(1), ctx)?)
            };

            *data.current_transform_feedback.lock().unwrap() = tf_id;
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // beginTransformFeedback
    obj.set(
        js_string!("beginTransformFeedback"),
        NativeFunction::from_fn_ptr(|this, _args, _ctx| {
            with_webgl2_context!(this => _ctx_obj, data);

            let tf_id = *data.current_transform_feedback.lock().unwrap();
            if let Some(tf_id) = tf_id {
                if let Some(tf) = data.transform_feedbacks.lock().unwrap().get_mut(&tf_id) {
                    tf.active = true;
                    tf.paused = false;
                }
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // endTransformFeedback
    obj.set(
        js_string!("endTransformFeedback"),
        NativeFunction::from_fn_ptr(|this, _args, _ctx| {
            with_webgl2_context!(this => _ctx_obj, data);

            let tf_id = *data.current_transform_feedback.lock().unwrap();
            if let Some(tf_id) = tf_id {
                if let Some(tf) = data.transform_feedbacks.lock().unwrap().get_mut(&tf_id) {
                    tf.active = false;
                }
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // pauseTransformFeedback
    obj.set(
        js_string!("pauseTransformFeedback"),
        NativeFunction::from_fn_ptr(|this, _args, _ctx| {
            with_webgl2_context!(this => _ctx_obj, data);

            let tf_id = *data.current_transform_feedback.lock().unwrap();
            if let Some(tf_id) = tf_id {
                if let Some(tf) = data.transform_feedbacks.lock().unwrap().get_mut(&tf_id) {
                    tf.paused = true;
                }
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // resumeTransformFeedback
    obj.set(
        js_string!("resumeTransformFeedback"),
        NativeFunction::from_fn_ptr(|this, _args, _ctx| {
            with_webgl2_context!(this => _ctx_obj, data);

            let tf_id = *data.current_transform_feedback.lock().unwrap();
            if let Some(tf_id) = tf_id {
                if let Some(tf) = data.transform_feedbacks.lock().unwrap().get_mut(&tf_id) {
                    tf.paused = false;
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

/// Add uniform buffer methods
pub fn add_uniform_buffer_methods(obj: &JsObject, context: &mut Context) {
    // getUniformBlockIndex
    obj.set(
        js_string!("getUniformBlockIndex"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
            // Would return the index of a uniform block
            Ok(JsValue::from(0))
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // uniformBlockBinding
    obj.set(
        js_string!("uniformBlockBinding"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // bindBufferBase
    obj.set(
        js_string!("bindBufferBase"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);
            let _target = args.get_or_undefined(0).to_u32(ctx)?;
            let index = args.get_or_undefined(1).to_u32(ctx)?;
            let buffer_id = if args.get_or_undefined(2).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(2), ctx)?)
            };

            data.uniform_buffer_bindings
                .lock()
                .unwrap()
                .insert(index, buffer_id);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // bindBufferRange
    obj.set(
        js_string!("bindBufferRange"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);
            let _target = args.get_or_undefined(0).to_u32(ctx)?;
            let index = args.get_or_undefined(1).to_u32(ctx)?;
            let buffer_id = if args.get_or_undefined(2).is_null() {
                None
            } else {
                Some(get_object_id(args.get_or_undefined(2), ctx)?)
            };

            data.uniform_buffer_bindings
                .lock()
                .unwrap()
                .insert(index, buffer_id);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();
}
