//! One long-lived OS thread per browser.
//!
//! A browser is `!Send` (it is shared as `Rc<Mutex<_>>` and held across
//! `.await`), and a JavaScript engine's heap is bound to the thread that
//! created it, so a browser must be created, used and dropped on a single
//! thread. [`BrowserThread`] owns a [`HeadlessWebBrowser`] on a dedicated
//! thread and runs work on it as jobs sent over a channel, so any caller —
//! async MCP handlers on any runtime, FFI calls from other threads — can
//! drive the browser without touching it from the wrong thread.
//!
//! Jobs run one at a time, to completion (a job may hold the browser's
//! `MutexGuard` across `.await`).
//!
//! # Page JavaScript
//!
//! The thread is started through the V8 root thread
//! ([`spawn_js_thread`](super::page_js)), so the pages its browser loads may
//! run JavaScript. Between jobs the thread keeps the current page's event
//! loop running (timers, fetches, promise jobs), so a page goes on working
//! between two tool calls as it does in a browser:
//!
//! - While the page has pending work, the loop alternates between waiting
//!   for a job and pumping the page in slices; an arriving job interrupts
//!   the slice between two page tasks.
//! - A page with nothing pending costs nothing: the loop only waits for
//!   jobs.
//! - Pumping stops [`BACKGROUND_PUMP`] after the last job (a page with a
//!   repeating timer would otherwise keep a core busy for as long as the
//!   session exists). The next job resumes it.
//! - A navigation the page asks for meanwhile (`location.href = …` from a
//!   timer) is performed between jobs, to completion.

// The browser is only touched on this thread, one job (or one pump slice) at
// a time, so holding its MutexGuard across .await cannot contend or deadlock.
#![allow(clippy::await_holding_lock)]

use super::HeadlessWebBrowser;
use anyhow::{Result, anyhow};
use futures::FutureExt;
use futures::future::LocalBoxFuture;
use std::rc::Rc;
use std::sync::Mutex;
use std::thread::{JoinHandle, ThreadId};

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

/// Stack size for browser threads (deeply nested layout).
const STACK_SIZE: usize = 16 * 1024 * 1024;

/// How long after the last job the current page's event loop keeps being
/// pumped in the background.
const BACKGROUND_PUMP: std::time::Duration = std::time::Duration::from_secs(30);

/// One slice of background pumping. A job that arrives interrupts it, so
/// this is not a latency; it bounds how often the loop re-arms.
const PUMP_SLICE: std::time::Duration = std::time::Duration::from_millis(250);

impl BrowserThread {
    /// Start a thread named `thalora-<name>` that creates and owns a browser.
    pub fn spawn(name: &str) -> Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let (stopped_tx, stopped_rx) = tokio::sync::oneshot::channel();
        let builder = std::thread::Builder::new()
            .name(format!("thalora-{name}"))
            .stack_size(STACK_SIZE);
        // Started by the V8 root thread, so its pages may run JavaScript
        let handle = super::page_js::spawn_js_thread(builder, move || {
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
            local.block_on(&runtime, run_loop(rx));
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
    pub async fn run_once<R, F>(name: &str, f: F) -> Result<R>
    where
        R: Send + 'static,
        F: FnOnce(BrowserHandle) -> LocalBoxFuture<'static, R> + Send + 'static,
    {
        let thread = Self::spawn(name)?;
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

async fn run_loop(mut rx: tokio::sync::mpsc::UnboundedReceiver<Message>) {
    let browser = HeadlessWebBrowser::new();
    // Until when the current page is pumped between jobs (see the module
    // docs); `None` while it has nothing pending.
    let mut pump_until: Option<std::time::Instant> = None;
    loop {
        let pumping = pump_until.is_some_and(|until| std::time::Instant::now() < until);
        let message = if pumping {
            tokio::select! {
                biased;
                message = rx.recv() => Some(message),
                pending = pump_slice(&browser) => {
                    if !pending {
                        pump_until = None;
                    }
                    None
                }
            }
        } else {
            Some(rx.recv().await)
        };
        match message {
            // A pump slice ended. Not inside the `select!`: a navigation
            // must not be dropped half-way when a job arrives.
            None => follow_page_navigation(&browser).await,
            Some(Some(Message::Run(job))) => {
                job(browser.clone()).await;
                pump_until = Some(std::time::Instant::now() + BACKGROUND_PUMP);
            }
            Some(Some(Message::Stop)) | Some(None) => break,
        }
    }
    // Dropped here, on the thread that created it
    drop(browser);
}

/// Pump the current page's event loop for one slice. Returns whether the
/// page still has pending work. Cancel-safe.
async fn pump_slice(browser: &BrowserHandle) -> bool {
    // A poisoned lock (a job panicked while holding it): leave the page alone
    let Ok(mut guard) = browser.lock() else {
        return false;
    };
    guard.pump_background(PUMP_SLICE).await || guard.has_script_navigation()
}

/// Perform the navigation the page asked for while it was pumped, if any.
async fn follow_page_navigation(browser: &BrowserHandle) {
    let Ok(mut guard) = browser.lock() else {
        return;
    };
    if guard.has_script_navigation() {
        guard.follow_script_navigations().await;
    }
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
        let thread = BrowserThread::spawn("test").unwrap();
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
        let thread = BrowserThread::spawn("test-panic").unwrap();
        let result: Result<()> = thread.call_blocking(|_| async { panic!("boom") }.boxed_local());
        assert!(result.is_err());
        set_marker(&thread, "still alive");
        assert_eq!(get_marker(&thread).as_deref(), Some("still alive"));
        thread.shutdown_blocking();
    }

    #[test]
    fn reentrant_blocking_call_is_refused() {
        let thread = BrowserThread::spawn("test-reentrant").unwrap();
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
            let thread = BrowserThread::spawn("test-async").unwrap();
            let url = thread
                .call(|b| async move { b.lock().unwrap().get_current_url() }.boxed_local())
                .await
                .unwrap();
            assert_eq!(url, None);
            thread.shutdown().await;
        });
    }

    #[test]
    fn calls_after_shutdown_fail_cleanly() {
        let thread = BrowserThread::spawn("test-stop").unwrap();
        let other = thread.clone();
        thread.shutdown_blocking();
        assert!(other.call_blocking(|_| async {}.boxed_local()).is_err());
    }
}
