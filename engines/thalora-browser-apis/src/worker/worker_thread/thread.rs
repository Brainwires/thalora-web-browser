//! WorkerThread implementation - main worker thread struct

use boa_engine::{Context, JsNativeError, JsResult, builtins::IntrinsicObject};
use crossbeam_channel::{Receiver, Sender, TryRecvError, unbounded};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::command_handler;
use super::script_loader;
use super::types::{WorkerCommand, WorkerConfig, WorkerEvent, WorkerStatus};
use crate::worker::worker_global_scope::{WorkerGlobalScope, WorkerGlobalScopeType};

/// Unique identifier for worker threads
static NEXT_WORKER_ID: AtomicU64 = AtomicU64::new(1);

/// Generate a unique worker ID
fn generate_worker_id() -> u64 {
    NEXT_WORKER_ID.fetch_add(1, Ordering::SeqCst)
}

/// Represents a real OS thread running worker JavaScript
pub struct WorkerThread {
    /// Unique worker ID
    worker_id: u64,
    /// Worker configuration
    config: WorkerConfig,
    /// Worker status
    status: Arc<Mutex<WorkerStatus>>,
    /// Flag indicating if worker should keep running
    running: Arc<AtomicBool>,
    /// Channel to send commands to worker thread
    command_sender: Sender<WorkerCommand>,
    /// Channel to receive events from worker thread
    event_receiver: Receiver<WorkerEvent>,
    /// Handle to the OS thread
    thread_handle: Option<JoinHandle<()>>,
    /// Worker creation timestamp
    created_at: Instant,
}

