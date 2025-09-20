use boa_engine::{Context, JsValue, NativeFunction, js_string};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};

/// Timer implementation for browser compatibility
pub struct TimerManager {
    active_timers: Arc<Mutex<HashMap<u32, bool>>>,
    next_id: Arc<AtomicU32>,
}

impl TimerManager {
    pub fn new() -> Self {
        Self {
            active_timers: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(AtomicU32::new(1)),
        }
    }

    /// Setup timer functions in JavaScript context
    pub fn setup_real_timers(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        let window_obj = context.global_object();

        // setTimeout implementation
        let active_timers_clone = Arc::clone(&self.active_timers);
        let next_id_clone = Arc::clone(&self.next_id);
        let set_timeout_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            if args.is_empty() {
                return Err(boa_engine::JsNativeError::typ()
                    .with_message("setTimeout requires a callback")
                    .into());
            }

            let callback = args[0].clone();
            let delay_ms = if args.len() > 1 {
                args[1].to_number(context).unwrap_or(0.0).max(0.0) as u64
            } else {
                0
            };

            let id = next_id_clone.fetch_add(1, Ordering::SeqCst);

            // Store timer as active
            {
                let mut timers = active_timers_clone.lock().unwrap();
                timers.insert(id, true);
            }

            // For now, execute immediately if delay is 0
            if delay_ms == 0 {
                if callback.is_callable() {
                    let _ = callback.as_callable().unwrap().call(&JsValue::undefined(), &[], context);
                }
                // Remove from active timers
                {
                    let mut timers = active_timers_clone.lock().unwrap();
                    timers.remove(&id);
                }
            }

            Ok(JsValue::from(id))
        }) };
        window_obj.set(js_string!("setTimeout"), JsValue::from(set_timeout_fn.to_js_function(context.realm())), false, context)?;

        // clearTimeout implementation
        let active_timers_clear = Arc::clone(&self.active_timers);
        let clear_timeout_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            if !args.is_empty() {
                if let Ok(id) = args[0].to_u32(context) {
                    let mut timers = active_timers_clear.lock().unwrap();
                    timers.remove(&id);
                }
            }
            Ok(JsValue::undefined())
        }) };
        window_obj.set(js_string!("clearTimeout"), JsValue::from(clear_timeout_fn.to_js_function(context.realm())), false, context)?;

        // setInterval implementation
        let active_timers_interval = Arc::clone(&self.active_timers);
        let next_id_interval = Arc::clone(&self.next_id);
        let set_interval_fn = unsafe { NativeFunction::from_closure(move |_, args, _context| {
            if args.is_empty() {
                return Err(boa_engine::JsNativeError::typ()
                    .with_message("setInterval requires a callback")
                    .into());
            }

            let id = next_id_interval.fetch_add(1, Ordering::SeqCst);

            // Store timer as active
            {
                let mut timers = active_timers_interval.lock().unwrap();
                timers.insert(id, true);
            }

            Ok(JsValue::from(id))
        }) };
        window_obj.set(js_string!("setInterval"), JsValue::from(set_interval_fn.to_js_function(context.realm())), false, context)?;

        // clearInterval implementation
        let active_timers_clear_interval = Arc::clone(&self.active_timers);
        let clear_interval_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            if !args.is_empty() {
                if let Ok(id) = args[0].to_u32(context) {
                    let mut timers = active_timers_clear_interval.lock().unwrap();
                    timers.remove(&id);
                }
            }
            Ok(JsValue::undefined())
        }) };
        window_obj.set(js_string!("clearInterval"), JsValue::from(clear_interval_fn.to_js_function(context.realm())), false, context)?;

        // requestAnimationFrame implementation
        let active_timers_raf = Arc::clone(&self.active_timers);
        let next_id_raf = Arc::clone(&self.next_id);
        let request_animation_frame_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            if args.is_empty() {
                return Err(boa_engine::JsNativeError::typ()
                    .with_message("requestAnimationFrame requires a callback")
                    .into());
            }

            let callback = args[0].clone();
            let id = next_id_raf.fetch_add(1, Ordering::SeqCst);

            // Store timer as active
            {
                let mut timers = active_timers_raf.lock().unwrap();
                timers.insert(id, true);
            }

            // Execute callback immediately with more realistic timestamp
            if callback.is_callable() {
                // Use performance.now() style timestamp (milliseconds since load)
                let timestamp = JsValue::from(std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as f64 % 1000000.0); // Keep reasonable range
                let _ = callback.as_callable().unwrap().call(&JsValue::undefined(), &[timestamp], context);
            }

            // Remove from active timers
            {
                let mut timers = active_timers_raf.lock().unwrap();
                timers.remove(&id);
            }

            Ok(JsValue::from(id))
        }) };
        window_obj.set(js_string!("requestAnimationFrame"), JsValue::from(request_animation_frame_fn.to_js_function(context.realm())), false, context)?;

        // cancelAnimationFrame implementation
        let active_timers_cancel_raf = Arc::clone(&self.active_timers);
        let cancel_animation_frame_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            if !args.is_empty() {
                if let Ok(id) = args[0].to_u32(context) {
                    let mut timers = active_timers_cancel_raf.lock().unwrap();
                    timers.remove(&id);
                }
            }
            Ok(JsValue::undefined())
        }) };
        window_obj.set(js_string!("cancelAnimationFrame"), JsValue::from(cancel_animation_frame_fn.to_js_function(context.realm())), false, context)?;

        Ok(())
    }

    /// Get active timer count for monitoring
    pub fn get_active_timer_count(&self) -> usize {
        self.active_timers.lock().unwrap().len()
    }

    /// Cancel all active timers
    pub fn cancel_all_timers(&self) {
        let mut timers = self.active_timers.lock().unwrap();
        timers.clear();
    }
}

impl Drop for TimerManager {
    fn drop(&mut self) {
        self.cancel_all_timers();
    }
}