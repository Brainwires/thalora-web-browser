//! WebGL2 Query, Sampler, and Sync Methods
//!
//! Query objects, sampler objects, and sync objects for WebGL2.

use boa_engine::{Context, JsArgs, JsObject, JsValue, NativeFunction, js_string};

use super::context2::{WebGLQuery, WebGLSampler, WebGLSync, get_object_id};
use super::state::{WebGL2Constants, WebGLConstants};
use crate::with_webgl2_context;

/// Add query methods
pub fn add_query_methods(obj: &JsObject, context: &mut Context) {
    static QUERY_ID_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

    // createQuery
    obj.set(
        js_string!("createQuery"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);

            let query = WebGLQuery {
                id: QUERY_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                target: 0,
                result: None,
            };
            let id = query.id;
            data.queries.lock().unwrap().insert(id, query);

            let query_obj = JsObject::with_null_proto();
            query_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(query_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // deleteQuery
    obj.set(
        js_string!("deleteQuery"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);
            let query_id = get_object_id(args.get_or_undefined(0), ctx)?;

            data.queries.lock().unwrap().remove(&query_id);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // beginQuery
    obj.set(
        js_string!("beginQuery"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);
            let target = args.get_or_undefined(0).to_u32(ctx)?;
            let query_id = get_object_id(args.get_or_undefined(1), ctx)?;

            if let Some(query) = data.queries.lock().unwrap().get_mut(&query_id) {
                query.target = target;
            }
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // endQuery
    obj.set(
        js_string!("endQuery"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // getQueryParameter
    obj.set(
        js_string!("getQueryParameter"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);
            let query_id = get_object_id(args.get_or_undefined(0), ctx)?;
            let _pname = args.get_or_undefined(1).to_u32(ctx)?;

            if let Some(query) = data.queries.lock().unwrap().get(&query_id)
                && let Some(result) = query.result
            {
                return Ok(JsValue::from(result as f64));
            }
            Ok(JsValue::null())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();
}

/// Add sampler methods
pub fn add_sampler_methods(obj: &JsObject, context: &mut Context) {
    static SAMPLER_ID_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

    // createSampler
    obj.set(
        js_string!("createSampler"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);

            let sampler = WebGLSampler {
                id: SAMPLER_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                min_filter: WebGLConstants::NEAREST_MIPMAP_LINEAR,
                mag_filter: WebGLConstants::LINEAR,
                wrap_s: WebGLConstants::REPEAT,
                wrap_t: WebGLConstants::REPEAT,
                wrap_r: WebGLConstants::REPEAT,
                min_lod: -1000.0,
                max_lod: 1000.0,
                compare_mode: 0,
                compare_func: WebGLConstants::LEQUAL,
            };
            let id = sampler.id;
            data.samplers.lock().unwrap().insert(id, sampler);

            let sampler_obj = JsObject::with_null_proto();
            sampler_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(sampler_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // deleteSampler
    obj.set(
        js_string!("deleteSampler"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);
            let sampler_id = get_object_id(args.get_or_undefined(0), ctx)?;

            data.samplers.lock().unwrap().remove(&sampler_id);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // bindSampler
    obj.set(
        js_string!("bindSampler"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // samplerParameteri
    obj.set(
        js_string!("samplerParameteri"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);
            let sampler_id = get_object_id(args.get_or_undefined(0), ctx)?;
            let pname = args.get_or_undefined(1).to_u32(ctx)?;
            let param = args.get_or_undefined(2).to_u32(ctx)?;

            if let Some(sampler) = data.samplers.lock().unwrap().get_mut(&sampler_id) {
                match pname {
                    WebGLConstants::TEXTURE_MIN_FILTER => sampler.min_filter = param,
                    WebGLConstants::TEXTURE_MAG_FILTER => sampler.mag_filter = param,
                    WebGLConstants::TEXTURE_WRAP_S => sampler.wrap_s = param,
                    WebGLConstants::TEXTURE_WRAP_T => sampler.wrap_t = param,
                    WebGL2Constants::TEXTURE_WRAP_R => sampler.wrap_r = param,
                    _ => {}
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

/// Add sync methods
pub fn add_sync_methods(obj: &JsObject, context: &mut Context) {
    static SYNC_ID_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

    // fenceSync
    obj.set(
        js_string!("fenceSync"),
        NativeFunction::from_fn_ptr(|this, _args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);

            let sync = WebGLSync {
                id: SYNC_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                signaled: false,
            };
            let id = sync.id;
            data.syncs.lock().unwrap().insert(id, sync);

            let sync_obj = JsObject::with_null_proto();
            sync_obj.set(js_string!("_id"), JsValue::from(id), false, ctx)?;
            Ok(sync_obj.into())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // deleteSync
    obj.set(
        js_string!("deleteSync"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);
            let sync_id = get_object_id(args.get_or_undefined(0), ctx)?;

            data.syncs.lock().unwrap().remove(&sync_id);
            Ok(JsValue::undefined())
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // clientWaitSync
    obj.set(
        js_string!("clientWaitSync"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);
            let sync_id = get_object_id(args.get_or_undefined(0), ctx)?;
            let _flags = args.get_or_undefined(1).to_u32(ctx)?;
            let _timeout = args.get_or_undefined(2).to_number(ctx)?;

            if let Some(sync) = data.syncs.lock().unwrap().get(&sync_id)
                && sync.signaled
            {
                return Ok(JsValue::from(WebGL2Constants::ALREADY_SIGNALED));
            }
            // For now, always return condition satisfied
            Ok(JsValue::from(WebGL2Constants::CONDITION_SATISFIED))
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // waitSync
    obj.set(
        js_string!("waitSync"),
        NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::undefined()))
            .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();

    // getSyncParameter
    obj.set(
        js_string!("getSyncParameter"),
        NativeFunction::from_fn_ptr(|this, args, ctx| {
            with_webgl2_context!(this => _ctx_obj, data);
            let sync_id = get_object_id(args.get_or_undefined(0), ctx)?;
            let _pname = args.get_or_undefined(1).to_u32(ctx)?;

            if let Some(sync) = data.syncs.lock().unwrap().get(&sync_id)
                && sync.signaled
            {
                return Ok(JsValue::from(1));
            }
            Ok(JsValue::from(0))
        })
        .to_js_function(context.realm()),
        false,
        context,
    )
    .unwrap();
}
