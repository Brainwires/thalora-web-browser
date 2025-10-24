//! Command handling for worker threads

use boa_engine::{Context, JsResult, JsValue};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use crossbeam_channel::Sender;

use crate::worker::worker_global_scope::WorkerGlobalScope;
use crate::misc::structured_clone::structured_deserialize;
use super::types::{WorkerCommand, WorkerEvent, WorkerStatus};

/// Handle a command from the main thread
pub fn handle_command(
    command: WorkerCommand,
    context: &mut Context,
    worker_scope: &Arc<WorkerGlobalScope>,
    status: &Arc<Mutex<WorkerStatus>>,
    running: &Arc<AtomicBool>,
    event_tx: &Sender<WorkerEvent>,
) -> JsResult<bool> {
    match command {
        WorkerCommand::ExecuteScript { script } => {
            match worker_scope.execute_script(context, &script) {
                Ok(_) => {
                    let _ = event_tx.send(WorkerEvent::ScriptExecuted { success: true });
                }
                Err(e) => {
                    let error_msg = format!("{:?}", e);
                    let _ = event_tx.send(WorkerEvent::Error {
                        message: error_msg,
                        filename: "eval".to_string(),
                        lineno: 0,
                        colno: 0,
                    });
                    let _ = event_tx.send(WorkerEvent::ScriptExecuted { success: false });
                }
            }
            Ok(true)
        }

        WorkerCommand::PostMessage { message } => {
            // Message is already structured-cloned, deserialize it
            let _deserialized = structured_deserialize(&message, context)?;

            // Create a WorkerMessage and send to the worker scope
            // The worker scope will dispatch the message event
            // (This is handled by process_message_from_main in the event loop)

            Ok(true)
        }

        WorkerCommand::ImportScripts { urls } => {
            eprintln!("[Worker] importScripts called with: {:?}", urls);

            // Get the base URL from worker scope
            let base_url = Some(worker_scope.get_location().href.clone());

            // Use the import_scripts implementation
            match crate::worker::import_scripts::import_scripts_impl(urls, base_url, context) {
                Ok(_) => {
                    eprintln!("[Worker] importScripts completed successfully");
                }
                Err(e) => {
                    let error_msg = format!("{:?}", e);
                    let _ = event_tx.send(WorkerEvent::Error {
                        message: error_msg,
                        filename: "importScripts".to_string(),
                        lineno: 0,
                        colno: 0,
                    });
                }
            }
            Ok(true)
        }

        WorkerCommand::Terminate => {
            eprintln!("[Worker] Received terminate command");
            let mut worker_status = status.lock().unwrap();
            *worker_status = WorkerStatus::Terminating;
            running.store(false, Ordering::SeqCst);
            Ok(false) // Signal to exit the event loop
        }

        WorkerCommand::Suspend => {
            eprintln!("[Worker] Suspending");
            let mut worker_status = status.lock().unwrap();
            *worker_status = WorkerStatus::Suspended;
            Ok(true)
        }

        WorkerCommand::Resume => {
            eprintln!("[Worker] Resuming");
            let mut worker_status = status.lock().unwrap();
            *worker_status = WorkerStatus::Running;
            Ok(true)
        }
    }
}
