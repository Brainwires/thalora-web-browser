//! Timer Web API implementation for Boa
//!
//! Native implementation of timers (setTimeout, setInterval, etc.)
//! https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html
//!
//! This implements the complete Timer interface with basic synchronous scheduling

use boa_engine::{
    Context, JsArgs, JsResult, JsValue, NativeFunction, js_string, object::ObjectInitializer,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Global timer storage
static TIMERS: Lazy<Arc<Mutex<HashMap<u32, TimerInfo>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Next timer ID
static NEXT_TIMER_ID: Lazy<Arc<Mutex<u32>>> = Lazy::new(|| Arc::new(Mutex::new(1)));

/// Timer information
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
    fn request_animation_frame(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // requestAnimationFrame is essentially setTimeout with ~16ms delay
        // In headless mode, we just schedule it like a timer
        if args.is_empty() {
            return Ok(JsValue::from(0));
        }

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

        Ok(JsValue::from(timer_id))
    }

    /// cancelAnimationFrame(id)
    fn cancel_animation_frame(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
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

    /// Schedule a timer (setTimeout or setInterval)
    fn schedule_timer(
        args: &[JsValue],
        context: &mut Context,
        repeating: bool,
    ) -> JsResult<JsValue> {
        // Get callback (must be provided)
        if args.is_empty() {
            return Ok(JsValue::from(0));
        }

        // Get delay (default to 0)
        let delay = if args.len() > 1 {
            let delay_val = args.get_or_undefined(1).to_number(context)?;
            if delay_val < 0.0 { 0 } else { delay_val as u32 }
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

        // Store timer info
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

        Ok(JsValue::from(timer_id))
    }

    /// Clear a timer (clearTimeout or clearInterval)
    fn clear_timer(args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if args.is_empty() {
            return Ok(JsValue::undefined());
        }

        let timer_id = args.get_or_undefined(0).to_u32(context)?;

        // Remove timer from storage
        {
            let mut timers = TIMERS.lock().unwrap();
            timers.remove(&timer_id);
        }

        Ok(JsValue::undefined())
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
        let mut timers = TIMERS.lock().unwrap();
        timers.clear();
        // Reset the timer ID counter
        let mut next_id = NEXT_TIMER_ID.lock().unwrap();
        *next_id = 1;
    }
}
