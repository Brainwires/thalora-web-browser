//! Timer Web API implementation for Boa
//!
//! setTimeout / setInterval / clearTimeout / clearInterval,
//! requestAnimationFrame / cancelAnimationFrame and queueMicrotask.
//! https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html
//!
//! Timer state (ids, active set, nesting level) lives in the context, so
//! sessions never share timers. Callbacks are scheduled on the context's job
//! executor: with [`ThaloraJobExecutor`] they are cancellable by id; with any
//! other executor they are enqueued as engine `TimeoutJob`s and skipped when
//! cleared.

use crate::event_loop::ThaloraJobExecutor;
use boa_engine::gc::{Gc, GcRefCell};
use boa_engine::job::{Job, NativeJob, PromiseJob, TimeoutJob};
use boa_engine::object::JsObject;
use boa_engine::{
    Context, Finalize, JsArgs, JsData, JsNativeError, JsResult, JsString, JsValue, NativeFunction,
    Source, Trace, js_string,
};
use std::collections::HashSet;

/// Frame interval used for requestAnimationFrame (~60 fps).
const FRAME_INTERVAL_MS: u64 = 16;

/// Per-context timer bookkeeping.
#[derive(Default, Trace, Finalize, JsData)]
struct TimerState {
    /// Ids of timers that have not fired (one-shot) or been cleared.
    active: HashSet<u32>,
    next_id: u32,
    /// Timer nesting level of the currently running timer task (0 outside).
    nesting: u32,
}

impl TimerState {
    fn from_context(context: &mut Context) -> Gc<GcRefCell<Self>> {
        if !context.has_data::<Gc<GcRefCell<TimerState>>>() {
            context.insert_data(Gc::new(GcRefCell::new(Self::default())));
        }
        context
            .get_data::<Gc<GcRefCell<Self>>>()
            .expect("timer state was just inserted")
            .clone()
    }

    fn allocate_id(&mut self) -> u32 {
        self.next_id = self.next_id.checked_add(1).unwrap_or(1);
        self.active.insert(self.next_id);
        self.next_id
    }
}

/// What a timer runs when it fires.
#[derive(Clone)]
enum TimerCallback {
    /// A function, called with the extra arguments given to setTimeout.
    Function(JsObject, Vec<JsValue>),
    /// A string of code (legacy `setTimeout("code", ms)`), evaluated globally.
    Code(JsString),
    /// A requestAnimationFrame callback, called with a timestamp.
    AnimationFrame(JsObject),
}

impl TimerCallback {
    fn invoke(&self, context: &mut Context) -> JsResult<()> {
        match self {
            Self::Function(function, args) => {
                function.call(&JsValue::undefined(), args, context)?;
            }
            Self::Code(code) => {
                let code = code.to_std_string_escaped();
                context.eval(Source::from_bytes(code.as_bytes()))?;
            }
            Self::AnimationFrame(function) => {
                let timestamp = context.clock().now().millis_since_epoch() as f64;
                function.call(&JsValue::undefined(), &[JsValue::from(timestamp)], context)?;
            }
        }
        Ok(())
    }
}

/// Timer API implementation
pub struct Timers;

impl Timers {
    /// Initialize timer functions in the global scope
    pub fn init(context: &mut Context) {
        let functions: [(JsString, usize, NativeFunction); 7] = [
            (
                js_string!("setTimeout"),
                2,
                NativeFunction::from_fn_ptr(Self::set_timeout),
            ),
            (
                js_string!("setInterval"),
                2,
                NativeFunction::from_fn_ptr(Self::set_interval),
            ),
            (
                js_string!("clearTimeout"),
                1,
                NativeFunction::from_fn_ptr(Self::clear_timer),
            ),
            (
                js_string!("clearInterval"),
                1,
                NativeFunction::from_fn_ptr(Self::clear_timer),
            ),
            (
                js_string!("requestAnimationFrame"),
                1,
                NativeFunction::from_fn_ptr(Self::request_animation_frame),
            ),
            (
                js_string!("cancelAnimationFrame"),
                1,
                NativeFunction::from_fn_ptr(Self::clear_timer),
            ),
            (
                js_string!("queueMicrotask"),
                1,
                NativeFunction::from_fn_ptr(Self::queue_microtask),
            ),
        ];
        for (name, length, function) in functions {
            let message = format!("Failed to register {}", name.to_std_string_escaped());
            context
                .register_global_builtin_callable(name, length, function)
                .expect(&message);
        }
    }

    /// setTimeout(callback, delay, ...args)
    fn set_timeout(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::start_timer(args, context, false)
    }

    /// setInterval(callback, delay, ...args)
    fn set_interval(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::start_timer(args, context, true)
    }

    /// requestAnimationFrame(callback): runs the callback on the next frame
    /// (~16 ms) with a timestamp argument.
    fn request_animation_frame(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let Some(function) = args.get_or_undefined(0).as_callable() else {
            return Err(JsNativeError::typ()
                .with_message("requestAnimationFrame: callback is not a function")
                .into());
        };
        let state = TimerState::from_context(context);
        let id = state.borrow_mut().allocate_id();
        schedule(
            context,
            id,
            FRAME_INTERVAL_MS,
            false,
            TimerCallback::AnimationFrame(function),
            0,
        );
        Ok(JsValue::from(id))
    }

