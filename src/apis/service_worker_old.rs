use anyhow::Result;
use boa_engine::{Context, JsObject, JsValue, NativeFunction, property::Attribute, js_string};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Service Worker implementation for PWA support and modern web app compatibility
pub struct ServiceWorkerManager {
    registrations: Arc<Mutex<HashMap<String, ServiceWorkerRegistration>>>,
    active_workers: Arc<Mutex<HashMap<String, ServiceWorker>>>,
}

#[derive(Debug, Clone)]
pub struct ServiceWorkerRegistration {
    pub scope: String,
    pub script_url: String,
    pub state: RegistrationState,
    pub update_via_cache: String,
}

#[derive(Debug, Clone)]
pub struct ServiceWorker {
    pub script_url: String,
    pub state: WorkerState,
}

#[derive(Debug, Clone)]
pub enum RegistrationState {
    Installing,
    Waiting,
    Active,
}

#[derive(Debug, Clone)]
pub enum WorkerState {
    Parsed,
    Installing,
    Installed,
    Activating,
    Activated,
    Redundant,
}

impl ServiceWorkerManager {
    pub fn new() -> Self {
        Self {
            registrations: Arc::new(Mutex::new(HashMap::new())),
            active_workers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Setup Service Worker API in navigator
    pub fn setup_service_worker_api(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        // Get navigator object
        let navigator_obj = context.global_object().get(js_string!("navigator"), context)?;

        if let Some(nav_obj) = navigator_obj.as_object() {
            let service_worker_obj = JsObject::default();

            // navigator.serviceWorker.register()
            let register_fn = unsafe { NativeFunction::from_closure({
                let registrations_clone = Arc::clone(&self.registrations);
                let workers_clone = Arc::clone(&self.active_workers);
                move |_, args, context| {
                    if args.is_empty() {
                        return Err(boa_engine::JsNativeError::typ()
                            .with_message("register() requires a script URL")
                            .into());
                    }

                    let script_url = args[0].to_string(context)?.to_std_string_escaped();
                    let scope = if args.len() > 1 && args[1].is_object() {
                        if let Some(options) = args[1].as_object() {
                            if let Ok(scope_val) = options.get(js_string!("scope"), context) {
                                scope_val.to_string(context)?.to_std_string_escaped()
                            } else {
                                "/".to_string()
                            }
                        } else {
                            "/".to_string()
                        }
                    } else {
                        "/".to_string()
                    };

                    // Store registration and worker
                    {
                        let mut regs = registrations_clone.lock().unwrap();
                        let registration = ServiceWorkerRegistration {
                            scope: scope.clone(),
                            script_url: script_url.clone(),
                            state: RegistrationState::Installing,
                            update_via_cache: "imports".to_string(),
                        };
                        regs.insert(scope.clone(), registration);

                        let mut workers = workers_clone.lock().unwrap();
                        let worker = ServiceWorker {
                            script_url: script_url.clone(),
                            state: WorkerState::Installing,
                        };
                        workers.insert(script_url.clone(), worker);
                    }

                    // Return Promise-like object
                    let promise_obj = JsObject::default();
                    let then_fn = unsafe { NativeFunction::from_closure(move |_, _args, _context| {
                        Ok(JsValue::undefined())
                    }) };
                    promise_obj.set(js_string!("then"), JsValue::from(then_fn.to_js_function(context.realm())), false, context)?;

                    Ok(JsValue::from(promise_obj))
                }
            }) };
            service_worker_obj.set(js_string!("register"), JsValue::from(register_fn.to_js_function(context.realm())), false, context)?;

            // navigator.serviceWorker.ready (Promise)
            let ready_promise = JsObject::default();
            let ready_then_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
                Ok(JsValue::undefined())
            }) };
            ready_promise.set(js_string!("then"), JsValue::from(ready_then_fn.to_js_function(context.realm())), false, context)?;
            service_worker_obj.set(js_string!("ready"), JsValue::from(ready_promise), false, context)?;

            // navigator.serviceWorker.controller
            service_worker_obj.set(js_string!("controller"), JsValue::null(), false, context)?;

            nav_obj.set(js_string!("serviceWorker"), JsValue::from(service_worker_obj), false, context)?;
        }

        Ok(())
    }
}