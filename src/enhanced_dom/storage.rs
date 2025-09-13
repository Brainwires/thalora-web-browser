use anyhow::Result;
use boa_engine::{Context, JsObject, JsValue, NativeFunction, property::Attribute, js_string};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct WebStorage {
    local_storage: Arc<Mutex<HashMap<String, String>>>,
    session_storage: Arc<Mutex<HashMap<String, String>>>,
}

impl WebStorage {
    pub fn new() -> Self {
        Self {
            local_storage: Arc::new(Mutex::new(HashMap::new())),
            session_storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn setup_storage_globals(&self, context: &mut Context) -> Result<()> {
        let local_storage = self.create_storage_object(self.local_storage.clone(), context)?;
        let session_storage = self.create_storage_object(self.session_storage.clone(), context)?;

        context.register_global_property(js_string!("localStorage"), local_storage, Attribute::all(), context)?;
        context.register_global_property(js_string!("sessionStorage"), session_storage, Attribute::all(), context)?;

        Ok(())
    }

    fn create_storage_object(
        &self,
        storage: Arc<Mutex<HashMap<String, String>>>,
        context: &mut Context,
    ) -> Result<JsObject, boa_engine::JsError> {
        let storage_obj = JsObject::default();

        // getItem method
        let storage_clone = storage.clone();
        let get_item_fn = NativeFunction::from_fn_ptr(move |_, args, context| {
            if args.is_empty() {
                return Ok(JsValue::null());
            }

            let key = args[0].to_string(context)?.to_std_string_escaped();
            let storage_guard = storage_clone.lock().unwrap();
            
            match storage_guard.get(&key) {
                Some(value) => Ok(JsValue::from(value.clone())),
                None => Ok(JsValue::null()),
            }
        });
        storage_obj.set(js_string!("getItem"), get_item_fn, Attribute::WRITABLE | Attribute::CONFIGURABLE, context)?;

        // setItem method
        let storage_clone = storage.clone();
        let set_item_fn = NativeFunction::from_fn_ptr(move |_, args, context| {
            if args.len() < 2 {
                return Ok(JsValue::undefined());
            }

            let key = args[0].to_string(context)?.to_std_string_escaped();
            let value = args[1].to_string(context)?.to_std_string_escaped();
            
            let mut storage_guard = storage_clone.lock().unwrap();
            storage_guard.insert(key, value);
            
            Ok(JsValue::undefined())
        });
        storage_obj.set(js_string!("setItem"), set_item_fn, Attribute::WRITABLE | Attribute::CONFIGURABLE, context)?;

        // removeItem method
        let storage_clone = storage.clone();
        let remove_item_fn = NativeFunction::from_fn_ptr(move |_, args, context| {
            if args.is_empty() {
                return Ok(JsValue::undefined());
            }

            let key = args[0].to_string(context)?.to_std_string_escaped();
            let mut storage_guard = storage_clone.lock().unwrap();
            storage_guard.remove(&key);
            
            Ok(JsValue::undefined())
        });
        storage_obj.set(js_string!("removeItem"), remove_item_fn, Attribute::WRITABLE | Attribute::CONFIGURABLE, context)?;

        // clear method
        let storage_clone = storage.clone();
        let clear_fn = NativeFunction::from_fn_ptr(move |_, _, _| {
            let mut storage_guard = storage_clone.lock().unwrap();
            storage_guard.clear();
            Ok(JsValue::undefined())
        });
        storage_obj.set(js_string!("clear"), clear_fn, Attribute::WRITABLE | Attribute::CONFIGURABLE, context)?;

        // length property getter
        let storage_clone = storage.clone();
        let length_getter = NativeFunction::from_fn_ptr(move |_, _, _| {
            let storage_guard = storage_clone.lock().unwrap();
            Ok(JsValue::from(storage_guard.len() as f64))
        });
        storage_obj.define_property_or_throw(
            js_string!("length"),
            boa_engine::property::PropertyDescriptor::builder()
                .get(length_getter)
                .configurable(true)
                .build(),
            context,
        )?;

        // key method
        let storage_clone = storage.clone();
        let key_fn = NativeFunction::from_fn_ptr(move |_, args, context| {
            if args.is_empty() {
                return Ok(JsValue::null());
            }

            let index = args[0].to_number(context)? as usize;
            let storage_guard = storage_clone.lock().unwrap();
            
            let keys: Vec<&String> = storage_guard.keys().collect();
            match keys.get(index) {
                Some(key) => Ok(JsValue::from((*key).clone())),
                None => Ok(JsValue::null()),
            }
        });
        storage_obj.set(js_string!("key"), key_fn, Attribute::WRITABLE | Attribute::CONFIGURABLE, context)?;

        Ok(storage_obj)
    }

    pub fn get_local_storage_data(&self) -> HashMap<String, String> {
        self.local_storage.lock().unwrap().clone()
    }

    pub fn get_session_storage_data(&self) -> HashMap<String, String> {
        self.session_storage.lock().unwrap().clone()
    }

    pub fn clear_all_storage(&self) {
        self.local_storage.lock().unwrap().clear();
        self.session_storage.lock().unwrap().clear();
    }
}