    /// queueMicrotask(callback): runs the callback at the next microtask
    /// checkpoint (after the current script, before timers).
    fn queue_microtask(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let Some(function) = args.get_or_undefined(0).as_callable() else {
            return Err(JsNativeError::typ()
                .with_message("queueMicrotask: callback is not a function")
                .into());
        };
        context.enqueue_job(Job::PromiseJob(PromiseJob::new(move |ctx| {
            function.call(&JsValue::undefined(), &[], ctx)?;
            Ok(JsValue::undefined())
        })));
        Ok(JsValue::undefined())
    }

    /// Shared implementation of setTimeout / setInterval.
    fn start_timer(args: &[JsValue], context: &mut Context, repeating: bool) -> JsResult<JsValue> {
        let handler = args.get_or_undefined(0);
        let callback = if let Some(function) = handler.as_callable() {
            TimerCallback::Function(function, args.iter().skip(2).cloned().collect())
        } else if handler.is_undefined() {
            // Nothing to run; browsers still return a valid id.
            TimerCallback::Code(JsString::default())
        } else {
            TimerCallback::Code(handler.to_string(context)?)
        };

        let delay = timeout_ms(args.get_or_undefined(1), context)?;
        let state = TimerState::from_context(context);
        let (id, nesting) = {
            let mut state = state.borrow_mut();
            (state.allocate_id(), state.nesting)
        };
        schedule(
            context,
            id,
            clamp_for_nesting(delay, nesting),
            repeating,
            callback,
            nesting,
        );
        Ok(JsValue::from(id))
    }

    /// clearTimeout / clearInterval / cancelAnimationFrame(id)
    fn clear_timer(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let id_value = args.get_or_undefined(0);
        if id_value.is_undefined() || id_value.is_null() {
            return Ok(JsValue::undefined());
        }
        let id = id_value.to_u32(context)?;
        let state = TimerState::from_context(context);
        state.borrow_mut().active.remove(&id);
        if let Some(executor) = context.downcast_job_executor::<ThaloraJobExecutor>() {
            executor.cancel_timer(id);
        }
        Ok(JsValue::undefined())
    }

    /// Number of timers that are scheduled and not cleared (for tests).
    pub fn active_timers_count(context: &mut Context) -> usize {
        TimerState::from_context(context).borrow().active.len()
    }
}

/// Convert the delay argument per HTML: non-finite or negative → 0, values
/// beyond the 32-bit signed range overflow to 1 ms as in browsers.
fn timeout_ms(value: &JsValue, context: &mut Context) -> JsResult<u64> {
    if value.is_undefined() {
        return Ok(0);
    }
    let ms = value.to_number(context)?;
    Ok(if !ms.is_finite() || ms <= 0.0 {
        0
    } else if ms > f64::from(i32::MAX) {
        1
    } else {
        ms as u64
    })
}

/// HTML: timers nested more than 5 levels deep are clamped to at least 4 ms.
fn clamp_for_nesting(delay: u64, nesting: u32) -> u64 {
    if nesting > 5 && delay < 4 { 4 } else { delay }
}

/// Queue the timer task on the context's executor.
fn schedule(
    context: &mut Context,
    id: u32,
    delay: u64,
    repeating: bool,
    callback: TimerCallback,
    nesting: u32,
) {
    let job = NativeJob::new(move |ctx| fire(ctx, id, delay, repeating, callback, nesting));
    if let Some(executor) = context.downcast_job_executor::<ThaloraJobExecutor>() {
        let now = context.clock().now();
        executor.schedule_timer(now, delay, Some(id), repeating, job);
    } else if repeating {
        context.enqueue_job(Job::TimeoutJob(TimeoutJob::recurring(job, delay)));
    } else {
        context.enqueue_job(Job::TimeoutJob(TimeoutJob::new(job, delay)));
    }
}

/// Run a timer task: skip if cleared, run the callback, re-arm intervals.
fn fire(
    context: &mut Context,
    id: u32,
    delay: u64,
    repeating: bool,
    callback: TimerCallback,
    nesting: u32,
) -> JsResult<JsValue> {
    let state = TimerState::from_context(context);
    if !state.borrow().active.contains(&id) {
        return Ok(JsValue::undefined());
    }
    if !repeating {
        state.borrow_mut().active.remove(&id);
    }

    let task_nesting = nesting.saturating_add(1);
    state.borrow_mut().nesting = task_nesting;
    let result = callback.invoke(context);
    state.borrow_mut().nesting = 0;

    if repeating && state.borrow().active.contains(&id) {
        schedule(
            context,
            id,
            clamp_for_nesting(delay, task_nesting),
            true,
            callback,
            task_nesting,
        );
    }
    result.map(|()| JsValue::undefined())
}
