//! Event loop for Thalora pages.
//!
//! [`ThaloraJobExecutor`] is a Boa [`JobExecutor`] that keeps microtasks
//! (promise reactions, `queueMicrotask`), timers and async host jobs (fetch,
//! XHR, module loading) in separate queues, and [`ThaloraJobExecutor::pump`]
//! drives them HTML-style under a time budget:
//!
//! 1. microtask checkpoint
//! 2. start queued async jobs and poll the ones in flight
//! 3. run one due timer, followed by a microtask checkpoint
//! 4. stop when the requested [`WaitMode`] is satisfied or the budget is spent
//!
//! Unlike Boa's `SimpleJobExecutor`, [`JobExecutor::run_jobs`] never waits for
//! timers that are not yet due, so a live `setInterval` cannot hang it.
//!
//! Async jobs borrow the context for as long as they run, so they only make
//! progress inside a `pump`/`run_jobs` call; jobs still in flight when a pump's
//! budget runs out are dropped (their promises stay pending). A `NoWait` pump
//! therefore doesn't start async jobs at all — it leaves them queued for the
//! next pump that is allowed to wait.

use boa_engine::context::time::{JsDuration, JsInstant};
use boa_engine::job::{
    GenericJob, Job, JobExecutor, NativeAsyncJob, NativeJob, PromiseJob, TimeoutJob,
};
use boa_engine::{Context, JsError, JsResult, JsValue};
use futures_util::stream::{FuturesUnordered, StreamExt};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;
use std::task::Poll;
use std::time::Duration;

#[cfg(test)]
mod tests;

/// Microtasks run per checkpoint before the queue is considered runaway.
const MAX_MICROTASKS_PER_CHECKPOINT: usize = 100_000;

/// Default wall-clock limit for a single task (timer callback, microtask).
pub const DEFAULT_TASK_SLICE: Duration = Duration::from_millis(500);

/// Upper bound for `run_jobs`, which must settle async jobs (e.g. module
/// loading) but must never hang the caller.
const RUN_JOBS_LIMIT: Duration = Duration::from_secs(30);

/// How often in-flight async jobs are polled while waiting.
const POLL_INTERVAL: Duration = Duration::from_millis(5);

/// When a [`ThaloraJobExecutor::pump`] call may return.
pub enum WaitMode {
    /// Run whatever is ready right now; don't wait and don't start async jobs.
    NoWait,
    /// Run until no microtasks, due timers or async jobs remain and no
    /// one-shot timer is due within the budget (intervals are ignored).
    UntilIdle,
    /// Run until no async jobs are queued or in flight, none has started or
    /// completed for `quiet`, nothing else is ready, and no one-shot timer is
    /// due within `quiet`.
    UntilNetworkIdle { quiet: Duration },
    /// Run until the predicate returns true.
    Until(Box<dyn FnMut(&mut Context) -> bool>),
    /// Run microtasks, due timers and async jobs to completion without
    /// waiting for timers that aren't due yet (the `run_jobs` contract).
    UntilSettled,
}

/// Limits for a [`ThaloraJobExecutor::pump`] call.
pub struct PumpBudget {
    /// Maximum wall-clock time to spend.
    pub wall: Duration,
    /// When to return early.
    pub wait: WaitMode,
}

impl PumpBudget {
    /// Run only what is ready now.
    pub fn no_wait() -> Self {
        Self {
            wall: Duration::ZERO,
            wait: WaitMode::NoWait,
        }
    }

    /// Wait up to `wall` for the page to go idle.
    pub fn until_idle(wall: Duration) -> Self {
        Self {
            wall,
            wait: WaitMode::UntilIdle,
        }
    }

    /// Wait up to `wall` for the network to be quiet for `quiet`.
    pub fn until_network_idle(wall: Duration, quiet: Duration) -> Self {
        Self {
            wall,
            wait: WaitMode::UntilNetworkIdle { quiet },
        }
    }

    /// Wait up to `wall` for `predicate` to hold.
    pub fn until(wall: Duration, predicate: impl FnMut(&mut Context) -> bool + 'static) -> Self {
        Self {
            wall,
            wait: WaitMode::Until(Box::new(predicate)),
        }
    }
}

/// Why a [`ThaloraJobExecutor::pump`] call returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PumpOutcome {
    /// Nothing left to do (for the chosen wait mode).
    Idle,
    /// No network activity for the requested quiet period.
    NetworkIdle,
    /// The `Until` predicate returned true.
    ConditionMet,
    /// The wall-clock budget ran out first.
    BudgetExhausted,
}

