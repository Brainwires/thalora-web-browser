//! Timer Web API implementation for Boa
//!
//! Native implementation of timers (setTimeout, setInterval, etc.)
//! https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html
//!
//! This implements the complete Timer interface with callback execution support.
//!
//! ## How it works:
//! - `setTimeout`/`setInterval` store the callback function and schedule info
//! - `process_timers()` must be called periodically to execute due callbacks
//! - This is typically done by the event loop in `evaluate_javascript_with_async_wait`

use boa_engine::{
    Context, JsArgs, JsResult, JsValue, NativeFunction,
    object::{ObjectInitializer, JsObject},
    js_string,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use once_cell::sync::Lazy;

/// Global timer storage - stores timer info (without callbacks due to thread-safety)
static TIMERS: Lazy<Arc<Mutex<HashMap<u32, TimerInfo>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Next timer ID
static NEXT_TIMER_ID: Lazy<Arc<Mutex<u32>>> =
    Lazy::new(|| Arc::new(Mutex::new(1)));

/// Timer callbacks stored separately (JsObject is not Send/Sync, so context-local)
/// This is stored in the JS context via a global variable
thread_local! {
    static TIMER_CALLBACKS: std::cell::RefCell<HashMap<u32, (JsObject, Vec<JsValue>)>> =
        std::cell::RefCell::new(HashMap::new());
}

/// Timer information (metadata only - callback stored separately)
#[derive(Debug, Clone)]
pub struct TimerInfo {
    pub id: u32,
    pub created_at: Instant,
    pub delay: u32,
    pub repeating: bool,
    pub active: bool,
}

/// Timer API implementation
pub struct Timers;

impl Timers {
    /// Initialize timer functions in the global scope
    pub fn init(context: &mut Context) {
        // setTimeout
        context
            .register_global_builtin_callable(
                js_string!("setTimeout"),
                2,
                NativeFunction::from_fn_ptr(Self::set_timeout),
            )
            .expect("Failed to register setTimeout");

        // setInterval
        context
            .register_global_builtin_callable(
                js_string!("setInterval"),
                2,
                NativeFunction::from_fn_ptr(Self::set_interval),
            )
            .expect("Failed to register setInterval");

        // clearTimeout
        context
            .register_global_builtin_callable(
                js_string!("clearTimeout"),
                1,
                NativeFunction::from_fn_ptr(Self::clear_timeout),
            )
            .expect("Failed to register clearTimeout");

        // clearInterval
        context
            .register_global_builtin_callable(
                js_string!("clearInterval"),
                1,
                NativeFunction::from_fn_ptr(Self::clear_interval),
            )
            .expect("Failed to register clearInterval");

        // requestAnimationFrame
        context
            .register_global_builtin_callable(
                js_string!("requestAnimationFrame"),
                1,
                NativeFunction::from_fn_ptr(Self::request_animation_frame),
            )
            .expect("Failed to register requestAnimationFrame");

        // cancelAnimationFrame
        context
            .register_global_builtin_callable(
                js_string!("cancelAnimationFrame"),
                1,
                NativeFunction::from_fn_ptr(Self::cancel_animation_frame),
            )
            .expect("Failed to register cancelAnimationFrame");

        // queueMicrotask
        context
            .register_global_builtin_callable(
                js_string!("queueMicrotask"),
                1,
                NativeFunction::from_fn_ptr(Self::queue_microtask),
            )
            .expect("Failed to register queueMicrotask");

        // requestIdleCallback - schedules low-priority background tasks
        context
            .register_global_builtin_callable(
                js_string!("requestIdleCallback"),
                1,
                NativeFunction::from_fn_ptr(Self::request_idle_callback),
            )
            .expect("Failed to register requestIdleCallback");

        // cancelIdleCallback
        context
            .register_global_builtin_callable(
                js_string!("cancelIdleCallback"),
                1,
                NativeFunction::from_fn_ptr(Self::cancel_idle_callback),
            )
            .expect("Failed to register cancelIdleCallback");
    }

    /// setTimeout(callback, delay, ...args)
    fn set_timeout(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::schedule_timer(args, context, false)
    }

    /// setInterval(callback, delay, ...args)
    fn set_interval(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::schedule_timer(args, context, true)
    }

    /// clearTimeout(id)
    fn clear_timeout(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::clear_timer(args, context)
    }

    /// clearInterval(id)
    fn clear_interval(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::clear_timer(args, context)
    }

    /// requestAnimationFrame(callback)
    /// In a headless environment, we simulate ~60fps (16ms delay)
    fn request_animation_frame(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // requestAnimationFrame is essentially setTimeout with ~16ms delay
        // In headless mode, we just schedule it like a timer
        if args.is_empty() {
            return Ok(JsValue::from(0));
        }

        let callback = args.get_or_undefined(0);

        // Callback must be callable
        let callable = match callback.as_object() {
            Some(obj) if obj.is_callable() => obj.clone(),
            _ => return Ok(JsValue::from(0)),
        };

        // Generate timer ID
        let timer_id = {
            let mut next_id = NEXT_TIMER_ID.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        // Store timer info (16ms delay to simulate ~60fps)
        let timer_info = TimerInfo {
            id: timer_id,
            created_at: Instant::now(),
            delay: 16, // ~60fps
            repeating: false,
            active: true,
        };

        {
            let mut timers = TIMERS.lock().unwrap();
            timers.insert(timer_id, timer_info);
        }

        // Store callback in thread-local storage
        // RAF callbacks receive a DOMHighResTimeStamp (milliseconds since page load)
        // We'll pass performance.now() value when executing
        TIMER_CALLBACKS.with(|callbacks| {
            callbacks.borrow_mut().insert(timer_id, (callable, Vec::new()));
        });

        Ok(JsValue::from(timer_id))
    }

    /// cancelAnimationFrame(id)
    fn cancel_animation_frame(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::clear_timer(args, context)
    }

    /// queueMicrotask(callback)
    /// Queues a microtask to be executed
    fn queue_microtask(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // In a headless environment without an event loop, we execute immediately
        // This is a simplified implementation - a full implementation would queue for next microtask checkpoint
        if args.is_empty() {
            return Ok(JsValue::undefined());
        }

        let callback = args.get_or_undefined(0);
        if let Some(callable) = callback.as_callable() {
            // Execute the callback immediately (simplified behavior)
            // A full implementation would queue this for the microtask checkpoint
            let _ = callable.call(&JsValue::undefined(), &[], context);
        }

        Ok(JsValue::undefined())
    }

    /// requestIdleCallback(callback, options?)
    /// Schedules a callback to run during browser's idle periods
    /// In our headless implementation, we treat it like a setTimeout with 0 delay
    fn request_idle_callback(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if args.is_empty() {
            return Ok(JsValue::from(0));
        }

        let callback = args.get_or_undefined(0);

        // Callback must be callable
        let callable = match callback.as_object() {
            Some(obj) if obj.is_callable() => obj.clone(),
            _ => return Ok(JsValue::from(0)),
        };

        // Get timeout option if provided
        let timeout = if args.len() > 1 {
            if let Some(options) = args.get_or_undefined(1).as_object() {
                if let Ok(timeout_val) = options.get(js_string!("timeout"), context) {
                    timeout_val.to_number(context).unwrap_or(0.0) as u32
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        };

        // Generate timer ID
        let timer_id = {
            let mut next_id = NEXT_TIMER_ID.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        // Store timer info with minimal delay (1ms for idle callbacks)
        let timer_info = TimerInfo {
            id: timer_id,
            created_at: Instant::now(),
            delay: if timeout > 0 { timeout } else { 1 },
            repeating: false,
            active: true,
        };

        {
            let mut timers = TIMERS.lock().unwrap();
            timers.insert(timer_id, timer_info);
        }

        // Create IdleDeadline-like object to pass to callback
        // For simplicity, we'll pass an object with timeRemaining() and didTimeout
        let idle_deadline = context.eval(boa_engine::Source::from_bytes(
            "({ timeRemaining: function() { return 50; }, didTimeout: false })"
        )).unwrap_or(JsValue::undefined());

        // Store callback with the deadline argument
        TIMER_CALLBACKS.with(|callbacks| {
            callbacks.borrow_mut().insert(timer_id, (callable, vec![idle_deadline]));
        });

        Ok(JsValue::from(timer_id))
    }

    /// cancelIdleCallback(id)
    fn cancel_idle_callback(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::clear_timer(args, context)
    }

    /// Schedule a timer (setTimeout or setInterval)
    fn schedule_timer(args: &[JsValue], context: &mut Context, repeating: bool) -> JsResult<JsValue> {
        // Get callback (must be provided)
        if args.is_empty() {
            return Ok(JsValue::from(0));
        }

        let callback = args.get_or_undefined(0);

        // Callback must be callable
        let callable = match callback.as_object() {
            Some(obj) if obj.is_callable() => obj.clone(),
            _ => {
                // Per spec, non-callable first argument should still return a timer ID
                // but the callback won't fire
                return Ok(JsValue::from(0));
            }
        };

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
        let extra_args: Vec<JsValue> = if args.len() > 2 {
            args[2..].to_vec()
        } else {
            Vec::new()
        };

        // Generate timer ID
        let timer_id = {
            let mut next_id = NEXT_TIMER_ID.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        // Store timer metadata
        let timer_info = TimerInfo {
            id: timer_id,
            created_at: Instant::now(),
            delay,
            repeating,
            active: true,
        };

        {
            let mut timers = TIMERS.lock().unwrap();
            timers.insert(timer_id, timer_info);
        }

        // Store callback in thread-local storage
        TIMER_CALLBACKS.with(|callbacks| {
            callbacks.borrow_mut().insert(timer_id, (callable, extra_args));
        });

        Ok(JsValue::from(timer_id))
    }

    /// Clear a timer (clearTimeout or clearInterval)
    fn clear_timer(args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if args.is_empty() {
            return Ok(JsValue::undefined());
        }

        let timer_id = args.get_or_undefined(0).to_u32(context)?;

        // Remove timer from metadata storage
        {
            let mut timers = TIMERS.lock().unwrap();
            timers.remove(&timer_id);
        }

        // Remove callback from thread-local storage
        TIMER_CALLBACKS.with(|callbacks| {
            callbacks.borrow_mut().remove(&timer_id);
        });

        Ok(JsValue::undefined())
    }

    /// Process all due timers and execute their callbacks.
    /// This should be called periodically by the event loop.
    /// Returns the number of timers that were executed.
    pub fn process_timers(context: &mut Context) -> usize {
        let now = Instant::now();
        let mut executed_count = 0;
        let mut to_execute: Vec<u32> = Vec::new();
        let mut to_reschedule: Vec<(u32, TimerInfo)> = Vec::new();
        let mut to_remove: Vec<u32> = Vec::new();

        // First pass: find due timers
        {
            let timers = TIMERS.lock().unwrap();
            for (id, info) in timers.iter() {
                if !info.active {
                    continue;
                }
                let elapsed = now.duration_since(info.created_at).as_millis() as u32;
                if elapsed >= info.delay {
                    to_execute.push(*id);
                    if info.repeating {
                        // Reschedule for next interval
                        let mut new_info = info.clone();
                        new_info.created_at = now;
                        to_reschedule.push((*id, new_info));
                    } else {
                        to_remove.push(*id);
                    }
                }
            }
        }

        // Execute callbacks
        for timer_id in to_execute {
            let callback_data = TIMER_CALLBACKS.with(|callbacks| {
                callbacks.borrow().get(&timer_id).cloned()
            });

            if let Some((callable, args)) = callback_data {
                // Execute the callback
                match callable.call(&JsValue::undefined(), &args, context) {
                    Ok(_) => {
                        executed_count += 1;
                    }
                    Err(e) => {
                        // Log error but continue processing other timers
                        eprintln!("⏱️ TIMER: Callback error for timer {}: {:?}", timer_id, e);
                    }
                }
            }
        }

        // Update timer storage
        {
            let mut timers = TIMERS.lock().unwrap();
            for id in &to_remove {
                timers.remove(id);
            }
            for (id, info) in to_reschedule {
                timers.insert(id, info);
            }
        }

        // Remove non-repeating timer callbacks
        TIMER_CALLBACKS.with(|callbacks| {
            let mut cbs = callbacks.borrow_mut();
            for id in to_remove {
                cbs.remove(&id);
            }
        });

        executed_count
    }

    /// Check if any timers are pending (for event loop to know if it should keep running)
    pub fn has_pending_timers() -> bool {
        let timers = TIMERS.lock().unwrap();
        !timers.is_empty()
    }

    /// Get count of pending timers
    pub fn pending_timers_count() -> usize {
        let timers = TIMERS.lock().unwrap();
        timers.len()
    }

    /// Get timer info (for testing)
    #[cfg(test)]
    pub fn get_timer_info(timer_id: u32) -> Option<TimerInfo> {
        let timers = TIMERS.lock().unwrap();
        timers.get(&timer_id).cloned()
    }

    /// Get all active timers count (for testing)
    #[cfg(test)]
    pub fn active_timers_count() -> usize {
        let timers = TIMERS.lock().unwrap();
        timers.len()
    }

    /// Clear all timers (for testing)
    #[cfg(test)]
    pub fn clear_all_timers() {
        {
            let mut timers = TIMERS.lock().unwrap();
            timers.clear();
        }
        // Clear callbacks
        TIMER_CALLBACKS.with(|callbacks| {
            callbacks.borrow_mut().clear();
        });
        // Reset the timer ID counter
        let mut next_id = NEXT_TIMER_ID.lock().unwrap();
        *next_id = 1;
    }
}
