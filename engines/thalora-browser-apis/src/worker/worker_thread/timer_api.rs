//! Worker timer API implementation
//!
//! Provides setTimeout, setInterval, clearTimeout, clearInterval, queueMicrotask for workers
//! This is spec-compliant: callbacks are stored in Context, event loop only has IDs

use boa_engine::{Context, JsResult, JsValue, JsArgs, JsNativeError, js_string};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use once_cell::sync::Lazy;
use super::event_loop::WorkerEventLoop;
use super::callback_registry;

/// Global registry mapping thread IDs to event loops
static EVENT_LOOP_REGISTRY: Lazy<Mutex<HashMap<std::thread::ThreadId, Arc<Mutex<WorkerEventLoop>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Register an event loop for the current thread
pub fn register_event_loop(event_loop: Arc<Mutex<WorkerEventLoop>>) {
    let thread_id = std::thread::current().id();
    let mut registry = EVENT_LOOP_REGISTRY.lock().unwrap();
    registry.insert(thread_id, event_loop);
}

/// Unregister the event loop for the current thread
pub fn unregister_event_loop() {
    let thread_id = std::thread::current().id();
    let mut registry = EVENT_LOOP_REGISTRY.lock().unwrap();
    registry.remove(&thread_id);

    // Also clear all callbacks for this thread
    callback_registry::clear_all_callbacks();
}

/// Get the event loop for the current thread
fn get_current_event_loop() -> Option<Arc<Mutex<WorkerEventLoop>>> {
    let thread_id = std::thread::current().id();
    let registry = EVENT_LOOP_REGISTRY.lock().unwrap();
    registry.get(&thread_id).cloned()
}

/// Initialize timer functions in a worker context
pub fn init_worker_timers(context: &mut Context, event_loop: Arc<Mutex<WorkerEventLoop>>) -> JsResult<()> {
    // Register the event loop for this thread
    register_event_loop(event_loop);

    let global = context.global_object();

    // setTimeout
    let set_timeout_func = boa_engine::object::FunctionObjectBuilder::new(
        context.realm(),
        boa_engine::NativeFunction::from_fn_ptr(set_timeout_impl),
    )
    .name(js_string!("setTimeout"))
    .length(2)
    .build();

    global.set(js_string!("setTimeout"), set_timeout_func, false, context)?;

    // setInterval
    let set_interval_func = boa_engine::object::FunctionObjectBuilder::new(
        context.realm(),
        boa_engine::NativeFunction::from_fn_ptr(set_interval_impl),
    )
    .name(js_string!("setInterval"))
    .length(2)
    .build();

    global.set(js_string!("setInterval"), set_interval_func, false, context)?;

    // clearTimeout
    let clear_timeout_func = boa_engine::object::FunctionObjectBuilder::new(
        context.realm(),
        boa_engine::NativeFunction::from_fn_ptr(clear_timer_impl),
    )
    .name(js_string!("clearTimeout"))
    .length(1)
    .build();

    global.set(js_string!("clearTimeout"), clear_timeout_func, false, context)?;

    // clearInterval
    let clear_interval_func = boa_engine::object::FunctionObjectBuilder::new(
        context.realm(),
        boa_engine::NativeFunction::from_fn_ptr(clear_timer_impl),
    )
    .name(js_string!("clearInterval"))
    .length(1)
    .build();

    global.set(js_string!("clearInterval"), clear_interval_func, false, context)?;

    // queueMicrotask
    let queue_microtask_func = boa_engine::object::FunctionObjectBuilder::new(
        context.realm(),
        boa_engine::NativeFunction::from_fn_ptr(queue_microtask_impl),
    )
    .name(js_string!("queueMicrotask"))
    .length(1)
    .build();

    global.set(js_string!("queueMicrotask"), queue_microtask_func, false, context)?;

    Ok(())
}

/// setTimeout implementation
fn set_timeout_impl(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    schedule_timer(args, context, false)
}

/// setInterval implementation
fn set_interval_impl(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    schedule_timer(args, context, true)
}

/// Schedule a timer (shared by setTimeout and setInterval)
fn schedule_timer(args: &[JsValue], context: &mut Context, repeating: bool) -> JsResult<JsValue> {
    // Get the event loop for this thread
    let event_loop = get_current_event_loop()
        .ok_or_else(|| JsNativeError::error()
            .with_message("Event loop not available"))?;

    // Get callback function
    let callback = args.get_or_undefined(0);
    if !callback.is_callable() {
        return Err(JsNativeError::typ()
            .with_message("First argument must be a function")
            .into());
    }

    let callback_obj = callback.as_object().unwrap().clone();

    // Get delay (default to 0)
    let delay = if args.len() > 1 {
        let delay_val = args.get_or_undefined(1).to_number(context)?;
        if delay_val < 0.0 {
            0
        } else {
            delay_val as u32
        }
    } else {
        0
    };

    // Collect additional arguments to pass to callback
    let callback_args: Vec<JsValue> = args.iter().skip(2).cloned().collect();

    // Store the callback in the thread-local registry
    let callback_id = callback_registry::store_callback(callback_obj, callback_args);

    // Schedule the timer in the event loop (only stores the callback ID)
    let timer_id = {
        let mut event_loop_guard = event_loop.lock().unwrap();
        event_loop_guard.schedule_timer(callback_id, delay, repeating)
    };

    Ok(JsValue::from(timer_id))
}

/// Clear a timer (clearTimeout/clearInterval)
fn clear_timer_impl(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Get the event loop for this thread
    let event_loop = get_current_event_loop()
        .ok_or_else(|| JsNativeError::error()
            .with_message("Event loop not available"))?;

    if args.is_empty() {
        return Ok(JsValue::undefined());
    }

    let timer_id = args.get_or_undefined(0).to_u32(context)?;

    // Cancel the timer
    {
        let mut event_loop_guard = event_loop.lock().unwrap();
        event_loop_guard.cancel_timer(timer_id);
    }

    Ok(JsValue::undefined())
}

/// queueMicrotask implementation
fn queue_microtask_impl(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // Get the event loop for this thread
    let event_loop = get_current_event_loop()
        .ok_or_else(|| JsNativeError::error()
            .with_message("Event loop not available"))?;

    // Get callback function
    let callback = args.get_or_undefined(0);
    if !callback.is_callable() {
        return Err(JsNativeError::typ()
            .with_message("Argument must be a function")
            .into());
    }

    let callback_obj = callback.as_object().unwrap().clone();

    // Store the callback
    let callback_id = callback_registry::store_callback(callback_obj, vec![]);

    // Queue the microtask (only stores the callback ID)
    {
        let mut event_loop_guard = event_loop.lock().unwrap();
        event_loop_guard.queue_microtask(callback_id);
    }

    Ok(JsValue::undefined())
}