enum Microtask {
    Promise(PromiseJob),
    Generic(GenericJob),
}

impl Microtask {
    fn call(self, context: &mut Context) -> JsResult<JsValue> {
        match self {
            Self::Promise(job) => job.call(context),
            Self::Generic(job) => job.call(context),
        }
    }
}

enum TimerJob {
    /// Scheduled through [`ThaloraJobExecutor::schedule_timer`]; cancellable by id.
    Native(NativeJob),
    /// Enqueued by the engine as a `Job::TimeoutJob`.
    Engine(TimeoutJob),
}

struct TimerEntry {
    id: Option<u32>,
    recurring: bool,
    job: TimerJob,
}

/// Job executor with an HTML-like event loop. See the module docs.
pub struct ThaloraJobExecutor {
    microtasks: RefCell<VecDeque<Microtask>>,
    /// Keyed by (due time, insertion sequence) so equal due times keep FIFO order.
    timers: RefCell<BTreeMap<(JsInstant, u64), TimerEntry>>,
    async_jobs: RefCell<VecDeque<NativeAsyncJob>>,
    finalization_jobs: RefCell<VecDeque<NativeAsyncJob>>,
    seq: Cell<u64>,
    in_flight: Cell<usize>,
    last_network_activity: Cell<Option<JsInstant>>,
    task_slice: Cell<Duration>,
}

impl Default for ThaloraJobExecutor {
    fn default() -> Self {
        Self {
            microtasks: RefCell::default(),
            timers: RefCell::default(),
            async_jobs: RefCell::default(),
            finalization_jobs: RefCell::default(),
            seq: Cell::new(0),
            in_flight: Cell::new(0),
            last_network_activity: Cell::new(None),
            task_slice: Cell::new(DEFAULT_TASK_SLICE),
        }
    }
}

impl std::fmt::Debug for ThaloraJobExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThaloraJobExecutor")
            .field("microtasks", &self.microtasks.borrow().len())
            .field("timers", &self.timers.borrow().len())
            .field("async_jobs", &self.async_jobs.borrow().len())
            .field("in_flight", &self.in_flight.get())
            .finish()
    }
}

impl ThaloraJobExecutor {
    /// Create an executor (wrap it in an `Rc` and pass it to
    /// `ContextBuilder::job_executor`).
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the wall-clock limit for a single task (default 500 ms). A task
    /// that exceeds it is aborted and reported; the loop continues.
    pub fn set_task_slice(&self, slice: Duration) {
        self.task_slice.set(slice);
    }

    /// Schedule `job` to run `delay_ms` after `now`. Timers with an `id` can
    /// be cancelled with [`cancel_timer`](Self::cancel_timer); `recurring`
    /// timers (intervals) don't keep [`WaitMode::UntilIdle`] waiting.
    pub fn schedule_timer(
        &self,
        now: JsInstant,
        delay_ms: u64,
        id: Option<u32>,
        recurring: bool,
        job: NativeJob,
    ) {
        let due = now + JsDuration::from_millis(delay_ms);
        self.insert_timer(due, id, recurring, TimerJob::Native(job));
    }

    /// Remove every pending timer scheduled with `id`.
    pub fn cancel_timer(&self, id: u32) {
        self.timers
            .borrow_mut()
            .retain(|_, entry| entry.id != Some(id));
    }

    /// Number of pending timers (including intervals).
    pub fn pending_timers(&self) -> usize {
        self.timers.borrow().len()
    }

    /// Number of async jobs (fetch, XHR, …) queued or in flight.
    pub fn pending_network(&self) -> usize {
        self.async_jobs.borrow().len() + self.in_flight.get()
    }

    /// Whether any work is queued (microtasks, timers or async jobs).
    pub fn has_pending_work(&self) -> bool {
        !self.microtasks.borrow().is_empty()
            || !self.timers.borrow().is_empty()
            || self.pending_network() > 0
    }

    fn insert_timer(&self, due: JsInstant, id: Option<u32>, recurring: bool, job: TimerJob) {
        let seq = self.seq.get();
        self.seq.set(seq.wrapping_add(1));
        self.timers
            .borrow_mut()
            .insert((due, seq), TimerEntry { id, recurring, job });
    }

    fn note_network_activity(&self, now: JsInstant) {
        self.last_network_activity.set(Some(now));
    }

    fn has_due_timer(&self, now: JsInstant) -> bool {
        self.timers
            .borrow()
            .keys()
            .next()
            .is_some_and(|(due, _)| *due <= now)
    }

