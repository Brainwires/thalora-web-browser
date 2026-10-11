//! One long-lived OS thread per browser.
//!
//! Boa's garbage collector is thread-local: objects belong to the thread that
//! allocated them, and are freed when that thread exits. A browser (its
//! renderer and JS context) must therefore be created, used and dropped on a
//! single thread. [`BrowserThread`] owns a [`HeadlessWebBrowser`] on a
//! dedicated thread and runs work on it as jobs sent over a channel, so any
//! caller — async MCP handlers on any runtime, FFI calls from .NET threads —
//! can drive the browser without touching it from the wrong thread.
//!
//! Jobs run one at a time, to completion (a job may hold the browser's
//! `MutexGuard` across `.await`). Between jobs the thread pumps the page's
//! event loop so timers keep running.

use super::HeadlessWebBrowser;
use crate::engine::engine_trait::EngineType;
use anyhow::{Result, anyhow};
use futures::FutureExt;
use futures::future::LocalBoxFuture;
use std::rc::Rc;
use std::sync::Mutex;
use std::thread::{JoinHandle, ThreadId};
use std::time::Duration;

/// The browser as seen by jobs running on its thread.
pub type BrowserHandle = Rc<Mutex<HeadlessWebBrowser>>;

type Job = Box<dyn FnOnce(BrowserHandle) -> LocalBoxFuture<'static, ()> + Send>;

enum Message {
    Run(Job),
    Stop,
}

struct Inner {
    tx: tokio::sync::mpsc::UnboundedSender<Message>,
    thread_id: ThreadId,
    join: Mutex<Option<JoinHandle<()>>>,
    stopped: Mutex<Option<tokio::sync::oneshot::Receiver<()>>>,
}

/// Handle to a browser living on its own thread. Cheap to clone; `Send` and
/// `Sync`. The thread exits when [`shutdown`](Self::shutdown) is called or
/// the last handle is dropped.
#[derive(Clone)]
pub struct BrowserThread {
    inner: std::sync::Arc<Inner>,
}

impl std::fmt::Debug for BrowserThread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrowserThread")
            .field("thread", &self.inner.thread_id)
            .finish()
    }
}

/// Stack size for browser threads (deep JS recursion, layout).
const STACK_SIZE: usize = 16 * 1024 * 1024;

