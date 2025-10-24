//! Worker thread types and enums

use crate::misc::structured_clone::StructuredCloneValue;

/// Commands that can be sent to the worker thread
#[derive(Debug, Clone)]
pub enum WorkerCommand {
    /// Execute a script in the worker context
    ExecuteScript { script: String },
    /// Send a message to the worker
    PostMessage { message: StructuredCloneValue },
    /// Import external scripts
    ImportScripts { urls: Vec<String> },
    /// Terminate the worker
    Terminate,
    /// Suspend the worker (pause execution)
    Suspend,
    /// Resume the worker
    Resume,
}

/// Events that can be sent from the worker thread to the main thread
#[derive(Debug, Clone)]
pub enum WorkerEvent {
    /// Worker has started successfully
    Started,
    /// Worker has sent a message
    Message { data: StructuredCloneValue },
    /// Worker encountered an error
    Error { message: String, filename: String, lineno: u32, colno: u32 },
    /// Worker has terminated
    Terminated,
    /// Script execution completed
    ScriptExecuted { success: bool },
}

/// Status of a worker thread
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerStatus {
    /// Worker is being initialized
    Initializing,
    /// Worker is running normally
    Running,
    /// Worker is suspended (paused)
    Suspended,
    /// Worker is terminating
    Terminating,
    /// Worker has terminated
    Terminated,
}

/// Worker type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerType {
    /// Classic script worker
    Classic,
    /// Module script worker (ES6 modules)
    Module,
}

/// Configuration for worker thread creation
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Worker name (for debugging)
    pub name: Option<String>,
    /// Worker type (classic or module)
    pub worker_type: WorkerType,
    /// Script URL or content
    pub script_url: String,
    /// Maximum stack size for the worker thread
    pub stack_size: Option<usize>,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            name: None,
            worker_type: WorkerType::Classic,
            script_url: String::new(),
            stack_size: Some(2 * 1024 * 1024), // 2MB default stack
        }
    }
}
