//! Cache API implementation
//!
//! The Cache API provides a persistent storage mechanism for Request/Response object pairs.
//! https://developer.mozilla.org/en-US/docs/Web/API/Cache_API

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{JsObject, builtins::JsPromise},
    realm::Realm,
    string::JsString,
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Internal cache entry storing request/response pairs
#[derive(Debug, Clone)]
pub struct CacheEntry {
    url: String,
    body: Vec<u8>,
    headers: HashMap<String, String>,
    status: u16,
    status_text: String,
}

/// The Cache data implementation
#[derive(Debug, Trace, Finalize, JsData)]
pub struct CacheData {
    #[unsafe_ignore_trace]
    name: String,
    #[unsafe_ignore_trace]
    entries: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl CacheData {
    pub fn new(name: String) -> Self {
        Self {
            name,
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn has(&self, url: &str) -> bool {
        self.entries.lock().unwrap().contains_key(url)
    }

    pub fn get(&self, url: &str) -> Option<CacheEntry> {
        self.entries.lock().unwrap().get(url).cloned()
    }

    pub fn put(&self, url: String, entry: CacheEntry) {
        self.entries.lock().unwrap().insert(url, entry);
    }

    pub fn delete(&self, url: &str) -> bool {
        self.entries.lock().unwrap().remove(url).is_some()
    }

    pub fn keys(&self) -> Vec<String> {
        self.entries.lock().unwrap().keys().cloned().collect()
    }
}

/// The `Cache` object
#[derive(Debug, Trace, Finalize)]
pub struct Cache;

impl Cache {
    pub fn create(name: String, context: &mut Context) -> JsResult<JsObject> {
        let data = CacheData::new(name);
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().cache().prototype(),
            data,
        );
        Ok(obj.upcast())
    }

    fn match_request(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("Cache.match called on non-object"))?;

        let data = this_obj.downcast_ref::<CacheData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Cache.match called on non-Cache object")
        })?;

        let request = args.get_or_undefined(0);
        let url = if let Some(req_obj) = request.as_object() {
            req_obj
                .get(js_string!("url"), context)?
                .to_string(context)?
                .to_std_string_escaped()
        } else {
            request.to_string(context)?.to_std_string_escaped()
        };

        let (promise, resolvers) = JsPromise::new_pending(context);

        if let Some(entry) = data.get(&url) {
            let response = JsObject::default(context.intrinsics());
            response.set(
                js_string!("ok"),
                JsValue::from(entry.status >= 200 && entry.status < 300),
                false,
                context,
            )?;
            response.set(
                js_string!("status"),
                JsValue::from(entry.status),
                false,
                context,
            )?;
            response.set(
                js_string!("statusText"),
                js_string!(entry.status_text),
                false,
                context,
            )?;
            response.set(js_string!("url"), js_string!(entry.url), false, context)?;

            let body_str = String::from_utf8_lossy(&entry.body).to_string();
            response.set(js_string!("_body"), js_string!(body_str), false, context)?;

            resolvers
                .resolve
                .call(&JsValue::undefined(), &[response.into()], context)?;
        } else {
            resolvers
                .resolve
                .call(&JsValue::undefined(), &[JsValue::undefined()], context)?;
        }