/// How often an idle browser thread runs ready timers/microtasks
/// (`THALORA_IDLE_PUMP_MS`, default 50; 0 disables).
fn idle_tick() -> Option<Duration> {
    let ms = std::env::var("THALORA_IDLE_PUMP_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(50);
    (ms > 0).then(|| Duration::from_millis(ms))
}

impl BrowserThread {
    /// Start a thread named `thalora-<name>` that creates and owns a browser.
    pub fn spawn(name: &str, engine: EngineType) -> Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let (stopped_tx, stopped_rx) = tokio::sync::oneshot::channel();
        let handle = std::thread::Builder::new()
            .name(format!("thalora-{name}"))
            .stack_size(STACK_SIZE)
            .spawn(move || {
                super::core::mark_owner_thread();
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(e) => {
                        eprintln!("⚠️ browser thread: failed to start runtime: {e}");
                        return;
                    }
                };
                let local = tokio::task::LocalSet::new();
                local.block_on(&runtime, run_loop(rx, engine));
                let _ = stopped_tx.send(());
            })
            .map_err(|e| anyhow!("failed to spawn browser thread: {e}"))?;
        Ok(Self {
            inner: std::sync::Arc::new(Inner {
                tx,
                thread_id: handle.thread().id(),
                join: Mutex::new(Some(handle)),
                stopped: Mutex::new(Some(stopped_rx)),
            }),
        })
    }

    /// Spawn a browser thread, run one job on it, and shut it down.
    pub async fn run_once<R, F>(name: &str, engine: EngineType, f: F) -> Result<R>
    where
        R: Send + 'static,
        F: FnOnce(BrowserHandle) -> LocalBoxFuture<'static, R> + Send + 'static,
    {
        let thread = Self::spawn(name, engine)?;
        let result = thread.call(f).await;
        thread.shutdown().await;
        result
    }

    /// Whether the caller is running on this browser's thread.
    pub fn is_current_thread(&self) -> bool {
        std::thread::current().id() == self.inner.thread_id
    }

    fn wrap<R, F>(f: F, reply: impl FnOnce(Option<R>) + Send + 'static) -> Job
    where
        R: Send + 'static,
        F: FnOnce(BrowserHandle) -> LocalBoxFuture<'static, R> + Send + 'static,
    {
        Box::new(move |browser| {
            async move {
                // Call `f` inside the guarded future so a panic while
                // building the job's future is caught too.
                let result = std::panic::AssertUnwindSafe(async move { f(browser).await })
                    .catch_unwind()
                    .await;
                reply(result.ok());
            }
            .boxed_local()
        })
    }

    /// Run `f` with the browser on its thread and return its result.
    pub async fn call<R, F>(&self, f: F) -> Result<R>
    where
        R: Send + 'static,
        F: FnOnce(BrowserHandle) -> LocalBoxFuture<'static, R> + Send + 'static,
    {
        if self.is_current_thread() {
            return Err(anyhow!(
                "BrowserThread::call from its own thread would deadlock"
            ));
        }
        let (tx, rx) = tokio::sync::oneshot::channel();
        let job = Self::wrap(f, move |result| {
            let _ = tx.send(result);
        });
        self.inner
            .tx
            .send(Message::Run(job))
            .map_err(|_| anyhow!("browser thread has stopped"))?;
        match rx.await {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(anyhow!("browser job panicked")),
            Err(_) => Err(anyhow!("browser thread stopped before replying")),
        }
    }

    /// Blocking version of [`call`](Self::call) for synchronous callers
    /// (FFI). Must not be called from inside an async task.
    pub fn call_blocking<R, F>(&self, f: F) -> Result<R>
    where
        R: Send + 'static,
        F: FnOnce(BrowserHandle) -> LocalBoxFuture<'static, R> + Send + 'static,
    {
        if self.is_current_thread() {
            return Err(anyhow!(
                "BrowserThread::call_blocking from its own thread would deadlock"
            ));
        }
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let job = Self::wrap(f, move |result| {
            let _ = tx.send(result);
        });
        self.inner
            .tx
            .send(Message::Run(job))
            .map_err(|_| anyhow!("browser thread has stopped"))?;
        match rx.recv() {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(anyhow!("browser job panicked")),
            Err(_) => Err(anyhow!("browser thread stopped before replying")),
        }
    }

    /// Stop the thread after queued jobs finish and wait for it to exit
    /// (the browser is dropped on its own thread).
    pub async fn shutdown(self) {
        let _ = self.inner.tx.send(Message::Stop);
        if self.is_current_thread() {
            return;
        }
        let stopped = self.inner.stopped.lock().ok().and_then(|mut s| s.take());
        if let Some(stopped) = stopped {
            let _ = stopped.await;
        }
    }

    /// Blocking version of [`shutdown`](Self::shutdown).
    pub fn shutdown_blocking(self) {
        let _ = self.inner.tx.send(Message::Stop);
        if self.is_current_thread() {
            return;
        }
        let join = self.inner.join.lock().ok().and_then(|mut j| j.take());
        if let Some(join) = join {
            let _ = join.join();
        }
    }
}

