//! Implementation of the `Storage` Web API.
//!
//! The `Storage` interface provides access to the local storage and session storage for a particular domain.
//! It allows you to add, modify or delete stored data items.
//!
//! ## Security
//!
//! Storage data is encrypted at rest using AES-256-GCM with a key derived from the
//! `THALORA_SESSION_SECRET` environment variable or an auto-generated secret.
//! This prevents sensitive data from being stored in plaintext on disk.
//!
//! More information:
//! - [WHATWG Specification](https://html.spec.whatwg.org/multipage/webstorage.html#the-storage-interface)
//! - [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/API/Storage)

use boa_engine::builtins::{BuiltInConstructor, BuiltInObject, IntrinsicObject};
use boa_engine::context::intrinsics::StandardConstructor;
use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, JsValue, builtins::BuiltInBuilder,
    context::intrinsics::Intrinsics, js_string, object::JsObject, property::Attribute,
    realm::Realm,
};
use boa_gc::{Finalize, Trace};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

// Encryption imports
use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use sha2::{Digest, Sha256};

/// Encryption module for Storage API
mod encryption {
    use super::*;

    /// Get the path for the secret file.
    /// Uses platform-appropriate secure location.
    fn get_secret_path() -> PathBuf {
        // Try to use a secure location (native builds only)
        #[cfg(feature = "native")]
        {
            if let Some(data_dir) = dirs::data_local_dir() {
                return data_dir.join("thalora").join(".session_secret");
            }
        }

        // Fallback to temp directory
        std::env::temp_dir().join(".thalora_session_secret")
    }

    /// Derive a 256-bit key from the storage type and session secret.
    fn derive_key(storage_type: &str) -> [u8; 32] {
        // Get secret from environment or use auto-generated fallback
        let secret = std::env::var("THALORA_SESSION_SECRET").unwrap_or_else(|_| {
            // Try to load from persisted secret file
            let secret_path = get_secret_path();

            if let Ok(secret) = fs::read_to_string(&secret_path) {
                let secret = secret.trim().to_string();
                if secret.len() >= 32 {
                    return secret;
                }
            }

            // Generate and save new secret
            let mut bytes = [0u8; 32];
            OsRng.fill_bytes(&mut bytes);
            let new_secret: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();

            // Try to create parent directory and save
            if let Some(parent) = secret_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                if let Ok(mut file) = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .mode(0o600)
                    .open(&secret_path)
                {
                    use std::io::Write;
                    let _ = file.write_all(new_secret.as_bytes());
                }
            }
            #[cfg(not(unix))]
            {
                let _ = fs::write(&secret_path, &new_secret);
            }

            new_secret
        });

        // Derive key using SHA-256(secret || storage_type)
        let mut hasher = Sha256::new();
        hasher.update(secret.as_bytes());
        hasher.update(storage_type.as_bytes());
        let result = hasher.finalize();

        let mut key = [0u8; 32];
        key.copy_from_slice(&result);
        key
    }

    /// Encrypt data using AES-256-GCM.
    ///
    /// Returns: [12-byte nonce][ciphertext with auth tag]
    pub fn encrypt(plaintext: &[u8], storage_type: &str) -> Result<Vec<u8>, String> {
        let key = derive_key(storage_type);
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| format!("Failed to create cipher: {}", e))?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt data using AES-256-GCM.
    ///
    /// Expects: [12-byte nonce][ciphertext with auth tag]
    pub fn decrypt(encrypted: &[u8], storage_type: &str) -> Result<Vec<u8>, String> {
        if encrypted.len() < 12 {
            return Err("Encrypted data too short".to_string());
        }

        let key = derive_key(storage_type);
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| format!("Failed to create cipher: {}", e))?;

        let (nonce_bytes, ciphertext) = encrypted.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| "Decryption failed: invalid key or corrupted data".to_string())
    }
}

/// Serializable storage data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StorageData {
    data: HashMap<String, String>,
}

/// `Storage` implementation for the Web Storage API.
#[derive(Debug, Clone, Finalize)]
pub struct Storage {
    /// The storage data, shared between instances for the same storage type
    data: Arc<RwLock<HashMap<String, String>>>,
    /// Storage type identifier for debugging
    storage_type: &'static str,
    /// Storage path for persistence
    storage_path: PathBuf,
}

// SAFETY: Storage is safe to trace because HashMap<String, String> doesn't contain any GC'd objects
unsafe impl Trace for Storage {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // No GC'd objects in Storage, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC'd objects in Storage, nothing to trace
    }

    fn run_finalizer(&self) {
        // No cleanup needed for Storage
    }
}

impl JsData for Storage {}

impl Storage {
    /// Creates a new `Storage` instance.
    pub(crate) fn new(storage_type: &'static str) -> Self {
        let storage_path = Self::get_storage_path(storage_type);
        let data = Self::load_storage_data(&storage_path, storage_type);
        Self {
            data: Arc::new(RwLock::new(data)),
            storage_type,
            storage_path,
        }
    }

