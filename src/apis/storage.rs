use anyhow::Result;
use boa_engine::{Context, JsObject, JsValue, NativeFunction, js_string, property::Attribute};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::fs;
use serde_json;

/// Real WebStorage implementation with persistent localStorage and sessionStorage
pub struct WebStorage {
    local_storage: Arc<Mutex<HashMap<String, String>>>,
    session_storage: Arc<Mutex<HashMap<String, String>>>,
    storage_dir: PathBuf,
}

impl WebStorage {
    pub fn new() -> Self {
        let storage_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("thalora")
            .join("storage");

        fs::create_dir_all(&storage_dir).ok();

        let instance = Self {
            local_storage: Arc::new(Mutex::new(HashMap::new())),
            session_storage: Arc::new(Mutex::new(HashMap::new())),
            storage_dir,
        };

        // Load persistent localStorage from disk
        instance.load_local_storage();
        instance
    }

    /// Setup real localStorage and sessionStorage APIs
    pub fn setup_storage_globals(&self, context: &mut Context) -> Result<()> {
        self.setup_local_storage(context).map_err(|e| anyhow::Error::msg(format!("localStorage setup failed: {:?}", e)))?;
        self.setup_session_storage(context).map_err(|e| anyhow::Error::msg(format!("sessionStorage setup failed: {:?}", e)))?;
        Ok(())
    }

    fn load_local_storage(&self) {
        let local_storage_file = self.storage_dir.join("localStorage.json");
        if let Ok(data) = fs::read_to_string(&local_storage_file) {
            if let Ok(storage_data) = serde_json::from_str::<HashMap<String, String>>(&data) {
                *self.local_storage.lock().unwrap() = storage_data;
            }
        }
    }

    fn save_local_storage(&self) {
        let local_storage_file = self.storage_dir.join("localStorage.json");
        let storage_data = self.local_storage.lock().unwrap().clone();
        if let Ok(json_data) = serde_json::to_string_pretty(&storage_data) {
            fs::write(&local_storage_file, json_data).ok();
        }
    }

