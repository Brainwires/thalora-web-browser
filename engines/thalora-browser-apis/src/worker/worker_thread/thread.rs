//! WorkerThread implementation - main worker thread struct

use boa_engine::{Context, JsNativeError, JsResult};
use crossbeam_channel::{Receiver, Sender, TryRecvError, unbounded};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::command_handler;
use super::event_loop::WorkerEventLoop;
use super::script_loader;
use super::timer_api;
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

        // Create a new JavaScript context for this worker
        let mut context = Context::default();

        // Initialize the worker global scope
        let scope_type = WorkerGlobalScopeType::Dedicated;
        let worker_scope =
            WorkerGlobalScope::new(scope_type, &config.script_url, Some(event_tx.clone()))?;
        let worker_scope_arc = Arc::new(worker_scope);

        // Register the scope in the global registry
        WorkerGlobalScope::register_scope(worker_scope_arc.clone());

        // Initialize worker global scope APIs in the context
        worker_scope_arc.initialize_in_context(&mut context)?;

        // Create the event loop for this worker
        let event_loop = Arc::new(Mutex::new(WorkerEventLoop::new()));

        // Initialize timer APIs with the event loop
        timer_api::init_worker_timers(&mut context, event_loop.clone())?;

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

        // Main event loop
        while running.load(Ordering::SeqCst) {
            // Check if we're suspended
            let current_status = {
                let worker_status = status.lock().unwrap();
                *worker_status
            };

            if current_status == WorkerStatus::Suspended {
                thread::sleep(Duration::from_millis(10));
                continue;
            }

            if current_status == WorkerStatus::Terminating {
                break;
            }

            // Process incoming messages from main thread
            let _ = worker_scope_arc.process_main_thread_messages(&mut context);

            // Process event loop (get pending work)
            let (timer_callbacks, microtask_callbacks, next_timer_delay) = {
                let mut event_loop_guard = event_loop.lock().unwrap();
                event_loop_guard.get_pending_work()
            };

            // Execute timer callbacks
            for (callback_id, _is_repeating) in timer_callbacks {
                let _ = super::callback_registry::execute_callback(callback_id, &mut context);
            }

            // Execute microtask callbacks
            for callback_id in microtask_callbacks {
                let _ = super::callback_registry::execute_callback(callback_id, &mut context);
            }

            // Process commands from the main thread
            match command_rx.try_recv() {
                Ok(command) => {
                    match command_handler::handle_command(
                        command,
                        &mut context,
                        &worker_scope_arc,
                        &status,
                        &running,
                        &event_tx,
                    ) {
                        Ok(should_continue) => {
                            if !should_continue {
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("[Worker {}] Command handling error: {:?}", worker_id, e);
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    // No commands, sleep based on next timer or default
                    let sleep_duration = next_timer_delay
                        .unwrap_or_else(|| Duration::from_millis(10))
                        .min(Duration::from_millis(10)); // Cap at 10ms for responsiveness
                    thread::sleep(sleep_duration);
                }
                Err(TryRecvError::Disconnected) => {
                    // Main thread disconnected, terminate
                    eprintln!("[Worker {}] Command channel disconnected", worker_id);
                    break;
                }
            }
        }

        // Cleanup
        WorkerGlobalScope::unregister_scope(worker_scope_arc.get_scope_id());

        // Unregister event loop
        timer_api::unregister_event_loop();

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