        Ok(JsValue::from(promise))
    }

    fn match_all(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Cache.matchAll called on non-object")
        })?;

        let data = this_obj.downcast_ref::<CacheData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Cache.matchAll called on non-Cache object")
        })?;

        let url_filter = if args.is_empty() || args.get_or_undefined(0).is_undefined() {
            None
        } else {
            let request = args.get_or_undefined(0);
            Some(if let Some(req_obj) = request.as_object() {
                req_obj
                    .get(js_string!("url"), context)?
                    .to_string(context)?
                    .to_std_string_escaped()
            } else {
                request.to_string(context)?.to_std_string_escaped()
            })
        };

        let (promise, resolvers) = JsPromise::new_pending(context);

        let results = boa_engine::builtins::array::Array::array_create(0, None, context)?;
        let mut idx = 0u64;

        for url in data.keys() {
            if let Some(ref filter) = url_filter {
                if !url.starts_with(filter) {
                    continue;
                }
            }

            if let Some(entry) = data.get(&url) {
                let response = JsObject::default(context.intrinsics());
                response.set(
                    js_string!("ok"),
                    JsValue::from(entry.status >= 200 && entry.status < 300),
                    false,
                    context,
                )?;
                response.set(
                    js_string!("status"),
                    JsValue::from(entry.status),
                    false,
                    context,
                )?;
                response.set(js_string!("url"), js_string!(entry.url), false, context)?;

                let body_str = String::from_utf8_lossy(&entry.body).to_string();
                response.set(js_string!("_body"), js_string!(body_str), false, context)?;

                results.create_data_property_or_throw(idx, response, context)?;
                idx += 1;
            }
        }

        resolvers
            .resolve
            .call(&JsValue::undefined(), &[results.into()], context)?;
        Ok(JsValue::from(promise))
    }

    fn add(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("Cache.add called on non-object"))?;

        let data = this_obj.downcast_ref::<CacheData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Cache.add called on non-Cache object")
        })?;

        let request = args.get_or_undefined(0);
        let url = if let Some(req_obj) = request.as_object() {
            req_obj
                .get(js_string!("url"), context)?
                .to_string(context)?
                .to_std_string_escaped()
        } else {
            request.to_string(context)?.to_std_string_escaped()
        };

        // In a real implementation, we would fetch the URL
        let entry = CacheEntry {
            url: url.clone(),
            body: Vec::new(),
            headers: HashMap::new(),
            status: 200,
            status_text: "OK".to_string(),
        };

        data.put(url, entry);

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers
            .resolve
            .call(&JsValue::undefined(), &[JsValue::undefined()], context)?;
        Ok(JsValue::from(promise))
    }

    fn add_all(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers
            .resolve
            .call(&JsValue::undefined(), &[JsValue::undefined()], context)?;
        Ok(JsValue::from(promise))
    }

    fn put(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("Cache.put called on non-object"))?;

        let data = this_obj.downcast_ref::<CacheData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Cache.put called on non-Cache object")
        })?;

        let request = args.get_or_undefined(0);
        let response = args.get_or_undefined(1);

        let url = if let Some(req_obj) = request.as_object() {
            req_obj
                .get(js_string!("url"), context)?
                .to_string(context)?
                .to_std_string_escaped()
        } else {
            request.to_string(context)?.to_std_string_escaped()
        };

        let (status, status_text, body) = if let Some(resp_obj) = response.as_object() {
            let status = resp_obj
                .get(js_string!("status"), context)?
                .to_u32(context)? as u16;
            let status_text = resp_obj
                .get(js_string!("statusText"), context)?
                .to_string(context)?
                .to_std_string_escaped();
            let body_val = resp_obj.get(js_string!("_body"), context)?;
            let body = if body_val.is_undefined() {
                Vec::new()
            } else {
                body_val
                    .to_string(context)?
                    .to_std_string_escaped()
                    .into_bytes()
            };
            (status, status_text, body)
        } else {
            (200, "OK".to_string(), Vec::new())
        };

        let entry = CacheEntry {
            url: url.clone(),
            body,
            headers: HashMap::new(),
            status,
            status_text,
        };

        data.put(url, entry);

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers
            .resolve
            .call(&JsValue::undefined(), &[JsValue::undefined()], context)?;
        Ok(JsValue::from(promise))
    }

    fn delete(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Cache.delete called on non-object")
        })?;

        let data = this_obj.downcast_ref::<CacheData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Cache.delete called on non-Cache object")
        })?;

        let request = args.get_or_undefined(0);
        let url = if let Some(req_obj) = request.as_object() {
            req_obj
                .get(js_string!("url"), context)?
                .to_string(context)?
                .to_std_string_escaped()
        } else {
            request.to_string(context)?.to_std_string_escaped()
        };

        let deleted = data.delete(&url);

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers
            .resolve
            .call(&JsValue::undefined(), &[JsValue::from(deleted)], context)?;
        Ok(JsValue::from(promise))
    }

    fn keys(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("Cache.keys called on non-object"))?;

        let data = this_obj.downcast_ref::<CacheData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Cache.keys called on non-Cache object")
        })?;

        let urls = data.keys();

        let (promise, resolvers) = JsPromise::new_pending(context);

        let results =
            boa_engine::builtins::array::Array::array_create(urls.len() as u64, None, context)?;

        for (idx, url) in urls.iter().enumerate() {
            let req = JsObject::default(context.intrinsics());
            req.set(js_string!("url"), js_string!(url.as_str()), false, context)?;
            results.create_data_property_or_throw(idx, req, context)?;
        }

        resolvers
            .resolve
            .call(&JsValue::undefined(), &[results.into()], context)?;
        Ok(JsValue::from(promise))
    }
}

