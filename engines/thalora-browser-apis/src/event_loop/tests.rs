//! Tests for the event loop executor and the timer APIs built on it.

use super::{PumpBudget, PumpOutcome, ThaloraJobExecutor};
use crate::timers::timers::Timers;
use boa_engine::job::NativeAsyncJob;
use boa_engine::{Context, JsValue, Source};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

fn context_with_loop() -> (Context, Rc<ThaloraJobExecutor>) {
    let executor = Rc::new(ThaloraJobExecutor::new());
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .build()
        .expect("context builds");
    Timers::init(&mut context);
    (context, executor)
}

fn eval(context: &mut Context, code: &str) -> JsValue {
    context
        .eval(Source::from_bytes(code))
        .unwrap_or_else(|e| panic!("eval failed for `{code}`: {e}"))
}

fn eval_string(context: &mut Context, code: &str) -> String {
    eval(context, code)
        .to_string(context)
        .expect("stringifiable")
        .to_std_string_escaped()
}

fn global_is_true(name: &'static str) -> impl FnMut(&mut Context) -> bool + 'static {
    move |ctx: &mut Context| {
        ctx.eval(Source::from_bytes(format!("{name} === true").as_bytes()))
            .map(|v| v.to_boolean())
            .unwrap_or(false)
    }
}

#[test]
fn microtasks_run_before_timers() {
    let (mut ctx, executor) = context_with_loop();
    eval(
        &mut ctx,
        r#"
        var log = [];
        setTimeout(function () { log.push('timeout'); }, 0);
        Promise.resolve().then(function () { log.push('promise'); });
        queueMicrotask(function () { log.push('microtask'); });
        log.push('sync');
        "#,
    );
    let outcome = executor.pump(&mut ctx, PumpBudget::until_idle(Duration::from_secs(2)));
    assert_eq!(outcome, PumpOutcome::Idle);
    assert_eq!(
        eval_string(&mut ctx, "log.join(',')"),
        "sync,promise,microtask,timeout"
    );
}

#[test]
fn timers_fire_in_due_order_with_fifo_ties() {
    let (mut ctx, executor) = context_with_loop();
    eval(
        &mut ctx,
        r#"
        var order = [];
        setTimeout(function () { order.push('late'); }, 30);
        setTimeout(function () { order.push('first'); }, 0);
        setTimeout(function () { order.push('second'); }, 0);
        "#,
    );
    executor.pump(&mut ctx, PumpBudget::until_idle(Duration::from_secs(2)));
    assert_eq!(
        eval_string(&mut ctx, "order.join(',')"),
        "first,second,late"
    );
}

#[test]
fn timer_arguments_are_forwarded() {
    let (mut ctx, executor) = context_with_loop();
    eval(
        &mut ctx,
        "var sum = 0; setTimeout(function (a, b) { sum = a + b; }, 0, 2, 3);",
    );
    executor.pump(&mut ctx, PumpBudget::until_idle(Duration::from_secs(1)));
    assert_eq!(eval_string(&mut ctx, "sum"), "5");
}

#[test]
fn cleared_timers_never_fire() {
    let (mut ctx, executor) = context_with_loop();
    eval(
        &mut ctx,
        r#"
        var fired = false;
        var id = setTimeout(function () { fired = true; }, 10);
        clearTimeout(id);
        "#,
    );
    assert_eq!(executor.pending_timers(), 0, "cancelled timer is removed");
    executor.pump(&mut ctx, PumpBudget::until_idle(Duration::from_millis(200)));
    assert_eq!(eval_string(&mut ctx, "fired"), "false");
}

#[test]
fn string_handler_is_evaluated() {
    let (mut ctx, executor) = context_with_loop();
    eval(
        &mut ctx,
        "var viaString = 0; setTimeout('viaString = 42', 0);",
    );
    executor.pump(&mut ctx, PumpBudget::until_idle(Duration::from_secs(1)));
    assert_eq!(eval_string(&mut ctx, "viaString"), "42");
}

#[test]
fn intervals_repeat_and_do_not_hang_run_jobs() {
    let (mut ctx, executor) = context_with_loop();
    eval(
        &mut ctx,
        "var ticks = 0; var done = false; var id = setInterval(function () { if (++ticks >= 3) done = true; }, 5);",
    );
    let outcome = executor.pump(
        &mut ctx,
        PumpBudget::until(Duration::from_secs(2), global_is_true("done")),
    );
    assert_eq!(outcome, PumpOutcome::ConditionMet);

    // UntilIdle ignores intervals, and run_jobs never waits for future timers.
    let start = Instant::now();
    assert_eq!(
        executor.pump(&mut ctx, PumpBudget::until_idle(Duration::from_secs(5))),
        PumpOutcome::Idle
    );
    ctx.run_jobs().expect("run_jobs");
    assert!(
        start.elapsed() < Duration::from_secs(1),
        "must not wait on intervals"
    );

    eval(&mut ctx, "clearInterval(id);");
    assert_eq!(executor.pending_timers(), 0);
}

#[test]
fn long_running_task_is_aborted_and_loop_continues() {
    let (mut ctx, executor) = context_with_loop();
    executor.set_task_slice(Duration::from_millis(50));
    eval(
        &mut ctx,
        r#"
        var after = false;
        setTimeout(function () { while (true) {} }, 0);
        setTimeout(function () { after = true; }, 5);
        "#,
    );
    let outcome = executor.pump(
        &mut ctx,
        PumpBudget::until(Duration::from_secs(5), global_is_true("after")),
    );
    assert_eq!(outcome, PumpOutcome::ConditionMet);
}

#[test]
fn animation_frames_receive_a_timestamp() {
    let (mut ctx, executor) = context_with_loop();
    eval(
        &mut ctx,
        "var kind = 'none'; requestAnimationFrame(function (t) { kind = typeof t; });",
    );
    executor.pump(&mut ctx, PumpBudget::until_idle(Duration::from_secs(1)));
    assert_eq!(eval_string(&mut ctx, "kind"), "number");
}

#[test]
fn queue_microtask_rejects_non_functions() {
    let (mut ctx, _executor) = context_with_loop();
    assert!(ctx.eval(Source::from_bytes("queueMicrotask(42)")).is_err());
}

#[test]
fn timer_state_is_per_context() {
    let (mut a, _) = context_with_loop();
    let (mut b, _) = context_with_loop();
    assert_eq!(eval_string(&mut a, "setTimeout(function () {}, 1000)"), "1");
    assert_eq!(eval_string(&mut a, "setTimeout(function () {}, 1000)"), "2");
    assert_eq!(eval_string(&mut b, "setTimeout(function () {}, 1000)"), "1");
    assert_eq!(Timers::active_timers_count(&mut a), 2);
    assert_eq!(Timers::active_timers_count(&mut b), 1);
}

#[test]
fn async_jobs_wait_for_a_waiting_pump() {
    let (mut ctx, executor) = context_with_loop();
    let done = Rc::new(RefCell::new(false));
    let flag = done.clone();
    ctx.enqueue_job(
        NativeAsyncJob::new(async move |_ctx: &RefCell<&mut Context>| {
            *flag.borrow_mut() = true;
            Ok(JsValue::undefined())
        })
        .into(),
    );

    // A NoWait pump leaves async jobs queued rather than starting and
    // cancelling them.
    executor.pump(&mut ctx, PumpBudget::no_wait());
    assert_eq!(executor.pending_network(), 1);
    assert!(!*done.borrow());

    let outcome = executor.pump(
        &mut ctx,
        PumpBudget::until_network_idle(Duration::from_secs(1), Duration::from_millis(10)),
    );
    assert_eq!(outcome, PumpOutcome::NetworkIdle);
    assert!(*done.borrow());
    assert_eq!(executor.pending_network(), 0);
}