impl WorkerThread {
    /// Create and start a new worker thread
    pub fn spawn(config: WorkerConfig) -> JsResult<Self> {
        let worker_id = generate_worker_id();
        let status = Arc::new(Mutex::new(WorkerStatus::Initializing));
        let running = Arc::new(AtomicBool::new(true));

        // Create channels for bidirectional communication
        let (command_tx, command_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();

        // Clone references for the worker thread
        let worker_config = config.clone();
        let worker_status = status.clone();
        let worker_running = running.clone();
        let worker_id_clone = worker_id;

        // Build the thread
        let mut thread_builder = thread::Builder::new().name(format!("Worker-{}", worker_id));

        if let Some(stack_size) = config.stack_size {
            thread_builder = thread_builder.stack_size(stack_size);
        }

        // Spawn the worker thread
        let thread_handle = thread_builder
            .spawn(move || {
                let result = Self::run_worker_thread(
                    worker_id_clone,
                    worker_config,
                    worker_status,
                    worker_running,
                    command_rx,
                    event_tx,
                );

                if let Err(e) = result {
                    eprintln!("Worker thread {} failed: {:?}", worker_id_clone, e);
                }
            })
            .map_err(|e| {
                JsNativeError::error().with_message(format!("Failed to spawn worker thread: {}", e))
            })?;

        Ok(Self {
            worker_id,
            config,
            status,
            running,
            command_sender: command_tx,
            event_receiver: event_rx,
            thread_handle: Some(thread_handle),
            created_at: Instant::now(),
        })
    }

    /// Main worker thread execution loop
    fn run_worker_thread(
        worker_id: u64,
        config: WorkerConfig,
        status: Arc<Mutex<WorkerStatus>>,
        running: Arc<AtomicBool>,
        command_rx: Receiver<WorkerCommand>,
        event_tx: Sender<WorkerEvent>,
    ) -> JsResult<()> {
        eprintln!("[Worker {}] Thread started", worker_id);

        // A JS context driven by the same event loop as pages: timers,
        // promises and async jobs (fetch) all go through the executor.
        let executor = std::rc::Rc::new(crate::event_loop::ThaloraJobExecutor::new());
        let mut context = Context::builder()
            .job_executor(executor.clone())
            .build()
            .map_err(|e| JsNativeError::error().with_message(e.to_string()))?;

        // Install browsing-context state (origin + is_worker=true) so OPFS and
        // FileSystemSyncAccessHandle can detect the worker realm and resolve
        // the active origin.
        crate::realm_ext::install(&mut context, config.origin.clone(), true);

        // Initialize the FileSystem* prototypes inside the worker realm so
        // OPFS handles handed out via `navigator.storage.getDirectory()` have
        // working method tables.
        crate::file::file_system::FileSystemHandle::init(context.realm());
        crate::file::file_system::FileSystemFileHandle::init(context.realm());
        crate::file::file_system::FileSystemDirectoryHandle::init(context.realm());
        crate::file::file_system::writable_stream::FileSystemWritableFileStream::init(
            context.realm(),
        );
        crate::file::file_system::sync_access::FileSystemSyncAccessHandle::init(context.realm());
        crate::storage::storage_manager::StorageManager::init(context.realm());

        // Initialize the worker global scope
        let scope_type = WorkerGlobalScopeType::Dedicated;
        let worker_scope =
            WorkerGlobalScope::new(scope_type, &config.script_url, Some(event_tx.clone()))?;
        let worker_scope_arc = Arc::new(worker_scope);

        // Register the scope in the global registry
        WorkerGlobalScope::register_scope(worker_scope_arc.clone());

        // Initialize worker global scope APIs in the context
        worker_scope_arc.initialize_in_context(&mut context)?;

        // Update status to running
        {
            let mut worker_status = status.lock().unwrap();
            *worker_status = WorkerStatus::Running;
        }

        // Send started event
        let _ = event_tx.send(WorkerEvent::Started);

        // Load and execute the initial worker script if provided
        if !config.script_url.is_empty() {
            match script_loader::load_and_execute_script(
                &config.script_url,
                &mut context,
                &worker_scope_arc,
            ) {
                Ok(_) => {
                    let _ = event_tx.send(WorkerEvent::ScriptExecuted { success: true });
                }
                Err(e) => {
                    let error_msg = format!("{:?}", e);
                    let _ = event_tx.send(WorkerEvent::Error {
                        message: error_msg,
                        filename: config.script_url.clone(),
                        lineno: 0,
                        colno: 0,
                    });
                    let _ = event_tx.send(WorkerEvent::ScriptExecuted { success: false });
                }
            }
        }

        // Main event loop: one long pump per iteration. Between tasks the
        // pump asks the predicate below, which handles messages and commands
        // from the main thread; async jobs (fetch) stay in flight across it.
        let exit = std::rc::Rc::new(std::cell::Cell::new(false));
        let command_rx = std::rc::Rc::new(command_rx);
        while running.load(Ordering::SeqCst) && !exit.get() {
            let exit = exit.clone();
            let command_rx = command_rx.clone();
            let worker_scope = worker_scope_arc.clone();
            let status = status.clone();
            let running = running.clone();
            let event_tx = event_tx.clone();
            let budget =
                crate::event_loop::PumpBudget::until(Duration::from_secs(3600), move |context| {
                    let current_status = *status.lock().unwrap();
                    if current_status == WorkerStatus::Terminating
                        || !running.load(Ordering::SeqCst)
                    {
                        exit.set(true);
                        return true;
                    }
                    // Messages from the main thread (onmessage)
                    if current_status != WorkerStatus::Suspended {
                        let _ = worker_scope.process_main_thread_messages(context);
                    }
                    // Commands (execute script, suspend/resume, terminate)
                    loop {
                        match command_rx.try_recv() {
                            Ok(command) => match command_handler::handle_command(
                                command,
                                context,
                                &worker_scope,
                                &status,
                                &running,
                                &event_tx,
                            ) {
                                Ok(true) => {}
                                Ok(false) => {
                                    exit.set(true);
                                    return true;
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[Worker {}] Command handling error: {:?}",
                                        worker_id, e
                                    );
                                }
                            },
                            Err(TryRecvError::Empty) => break,
                            Err(TryRecvError::Disconnected) => {
                                eprintln!("[Worker {}] Command channel disconnected", worker_id);
                                exit.set(true);
                                return true;
                            }
                        }
                    }
                    false
                });
            executor.pump(&mut context, budget);
        }

        // Cleanup
        WorkerGlobalScope::unregister_scope(worker_scope_arc.get_scope_id());

        // Update status to terminated
        {
            let mut worker_status = status.lock().unwrap();
            *worker_status = WorkerStatus::Terminated;
        }

        let _ = event_tx.send(WorkerEvent::Terminated);
        eprintln!("[Worker {}] Thread terminated", worker_id);

        Ok(())
    }