impl IntrinsicObject for Cache {
    fn init(realm: &Realm) {
        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::match_request, js_string!("match"), 1)
            .method(Self::match_all, js_string!("matchAll"), 0)
            .method(Self::add, js_string!("add"), 1)
            .method(Self::add_all, js_string!("addAll"), 1)
            .method(Self::put, js_string!("put"), 2)
            .method(Self::delete, js_string!("delete"), 1)
            .method(Self::keys, js_string!("keys"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Cache {
    const NAME: JsString = js_string!("Cache");
}

impl BuiltInConstructor for Cache {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::cache;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("Cache cannot be constructed directly")
            .into())
    }
}

// ============================================================================
// CacheStorage Implementation
// ============================================================================

#[derive(Debug, Trace, Finalize, JsData)]
pub struct CacheStorageData {
    #[unsafe_ignore_trace]
    caches: Arc<Mutex<HashMap<String, JsObject>>>,
}

impl CacheStorageData {
    pub fn new() -> Self {
        Self {
            caches: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn has(&self, name: &str) -> bool {
        self.caches.lock().unwrap().contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<JsObject> {
        self.caches.lock().unwrap().get(name).cloned()
    }

    pub fn set(&self, name: String, cache: JsObject) {
        self.caches.lock().unwrap().insert(name, cache);
    }

    pub fn delete(&self, name: &str) -> bool {
        self.caches.lock().unwrap().remove(name).is_some()
    }

    pub fn keys(&self) -> Vec<String> {
        self.caches.lock().unwrap().keys().cloned().collect()
    }
}

#[derive(Debug, Trace, Finalize)]
pub struct CacheStorage;

impl CacheStorage {
    pub fn create(context: &mut Context) -> JsResult<JsObject> {
        let data = CacheStorageData::new();
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .constructors()
                .cache_storage()
                .prototype(),
            data,
        );
        Ok(obj.upcast())
    }

    fn open(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CacheStorage.open called on non-object")
        })?;

        let data = this_obj.downcast_ref::<CacheStorageData>().ok_or_else(|| {
            JsNativeError::typ().with_message("CacheStorage.open called on non-CacheStorage object")
        })?;

        let cache_name = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();

        let cache = if let Some(existing) = data.get(&cache_name) {
            existing
        } else {
            let new_cache = Cache::create(cache_name.clone(), context)?;
            data.set(cache_name, new_cache.clone());
            new_cache
        };

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers
            .resolve
            .call(&JsValue::undefined(), &[cache.into()], context)?;
        Ok(JsValue::from(promise))
    }

    fn has(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CacheStorage.has called on non-object")
        })?;

        let data = this_obj.downcast_ref::<CacheStorageData>().ok_or_else(|| {
            JsNativeError::typ().with_message("CacheStorage.has called on non-CacheStorage object")
        })?;

