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
    pub fn setup_service_worker_api(&self, context: &mut Context) -> Result<()> {
        // Get navigator object
        let navigator_obj = context.global_object().get(js_string!("navigator"), context)?;
        
        if let Ok(nav_obj) = navigator_obj.as_object() {
            let service_worker_obj = JsObject::default();
            
            // navigator.serviceWorker.register()
            let registrations_clone = Arc::clone(&self.registrations);
            let workers_clone = Arc::clone(&self.active_workers);
            
            let register_fn = NativeFunction::from_fn_ptr(move |_, args, context| {
                if args.is_empty() {
                    return Err(boa_engine::JsError::from_native("register() requires a script URL"));
                }

                let script_url = args[0].to_string(context)?.to_std_string_escaped();
                let scope = if args.len() > 1 && args[1].is_object() {
                    // Parse options object for scope
                    if let Ok(options) = args[1].as_object() {
                        if let Ok(scope_val) = options.get("scope", context) {
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

                // Create registration
                let registration = ServiceWorkerRegistration {
                    scope: scope.clone(),
                    script_url: script_url.clone(),
                    state: RegistrationState::Installing,
                    update_via_cache: "imports".to_string(),
                };

                // Store registration
                {
                    let mut regs = registrations_clone.lock().unwrap();
                    regs.insert(scope.clone(), registration);
                }

                // Create worker
                let worker = ServiceWorker {
                    script_url: script_url.clone(),
                    state: WorkerState::Installing,
                };
                
                {
                    let mut workers = workers_clone.lock().unwrap();
                    workers.insert(script_url.clone(), worker);
                }

                // Return a Promise that resolves to a ServiceWorkerRegistration
                let promise_obj = JsObject::default();
                
                let then_fn = NativeFunction::from_fn_ptr(move |_, args, context| {
                    if !args.is_empty() && args[0].is_callable() {
                        // Create ServiceWorkerRegistration object
                        let reg_obj = Self::create_registration_object(context, &scope, &script_url)?;
                        
                        if let Ok(callback) = args[0].as_callable() {
                            let _ = callback.call(&JsValue::undefined(), &[JsValue::from(reg_obj)], context);
                        }
                    }
                    Ok(JsValue::undefined())
                });
                promise_obj.set("then", then_fn, false, context)?;
                
                let catch_fn = NativeFunction::from_fn_ptr(|_, args, _context| {
                    Ok(JsValue::undefined())
                });
                promise_obj.set("catch", catch_fn, false, context)?;

                Ok(JsValue::from(promise_obj))
            });
            service_worker_obj.set("register", register_fn, false, context)?;

            // navigator.serviceWorker.ready (Promise)
            let ready_promise = JsObject::default();
            let ready_then_fn = NativeFunction::from_fn_ptr(|_, args, context| {
                if !args.is_empty() && args[0].is_callable() {
                    // Return a mock ready registration
                    let reg_obj = Self::create_registration_object(context, "/", "/sw.js")?;
                    
                    if let Ok(callback) = args[0].as_callable() {
                        let _ = callback.call(&JsValue::undefined(), &[JsValue::from(reg_obj)], context);
                    }
                }
                Ok(JsValue::undefined())
            });
            ready_promise.set("then", ready_then_fn, false, context)?;
            service_worker_obj.set("ready", ready_promise, false, context)?;

            // navigator.serviceWorker.controller
            service_worker_obj.set("controller", JsValue::null(), true, context)?;

            // navigator.serviceWorker.getRegistrations()
            let get_registrations_fn = NativeFunction::from_fn_ptr(|_, args, context| {
                let promise_obj = JsObject::default();
                
                let then_fn = NativeFunction::from_fn_ptr(|_, args, context| {
                    if !args.is_empty() && args[0].is_callable() {
                        // Return empty array of registrations
                        let regs_array = context.construct_array(&[])?;
                        
                        if let Ok(callback) = args[0].as_callable() {
                            let _ = callback.call(&JsValue::undefined(), &[JsValue::from(regs_array)], context);
                        }
                    }
                    Ok(JsValue::undefined())
                });
                promise_obj.set("then", then_fn, false, context)?;
                
                Ok(JsValue::from(promise_obj))
            });
            service_worker_obj.set("getRegistrations", get_registrations_fn, false, context)?;

            // Add event listener support
            service_worker_obj.set("addEventListener", 
                NativeFunction::from_fn_ptr(|_, args, _context| Ok(JsValue::undefined())), 
                false, context)?;
            service_worker_obj.set("removeEventListener", 
                NativeFunction::from_fn_ptr(|_, args, _context| Ok(JsValue::undefined())), 
                false, context)?;

            nav_obj.set("serviceWorker", service_worker_obj, false, context)?;
        }

        // Setup Cache API (companion to Service Workers)
        self.setup_cache_api(context)?;

        Ok(())
    }

    fn create_registration_object(context: &mut Context, scope: &str, script_url: &str) -> Result<JsObject, boa_engine::JsError> {
        let reg_obj = JsObject::default();
        
        reg_obj.set("scope", scope, false, context)?;
        reg_obj.set("updateViaCache", "imports", false, context)?;
        
        // Create ServiceWorker objects
        let installing_worker = Self::create_worker_object(context, script_url, WorkerState::Installing)?;
        let waiting_worker = JsValue::null();
        let active_worker = Self::create_worker_object(context, script_url, WorkerState::Activated)?;
        
        reg_obj.set("installing", installing_worker, false, context)?;
        reg_obj.set("waiting", waiting_worker, false, context)?;
        reg_obj.set("active", active_worker, false, context)?;
        
        // Methods
        let update_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            let promise_obj = JsObject::default();
            let then_fn = NativeFunction::from_fn_ptr(|_, args, _context| Ok(JsValue::undefined()));
            promise_obj.set("then", then_fn, false, context)?;
            Ok(JsValue::from(promise_obj))
        });
        reg_obj.set("update", update_fn, false, context)?;
        
        let unregister_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            let promise_obj = JsObject::default();
            let then_fn = NativeFunction::from_fn_ptr(|_, args, context| {
                if !args.is_empty() && args[0].is_callable() {
                    if let Ok(callback) = args[0].as_callable() {
                        let _ = callback.call(&JsValue::undefined(), &[JsValue::from(true)], context);
                    }
                }
                Ok(JsValue::undefined())
            });
            promise_obj.set("then", then_fn, false, context)?;
            Ok(JsValue::from(promise_obj))
        });
        reg_obj.set("unregister", unregister_fn, false, context)?;
        
        // Event listener support
        reg_obj.set("addEventListener", 
            NativeFunction::from_fn_ptr(|_, args, _context| Ok(JsValue::undefined())), 
            false, context)?;
        reg_obj.set("removeEventListener", 
            NativeFunction::from_fn_ptr(|_, args, _context| Ok(JsValue::undefined())), 
            false, context)?;
        
        Ok(reg_obj)
    }

    fn create_worker_object(context: &mut Context, script_url: &str, state: WorkerState) -> Result<JsValue, boa_engine::JsError> {
        let worker_obj = JsObject::default();
        
        worker_obj.set("scriptURL", script_url, false, context)?;
        worker_obj.set("state", format!("{:?}", state).to_lowercase(), false, context)?;
        
        // postMessage method
        let post_message_fn = NativeFunction::from_fn_ptr(|_, args, _context| Ok(JsValue::undefined()));
        worker_obj.set("postMessage", post_message_fn, false, context)?;
        
        // Event listener support
        worker_obj.set("addEventListener", 
            NativeFunction::from_fn_ptr(|_, args, _context| Ok(JsValue::undefined())), 
            false, context)?;
        worker_obj.set("removeEventListener", 
            NativeFunction::from_fn_ptr(|_, args, _context| Ok(JsValue::undefined())), 
            false, context)?;
        
        Ok(JsValue::from(worker_obj))
    }

    /// Setup Cache API (used by Service Workers)
    fn setup_cache_api(&self, context: &mut Context) -> Result<()> {
        let caches_obj = JsObject::default();
        
        // caches.open()
        let open_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            let promise_obj = JsObject::default();
            
            let then_fn = NativeFunction::from_fn_ptr(|_, args, context| {
                if !args.is_empty() && args[0].is_callable() {
                    // Create Cache object
                    let cache_obj = Self::create_cache_object(context)?;
                    
                    if let Ok(callback) = args[0].as_callable() {
                        let _ = callback.call(&JsValue::undefined(), &[JsValue::from(cache_obj)], context);
                    }
                }
                Ok(JsValue::undefined())
            });
            promise_obj.set("then", then_fn, false, context)?;
            
            Ok(JsValue::from(promise_obj))
        });
        caches_obj.set("open", open_fn, false, context)?;
        
        // caches.match()
        let match_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            let promise_obj = JsObject::default();
            
            let then_fn = NativeFunction::from_fn_ptr(|_, args, context| {
                if !args.is_empty() && args[0].is_callable() {
                    if let Ok(callback) = args[0].as_callable() {
                        let _ = callback.call(&JsValue::undefined(), &[JsValue::undefined()], context);
                    }
                }
                Ok(JsValue::undefined())
            });
            promise_obj.set("then", then_fn, false, context)?;
            
            Ok(JsValue::from(promise_obj))
        });
        caches_obj.set("match", match_fn, false, context)?;
        
        // caches.keys()
        let keys_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            let promise_obj = JsObject::default();
            
            let then_fn = NativeFunction::from_fn_ptr(|_, args, context| {
                if !args.is_empty() && args[0].is_callable() {
                    let keys_array = context.construct_array(&[])?;
                    
                    if let Ok(callback) = args[0].as_callable() {
                        let _ = callback.call(&JsValue::undefined(), &[JsValue::from(keys_array)], context);
                    }
                }
                Ok(JsValue::undefined())
            });
            promise_obj.set("then", then_fn, false, context)?;
            
            Ok(JsValue::from(promise_obj))
        });
        caches_obj.set("keys", keys_fn, false, context)?;
        
        context.register_global_property(js_string!("caches"), caches_obj, Attribute::all())?;
        
        Ok(())
    }

    fn create_cache_object(context: &mut Context) -> Result<JsObject, boa_engine::JsError> {
        let cache_obj = JsObject::default();
        
        // cache.put()
        let put_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            let promise_obj = JsObject::default();
            let then_fn = NativeFunction::from_fn_ptr(|_, args, _context| Ok(JsValue::undefined()));
            promise_obj.set("then", then_fn, false, context)?;
            Ok(JsValue::from(promise_obj))
        });
        cache_obj.set("put", put_fn, false, context)?;
        
        // cache.match()
        let match_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            let promise_obj = JsObject::default();
            let then_fn = NativeFunction::from_fn_ptr(|_, args, context| {
                if !args.is_empty() && args[0].is_callable() {
                    if let Ok(callback) = args[0].as_callable() {
                        let _ = callback.call(&JsValue::undefined(), &[JsValue::undefined()], context);
                    }
                }
                Ok(JsValue::undefined())
            });
            promise_obj.set("then", then_fn, false, context)?;
            Ok(JsValue::from(promise_obj))
        });
        cache_obj.set("match", match_fn, false, context)?;
        
        // cache.keys()
        let keys_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            let promise_obj = JsObject::default();
            let then_fn = NativeFunction::from_fn_ptr(|_, args, context| {
                if !args.is_empty() && args[0].is_callable() {
                    let keys_array = context.construct_array(&[])?;
                    if let Ok(callback) = args[0].as_callable() {
                        let _ = callback.call(&JsValue::undefined(), &[JsValue::from(keys_array)], context);
                    }
                }
                Ok(JsValue::undefined())
            });
            promise_obj.set("then", then_fn, false, context)?;
            Ok(JsValue::from(promise_obj))
        });
        cache_obj.set("keys", keys_fn, false, context)?;
        
        Ok(cache_obj)
    }
}