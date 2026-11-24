//! Page JavaScript: the V8 page runtime of the Brainwires deno fork
//! (`deno_web_page::PageRuntime`), and what Thalora adds around it.
//!
//! - [`JsSettings`]: whether pages run script and with which limits.
//! - The **V8 root thread** ([`spawn_js_thread`]): the thread that
//!   initializes the V8 platform and spawns every thread that runs page
//!   JavaScript (the `BrowserThread`s).
//! - The **watchdog** ([`arm`]): ends a script, an evaluation or an event
//!   loop task that runs longer than the configured limit.
//! - [`JsPage`]: one page's runtime. Every call into JavaScript is made
//!   under the watchdog and reports its failure as a plain message.
//!
//! # Threads
//!
//! A `PageRuntime` is `!Send`: it is created, used and dropped on the
//! browser's thread. V8 additionally wants every thread that runs an isolate
//! to descend from the thread that initialized the platform (on x86-64 Linux
//! the JIT's memory protection key is inherited at thread creation; another
//! thread faults in JIT code). Nothing guarantees that for a library (test
//! threads are siblings), so the platform is initialized on a dedicated root
//! thread and browser threads are spawned *by* that thread. A page loaded on
//! any other thread stays static (see [`current_thread_can_run_js`]).
//!
//! Only one `PageRuntime` exists per thread at a time: V8 isolates are
//! entered on creation and must be dropped in reverse order of creation, so
//! the old page's runtime is dropped before the next one is created.

use std::cell::Cell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use deno_core::v8;
use deno_web_page::{
    ClickOutcome, ConsoleLevel, FormState, Idle, NavigationRequest, PageError, PageOptions,
    PageRuntime,
};
use thalora_dom::{Document, NodeId};

/// Default limit for one script, evaluation or event loop task.
const DEFAULT_SCRIPT_TIMEOUT: Duration = Duration::from_secs(10);
/// Default V8 heap limit of a page.
const DEFAULT_HEAP_LIMIT_MB: usize = 512;

/// Whether pages run JavaScript, and with which limits. Read from the
/// environment by [`JsSettings::from_env`]; a browser's copy can be changed
/// with `HeadlessWebBrowser::js_settings_mut`.
#[derive(Debug, Clone)]
pub struct JsSettings {
    /// Run page scripts. Off: pages are static HTML, as fetched
    /// (`THALORA_JS=0`).
    pub enabled: bool,
    /// The longest a single script, evaluation or event loop task may run
    /// before the watchdog ends it (`THALORA_JS_SCRIPT_TIMEOUT_MS`,
    /// default 10000).
    pub script_timeout: Duration,
    /// The V8 heap limit of a page; reaching it ends the page's JavaScript,
    /// not the process (`THALORA_JS_HEAP_LIMIT_MB`, default 512, 0 = V8's
    /// default).
    pub heap_limit_bytes: Option<usize>,
    /// Where `localStorage` lives (one subdirectory per origin), on the real
    /// file system: `<profile>/storage`, the profile being
    /// `THALORA_PROFILE_DIR` or `~/.thalora/profile`. `None`: no
    /// `localStorage`.
    pub storage_dir: Option<PathBuf>,
    /// Seed of `crypto.getRandomValues` / `crypto.randomUUID`, for
    /// deterministic runs (`THALORA_JS_SEED`).
    pub seed: Option<u64>,
}

fn env_number<T: std::str::FromStr>(name: &str) -> Option<T> {
    std::env::var(name).ok()?.trim().parse().ok()
}