async fn run_loop(mut rx: tokio::sync::mpsc::UnboundedReceiver<Message>, engine: EngineType) {
    let browser = HeadlessWebBrowser::new_with_engine(engine);
    let tick = idle_tick();
    loop {
        let message = match tick {
            Some(tick) => {
                tokio::select! {
                    message = rx.recv() => message,
                    _ = tokio::time::sleep(tick) => {
                        // Keep page timers running between jobs
                        if let Ok(mut b) = browser.try_lock() {
                            b.pump_event_loop(
                                thalora_browser_apis::event_loop::PumpBudget::no_wait(),
                            );
                        }
                        continue;
                    }
                }
            }
            None => rx.recv().await,
        };
        match message {
            Some(Message::Run(job)) => job(browser.clone()).await,
            Some(Message::Stop) | None => break,
        }
    }
    // Dropped here, on the thread that created it
    drop(browser);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_marker(thread: &BrowserThread, value: &str) {
        let value = value.to_string();
        thread
            .call_blocking(move |b| {
                async move {
                    b.lock()
                        .unwrap()
                        .get_storage_mut()
                        .session_storage
                        .insert("marker".to_string(), value);
                }
                .boxed_local()
            })
            .unwrap();
    }

    fn get_marker(thread: &BrowserThread) -> Option<String> {
        thread
            .call_blocking(|b| {
                async move {
                    b.lock()
                        .unwrap()
                        .get_storage_mut()
                        .session_storage
                        .get("marker")
                        .cloned()
                }
                .boxed_local()
            })
            .unwrap()
    }

    #[test]
    fn calls_run_on_one_browser_and_thread() {
        let thread = BrowserThread::spawn("test", EngineType::Boa).unwrap();
        assert_eq!(get_marker(&thread), None);
        set_marker(&thread, "one");
        assert_eq!(get_marker(&thread).as_deref(), Some("one"));

        let ids: Vec<ThreadId> = (0..5)
            .map(|_| {
                thread
                    .call_blocking(|_| async { std::thread::current().id() }.boxed_local())
                    .unwrap()
            })
            .collect();
        assert!(ids.iter().all(|id| *id == thread.inner.thread_id));
        thread.shutdown_blocking();
    }

    #[test]
    fn a_panicking_job_does_not_kill_the_thread() {
        let thread = BrowserThread::spawn("test-panic", EngineType::Boa).unwrap();
        let result: Result<()> = thread.call_blocking(|_| async { panic!("boom") }.boxed_local());
        assert!(result.is_err());
        set_marker(&thread, "still alive");
        assert_eq!(get_marker(&thread).as_deref(), Some("still alive"));
        thread.shutdown_blocking();
    }

    #[test]
    fn reentrant_blocking_call_is_refused() {
        let thread = BrowserThread::spawn("test-reentrant", EngineType::Boa).unwrap();
        let inner = thread.clone();
        let nested = thread
            .call_blocking(move |_| {
                async move { inner.call_blocking(|_| async {}.boxed_local()).is_err() }
                    .boxed_local()
            })
            .unwrap();
        assert!(nested, "re-entrant call must error instead of deadlocking");
        thread.shutdown_blocking();
    }

    #[test]
    fn call_works_from_a_current_thread_runtime() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let thread = BrowserThread::spawn("test-async", EngineType::Boa).unwrap();
            let url = thread
                .call(|b| async move { b.lock().unwrap().get_current_url() }.boxed_local())
                .await
                .unwrap();
            assert_eq!(url, None);
            thread.shutdown().await;
        });
    }

    /// Old renderers are dropped on the owner thread (not leaked), and the
    /// browser keeps working after many resets.
    #[test]
    fn renderer_resets_drop_old_renderers_and_js_still_runs() {
        if std::env::var("THALORA_LEAK_RENDERERS").is_ok() {
            return;
        }
        let thread = BrowserThread::spawn("test-resets", EngineType::Boa).unwrap();
        let before =
            super::super::core::RENDERERS_DROPPED.load(std::sync::atomic::Ordering::Relaxed);
        let result = thread
            .call_blocking(|b| {
                async move {
                    let mut guard = b.lock().unwrap();
                    for _ in 0..20 {
                        guard.reset_renderer();
                    }
                    guard.execute_javascript("6 * 7").await
                }
                .boxed_local()
            })
            .unwrap()
            .unwrap();
        assert!(result.contains("42"), "unexpected result: {result}");
        let after =
            super::super::core::RENDERERS_DROPPED.load(std::sync::atomic::Ordering::Relaxed);
        assert!(
            after - before >= 20,
            "only {} renderers dropped",
            after - before
        );
        thread.shutdown_blocking();
    }

    /// Resident set size in KiB (Linux only).
    fn rss_kib() -> Option<u64> {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        let line = status.lines().find(|l| l.starts_with("VmRSS:"))?;
        line.split_whitespace().nth(1)?.parse().ok()
    }

    /// Soak test: memory stays bounded across many page loads.
    /// Run with `cargo test --lib renderer_soak -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn renderer_soak_memory_is_bounded() {
        let thread = BrowserThread::spawn("test-soak", EngineType::Boa).unwrap();
        let cycle = |n: usize| {
            thread
                .call_blocking(move |b| {
                    async move {
                        let mut guard = b.lock().unwrap();
                        for i in 0..n {
                            guard.reset_renderer();
                            let script = format!(
                                "var a = []; for (var j = 0; j < 20000; j++) a.push({{i: {i}, j}}); a.length"
                            );
                            let _ = guard.execute_javascript(&script).await;
                        }
                    }
                    .boxed_local()
                })
                .unwrap()
        };
        cycle(20);
        let Some(baseline) = rss_kib() else {
            return;
        };
        cycle(200);
        let grown = rss_kib().unwrap_or(baseline).saturating_sub(baseline);
        eprintln!("RSS grew {grown} KiB over 200 renderer cycles");
        // Leaking every renderer costs 5-15 MB each (1-3 GB here)
        assert!(grown < 300 * 1024, "RSS grew {grown} KiB");
        thread.shutdown_blocking();
    }

    #[test]
    fn calls_after_shutdown_fail_cleanly() {
        let thread = BrowserThread::spawn("test-stop", EngineType::Boa).unwrap();
        let other = thread.clone();
        thread.shutdown_blocking();
        assert!(other.call_blocking(|_| async {}.boxed_local()).is_err());
    }
}
