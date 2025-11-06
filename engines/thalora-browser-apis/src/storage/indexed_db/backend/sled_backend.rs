//! Sled-based Persistent Storage Backend
//!
//! Provides persistent storage for IndexedDB using the Sled embedded database.
//! Data survives process restarts and is stored on disk.
//!
//! Sled is a high-performance embedded database that provides:
//! - ACID transactions
//! - Crash recovery
//! - Lock-free operations
//! - Ordered key-value storage (perfect for IndexedDB)

use super::*;
use sled::{Db, Tree};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Sled-based persistent storage backend
pub struct SledBackend {
    /// Base path for database storage
    base_path: PathBuf,

    /// Main Sled database instance
    db: Db,

    /// Cache of open databases
    databases: Arc<Mutex<HashMap<String, DatabaseInfo>>>,

    /// Transaction counter
    transaction_counter: AtomicU64,

    /// Active transactions
    active_transactions: Arc<Mutex<HashMap<u64, TransactionInfo>>>,
}

/// Database information
#[derive(Debug, Clone)]
struct DatabaseInfo {
    name: String,
    version: u32,
    object_stores: Vec<String>,
}

/// Transaction information
#[derive(Debug, Clone)]
struct TransactionInfo {
    id: u64,
    mode: TransactionMode,
    stores: Vec<String>,
    // In Sled, we use snapshot isolation for transactions
}