impl JsSettings {
    /// The settings the environment asks for (see the fields).
    pub fn from_env() -> Self {
        let enabled = !std::env::var("THALORA_JS").is_ok_and(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "0" | "false" | "off" | "no"
            )
        });
        let script_timeout = env_number::<u64>("THALORA_JS_SCRIPT_TIMEOUT_MS")
            .filter(|ms| *ms > 0)
            .map(Duration::from_millis)
            .unwrap_or(DEFAULT_SCRIPT_TIMEOUT);
        let heap_limit_mb =
            env_number::<usize>("THALORA_JS_HEAP_LIMIT_MB").unwrap_or(DEFAULT_HEAP_LIMIT_MB);
        let heap_limit_bytes = (heap_limit_mb > 0).then(|| heap_limit_mb.saturating_mul(1 << 20));
        let profile = std::env::var_os("THALORA_PROFILE_DIR")
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|home| home.join(".thalora").join("profile")));
        Self {
            enabled,
            script_timeout,
            heap_limit_bytes,
            storage_dir: profile.map(|profile| profile.join("storage")),
            seed: env_number("THALORA_JS_SEED"),
        }
    }

    /// The options of a page runtime for `html` at `url`.
    pub(crate) fn page_options(&self, url: url::Url, html: &str) -> PageOptions {
        let mut options = PageOptions::new(url, html);
        options.user_agent = super::USER_AGENT.to_string();
        // Which hosts a page may talk to is not restricted by name; the
        // addresses they resolve to are (the same classes the navigation
        // SSRF guard refuses), on every connection and redirect.
        options.net_allow = vec![deno_web_page::NET_ALLOW_ANY.to_string()];
        options.deny_addresses = if crate::engine::security::loopback_override_enabled() {
            // The test override lifts loopback only, like the SSRF guard.
            Some(
                deno_web_page::DEFAULT_DENY_ADDRESSES
                    .iter()
                    .filter(|class| **class != "loopback")
                    .map(|class| class.to_string())
                    .collect(),
            )
        } else {
            None
        };
        // Host-owned state on the real disk (std::fs, never the VFS). No
        // directory, no localStorage.
        options.storage_dir = self
            .storage_dir
            .clone()
            .filter(|dir| std::fs::create_dir_all(dir).is_ok());
        options.heap_limit_bytes = self.heap_limit_bytes;
        options.seed = self.seed;
        options
    }
}

impl Default for JsSettings {
    fn default() -> Self {
        Self::from_env()
    }
}

// ----- the V8 root thread -----------------------------------------------------

type SpawnRequest = Box<dyn FnOnce() + Send>;

thread_local! {
    static CAN_RUN_JS: Cell<bool> = const { Cell::new(false) };
}

/// Whether the calling thread may create a page runtime: it was started by
/// [`spawn_js_thread`], so it descends from the thread that initialized V8.
pub(crate) fn current_thread_can_run_js() -> bool {
    CAN_RUN_JS.with(Cell::get)
}

fn v8_root() -> Option<&'static std::sync::mpsc::Sender<SpawnRequest>> {
    static ROOT: OnceLock<Option<std::sync::mpsc::Sender<SpawnRequest>>> = OnceLock::new();
    ROOT.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel::<SpawnRequest>();
        std::thread::Builder::new()
            .name("thalora-v8-root".to_string())
            .spawn(move || {
                // Once per process, before any isolate exists.
                deno_core::JsRuntime::init_platform(None);
                while let Ok(request) = rx.recv() {
                    request();
                }
            })
            .ok()
            .map(|_| tx)
    })
    .as_ref()
}

/// Spawn a thread that may run page JavaScript: it is created by the V8 root
/// thread (see the module docs). If the root thread cannot be started the
/// thread is spawned directly and its pages stay static.
pub(crate) fn spawn_js_thread(
    builder: std::thread::Builder,
    body: impl FnOnce() + Send + 'static,
) -> std::io::Result<JoinHandle<()>> {
    let Some(root) = v8_root() else {
        return builder.spawn(body);
    };
    let (reply_tx, reply_rx) = std::sync::mpsc::sync_channel(1);
    let request: SpawnRequest = Box::new(move || {
        let spawned = builder.spawn(move || {
            CAN_RUN_JS.with(|flag| flag.set(true));
            body();
        });
        let _ = reply_tx.send(spawned);
    });
    root.send(request)
        .map_err(|_| std::io::Error::other("the V8 root thread has stopped"))?;
    reply_rx
        .recv()
        .map_err(|_| std::io::Error::other("the V8 root thread has stopped"))?
}

// ----- the watchdog -----------------------------------------------------------

struct Watched {
    deadline: Instant,
    isolate: v8::IsolateHandle,
    fired: Arc<AtomicBool>,
}

struct Watchdog {
    watched: parking_lot::Mutex<HashMap<u64, Watched>>,
    changed: parking_lot::Condvar,
    next_id: AtomicU64,
}