    /// Due time of the next timer, optionally skipping recurring ones.
    fn next_timer_due(&self, include_recurring: bool) -> Option<JsInstant> {
        self.timers
            .borrow()
            .iter()
            .find(|(_, entry)| include_recurring || !entry.recurring)
            .map(|((due, _), _)| *due)
    }

    /// Run `f` with the per-task execution deadline set (native only).
    fn with_task_slice<T>(
        &self,
        context: &mut Context,
        f: impl FnOnce(&mut Context) -> JsResult<T>,
    ) -> JsResult<T> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            context
                .runtime_limits_mut()
                .set_execution_deadline(std::time::Instant::now() + self.task_slice.get());
            let result = f(context);
            context.runtime_limits_mut().clear_execution_deadline();
            result
        }
        #[cfg(target_arch = "wasm32")]
        {
            f(context)
        }
    }

    /// Drain the microtask queue, including microtasks queued while draining.
    fn run_microtask_checkpoint(&self, context: &mut Context) {
        let mut ran = 0usize;
        loop {
            // Release the borrow before running: the task may queue more.
            let task = self.microtasks.borrow_mut().pop_front();
            let Some(task) = task else { break };
            if let Err(err) = self.with_task_slice(context, |ctx| task.call(ctx)) {
                report_error("microtask", &err);
            }
            ran += 1;
            if ran >= MAX_MICROTASKS_PER_CHECKPOINT {
                let dropped = self.microtasks.borrow().len();
                eprintln!(
                    "⚠️ event loop: microtask limit ({MAX_MICROTASKS_PER_CHECKPOINT}) reached, dropping {dropped} queued microtasks"
                );
                self.microtasks.borrow_mut().clear();
                break;
            }
        }
        context.clear_kept_objects();
    }

    /// Run the earliest timer if it is due. Returns whether one ran.
    fn run_one_due_timer(&self, context: &mut Context, now: JsInstant) -> bool {
        let entry = {
            let mut timers = self.timers.borrow_mut();
            match timers.keys().next().copied() {
                Some(key) if key.0 <= now => timers.remove(&key),
                _ => None,
            }
        };
        let Some(entry) = entry else { return false };
        let result = match entry.job {
            TimerJob::Native(job) => self.with_task_slice(context, |ctx| job.call(ctx)),
            TimerJob::Engine(job) => {
                if job.cancelled() {
                    return true;
                }
                self.with_task_slice(context, |ctx| job.call(ctx))
            }
        };
        if let Err(err) = result {
            report_error("timer", &err);
        }
        true
    }

    /// Drive the event loop until `budget` says to stop. See [`WaitMode`].
    pub fn pump(&self, context: &mut Context, budget: PumpBudget) -> PumpOutcome {
        let PumpBudget { wall, mut wait } = budget;
        let start_async = !matches!(wait, WaitMode::NoWait);
        let deadline = context.clock().now() + JsDuration::from(wall);

        let ctx = RefCell::new(context);
        // Declared after `ctx` so it is dropped first: in-flight futures borrow it.
        let mut in_flight = FuturesUnordered::new();

        let outcome = loop {
            self.run_microtask_checkpoint(&mut ctx.borrow_mut());

            // Start queued async jobs (only when this pump may wait for them).
            if start_async {
                loop {
                    let job = self.async_jobs.borrow_mut().pop_front();
                    let Some(job) = job else { break };
                    in_flight.push(job.call(&ctx));
                }
                loop {
                    let job = self.finalization_jobs.borrow_mut().pop_front();
                    let Some(job) = job else { break };
                    in_flight.push(job.call(&ctx));
                }
            }

            // Poll in-flight async jobs without blocking.
            let mut progressed = false;
            if !in_flight.is_empty() {
                let mut task_cx =
                    std::task::Context::from_waker(futures_util::task::noop_waker_ref());
                while let Poll::Ready(Some(result)) = in_flight.poll_next_unpin(&mut task_cx) {
                    progressed = true;
                    let now = ctx.borrow().clock().now();
                    self.note_network_activity(now);
                    if let Err(err) = result {
                        report_error("async job", &err);
                    }
                }
            }
            self.in_flight.set(in_flight.len());
            if progressed {
                self.run_microtask_checkpoint(&mut ctx.borrow_mut());
            }

            // Run one due timer task, then a microtask checkpoint.
            let now = ctx.borrow().clock().now();
            let ran_timer = self.run_one_due_timer(&mut ctx.borrow_mut(), now);
            if ran_timer {
                self.run_microtask_checkpoint(&mut ctx.borrow_mut());
            }

            // Decide whether to stop.
            let now = ctx.borrow().clock().now();
            let has_ready = !self.microtasks.borrow().is_empty()
                || self.has_due_timer(now)
                || (start_async && !self.async_jobs.borrow().is_empty());
            let network_busy = !in_flight.is_empty() || !self.async_jobs.borrow().is_empty();
            let mut poll_soon = !in_flight.is_empty();

            match &mut wait {
                WaitMode::NoWait => {
                    if !has_ready {
                        break PumpOutcome::Idle;
                    }
                }
                WaitMode::UntilIdle => {
                    let timer_soon = self
                        .next_timer_due(false)
                        .is_some_and(|due| due <= deadline);
                    if !has_ready && !network_busy && !timer_soon {
                        break PumpOutcome::Idle;
                    }
                }
                WaitMode::UntilSettled => {
                    if !has_ready && !network_busy {
                        break PumpOutcome::Idle;
                    }
                }
                WaitMode::UntilNetworkIdle { quiet } => {
                    let quiet_enough = self
                        .last_network_activity
                        .get()
                        .is_none_or(|last| now - last >= JsDuration::from(*quiet));
                    // Also wait for one-shot timers due within the quiet
                    // window (e.g. a short setTimeout that renders content).
                    let timer_soon = self
                        .next_timer_due(false)
                        .is_some_and(|due| due <= now + JsDuration::from(*quiet));
                    if !has_ready && !network_busy && quiet_enough && !timer_soon {
                        break PumpOutcome::NetworkIdle;
                    }
                    poll_soon = true;
                }
                WaitMode::Until(predicate) => {
                    if predicate(&mut ctx.borrow_mut()) {
                        break PumpOutcome::ConditionMet;
                    }
                    poll_soon = true;
                }
            }

            if now >= deadline {
                break PumpOutcome::BudgetExhausted;
            }
            if has_ready || progressed || ran_timer {
                continue;
            }

            // Nothing ready: sleep until the next timer, the deadline, or the
            // next poll of in-flight jobs / the wait condition.
            let mut sleep_for = deadline - now;
            if let Some(due) = self.next_timer_due(true) {
                sleep_for = sleep_for.min(due - now);
            }
            if poll_soon {
                sleep_for = sleep_for.min(JsDuration::from(POLL_INTERVAL));
            }
            if !sleep(sleep_for) {
                break PumpOutcome::BudgetExhausted;
            }
        };

        if !in_flight.is_empty() {
            eprintln!(
                "⚠️ event loop: budget exhausted with {} async job(s) in flight; they were cancelled",
                in_flight.len()
            );
        }
        drop(in_flight);
        self.in_flight.set(0);
        outcome
    }
}