impl SledBackend {
    /// Create a new Sled-based backend
    pub fn new(base_path: PathBuf) -> Result<Self, String> {
        // Create base directory if it doesn't exist
        std::fs::create_dir_all(&base_path)
            .map_err(|e| format!("Failed to create storage directory: {}", e))?;

        // Open main Sled database
        let db_path = base_path.join("indexeddb");
        let db = sled::open(&db_path)
            .map_err(|e| format!("Failed to open Sled database: {}", e))?;

        Ok(Self {
            base_path,
            db,
            databases: Arc::new(Mutex::new(HashMap::new())),
            transaction_counter: AtomicU64::new(1),
            active_transactions: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Get tree name for database metadata
    fn metadata_tree_name(db_name: &str) -> String {
        format!("__meta__{}", db_name)
    }

    /// Get tree name for object store
    fn store_tree_name(db_name: &str, store_name: &str) -> String {
        format!("{}::{}", db_name, store_name)
    }

    /// Get tree name for index
    fn index_tree_name(db_name: &str, store_name: &str, index_name: &str) -> String {
        format!("{}::{}::idx::{}", db_name, store_name, index_name)
    }

    /// Load database metadata
    fn load_metadata(&self, db_name: &str) -> Result<DatabaseInfo, String> {
        let meta_tree = self.db.open_tree(Self::metadata_tree_name(db_name))
            .map_err(|e| format!("Failed to open metadata tree: {}", e))?;

        // Get version
        let version = meta_tree.get(b"version")
            .map_err(|e| format!("Failed to read version: {}", e))?
            .map(|v| {
                let bytes: [u8; 4] = v.as_ref().try_into().unwrap_or([0, 0, 0, 0]);
                u32::from_be_bytes(bytes)
            })
            .unwrap_or(0);

        // Get object store list
        let stores_bytes = meta_tree.get(b"object_stores")
            .map_err(|e| format!("Failed to read object stores: {}", e))?;

        let object_stores = if let Some(bytes) = stores_bytes {
            serde_json::from_slice(&bytes)
                .map_err(|e| format!("Failed to deserialize object stores: {}", e))?
        } else {
            Vec::new()
        };

        Ok(DatabaseInfo {
            name: db_name.to_string(),
            version,
            object_stores,
        })
    }

    /// Save database metadata
    fn save_metadata(&self, info: &DatabaseInfo) -> Result<(), String> {
        let meta_tree = self.db.open_tree(Self::metadata_tree_name(&info.name))
            .map_err(|e| format!("Failed to open metadata tree: {}", e))?;

        // Save version
        meta_tree.insert(b"version", &info.version.to_be_bytes())
            .map_err(|e| format!("Failed to save version: {}", e))?;

        // Save object store list
        let stores_json = serde_json::to_vec(&info.object_stores)
            .map_err(|e| format!("Failed to serialize object stores: {}", e))?;

        meta_tree.insert(b"object_stores", stores_json)
            .map_err(|e| format!("Failed to save object stores: {}", e))?;

        meta_tree.flush()
            .map_err(|e| format!("Failed to flush metadata: {}", e))?;

        Ok(())
    }

    /// Load object store metadata
    fn load_store_metadata(&self, db_name: &str, store_name: &str) -> Result<ObjectStoreMetadata, String> {
        let meta_tree = self.db.open_tree(Self::metadata_tree_name(db_name))
            .map_err(|e| format!("Failed to open metadata tree: {}", e))?;

        let key = format!("store::{}", store_name);
        let metadata_bytes = meta_tree.get(key.as_bytes())
            .map_err(|e| format!("Failed to read store metadata: {}", e))?
            .ok_or_else(|| format!("Object store '{}' not found", store_name))?;

        serde_json::from_slice(&metadata_bytes)
            .map_err(|e| format!("Failed to deserialize store metadata: {}", e))
    }

    /// Save object store metadata
    fn save_store_metadata(&self, db_name: &str, metadata: &ObjectStoreMetadata) -> Result<(), String> {
        let meta_tree = self.db.open_tree(Self::metadata_tree_name(db_name))
            .map_err(|e| format!("Failed to open metadata tree: {}", e))?;

        let key = format!("store::{}", metadata.name);
        let metadata_json = serde_json::to_vec(metadata)
            .map_err(|e| format!("Failed to serialize store metadata: {}", e))?;

        meta_tree.insert(key.as_bytes(), metadata_json)
            .map_err(|e| format!("Failed to save store metadata: {}", e))?;

        Ok(())
    }

    /// Get tree for object store
    fn get_store_tree(&self, db_name: &str, store_name: &str) -> Result<Tree, String> {
        let tree_name = Self::store_tree_name(db_name, store_name);
        self.db.open_tree(tree_name)
            .map_err(|e| format!("Failed to open store tree: {}", e))
    }
}

impl StorageBackend for SledBackend {
    fn open_database(&mut self, name: &str, version: u32) -> Result<DatabaseHandle, String> {
        let mut databases = self.databases.lock().unwrap();

        // Try to load existing database
        let current_info = self.load_metadata(name).ok();
        let old_version = current_info.as_ref().map(|i| i.version).unwrap_or(0);
        let needs_upgrade = version > old_version || old_version == 0;

        let info = if needs_upgrade {
            let new_info = DatabaseInfo {
                name: name.to_string(),
                version,
                object_stores: current_info
                    .map(|i| i.object_stores)
                    .unwrap_or_default(),
            };
            self.save_metadata(&new_info)?;
            new_info
        } else {
            current_info.unwrap()
        };

        databases.insert(name.to_string(), info.clone());

        Ok(DatabaseHandle {
            name: name.to_string(),
            version,
            old_version,
            needs_upgrade,
            object_stores: info.object_stores,
        })
    }

    fn delete_database(&mut self, name: &str) -> Result<(), String> {
        let mut databases = self.databases.lock().unwrap();

        // Get database info to find all trees to delete
        if let Ok(info) = self.load_metadata(name) {
            // Delete all object store trees
            for store_name in &info.object_stores {
                let tree_name = Self::store_tree_name(name, store_name);
                self.db.drop_tree(tree_name)
                    .map_err(|e| format!("Failed to drop store tree: {}", e))?;
            }
        }

        // Delete metadata tree
        let meta_tree_name = Self::metadata_tree_name(name);
        self.db.drop_tree(meta_tree_name)
            .map_err(|e| format!("Failed to drop metadata tree: {}", e))?;

        databases.remove(name);
        Ok(())
    }

    fn databases(&self) -> Result<Vec<(String, u32)>, String> {
        let mut result = Vec::new();

        // Iterate through all trees looking for metadata trees
        for tree_name in self.db.tree_names() {
            let name_str = String::from_utf8_lossy(&tree_name);
            if name_str.starts_with("__meta__") {
                let db_name = name_str.strip_prefix("__meta__").unwrap();
                if let Ok(info) = self.load_metadata(db_name) {
                    result.push((info.name, info.version));
                }
            }
        }

        Ok(result)
    }

    fn get_version(&self, name: &str) -> Result<Option<u32>, String> {
        Ok(self.load_metadata(name).ok().map(|info| info.version))
    }

    fn create_object_store(
        &mut self,
        db: &str,
        store: &str,
        key_path: Option<String>,
        auto_increment: bool,
    ) -> Result<(), String> {
        let mut databases = self.databases.lock().unwrap();

        // Load current database info
        let mut info = self.load_metadata(db)?;

        // Check if store already exists
        if info.object_stores.contains(&store.to_string()) {
            return Err(format!("Object store '{}' already exists", store));
        }

        // Create the store tree
        let tree_name = Self::store_tree_name(db, store);
        self.db.open_tree(&tree_name)
            .map_err(|e| format!("Failed to create store tree: {}", e))?;

        // Add to object stores list
        info.object_stores.push(store.to_string());
        self.save_metadata(&info)?;

        // Save store metadata
        let metadata = ObjectStoreMetadata {
            name: store.to_string(),
            key_path,
            auto_increment,
            indexes: Vec::new(),
        };
        self.save_store_metadata(db, &metadata)?;

        databases.insert(db.to_string(), info);
        Ok(())
    }

    fn delete_object_store(&mut self, db: &str, store: &str) -> Result<(), String> {
        let mut databases = self.databases.lock().unwrap();

        // Load current database info
        let mut info = self.load_metadata(db)?;

        // Remove from object stores list
        info.object_stores.retain(|s| s != store);
        self.save_metadata(&info)?;

        // Drop the store tree
        let tree_name = Self::store_tree_name(db, store);
        self.db.drop_tree(tree_name)
            .map_err(|e| format!("Failed to drop store tree: {}", e))?;

        // Remove store metadata
        let meta_tree = self.db.open_tree(Self::metadata_tree_name(db))
            .map_err(|e| format!("Failed to open metadata tree: {}", e))?;
        let key = format!("store::{}", store);
        meta_tree.remove(key.as_bytes())
            .map_err(|e| format!("Failed to remove store metadata: {}", e))?;

        databases.insert(db.to_string(), info);
        Ok(())
    }

    fn get_object_store_metadata(&self, db: &str, store: &str) -> Result<ObjectStoreMetadata, String> {
        self.load_store_metadata(db, store)
    }

    fn list_object_stores(&self, db: &str) -> Result<Vec<String>, String> {
        let info = self.load_metadata(db)?;
        Ok(info.object_stores)
    }

    fn add(&mut self, db: &str, store: &str, key: &IDBKey, value: &[u8]) -> Result<IDBKey, String> {
        let tree = self.get_store_tree(db, store)?;
        let key_bytes = key.to_bytes();

        // Check if key already exists
        if tree.contains_key(&key_bytes)
            .map_err(|e| format!("Failed to check key existence: {}", e))? {
            return Err("Key already exists".to_string());
        }

        tree.insert(&key_bytes, value)
            .map_err(|e| format!("Failed to insert: {}", e))?;

        Ok(key.clone())
    }

    fn put(&mut self, db: &str, store: &str, key: &IDBKey, value: &[u8]) -> Result<IDBKey, String> {
        let tree = self.get_store_tree(db, store)?;
        let key_bytes = key.to_bytes();

        tree.insert(&key_bytes, value)
            .map_err(|e| format!("Failed to insert: {}", e))?;

        Ok(key.clone())
    }

    fn get(&self, db: &str, store: &str, key: &IDBKey) -> Result<Option<Vec<u8>>, String> {
        let tree = self.get_store_tree(db, store)?;
        let key_bytes = key.to_bytes();

        tree.get(&key_bytes)
            .map_err(|e| format!("Failed to get: {}", e))?
            .map(|ivec| Ok(ivec.to_vec()))
            .transpose()
    }

    fn delete(&mut self, db: &str, store: &str, key: &IDBKey) -> Result<(), String> {
        let tree = self.get_store_tree(db, store)?;
        let key_bytes = key.to_bytes();

        tree.remove(&key_bytes)
            .map_err(|e| format!("Failed to delete: {}", e))?;

        Ok(())
    }

    fn clear(&mut self, db: &str, store: &str) -> Result<(), String> {
        let tree = self.get_store_tree(db, store)?;

        tree.clear()
            .map_err(|e| format!("Failed to clear: {}", e))?;

        Ok(())
    }

    fn count(&self, db: &str, store: &str, range: Option<&IDBKeyRange>) -> Result<u64, String> {
        let tree = self.get_store_tree(db, store)?;

        if let Some(range) = range {
            let count = tree.iter()
                .filter_map(|item| item.ok())
                .filter(|(key_bytes, _)| {
                    if let Ok(key) = IDBKey::from_bytes(key_bytes) {
                        range.includes(&key)
                    } else {
                        false
                    }
                })
                .count() as u64;
            Ok(count)
        } else {
            Ok(tree.len() as u64)
        }
    }

    fn get_all_keys(
        &self,
        db: &str,
        store: &str,
        range: Option<&IDBKeyRange>,
        count: Option<u32>,
    ) -> Result<Vec<IDBKey>, String> {
        let tree = self.get_store_tree(db, store)?;

        let mut keys: Vec<IDBKey> = tree.iter()
            .filter_map(|item| item.ok())
            .filter_map(|(key_bytes, _)| IDBKey::from_bytes(&key_bytes).ok())
            .filter(|key| {
                if let Some(range) = range {
                    range.includes(key)
                } else {
                    true
                }
            })
            .collect();

        if let Some(limit) = count {
            keys.truncate(limit as usize);
        }

        Ok(keys)
    }

    fn get_all(
        &self,
        db: &str,
        store: &str,
        range: Option<&IDBKeyRange>,
        count: Option<u32>,
    ) -> Result<Vec<Vec<u8>>, String> {
        let tree = self.get_store_tree(db, store)?;

        let mut values: Vec<Vec<u8>> = tree.iter()
            .filter_map(|item| item.ok())
            .filter(|(key_bytes, _)| {
                if let Some(range) = range {
                    if let Ok(key) = IDBKey::from_bytes(key_bytes) {
                        range.includes(&key)
                    } else {
                        false
                    }
                } else {
                    true
                }
            })
            .map(|(_, value)| value.to_vec())
            .collect();

        if let Some(limit) = count {
            values.truncate(limit as usize);
        }

        Ok(values)
    }

    fn create_index(
        &mut self,
        db: &str,
        store: &str,
        index_name: &str,
        key_path: &str,
        unique: bool,
        multi_entry: bool,
    ) -> Result<(), String> {
        // Load current metadata
        let mut metadata = self.load_store_metadata(db, store)?;

        // Check if index already exists
        if metadata.indexes.iter().any(|idx| idx.name == index_name) {
            return Err(format!("Index '{}' already exists", index_name));
        }

        // Create index tree
        let index_tree_name = Self::index_tree_name(db, store, index_name);
        self.db.open_tree(&index_tree_name)
            .map_err(|e| format!("Failed to create index tree: {}", e))?;

        // Add to metadata
        metadata.indexes.push(IndexMetadata {
            name: index_name.to_string(),
            key_path: key_path.to_string(),
            unique,
            multi_entry,
        });

        self.save_store_metadata(db, &metadata)?;
        Ok(())
    }

    fn delete_index(&mut self, db: &str, store: &str, index_name: &str) -> Result<(), String> {
        // Load current metadata
        let mut metadata = self.load_store_metadata(db, store)?;

        // Remove from metadata
        metadata.indexes.retain(|idx| idx.name != index_name);
        self.save_store_metadata(db, &metadata)?;

        // Drop index tree
        let index_tree_name = Self::index_tree_name(db, store, index_name);
        self.db.drop_tree(index_tree_name)
            .map_err(|e| format!("Failed to drop index tree: {}", e))?;

        Ok(())
    }

    fn get_by_index(
        &self,
        db: &str,
        store: &str,
        index_name: &str,
        key: &IDBKey,
    ) -> Result<Option<Vec<u8>>, String> {
        let index_tree_name = Self::index_tree_name(db, store, index_name);
        let index_tree = self.db.open_tree(&index_tree_name)
            .map_err(|e| format!("Failed to open index tree: {}", e))?;

        let key_bytes = key.to_bytes();

        // Get primary key from index
        if let Some(primary_key_bytes) = index_tree.get(&key_bytes)
            .map_err(|e| format!("Failed to get from index: {}", e))? {

            // Get value from store using primary key
            let store_tree = self.get_store_tree(db, store)?;
            store_tree.get(&primary_key_bytes)
                .map_err(|e| format!("Failed to get from store: {}", e))?
                .map(|ivec| Ok(ivec.to_vec()))
                .transpose()
        } else {
            Ok(None)
        }
    }

    fn get_key_from_index(
        &self,
        db: &str,
        store: &str,
        index_name: &str,
        key: &IDBKey,
    ) -> Result<Option<IDBKey>, String> {
        let index_tree_name = Self::index_tree_name(db, store, index_name);
        let index_tree = self.db.open_tree(&index_tree_name)
            .map_err(|e| format!("Failed to open index tree: {}", e))?;

        let key_bytes = key.to_bytes();

        // Get primary key from index
        if let Some(primary_key_bytes) = index_tree.get(&key_bytes)
            .map_err(|e| format!("Failed to get from index: {}", e))? {
            IDBKey::from_bytes(&primary_key_bytes).map(Some)
        } else {
            Ok(None)
        }
    }

    fn get_all_from_index(
        &self,
        db: &str,
        store: &str,
        index_name: &str,
        range: Option<&IDBKeyRange>,
        count: Option<u32>,
    ) -> Result<Vec<Vec<u8>>, String> {
        let index_tree_name = Self::index_tree_name(db, store, index_name);
        let index_tree = self.db.open_tree(&index_tree_name)
            .map_err(|e| format!("Failed to open index tree: {}", e))?;

        let store_tree = self.get_store_tree(db, store)?;
        let mut results = Vec::new();
        let limit = count.unwrap_or(u32::MAX) as usize;

        for item in index_tree.iter() {
            if results.len() >= limit {
                break;
            }

            let (index_key_bytes, primary_key_bytes) = item
                .map_err(|e| format!("Failed to iterate index: {}", e))?;

            // Check if index key is in range
            if let Ok(index_key) = IDBKey::from_bytes(&index_key_bytes) {
                if let Some(range) = range {
                    if !range.includes(&index_key) {
                        continue;
                    }
                }

                // Get value from store
                if let Some(value) = store_tree.get(&primary_key_bytes)
                    .map_err(|e| format!("Failed to get from store: {}", e))? {
                    results.push(value.to_vec());
                }
            }
        }

        Ok(results)
    }

    fn get_all_keys_from_index(
        &self,
        db: &str,
        store: &str,
        index_name: &str,
        range: Option<&IDBKeyRange>,
        count: Option<u32>,
    ) -> Result<Vec<IDBKey>, String> {
        let index_tree_name = Self::index_tree_name(db, store, index_name);
        let index_tree = self.db.open_tree(&index_tree_name)
            .map_err(|e| format!("Failed to open index tree: {}", e))?;

        let mut results = Vec::new();
        let limit = count.unwrap_or(u32::MAX) as usize;

        for item in index_tree.iter() {
            if results.len() >= limit {
                break;
            }

            let (index_key_bytes, primary_key_bytes) = item
                .map_err(|e| format!("Failed to iterate index: {}", e))?;

            // Check if index key is in range
            if let Ok(index_key) = IDBKey::from_bytes(&index_key_bytes) {
                if let Some(range) = range {
                    if !range.includes(&index_key) {
                        continue;
                    }
                }

                // Parse and add primary key
                if let Ok(primary_key) = IDBKey::from_bytes(&primary_key_bytes) {
                    results.push(primary_key);
                }
            }
        }

        Ok(results)
    }

    fn count_index(
        &self,
        db: &str,
        store: &str,
        index_name: &str,
        range: Option<&IDBKeyRange>,
    ) -> Result<u32, String> {
        let index_tree_name = Self::index_tree_name(db, store, index_name);
        let index_tree = self.db.open_tree(&index_tree_name)
            .map_err(|e| format!("Failed to open index tree: {}", e))?;

        let mut count = 0u32;

        for item in index_tree.iter() {
            let (index_key_bytes, _) = item
                .map_err(|e| format!("Failed to iterate index: {}", e))?;

            // Check if index key is in range
            if let Ok(index_key) = IDBKey::from_bytes(&index_key_bytes) {
                if let Some(range) = range {
                    if !range.includes(&index_key) {
                        continue;
                    }
                }

                count += 1;
            }
        }

        Ok(count)
    }

    fn begin_transaction(&mut self, stores: &[String], mode: TransactionMode) -> Result<TransactionHandle, String> {
        let transaction_id = self.transaction_counter.fetch_add(1, Ordering::SeqCst);

        let info = TransactionInfo {
            id: transaction_id,
            mode,
            stores: stores.to_vec(),
        };

        let mut transactions = self.active_transactions.lock().unwrap();
        transactions.insert(transaction_id, info);

        Ok(TransactionHandle {
            id: transaction_id,
            mode,
            stores: stores.to_vec(),
        })
    }

    fn commit_transaction(&mut self, transaction_id: u64) -> Result<(), String> {
        let mut transactions = self.active_transactions.lock().unwrap();
        transactions.remove(&transaction_id)
            .ok_or_else(|| format!("Transaction {} not found", transaction_id))?;

        // Sled auto-commits, flush to ensure persistence
        self.db.flush()
            .map_err(|e| format!("Failed to flush: {}", e))?;

        Ok(())
    }

    fn abort_transaction(&mut self, transaction_id: u64) -> Result<(), String> {
        let mut transactions = self.active_transactions.lock().unwrap();
        transactions.remove(&transaction_id)
            .ok_or_else(|| format!("Transaction {} not found", transaction_id))?;

        // For abort, we would need to track changes and roll them back
        // This is a simplified implementation - full ACID would require more complex tracking
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn get_test_path() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);

        let mut path = env::temp_dir();
        path.push("thalora_indexeddb_test");
        path.push(format!("test_{}-{}", std::process::id(), counter));
        path
    }

    #[test]
    fn test_sled_backend_crud() {
        let test_path = get_test_path();
        let mut backend = SledBackend::new(test_path.clone()).unwrap();

        // Open database
        backend.open_database("test-db", 1).unwrap();
        backend.create_object_store("test-db", "store1", None, false).unwrap();

        // Add
        let key = IDBKey::String("key1".to_string());
        let value = b"value1";
        backend.add("test-db", "store1", &key, value).unwrap();

        // Get
        let retrieved = backend.get("test-db", "store1", &key).unwrap();
        assert_eq!(retrieved, Some(value.to_vec()));

        // Cleanup
        std::fs::remove_dir_all(test_path).ok();
    }

    #[test]
    fn test_sled_persistence() {
        let test_path = get_test_path();

        // Write data
        {
            let mut backend = SledBackend::new(test_path.clone()).unwrap();
            backend.open_database("test-db", 1).unwrap();
            backend.create_object_store("test-db", "store1", None, false).unwrap();

            let key = IDBKey::String("persistent-key".to_string());
            let value = b"persistent-value";
            backend.put("test-db", "store1", &key, value).unwrap();
        }

        // Read data in new instance
        {
            let backend = SledBackend::new(test_path.clone()).unwrap();
            let key = IDBKey::String("persistent-key".to_string());
            let retrieved = backend.get("test-db", "store1", &key).unwrap();
            assert_eq!(retrieved, Some(b"persistent-value".to_vec()));
        }

        // Cleanup
        std::fs::remove_dir_all(test_path).ok();
    }
}