fn watchdog() -> &'static Arc<Watchdog> {
    static WATCHDOG: OnceLock<Arc<Watchdog>> = OnceLock::new();
    WATCHDOG.get_or_init(|| {
        let watchdog = Arc::new(Watchdog {
            watched: parking_lot::Mutex::new(HashMap::new()),
            changed: parking_lot::Condvar::new(),
            next_id: AtomicU64::new(0),
        });
        let shared = watchdog.clone();
        // If the thread cannot be started nothing is ever terminated; the
        // heap limit and the event loop budget still apply.
        let _ = std::thread::Builder::new()
            .name("thalora-js-watchdog".to_string())
            .spawn(move || {
                let mut watched = shared.watched.lock();
                loop {
                    let now = Instant::now();
                    watched.retain(|_, entry| {
                        if entry.deadline > now {
                            return true;
                        }
                        entry.fired.store(true, Ordering::SeqCst);
                        entry.isolate.terminate_execution();
                        false
                    });
                    match watched.values().map(|entry| entry.deadline).min() {
                        Some(deadline) => {
                            shared.changed.wait_until(&mut watched, deadline);
                        }
                        None => shared.changed.wait(&mut watched),
                    }
                }
            });
        watchdog
    })
}

/// A running JavaScript call the watchdog ends at its deadline.
struct Armed {
    id: u64,
    isolate: v8::IsolateHandle,
    fired: Arc<AtomicBool>,
}

/// Have the watchdog end the JavaScript running in `isolate` if the returned
/// guard is not disarmed within `timeout`.
fn arm(isolate: v8::IsolateHandle, timeout: Duration) -> Armed {
    let watchdog = watchdog();
    let id = watchdog.next_id.fetch_add(1, Ordering::Relaxed);
    let fired = Arc::new(AtomicBool::new(false));
    watchdog.watched.lock().insert(
        id,
        Watched {
            deadline: Instant::now() + timeout,
            isolate: isolate.clone(),
            fired: fired.clone(),
        },
    );
    watchdog.changed.notify_one();
    Armed { id, isolate, fired }
}

impl Armed {
    /// Stop watching. Returns whether the watchdog ended the call.
    fn disarm(self) -> bool {
        // Drop does the work; the flag is final once the entry is gone.
        let fired = self.fired.clone();
        drop(self);
        fired.load(Ordering::SeqCst)
    }
}

impl Drop for Armed {
    fn drop(&mut self) {
        // The watchdog fires under this lock, so after it is released the
        // entry is either gone untouched or has fired.
        watchdog().watched.lock().remove(&self.id);
        if self.fired.load(Ordering::SeqCst) {
            // The termination may have been requested just after the call
            // returned: make sure it does not hit the next one.
            self.isolate.cancel_terminate_execution();
        }
    }
}

// ----- one page's runtime -----------------------------------------------------

/// Why a call into page JavaScript failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsError {
    /// JavaScript threw and nothing caught it (the message, with the stack
    /// when there is one).
    Exception(String),
    /// The call ran longer than the script timeout and was ended.
    Timeout(Duration),
    /// The page reached its heap limit; its JavaScript is stopped for good.
    OutOfMemory,
    /// Anything else (an invalid node, a runtime failure).
    Other(String),
}

impl std::fmt::Display for JsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsError::Exception(message) => f.write_str(message),
            JsError::Timeout(limit) => write!(
                f,
                "JavaScript ran longer than {} ms and was terminated",
                limit.as_millis()
            ),
            JsError::OutOfMemory => f.write_str(
                "The page ran out of memory; its JavaScript is stopped until the next navigation",
            ),
            JsError::Other(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for JsError {}

/// What a page's console and scripts produced since the last
/// [`JsPage::drain`].
#[derive(Debug, Default)]
pub(crate) struct JsOutput {
    /// `(level, text)` of console messages and uncaught errors, in order.
    pub console: Vec<(&'static str, String)>,
    /// Navigations scripts asked for outside a click, oldest first.
    pub navigations: Vec<NavigationRequest>,
}

/// The JavaScript side of one page: a `PageRuntime`, called under the
/// watchdog.
pub(crate) struct JsPage {
    runtime: PageRuntime,
    isolate: v8::IsolateHandle,
    script_timeout: Duration,
}

impl std::fmt::Debug for JsPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsPage")
            .field("script_timeout", &self.script_timeout)
            .finish_non_exhaustive()
    }
}