impl JobExecutor for ThaloraJobExecutor {
    fn enqueue_job(self: Rc<Self>, job: Job, context: &mut Context) {
        match job {
            Job::PromiseJob(job) => self
                .microtasks
                .borrow_mut()
                .push_back(Microtask::Promise(job)),
            Job::GenericJob(job) => self
                .microtasks
                .borrow_mut()
                .push_back(Microtask::Generic(job)),
            Job::AsyncJob(job) => {
                self.async_jobs.borrow_mut().push_back(job);
                self.note_network_activity(context.clock().now());
            }
            Job::TimeoutJob(job) => {
                let due = context.clock().now() + job.timeout();
                let recurring = job.is_recurring();
                self.insert_timer(due, None, recurring, TimerJob::Engine(job));
            }
            Job::FinalizationRegistryCleanupJob(job) => {
                self.finalization_jobs.borrow_mut().push_back(job);
            }
            _ => {}
        }
    }

    fn run_jobs(self: Rc<Self>, context: &mut Context) -> JsResult<()> {
        self.pump(
            context,
            PumpBudget {
                wall: RUN_JOBS_LIMIT,
                wait: WaitMode::UntilSettled,
            },
        );
        Ok(())
    }
}

fn report_error(kind: &str, err: &JsError) {
    eprintln!("⚠️ Uncaught error in {kind}: {err}");
}

/// Sleep for `duration`; returns false where blocking isn't possible.
#[cfg(not(target_arch = "wasm32"))]
fn sleep(duration: JsDuration) -> bool {
    std::thread::sleep(Duration::from_millis(duration.as_millis().max(1)));
    true
}

#[cfg(target_arch = "wasm32")]
fn sleep(_duration: JsDuration) -> bool {
    false
}