    fn setup_local_storage(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        let storage_obj = JsObject::default();
        let local_storage_data = Arc::clone(&self.local_storage);
        let storage_dir = self.storage_dir.clone();

        // localStorage.setItem(key, value)
        let set_item_data = Arc::clone(&local_storage_data);
        let set_item_dir = storage_dir.clone();
        let set_item_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            if args.len() >= 2 {
                let key = args[0].to_string(context)?.to_std_string_escaped();
                let value = args[1].to_string(context)?.to_std_string_escaped();

                set_item_data.lock().unwrap().insert(key, value);

                // Persist to disk
                let local_storage_file = set_item_dir.join("localStorage.json");
                let storage_data = set_item_data.lock().unwrap().clone();
                if let Ok(json_data) = serde_json::to_string_pretty(&storage_data) {
                    fs::write(&local_storage_file, json_data).ok();
                }
            }
            Ok(JsValue::undefined())
        }) };
        storage_obj.set(js_string!("setItem"), JsValue::from(set_item_fn.to_js_function(context.realm())), false, context)?;

        // localStorage.getItem(key)
        let get_item_data = Arc::clone(&local_storage_data);
        let get_item_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            if !args.is_empty() {
                let key = args[0].to_string(context)?.to_std_string_escaped();
                if let Some(value) = get_item_data.lock().unwrap().get(&key) {
                    return Ok(JsValue::from(js_string!(value.clone())));
                }
            }
            Ok(JsValue::null())
        }) };
        storage_obj.set(js_string!("getItem"), JsValue::from(get_item_fn.to_js_function(context.realm())), false, context)?;

        // localStorage.removeItem(key)
        let remove_item_data = Arc::clone(&local_storage_data);
        let remove_item_dir = storage_dir.clone();
        let remove_item_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            if !args.is_empty() {
                let key = args[0].to_string(context)?.to_std_string_escaped();
                remove_item_data.lock().unwrap().remove(&key);

                // Persist to disk
                let local_storage_file = remove_item_dir.join("localStorage.json");
                let storage_data = remove_item_data.lock().unwrap().clone();
                if let Ok(json_data) = serde_json::to_string_pretty(&storage_data) {
                    fs::write(&local_storage_file, json_data).ok();
                }
            }
            Ok(JsValue::undefined())
        }) };
        storage_obj.set(js_string!("removeItem"), JsValue::from(remove_item_fn.to_js_function(context.realm())), false, context)?;

        // localStorage.clear()
        let clear_data = Arc::clone(&local_storage_data);
        let clear_dir = storage_dir.clone();
        let clear_fn = unsafe { NativeFunction::from_closure(move |_, _args, _context| {
            clear_data.lock().unwrap().clear();

            // Persist to disk
            let local_storage_file = clear_dir.join("localStorage.json");
            fs::write(&local_storage_file, "{}").ok();

            Ok(JsValue::undefined())
        }) };
        storage_obj.set(js_string!("clear"), JsValue::from(clear_fn.to_js_function(context.realm())), false, context)?;

        // localStorage.length
        let length_data = Arc::clone(&local_storage_data);
        let length_fn = unsafe { NativeFunction::from_closure(move |_, _args, _context| {
            let len = length_data.lock().unwrap().len() as f64;
            Ok(JsValue::from(len))
        }) };
        storage_obj.set(js_string!("length"), JsValue::from(length_fn.to_js_function(context.realm())), false, context)?;

        // localStorage.key(index)
        let key_data = Arc::clone(&local_storage_data);
        let key_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            if !args.is_empty() {
                if let Ok(index) = args[0].to_u32(context) {
                    let keys: Vec<String> = key_data.lock().unwrap().keys().cloned().collect();
                    if let Some(key) = keys.get(index as usize) {
                        return Ok(JsValue::from(js_string!(key.clone())));
                    }
                }
            }
            Ok(JsValue::null())
        }) };
        storage_obj.set(js_string!("key"), JsValue::from(key_fn.to_js_function(context.realm())), false, context)?;

        context.register_global_property(js_string!("localStorage"), JsValue::from(storage_obj), Attribute::all())?;
        Ok(())
    }

    fn setup_session_storage(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        let storage_obj = JsObject::default();
        let session_storage_data = Arc::clone(&self.session_storage);

        // sessionStorage.setItem(key, value)
        let set_item_data = Arc::clone(&session_storage_data);
        let set_item_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            if args.len() >= 2 {
                let key = args[0].to_string(context)?.to_std_string_escaped();
                let value = args[1].to_string(context)?.to_std_string_escaped();
                set_item_data.lock().unwrap().insert(key, value);
            }
            Ok(JsValue::undefined())
        }) };
        storage_obj.set(js_string!("setItem"), JsValue::from(set_item_fn.to_js_function(context.realm())), false, context)?;

        // sessionStorage.getItem(key)
        let get_item_data = Arc::clone(&session_storage_data);
        let get_item_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            if !args.is_empty() {
                let key = args[0].to_string(context)?.to_std_string_escaped();
                if let Some(value) = get_item_data.lock().unwrap().get(&key) {
                    return Ok(JsValue::from(js_string!(value.clone())));
                }
            }
            Ok(JsValue::null())
        }) };
        storage_obj.set(js_string!("getItem"), JsValue::from(get_item_fn.to_js_function(context.realm())), false, context)?;

        // sessionStorage.removeItem(key)
        let remove_item_data = Arc::clone(&session_storage_data);
        let remove_item_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            if !args.is_empty() {
                let key = args[0].to_string(context)?.to_std_string_escaped();
                remove_item_data.lock().unwrap().remove(&key);
            }
            Ok(JsValue::undefined())
        }) };
        storage_obj.set(js_string!("removeItem"), JsValue::from(remove_item_fn.to_js_function(context.realm())), false, context)?;

        // sessionStorage.clear()
        let clear_data = Arc::clone(&session_storage_data);
        let clear_fn = unsafe { NativeFunction::from_closure(move |_, _args, _context| {
            clear_data.lock().unwrap().clear();
            Ok(JsValue::undefined())
        }) };
        storage_obj.set(js_string!("clear"), JsValue::from(clear_fn.to_js_function(context.realm())), false, context)?;

        // sessionStorage.length
        let length_data = Arc::clone(&session_storage_data);
        let length_fn = unsafe { NativeFunction::from_closure(move |_, _args, _context| {
            let len = length_data.lock().unwrap().len() as f64;
            Ok(JsValue::from(len))
        }) };
        storage_obj.set(js_string!("length"), JsValue::from(length_fn.to_js_function(context.realm())), false, context)?;

        // sessionStorage.key(index)
        let key_data = Arc::clone(&session_storage_data);
        let key_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            if !args.is_empty() {
                if let Ok(index) = args[0].to_u32(context) {
                    let keys: Vec<String> = key_data.lock().unwrap().keys().cloned().collect();
                    if let Some(key) = keys.get(index as usize) {
                        return Ok(JsValue::from(js_string!(key.clone())));
                    }
                }
            }
            Ok(JsValue::null())
        }) };
        storage_obj.set(js_string!("key"), JsValue::from(key_fn.to_js_function(context.realm())), false, context)?;

        context.register_global_property(js_string!("sessionStorage"), JsValue::from(storage_obj), Attribute::all())?;
        Ok(())
    }

    pub fn get_local_storage_data(&self) -> HashMap<String, String> {
        self.local_storage.lock().unwrap().clone()
    }

    pub fn get_session_storage_data(&self) -> HashMap<String, String> {
        self.session_storage.lock().unwrap().clone()
    }

    pub fn clear_storage(&self) {
        self.local_storage.lock().unwrap().clear();
        self.session_storage.lock().unwrap().clear();
    }
}