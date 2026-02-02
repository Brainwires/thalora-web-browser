use anyhow::{anyhow, Result};
use thalora_browser_apis::boa_engine::{js_string, property::Attribute, Context, JsObject, JsValue, NativeFunction};
use rquest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vfs::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Real Service Worker implementation for PWA support and modern web app compatibility
pub struct ServiceWorkerManager {
    registrations: Arc<Mutex<HashMap<String, ServiceWorkerRegistration>>>,
    active_workers: Arc<Mutex<HashMap<String, ServiceWorker>>>,
    cache_storage: Arc<Mutex<HashMap<String, CacheStorage>>>,
    http_client: Client,
    storage_dir: PathBuf,
    push_subscriptions: Arc<Mutex<Vec<PushSubscription>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceWorkerRegistration {
    pub scope: String,
    pub script_url: String,
    pub state: RegistrationState,
    pub update_via_cache: String,
    pub last_update_check: u64,
    pub install_time: u64,
    pub active_time: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceWorker {
    pub script_url: String,
    pub state: WorkerState,
    pub script_content: Option<String>,
    pub last_execution: Option<u64>,
    pub error_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistrationState {
    Installing,
    Waiting,
    Active,
    Redundant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerState {
    Parsed,
    Installing,
    Installed,
    Activating,
    Activated,
    Redundant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStorage {
    pub name: String,
    pub entries: HashMap<String, CacheEntry>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub url: String,
    pub response_body: Vec<u8>,
    pub headers: HashMap<String, String>,
    pub status: u16,
    pub cached_at: u64,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscription {
    pub endpoint: String,
    pub keys: PushKeys,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushKeys {
    pub p256dh: String,
    pub auth: String,
}

impl ServiceWorkerManager {
    pub fn new() -> Result<Self> {
        let storage_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("thalora")
            .join("service_workers");

        fs::create_dir_all(&storage_dir)
            .map_err(|e| anyhow!("Failed to create storage directory: {}", e))?;

        let mut instance = Self {
            registrations: Arc::new(Mutex::new(HashMap::new())),
            active_workers: Arc::new(Mutex::new(HashMap::new())),
            cache_storage: Arc::new(Mutex::new(HashMap::new())),
            http_client: Client::new(),
            storage_dir,
            push_subscriptions: Arc::new(Mutex::new(Vec::new())),
        };

        // Load persistent data
        instance.load_persistent_data()?;
        Ok(instance)
    }

    /// Load persistent Service Worker data from disk
    fn load_persistent_data(&mut self) -> Result<()> {
        // Load registrations
        let registrations_file = self.storage_dir.join("registrations.json");
        if let Ok(data) = fs::read_to_string(&registrations_file) {
            if let Ok(registrations) =
                serde_json::from_str::<HashMap<String, ServiceWorkerRegistration>>(&data)
            {
                *self.registrations.lock().unwrap() = registrations;
            }
        }

        // Load workers
        let workers_file = self.storage_dir.join("workers.json");
        if let Ok(data) = fs::read_to_string(&workers_file) {
            if let Ok(workers) = serde_json::from_str::<HashMap<String, ServiceWorker>>(&data) {
                *self.active_workers.lock().unwrap() = workers;
            }
        }

        // Load cache storage
        let cache_file = self.storage_dir.join("cache.json");
        if let Ok(data) = fs::read_to_string(&cache_file) {
            if let Ok(cache) = serde_json::from_str::<HashMap<String, CacheStorage>>(&data) {
                *self.cache_storage.lock().unwrap() = cache;
            }
        }

        Ok(())
    }

    /// Save persistent Service Worker data to disk
    fn save_persistent_data(&self) -> Result<()> {
        // Save registrations
        let registrations_file = self.storage_dir.join("registrations.json");
        let registrations_data = self.registrations.lock().unwrap().clone();
        if let Ok(json_data) = serde_json::to_string_pretty(&registrations_data) {
            fs::write(&registrations_file, json_data)?;
        }

        // Save workers
        let workers_file = self.storage_dir.join("workers.json");
        let workers_data = self.active_workers.lock().unwrap().clone();
        if let Ok(json_data) = serde_json::to_string_pretty(&workers_data) {
            fs::write(&workers_file, json_data)?;
        }

        // Save cache storage
        let cache_file = self.storage_dir.join("cache.json");
        let cache_data = self.cache_storage.lock().unwrap().clone();
        if let Ok(json_data) = serde_json::to_string_pretty(&cache_data) {
            fs::write(&cache_file, json_data)?;
        }

        Ok(())
    }

    /// Download and execute Service Worker script
    async fn fetch_and_install_worker(&self, script_url: &str) -> Result<String> {
        // Fetch the Service Worker script
        let response = self.http_client.get(script_url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch Service Worker script: {}",
                response.status()
            ));
        }

        let script_content = response.text().await?;

        // Store the script content
        {
            let mut workers = self.active_workers.lock().unwrap();
            if let Some(worker) = workers.get_mut(script_url) {
                worker.script_content = Some(script_content.clone());
                worker.state = WorkerState::Installed;
                worker.last_execution =
                    Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());
            }
        }

        // Execute the script (in a real implementation, this would run in a separate context)
        tracing::info!("Service Worker script downloaded and ready: {}", script_url);

        Ok(script_content)
    }

    /// Setup comprehensive Service Worker API in navigator
    pub fn setup_service_worker_api(
        &self,
        context: &mut Context,
    ) -> Result<(), thalora_browser_apis::boa_engine::JsError> {
        // Get navigator object
        let navigator_obj = context
            .global_object()
            .get(js_string!("navigator"), context)?;

        if let Some(nav_obj) = navigator_obj.as_object() {
            let service_worker_obj = JsObject::default(&context.intrinsics());

            // navigator.serviceWorker.register()
            let register_fn = unsafe {
                NativeFunction::from_closure({
                    let registrations_clone = Arc::clone(&self.registrations);
                    let workers_clone = Arc::clone(&self.active_workers);
                    move |_, args, context| {
                        if args.is_empty() {
                            return Err(thalora_browser_apis::boa_engine::JsNativeError::typ()
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

                        let current_time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();

                        // Store registration and worker
                        {
                            let mut regs = registrations_clone.lock().unwrap();
                            let registration = ServiceWorkerRegistration {
                                scope: scope.clone(),
                                script_url: script_url.clone(),
                                state: RegistrationState::Installing,
                                update_via_cache: "imports".to_string(),
                                last_update_check: current_time,
                                install_time: current_time,
                                active_time: None,
                            };
                            regs.insert(scope.clone(), registration);

                            let mut workers = workers_clone.lock().unwrap();
                            let worker = ServiceWorker {
                                script_url: script_url.clone(),
                                state: WorkerState::Installing,
                                script_content: None,
                                last_execution: None,
                                error_count: 0,
                            };
                            workers.insert(script_url.clone(), worker);
                        }

                        // Return Promise-like object
                        let promise_obj = JsObject::default(&context.intrinsics());
                        let registration_obj = JsObject::default(&context.intrinsics());
                        registration_obj.set(
                            js_string!("scope"),
                            JsValue::from(js_string!(scope)),
                            false,
                            context,
                        )?;
                        registration_obj.set(
                            js_string!("scriptURL"),
                            JsValue::from(js_string!(script_url)),
                            false,
                            context,
                        )?;

                        let then_fn = NativeFunction::from_closure(move |_, args, _context| {
                            if !args.is_empty() && args[0].is_callable() {
                                let callback = args[0].as_callable().unwrap();
                                callback.call(
                                    &JsValue::undefined(),
                                    &[JsValue::from(registration_obj.clone())],
                                    _context,
                                )?;
                            }
                            Ok(JsValue::undefined())
                        });
                        promise_obj.set(
                            js_string!("then"),
                            JsValue::from(then_fn.to_js_function(context.realm())),
                            false,
                            context,
                        )?;

                        Ok(JsValue::from(promise_obj))
                    }
                })
            };
            service_worker_obj.set(
                js_string!("register"),
                JsValue::from(register_fn.to_js_function(context.realm())),
                false,
                context,
            )?;

            // navigator.serviceWorker.ready (Promise)
            let ready_promise = JsObject::default(&context.intrinsics());
            let ready_then_fn = unsafe {
                NativeFunction::from_closure(|_, args, _context| {
                    if !args.is_empty() && args[0].is_callable() {
                        let callback = args[0].as_callable().unwrap();
                        let registration_obj = JsObject::default(&_context.intrinsics());
                        registration_obj.set(
                            js_string!("scope"),
                            JsValue::from(js_string!("/")),
                            false,
                            _context,
                        )?;
                        callback.call(
                            &JsValue::undefined(),
                            &[JsValue::from(registration_obj)],
                            _context,
                        )?;
                    }
                    Ok(JsValue::undefined())
                })
            };
            ready_promise.set(
                js_string!("then"),
                JsValue::from(ready_then_fn.to_js_function(context.realm())),
                false,
                context,
            )?;
            service_worker_obj.set(
                js_string!("ready"),
                JsValue::from(ready_promise),
                false,
                context,
            )?;

            // navigator.serviceWorker.controller
            service_worker_obj.set(js_string!("controller"), JsValue::null(), false, context)?;

            nav_obj.set(
                js_string!("serviceWorker"),
                JsValue::from(service_worker_obj),
                false,
                context,
            )?;
        }

        // Setup Cache API
        self.setup_cache_api(context)?;

        // Setup Push API
        self.setup_push_api(context)?;

        Ok(())
    }

    /// Setup Cache API for Service Workers
    fn setup_cache_api(&self, context: &mut Context) -> Result<(), thalora_browser_apis::boa_engine::JsError> {
        let caches_obj = JsObject::default(&context.intrinsics());

        // caches.open(cacheName)
        let cache_storage_clone = Arc::clone(&self.cache_storage);
        let open_fn = unsafe {
            NativeFunction::from_closure(move |_, args, context| {
                if args.is_empty() {
                    return Err(thalora_browser_apis::boa_engine::JsNativeError::typ()
                        .with_message("caches.open() requires a cache name")
                        .into());
                }

                let cache_name = args[0].to_string(context)?.to_std_string_escaped();
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                {
                    let mut caches = cache_storage_clone.lock().unwrap();
                    if !caches.contains_key(&cache_name) {
                        caches.insert(
                            cache_name.clone(),
                            CacheStorage {
                                name: cache_name.clone(),
                                entries: HashMap::new(),
                                created_at: current_time,
                            },
                        );
                    }
                }

                // Return Cache object
                let cache_obj = JsObject::default(&context.intrinsics());

                // cache.add(request)
                let cache_storage_add = Arc::clone(&cache_storage_clone);
                let cache_name_add = cache_name.clone();
                let add_fn = NativeFunction::from_closure(move |_, args, _context| {
                    if !args.is_empty() {
                        let url = args[0].to_string(_context)?.to_std_string_escaped();
                        tracing::info!("Cache add: {} to cache {}", url, cache_name_add);

                        // In real implementation, would fetch and store the resource
                        let mut caches = cache_storage_add.lock().unwrap();
                        if let Some(cache) = caches.get_mut(&cache_name_add) {
                            cache.entries.insert(
                                url.clone(),
                                CacheEntry {
                                    url: url.clone(),
                                    response_body: b"cached content".to_vec(),
                                    headers: HashMap::new(),
                                    status: 200,
                                    cached_at: SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs(),
                                    expires_at: None,
                                },
                            );
                        }
                    }
                    Ok(JsValue::undefined())
                });
                cache_obj.set(
                    js_string!("add"),
                    JsValue::from(add_fn.to_js_function(context.realm())),
                    false,
                    context,
                )?;

                // cache.match(request)
                let cache_storage_match = Arc::clone(&cache_storage_clone);
                let cache_name_match = cache_name.clone();
                let match_fn = NativeFunction::from_closure(move |_, args, context| {
                    if !args.is_empty() {
                        let url = args[0].to_string(context)?.to_std_string_escaped();

                        let caches = cache_storage_match.lock().unwrap();
                        if let Some(cache) = caches.get(&cache_name_match) {
                            if let Some(entry) = cache.entries.get(&url) {
                                // Return Response object
                                let response_obj = JsObject::default(&context.intrinsics());
                                response_obj.set(
                                    js_string!("status"),
                                    JsValue::from(entry.status),
                                    false,
                                    context,
                                )?;
                                response_obj.set(
                                    js_string!("ok"),
                                    JsValue::from(entry.status >= 200 && entry.status < 300),
                                    false,
                                    context,
                                )?;

                                let promise_obj = JsObject::default(&context.intrinsics());
                                let then_fn =
                                    NativeFunction::from_closure(move |_, args, _context| {
                                        if !args.is_empty() && args[0].is_callable() {
                                            let callback = args[0].as_callable().unwrap();
                                            callback.call(
                                                &JsValue::undefined(),
                                                &[JsValue::from(response_obj.clone())],
                                                _context,
                                            )?;
                                        }
                                        Ok(JsValue::undefined())
                                    });
                                promise_obj.set(
                                    js_string!("then"),
                                    JsValue::from(then_fn.to_js_function(context.realm())),
                                    false,
                                    context,
                                )?;
                                return Ok(JsValue::from(promise_obj));
                            }
                        }
                    }

                    // Return undefined for cache miss
                    let promise_obj = JsObject::default(&context.intrinsics());
                    let then_fn = NativeFunction::from_closure(move |_, args, _context| {
                        if !args.is_empty() && args[0].is_callable() {
                            let callback = args[0].as_callable().unwrap();
                            callback.call(
                                &JsValue::undefined(),
                                &[JsValue::undefined()],
                                _context,
                            )?;
                        }
                        Ok(JsValue::undefined())
                    });
                    promise_obj.set(
                        js_string!("then"),
                        JsValue::from(then_fn.to_js_function(context.realm())),
                        false,
                        context,
                    )?;
                    Ok(JsValue::from(promise_obj))
                });
                cache_obj.set(
                    js_string!("match"),
                    JsValue::from(match_fn.to_js_function(context.realm())),
                    false,
                    context,
                )?;

                let promise_obj = JsObject::default(&context.intrinsics());
                let then_fn = NativeFunction::from_closure(move |_, args, _context| {
                    if !args.is_empty() && args[0].is_callable() {
                        let callback = args[0].as_callable().unwrap();
                        callback.call(
                            &JsValue::undefined(),
                            &[JsValue::from(cache_obj.clone())],
                            _context,
                        )?;
                    }
                    Ok(JsValue::undefined())
                });
                promise_obj.set(
                    js_string!("then"),
                    JsValue::from(then_fn.to_js_function(context.realm())),
                    false,
                    context,
                )?;
                Ok(JsValue::from(promise_obj))
            })
        };
        caches_obj.set(
            js_string!("open"),
            JsValue::from(open_fn.to_js_function(context.realm())),
            false,
            context,
        )?;

        context.register_global_property(
            js_string!("caches"),
            JsValue::from(caches_obj),
            Attribute::all(),
        )?;
        Ok(())
    }

    /// Setup Push API for notifications
    fn setup_push_api(&self, context: &mut Context) -> Result<(), thalora_browser_apis::boa_engine::JsError> {
        // Setup PushManager
        let push_manager_obj = JsObject::default(&context.intrinsics());

        // pushManager.subscribe()
        let push_subscriptions_clone = Arc::clone(&self.push_subscriptions);
        let subscribe_fn = unsafe {
            NativeFunction::from_closure(move |_, _args, context| {
                // MOCK: Creates fake push subscription with hardcoded FCM endpoint
                let subscription_obj = JsObject::default(&context.intrinsics());
                subscription_obj.set(
                    js_string!("endpoint"),
                    JsValue::from(js_string!(
                        "https://fcm.googleapis.com/fcm/send/mock-endpoint" // MOCK endpoint
                    )),
                    false,
                    context,
                )?;

                let keys_obj = JsObject::default(&context.intrinsics());
                keys_obj.set(
                    js_string!("p256dh"),
                    JsValue::from(js_string!("mock-p256dh-key")), // MOCK key
                    false,
                    context,
                )?;
                keys_obj.set(
                    js_string!("auth"),
                    JsValue::from(js_string!("mock-auth-key")), // MOCK auth key
                    false,
                    context,
                )?;
                subscription_obj.set(
                    js_string!("keys"),
                    JsValue::from(keys_obj),
                    false,
                    context,
                )?;

                // Store subscription
                {
                    let mut subscriptions = push_subscriptions_clone.lock().unwrap();
                    subscriptions.push(PushSubscription {
                        endpoint: "https://fcm.googleapis.com/fcm/send/mock-endpoint".to_string(), // MOCK
                        keys: PushKeys {
                            p256dh: "mock-p256dh-key".to_string(), // MOCK
                            auth: "mock-auth-key".to_string(), // MOCK
                        },
                        created_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    });
                }

                let promise_obj = JsObject::default(&context.intrinsics());
                let then_fn = NativeFunction::from_closure(move |_, args, _context| {
                    if !args.is_empty() && args[0].is_callable() {
                        let callback = args[0].as_callable().unwrap();
                        callback.call(
                            &JsValue::undefined(),
                            &[JsValue::from(subscription_obj.clone())],
                            _context,
                        )?;
                    }
                    Ok(JsValue::undefined())
                });
                promise_obj.set(
                    js_string!("then"),
                    JsValue::from(then_fn.to_js_function(context.realm())),
                    false,
                    context,
                )?;
                Ok(JsValue::from(promise_obj))
            })
        };
        push_manager_obj.set(
            js_string!("subscribe"),
            JsValue::from(subscribe_fn.to_js_function(context.realm())),
            false,
            context,
        )?;

        // Get navigator and add pushManager
        let navigator_obj = context
            .global_object()
            .get(js_string!("navigator"), context)?;
        if let Some(nav_obj) = navigator_obj.as_object() {
            nav_obj.set(
                js_string!("pushManager"),
                JsValue::from(push_manager_obj),
                false,
                context,
            )?;
        }

        Ok(())
    }

    /// Get all active registrations
    pub fn get_active_registrations(&self) -> Vec<ServiceWorkerRegistration> {
        self.registrations
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// Unregister a Service Worker
    pub async fn unregister(&self, scope: &str) -> Result<bool> {
        let removed = {
            let mut registrations = self.registrations.lock().unwrap();
            registrations.remove(scope).is_some()
        };

        if removed {
            self.save_persistent_data()?;
            tracing::info!("Service Worker unregistered: {}", scope);
        }

        Ok(removed)
    }

    /// Update Service Worker registration
    pub async fn update_registration(&self, scope: &str) -> Result<bool> {
        let script_url = {
            let registrations = self.registrations.lock().unwrap();
            registrations.get(scope).map(|r| r.script_url.clone())
        };

        if let Some(url) = script_url {
            // Fetch the latest script
            match self.fetch_and_install_worker(&url).await {
                Ok(_) => {
                    // Update registration timestamp
                    {
                        let mut registrations = self.registrations.lock().unwrap();
                        if let Some(registration) = registrations.get_mut(scope) {
                            registration.last_update_check =
                                SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                        }
                    }
                    self.save_persistent_data()?;
                    Ok(true)
                }
                Err(e) => {
                    tracing::error!("Failed to update Service Worker {}: {}", scope, e);
                    Ok(false)
                }
            }
        } else {
            Ok(false)
        }
    }

    /// Clear all caches
    pub async fn clear_all_caches(&self) -> Result<()> {
        {
            let mut caches = self.cache_storage.lock().unwrap();
            caches.clear();
        }
        self.save_persistent_data()?;
        tracing::info!("All Service Worker caches cleared");
        Ok(())
    }
}