impl JsPage {
    /// Boot a page runtime. Must run on a thread started by
    /// [`spawn_js_thread`], inside its tokio runtime, with no other page
    /// runtime alive on the thread.
    pub fn new(options: PageOptions, script_timeout: Duration) -> Result<Self, String> {
        if !current_thread_can_run_js() {
            return Err("page JavaScript only runs on a browser thread".to_string());
        }
        let mut runtime = PageRuntime::new(options).map_err(|e| e.to_string())?;
        let isolate = runtime.terminate_handle();
        Ok(Self {
            runtime,
            isolate,
            script_timeout,
        })
    }

    fn error(&self, error: PageError, timed_out: bool) -> JsError {
        match error {
            PageError::Terminated if timed_out => JsError::Timeout(self.script_timeout),
            PageError::Js(info) => JsError::Exception(info.to_string()),
            PageError::OutOfMemory => JsError::OutOfMemory,
            other => JsError::Other(other.to_string()),
        }
    }

    /// Run `call` under the watchdog.
    fn guarded<R>(
        &mut self,
        call: impl FnOnce(&mut PageRuntime) -> Result<R, PageError>,
    ) -> Result<R, JsError> {
        let armed = arm(self.isolate.clone(), self.script_timeout);
        let result = call(&mut self.runtime);
        let timed_out = armed.disarm();
        result.map_err(|error| self.error(error, timed_out))
    }

    /// The page's live document.
    pub fn with_document<R>(&self, f: impl FnOnce(&Document) -> R) -> R {
        self.runtime.with_document(f)
    }

    /// Run a classic script (and the microtasks it queued).
    pub fn execute_script(&mut self, name: &str, source: String) -> Result<(), JsError> {
        self.guarded(|runtime| runtime.execute_script(name, source))
    }

    /// Evaluate a script and return its completion value as JSON. A promise
    /// is not awaited.
    pub fn evaluate_json(&mut self, source: String) -> Result<serde_json::Value, JsError> {
        self.guarded(|runtime| runtime.evaluate_json(source))
    }

    /// Click `node` as a user would (events, then the default action).
    pub fn click(&mut self, node: NodeId) -> Result<ClickOutcome, JsError> {
        self.guarded(|runtime| runtime.click(node))
    }

    /// Set the value of a form control as typing or choosing would.
    pub fn set_value(&mut self, node: NodeId, value: &str) -> Result<(), JsError> {
        self.guarded(|runtime| runtime.set_value(node, value))
    }

    /// The live value and checkedness of a form control.
    pub fn form_state(&mut self, node: NodeId) -> Result<Option<FormState>, JsError> {
        self.guarded(|runtime| runtime.form_state(node))
    }

    /// `DOMContentLoaded`.
    pub fn fire_dom_content_loaded(&mut self) -> Result<(), JsError> {
        self.guarded(|runtime| runtime.fire_dom_content_loaded())
    }

    /// `load`.
    pub fn fire_load(&mut self) -> Result<(), JsError> {
        self.guarded(|runtime| runtime.fire_load())
    }

    /// Pump the page's event loop until nothing is pending (`true`) or
    /// `budget` has passed with work still pending (`false`). A task that
    /// outlives the budget by more than the script timeout is ended.
    ///
    /// Cancel-safe: dropping the future between polls leaves the page as it
    /// was at the last completed task.
    pub async fn run_until_idle(&mut self, budget: Duration) -> Result<bool, JsError> {
        // The budget is only checked between tasks, so a call that has not
        // returned `script_timeout` after it is stuck in one.
        let armed = arm(self.isolate.clone(), budget + self.script_timeout);
        let result = self.runtime.run_until_idle(budget).await;
        let timed_out = armed.disarm();
        match result {
            Ok(Idle::Quiescent) => Ok(true),
            Ok(Idle::BudgetExhausted) => Ok(false),
            Err(error) => Err(self.error(error, timed_out)),
        }
    }

    /// Take what the page logged, the errors nothing caught, and the
    /// navigations scripts asked for.
    pub fn drain(&mut self) -> JsOutput {
        let mut output = JsOutput::default();
        for message in self.runtime.take_console_messages() {
            let level = match message.level {
                ConsoleLevel::Debug => "debug",
                ConsoleLevel::Info => "log",
                ConsoleLevel::Warn => "warning",
                ConsoleLevel::Error => "error",
            };
            output.console.push((level, message.text));
        }
        for error in self.runtime.take_uncaught_errors() {
            output.console.push(("error", error.to_string()));
        }
        output.navigations = self.runtime.take_navigation_requests();
        output
    }
}