        let cache_name = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        let exists = data.has(&cache_name);

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers
            .resolve
            .call(&JsValue::undefined(), &[JsValue::from(exists)], context)?;
        Ok(JsValue::from(promise))
    }

    fn delete(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CacheStorage.delete called on non-object")
        })?;

        let data = this_obj.downcast_ref::<CacheStorageData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("CacheStorage.delete called on non-CacheStorage object")
        })?;

        let cache_name = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        let deleted = data.delete(&cache_name);

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers
            .resolve
            .call(&JsValue::undefined(), &[JsValue::from(deleted)], context)?;
        Ok(JsValue::from(promise))
    }

    fn keys(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CacheStorage.keys called on non-object")
        })?;

        let data = this_obj.downcast_ref::<CacheStorageData>().ok_or_else(|| {
            JsNativeError::typ().with_message("CacheStorage.keys called on non-CacheStorage object")
        })?;

        let cache_names = data.keys();

        let (promise, resolvers) = JsPromise::new_pending(context);

        let results = boa_engine::builtins::array::Array::array_create(
            cache_names.len() as u64,
            None,
            context,
        )?;

        for (idx, name) in cache_names.iter().enumerate() {
            results.create_data_property_or_throw(idx, js_string!(name.as_str()), context)?;
        }

        resolvers
            .resolve
            .call(&JsValue::undefined(), &[results.into()], context)?;
        Ok(JsValue::from(promise))
    }

    fn match_request(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("CacheStorage.match called on non-object")
        })?;

        let data = this_obj.downcast_ref::<CacheStorageData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("CacheStorage.match called on non-CacheStorage object")
        })?;

        let request = args.get_or_undefined(0);
        let url = if let Some(req_obj) = request.as_object() {
            req_obj
                .get(js_string!("url"), context)?
                .to_string(context)?
                .to_std_string_escaped()
        } else {
            request.to_string(context)?.to_std_string_escaped()
        };

        let (promise, resolvers) = JsPromise::new_pending(context);

        // Search all caches for a match
        let mut found_response: Option<JsObject> = None;
        for cache_name in data.keys() {
            if let Some(cache_obj) = data.get(&cache_name) {
                if let Some(cache_data) = cache_obj.downcast_ref::<CacheData>() {
                    if let Some(entry) = cache_data.get(&url) {
                        let response = JsObject::default(context.intrinsics());
                        response.set(
                            js_string!("ok"),
                            JsValue::from(entry.status >= 200 && entry.status < 300),
                            false,
                            context,
                        )?;
                        response.set(
                            js_string!("status"),
                            JsValue::from(entry.status),
                            false,
                            context,
                        )?;
                        response.set(js_string!("url"), js_string!(entry.url), false, context)?;

                        let body_str = String::from_utf8_lossy(&entry.body).to_string();
                        response.set(js_string!("_body"), js_string!(body_str), false, context)?;

                        found_response = Some(response);
                        break;
                    }
                }
            }
        }

        let result = found_response
            .map(|r| JsValue::from(r))
            .unwrap_or(JsValue::undefined());
        resolvers
            .resolve
            .call(&JsValue::undefined(), &[result], context)?;
        Ok(JsValue::from(promise))
    }
}

impl IntrinsicObject for CacheStorage {
    fn init(realm: &Realm) {
        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::open, js_string!("open"), 1)
            .method(Self::has, js_string!("has"), 1)
            .method(Self::delete, js_string!("delete"), 1)
            .method(Self::keys, js_string!("keys"), 0)
            .method(Self::match_request, js_string!("match"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for CacheStorage {
    const NAME: JsString = js_string!("CacheStorage");
}

impl BuiltInConstructor for CacheStorage {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::cache_storage;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("CacheStorage cannot be constructed directly")
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_data_creation() {
        let data = CacheData::new("test".to_string());
        assert_eq!(data.name(), "test");
        assert_eq!(data.keys().len(), 0);
    }

    #[test]
    fn test_cache_storage_data_creation() {
        let data = CacheStorageData::new();
        assert!(!data.has("test"));
    }
}