    /// Creates a new `Storage` instance with empty data (for tests).
    pub fn new_empty(storage_type: &'static str) -> Self {
        let storage_path = Self::get_storage_path(storage_type);
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            storage_type,
            storage_path,
        }
    }

    /// Creates a new `Storage` instance with a custom storage path (for tests).
    #[cfg(test)]
    pub fn new_with_path(storage_type: &'static str, custom_path: PathBuf) -> Self {
        let data = Self::load_storage_data(&custom_path, storage_type);
        Self {
            data: Arc::new(RwLock::new(data)),
            storage_type,
            storage_path: custom_path,
        }
    }

    /// Creates a `Storage` instance with pre-populated data.
    pub(crate) fn with_data(data: HashMap<String, String>, storage_type: &'static str) -> Self {
        let storage_path = Self::get_storage_path(storage_type);
        let storage = Self {
            data: Arc::new(RwLock::new(data)),
            storage_type,
            storage_path,
        };
        storage.save_storage_data();
        storage
    }

    /// Gets the number of items in storage.
    #[cfg(test)]
    pub fn length_internal(&self) -> usize {
        self.data.read().unwrap().len()
    }

    #[cfg(not(test))]
    fn length_internal(&self) -> usize {
        self.data.read().unwrap().len()
    }

    /// Gets the key at the specified index.
    fn key_internal(&self, index: usize) -> Option<String> {
        let data = self.data.read().unwrap();
        let keys: Vec<_> = data.keys().collect();
        keys.get(index).map(|s| (*s).clone())
    }

    /// Gets an item from storage by key.
    #[cfg(test)]
    pub fn get_item_internal(&self, key: &str) -> Option<String> {
        self.data.read().unwrap().get(key).cloned()
    }

    #[cfg(not(test))]
    fn get_item_internal(&self, key: &str) -> Option<String> {
        self.data.read().unwrap().get(key).cloned()
    }

    /// Sets an item in storage.
    #[cfg(test)]
    pub fn set_item_internal(&self, key: String, value: String) -> JsResult<()> {
        let _old_value = {
            let data = self.data.read().unwrap();
            data.get(&key).cloned()
        };

        {
            let mut data = self.data.write().unwrap();
            data.insert(key.clone(), value.clone());
        }
        self.save_storage_data();

        // Fire storage event for cross-window communication
        // Note: In a real browser, this would only fire in other windows/contexts
        // For now, we'll implement the event structure for future use
        Ok(())
    }

    #[cfg(not(test))]
    fn set_item_internal(&self, key: String, value: String) -> JsResult<()> {
        let _old_value = {
            let data = self.data.read().unwrap();
            data.get(&key).cloned()
        };

        {
            let mut data = self.data.write().unwrap();
            data.insert(key.clone(), value.clone());
        }
        self.save_storage_data();

        // Fire storage event for cross-window communication
        // Note: In a real browser, this would only fire in other windows/contexts
        // For now, we'll implement the event structure for future use
        Ok(())
    }

    /// Removes an item from storage by key.
    fn remove_item_internal(&self, key: &str) {
        let old_value = {
            let data = self.data.read().unwrap();
            data.get(key).cloned()
        };

        if old_value.is_some() {
            {
                let mut data = self.data.write().unwrap();
                data.remove(key);
            }
            self.save_storage_data();

            // Fire storage event for cross-window communication
            // Note: In a real browser, this would only fire in other windows/contexts
            // Event would have: key=key, oldValue=old_value, newValue=null
        }
    }

    /// Clears all items from storage.
    fn clear_internal(&self) {
        let had_items = {
            let data = self.data.read().unwrap();
            !data.is_empty()
        };

        if had_items {
            {
                let mut data = self.data.write().unwrap();
                data.clear();
            }
            self.save_storage_data();

            // Fire storage event for cross-window communication
            // Note: In a real browser, this would only fire in other windows/contexts
            // Event would have: key=null, oldValue=null, newValue=null (clear operation)
        }
    }

    /// Get the storage path for a storage type.
    /// Uses `.enc` extension to indicate encrypted storage.
    fn get_storage_path(storage_type: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("boa_web_storage");
        if !path.exists() {
            fs::create_dir_all(&path).ok();
        }
        // Use .enc extension to indicate encrypted storage
        path.push(format!("{}.enc", storage_type));
        path
    }

    /// Load storage data from disk with decryption.
    ///
    /// # Security
    ///
    /// Data is decrypted using AES-256-GCM with a key derived from the session secret.
    /// If decryption fails (wrong key, corrupted data), returns empty HashMap.
    fn load_storage_data(
        storage_path: &PathBuf,
        storage_type: &'static str,
    ) -> HashMap<String, String> {
        if storage_path.exists() {
            // Try to read encrypted data
            if let Ok(encrypted_bytes) = fs::read(storage_path) {
                // Try to decrypt
                if let Ok(decrypted) = encryption::decrypt(&encrypted_bytes, storage_type) {
                    // Try to deserialize
                    if let Ok(storage_data) = serde_json::from_slice::<StorageData>(&decrypted) {
                        return storage_data.data;
                    }
                }
            }

            // Try legacy unencrypted format for migration
            let legacy_path = storage_path.with_extension("json");
            if legacy_path.exists()
                && let Ok(content) = fs::read_to_string(&legacy_path)
                && let Ok(storage_data) = serde_json::from_str::<StorageData>(&content)
            {
                // Delete legacy unencrypted file after reading
                let _ = fs::remove_file(&legacy_path);
                return storage_data.data;
            }
        }
        HashMap::new()
    }

    /// Save storage data to disk with encryption.
    ///
    /// # Security
    ///
    /// Data is encrypted using AES-256-GCM with a key derived from the session secret.
    /// The encrypted file contains: [12-byte nonce][ciphertext with auth tag]
    fn save_storage_data(&self) {
        let data = self.data.read().unwrap();
        let storage_data = StorageData { data: data.clone() };

        // Serialize to JSON
        let json_content = match serde_json::to_string(&storage_data) {
            Ok(c) => c,
            Err(_) => return,
        };

        // Encrypt the JSON content
        let encrypted = match encryption::encrypt(json_content.as_bytes(), self.storage_type) {
            Ok(e) => e,
            Err(_) => return,
        };

        // Write encrypted data to disk
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            if let Ok(mut file) = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600) // Owner read/write only
                .open(&self.storage_path)
            {
                use std::io::Write;
                let _ = file.write_all(&encrypted);
            }
        }

        #[cfg(not(unix))]
        {
            let _ = fs::write(&self.storage_path, &encrypted);
        }
    }

    /// Creates a localStorage instance
    pub fn create_local_storage() -> JsObject {
        let storage = Storage::new("localStorage");
        JsObject::from_proto_and_data(None, storage)
    }

    /// Creates a sessionStorage instance
    pub fn create_session_storage() -> JsObject {
        let storage = Storage::new("sessionStorage");
        JsObject::from_proto_and_data(None, storage)
    }
}