    /// Send a command to the worker thread
    pub fn send_command(&self, command: WorkerCommand) -> Result<(), String> {
        self.command_sender
            .send(command)
            .map_err(|e| format!("Failed to send command to worker: {}", e))
    }

    /// Try to receive an event from the worker thread (non-blocking)
    pub fn try_recv_event(&self) -> Option<WorkerEvent> {
        self.event_receiver.try_recv().ok()
    }

    /// Receive an event from the worker thread (blocking with timeout)
    pub fn recv_event_timeout(&self, timeout: Duration) -> Option<WorkerEvent> {
        self.event_receiver.recv_timeout(timeout).ok()
    }

    /// Get the worker ID
    pub fn id(&self) -> u64 {
        self.worker_id
    }

    /// Get the current worker status
    pub fn status(&self) -> WorkerStatus {
        *self.status.lock().unwrap()
    }

    /// Check if the worker is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Terminate the worker (non-blocking)
    pub fn terminate(&mut self) {
        eprintln!("[Worker {}] Terminating", self.worker_id);
        let _ = self.send_command(WorkerCommand::Terminate);
    }

    /// Wait for the worker thread to finish (blocking)
    pub fn join(mut self) -> Result<(), String> {
        self.terminate();

        if let Some(handle) = self.thread_handle.take() {
            handle
                .join()
                .map_err(|e| format!("Worker thread panicked: {:?}", e))
        } else {
            Ok(())
        }
    }

    /// Get worker uptime
    pub fn uptime(&self) -> Duration {
        self.created_at.elapsed()
    }
}

impl Drop for WorkerThread {
    fn drop(&mut self) {
        // Ensure worker is terminated when dropped
        if self.is_running() {
            eprintln!("[Worker {}] Dropping - sending terminate", self.worker_id);
            self.terminate();

            // Give the thread a short time to terminate gracefully
            if let Some(handle) = self.thread_handle.take() {
                std::thread::sleep(Duration::from_millis(100));
                // If it doesn't finish quickly, we just detach and let it terminate
                let _ = handle.join();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::WorkerType;
    use super::*;

    #[test]
    fn test_worker_thread_creation() {
        let config = WorkerConfig {
            name: Some("test-worker".to_string()),
            script_url: "console.log('Hello from worker');".to_string(),
            worker_type: WorkerType::Classic,
            stack_size: Some(2 * 1024 * 1024),
            origin: "thalora://test".to_string(),
        };

        let worker = WorkerThread::spawn(config);
        assert!(worker.is_ok());

        let mut worker = worker.unwrap();
        assert!(worker.is_running());

        // Wait a bit for the worker to start
        std::thread::sleep(Duration::from_millis(100));

        // Check for started event
        let event = worker.try_recv_event();
        assert!(event.is_some());

        worker.terminate();
        let _ = worker.join();
    }

    #[test]
    fn test_worker_data_url_script() {
        let script = "self.postMessage('test');";
        let data_url = format!(
            "data:application/javascript,{}",
            urlencoding::encode(script)
        );

        let config = WorkerConfig {
            script_url: data_url,
            worker_type: WorkerType::Classic,
            ..Default::default()
        };

        let worker = WorkerThread::spawn(config);
        assert!(worker.is_ok());
    }

    #[test]
    #[ignore = "requires cooperative JS interruption - Boa executes synchronously and cannot be interrupted mid-execution"]
    fn test_worker_terminate() {
        let config = WorkerConfig {
            script_url: "while(true) { }".to_string(), // Infinite loop
            worker_type: WorkerType::Classic,
            ..Default::default()
        };

        let mut worker = WorkerThread::spawn(config).unwrap();
        assert!(worker.is_running());

        worker.terminate();

        // Wait for termination
        std::thread::sleep(Duration::from_millis(200));

        assert!(!worker.is_running());
    }
}