impl IntrinsicObject for Storage {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("length"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_length)
                        .name(js_string!("get length"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(Self::key, js_string!("key"), 1)
            .method(Self::get_item, js_string!("getItem"), 1)
            .method(Self::set_item, js_string!("setItem"), 2)
            .method(Self::remove_item, js_string!("removeItem"), 1)
            .method(Self::clear, js_string!("clear"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Storage {
    const NAME: JsString = js_string!("Storage");
}

impl BuiltInConstructor for Storage {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(
        &boa_engine::context::intrinsics::StandardConstructors,
    ) -> &StandardConstructor = |constructors| constructors.storage();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // Storage constructor is not meant to be called directly
        Err(JsNativeError::typ()
            .with_message("Storage constructor cannot be called directly")
            .into())
    }
}

// Storage prototype methods
impl Storage {
    /// `Storage.prototype.length` getter
    fn get_length(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Storage object"))?;

        let storage = obj
            .downcast_ref::<Storage>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Storage object"))?;

        Ok(JsValue::from(storage.length_internal()))
    }

    /// `Storage.prototype.key(index)`
    fn key(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Storage object"))?;

        let storage = obj
            .downcast_ref::<Storage>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Storage object"))?;

        let index = args.get_or_undefined(0).to_length(context)?;

        match storage.key_internal(index as usize) {
            Some(key) => Ok(JsValue::from(JsString::from(key))),
            None => Ok(JsValue::null()),
        }
    }

    /// `Storage.prototype.getItem(key)`
    fn get_item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Storage object"))?;

        let storage = obj
            .downcast_ref::<Storage>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Storage object"))?;

        let key = args.get_or_undefined(0).to_string(context)?;

        match storage.get_item_internal(&key.to_std_string_escaped()) {
            Some(value) => Ok(JsValue::from(JsString::from(value))),
            None => Ok(JsValue::null()),
        }
    }

    /// `Storage.prototype.setItem(key, value)`
    fn set_item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Storage object"))?;

        let storage = obj
            .downcast_ref::<Storage>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Storage object"))?;

        let key = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        let value = args
            .get_or_undefined(1)
            .to_string(context)?
            .to_std_string_escaped();

        storage.set_item_internal(key, value)?;
        Ok(JsValue::undefined())
    }

    /// `Storage.prototype.removeItem(key)`
    fn remove_item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Storage object"))?;

        let storage = obj
            .downcast_ref::<Storage>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Storage object"))?;

        let key = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        storage.remove_item_internal(&key);
        Ok(JsValue::undefined())
    }

    /// `Storage.prototype.clear()`
    fn clear(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Storage object"))?;

        let storage = obj
            .downcast_ref::<Storage>()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not a Storage object"))?;

        storage.clear_internal();
        Ok(JsValue::undefined())
    }
}

#[cfg(test)]
mod tests;
