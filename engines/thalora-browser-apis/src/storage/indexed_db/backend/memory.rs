//! In-Memory Storage Backend
//!
//! A simple in-memory implementation of the storage backend for testing.
//! Data is not persisted and is lost when the process terminates.

use super::*;
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};

/// In-memory database
#[derive(Debug, Clone)]
struct MemoryDatabase {
    name: String,
    version: u32,
    object_stores: HashMap<String, MemoryObjectStore>,
}

/// In-memory object store
#[derive(Debug, Clone)]
struct MemoryObjectStore {
    name: String,
    key_path: Option<String>,
    auto_increment: bool,
    auto_increment_counter: u64,
    // Use BTreeMap for sorted key storage
    records: BTreeMap<Vec<u8>, Vec<u8>>,
    indexes: HashMap<String, MemoryIndex>,
}

/// In-memory index
#[derive(Debug, Clone)]
struct MemoryIndex {
    name: String,
    key_path: String,
    unique: bool,
    multi_entry: bool,
    // Map from index key to primary keys
    entries: BTreeMap<Vec<u8>, Vec<Vec<u8>>>,
}

/// In-memory storage backend
pub struct MemoryBackend {
    databases: HashMap<String, MemoryDatabase>,
    transaction_counter: AtomicU64,
    active_transactions: HashMap<u64, TransactionHandle>,
}

impl MemoryBackend {
    /// Create a new in-memory backend
    pub fn new() -> Self {
        Self {
            databases: HashMap::new(),
            transaction_counter: AtomicU64::new(1),
            active_transactions: HashMap::new(),
        }
    }

    /// Get or create database
    fn get_or_create_db(&mut self, name: &str) -> &mut MemoryDatabase {
        self.databases
            .entry(name.to_string())
            .or_insert_with(|| MemoryDatabase {
                name: name.to_string(),
                version: 0,
                object_stores: HashMap::new(),
            })
    }

    /// Get database
    fn get_db(&self, name: &str) -> Result<&MemoryDatabase, String> {
        self.databases
            .get(name)
            .ok_or_else(|| format!("Database '{}' not found", name))
    }

    /// Get mutable database
    fn get_db_mut(&mut self, name: &str) -> Result<&mut MemoryDatabase, String> {
        self.databases
            .get_mut(name)
            .ok_or_else(|| format!("Database '{}' not found", name))
    }

    /// Get object store
    fn get_store<'a>(db: &'a MemoryDatabase, store: &str) -> Result<&'a MemoryObjectStore, String> {
        db.object_stores
            .get(store)
            .ok_or_else(|| format!("Object store '{}' not found", store))
    }

    /// Get mutable object store
    fn get_store_mut<'a>(
        db: &'a mut MemoryDatabase,
        store: &str,
    ) -> Result<&'a mut MemoryObjectStore, String> {
        db.object_stores
            .get_mut(store)
            .ok_or_else(|| format!("Object store '{}' not found", store))
    }
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for MemoryBackend {
    fn open_database(&mut self, name: &str, version: u32) -> Result<DatabaseHandle, String> {
        let db = self.get_or_create_db(name);
        let old_version = db.version;
        let needs_upgrade = version > old_version || old_version == 0;

        if needs_upgrade {
            db.version = version;
        }

        let object_stores = db.object_stores.keys().cloned().collect();

        Ok(DatabaseHandle {
            name: name.to_string(),
            version,
            old_version,
            needs_upgrade,
            object_stores,
        })
    }

    fn delete_database(&mut self, name: &str) -> Result<(), String> {
        self.databases.remove(name);
        Ok(())
    }

    fn databases(&self) -> Result<Vec<(String, u32)>, String> {
        Ok(self
            .databases
            .iter()
            .map(|(name, db)| (name.clone(), db.version))
            .collect())
    }

    fn get_version(&self, name: &str) -> Result<Option<u32>, String> {
        Ok(self.databases.get(name).map(|db| db.version))
    }

    fn create_object_store(
        &mut self,
        db: &str,
        store: &str,
        key_path: Option<String>,
        auto_increment: bool,
    ) -> Result<(), String> {
        let database = self.get_db_mut(db)?;

        if database.object_stores.contains_key(store) {
            return Err(format!("Object store '{}' already exists", store));
        }

        database.object_stores.insert(
            store.to_string(),
            MemoryObjectStore {
                name: store.to_string(),
                key_path,
                auto_increment,
                auto_increment_counter: 1,
                records: BTreeMap::new(),
                indexes: HashMap::new(),
            },
        );

        Ok(())
    }

    fn delete_object_store(&mut self, db: &str, store: &str) -> Result<(), String> {
        let database = self.get_db_mut(db)?;
        database
            .object_stores
            .remove(store)
            .ok_or_else(|| format!("Object store '{}' not found", store))?;
        Ok(())
    }

    fn get_object_store_metadata(
        &self,
        db: &str,
        store: &str,
    ) -> Result<ObjectStoreMetadata, String> {
        let database = self.get_db(db)?;
        let obj_store = Self::get_store(database, store)?;

        let indexes = obj_store
            .indexes
            .values()
            .map(|idx| IndexMetadata {
                name: idx.name.clone(),
                key_path: idx.key_path.clone(),
                unique: idx.unique,
                multi_entry: idx.multi_entry,
            })
            .collect();

        Ok(ObjectStoreMetadata {
            name: obj_store.name.clone(),
            key_path: obj_store.key_path.clone(),
            auto_increment: obj_store.auto_increment,
            indexes,
        })
    }

    fn list_object_stores(&self, db: &str) -> Result<Vec<String>, String> {
        let database = self.get_db(db)?;
        Ok(database.object_stores.keys().cloned().collect())
    }

    fn add(&mut self, db: &str, store: &str, key: &IDBKey, value: &[u8]) -> Result<IDBKey, String> {
        let database = self.get_db_mut(db)?;
        let obj_store = Self::get_store_mut(database, store)?;

        // Generate key if auto-increment is enabled
        let actual_key = if obj_store.auto_increment {
            // Increment counter and use it as the key
            obj_store.auto_increment_counter += 1;
            IDBKey::Number(obj_store.auto_increment_counter as f64)
        } else {
            key.clone()
        };

        let key_bytes = actual_key.to_bytes();

        // Check if key already exists
        if obj_store.records.contains_key(&key_bytes) {
            return Err("Key already exists".to_string());
        }

        obj_store.records.insert(key_bytes, value.to_vec());
        Ok(actual_key)
    }

    fn put(&mut self, db: &str, store: &str, key: &IDBKey, value: &[u8]) -> Result<IDBKey, String> {
        let database = self.get_db_mut(db)?;
        let obj_store = Self::get_store_mut(database, store)?;

        let key_bytes = key.to_bytes();
        obj_store.records.insert(key_bytes, value.to_vec());
        Ok(key.clone())
    }

    fn get(&self, db: &str, store: &str, key: &IDBKey) -> Result<Option<Vec<u8>>, String> {
        let database = self.get_db(db)?;
        let obj_store = Self::get_store(database, store)?;

        let key_bytes = key.to_bytes();
        Ok(obj_store.records.get(&key_bytes).cloned())
    }

    fn delete(&mut self, db: &str, store: &str, key: &IDBKey) -> Result<(), String> {
        let database = self.get_db_mut(db)?;
        let obj_store = Self::get_store_mut(database, store)?;

        let key_bytes = key.to_bytes();
        obj_store.records.remove(&key_bytes);
        Ok(())
    }

    fn clear(&mut self, db: &str, store: &str) -> Result<(), String> {
        let database = self.get_db_mut(db)?;
        let obj_store = Self::get_store_mut(database, store)?;

        obj_store.records.clear();
        Ok(())
    }

    fn count(&self, db: &str, store: &str, range: Option<&IDBKeyRange>) -> Result<u64, String> {
        let database = self.get_db(db)?;
        let obj_store = Self::get_store(database, store)?;

        if let Some(range) = range {
            let count = obj_store
                .records
                .keys()
                .filter(|key_bytes| {
                    if let Ok(key) = IDBKey::from_bytes(key_bytes) {
                        range.includes(&key)
                    } else {
                        false
                    }
                })
                .count() as u64;
            Ok(count)
        } else {
            Ok(obj_store.records.len() as u64)
        }
    }

    fn get_all_keys(
        &self,
        db: &str,
        store: &str,
        range: Option<&IDBKeyRange>,
        count: Option<u32>,
    ) -> Result<Vec<IDBKey>, String> {
        let database = self.get_db(db)?;
        let obj_store = Self::get_store(database, store)?;

        let mut keys: Vec<IDBKey> = obj_store
            .records
            .keys()
            .filter_map(|key_bytes| IDBKey::from_bytes(key_bytes).ok())
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
        let database = self.get_db(db)?;
        let obj_store = Self::get_store(database, store)?;

        let mut values: Vec<Vec<u8>> = obj_store
            .records
            .iter()
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
            .map(|(_, value)| value.clone())
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
        let database = self.get_db_mut(db)?;
        let obj_store = Self::get_store_mut(database, store)?;

        if obj_store.indexes.contains_key(index_name) {
            return Err(format!("Index '{}' already exists", index_name));
        }

        obj_store.indexes.insert(
            index_name.to_string(),
            MemoryIndex {
                name: index_name.to_string(),
                key_path: key_path.to_string(),
                unique,
                multi_entry,
                entries: BTreeMap::new(),
            },
        );

        Ok(())
    }

    fn delete_index(&mut self, db: &str, store: &str, index_name: &str) -> Result<(), String> {
        let database = self.get_db_mut(db)?;
        let obj_store = Self::get_store_mut(database, store)?;

        obj_store
            .indexes
            .remove(index_name)
            .ok_or_else(|| format!("Index '{}' not found", index_name))?;
        Ok(())
    }

    fn get_by_index(
        &self,
        db: &str,
        store: &str,
        index_name: &str,
        key: &IDBKey,
    ) -> Result<Option<Vec<u8>>, String> {
        let database = self.get_db(db)?;
        let obj_store = Self::get_store(database, store)?;

        let index = obj_store
            .indexes
            .get(index_name)
            .ok_or_else(|| format!("Index '{}' not found", index_name))?;

        let key_bytes = key.to_bytes();

        if let Some(primary_keys) = index.entries.get(&key_bytes) {
            if let Some(primary_key_bytes) = primary_keys.first() {
                return Ok(obj_store.records.get(primary_key_bytes).cloned());
            }
        }

        Ok(None)
    }

    fn get_key_from_index(
        &self,
        db: &str,
        store: &str,
        index_name: &str,
        key: &IDBKey,
    ) -> Result<Option<IDBKey>, String> {
        let database = self.get_db(db)?;
        let obj_store = Self::get_store(database, store)?;

        let index = obj_store
            .indexes
            .get(index_name)
            .ok_or_else(|| format!("Index '{}' not found", index_name))?;

        let key_bytes = key.to_bytes();

        if let Some(primary_keys) = index.entries.get(&key_bytes) {
            if let Some(primary_key_bytes) = primary_keys.first() {
                return IDBKey::from_bytes(primary_key_bytes).map(Some);
            }
        }

        Ok(None)
    }

    fn get_all_from_index(
        &self,
        db: &str,
        store: &str,
        index_name: &str,
        range: Option<&IDBKeyRange>,
        count: Option<u32>,
    ) -> Result<Vec<Vec<u8>>, String> {
        let database = self.get_db(db)?;
        let obj_store = Self::get_store(database, store)?;

        let index = obj_store
            .indexes
            .get(index_name)
            .ok_or_else(|| format!("Index '{}' not found", index_name))?;

        let mut results = Vec::new();
        let limit = count.unwrap_or(u32::MAX) as usize;

        for (index_key_bytes, primary_keys) in &index.entries {
            if results.len() >= limit {
                break;
            }

            // Check if index key is in range
            if let Ok(index_key) = IDBKey::from_bytes(index_key_bytes) {
                if let Some(range) = range {
                    if !range.includes(&index_key) {
                        continue;
                    }
                }

                // Get all records for this index key
                for primary_key_bytes in primary_keys {
                    if results.len() >= limit {
                        break;
                    }

                    if let Some(value) = obj_store.records.get(primary_key_bytes) {
                        results.push(value.clone());
                    }
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
        let database = self.get_db(db)?;
        let obj_store = Self::get_store(database, store)?;

        let index = obj_store
            .indexes
            .get(index_name)
            .ok_or_else(|| format!("Index '{}' not found", index_name))?;

        let mut results = Vec::new();
        let limit = count.unwrap_or(u32::MAX) as usize;

        for (index_key_bytes, primary_keys) in &index.entries {
            if results.len() >= limit {
                break;
            }

            // Check if index key is in range
            if let Ok(index_key) = IDBKey::from_bytes(index_key_bytes) {
                if let Some(range) = range {
                    if !range.includes(&index_key) {
                        continue;
                    }
                }

                // Add all primary keys for this index key
                for primary_key_bytes in primary_keys {
                    if results.len() >= limit {
                        break;
                    }

                    if let Ok(primary_key) = IDBKey::from_bytes(primary_key_bytes) {
                        results.push(primary_key);
                    }
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
        let database = self.get_db(db)?;
        let obj_store = Self::get_store(database, store)?;

        let index = obj_store
            .indexes
            .get(index_name)
            .ok_or_else(|| format!("Index '{}' not found", index_name))?;

        let mut count = 0u32;

        for (index_key_bytes, primary_keys) in &index.entries {
            if let Ok(index_key) = IDBKey::from_bytes(index_key_bytes) {
                if let Some(range) = range {
                    if !range.includes(&index_key) {
                        continue;
                    }
                }

                count += primary_keys.len() as u32;
            }
        }

        Ok(count)
    }

    fn begin_transaction(
        &mut self,
        stores: &[String],
        mode: TransactionMode,
    ) -> Result<TransactionHandle, String> {
        let transaction_id = self.transaction_counter.fetch_add(1, Ordering::SeqCst);

        let handle = TransactionHandle {
            id: transaction_id,
            mode,
            stores: stores.to_vec(),
        };

        self.active_transactions
            .insert(transaction_id, handle.clone());

        Ok(handle)
    }

    fn commit_transaction(&mut self, transaction_id: u64) -> Result<(), String> {
        self.active_transactions
            .remove(&transaction_id)
            .ok_or_else(|| format!("Transaction {} not found", transaction_id))?;
        Ok(())
    }

    fn abort_transaction(&mut self, transaction_id: u64) -> Result<(), String> {
        self.active_transactions
            .remove(&transaction_id)
            .ok_or_else(|| format!("Transaction {} not found", transaction_id))?;
        // In-memory backend doesn't need rollback - changes aren't persisted until commit
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_operations() {
        let mut backend = MemoryBackend::new();

        // Open database
        let handle = backend.open_database("test-db", 1).unwrap();
        assert_eq!(handle.name, "test-db");
        assert_eq!(handle.version, 1);
        assert!(handle.needs_upgrade);

        // Create object store
        backend
            .create_object_store("test-db", "store1", None, false)
            .unwrap();

        // List object stores
        let stores = backend.list_object_stores("test-db").unwrap();
        assert_eq!(stores, vec!["store1"]);

        // Delete database
        backend.delete_database("test-db").unwrap();
        assert!(backend.get_db("test-db").is_err());
    }

    #[test]
    fn test_crud_operations() {
        let mut backend = MemoryBackend::new();
        backend.open_database("test-db", 1).unwrap();
        backend
            .create_object_store("test-db", "store1", None, false)
            .unwrap();

        let key = IDBKey::String("key1".to_string());
        let value = b"value1";

        // Add
        backend.add("test-db", "store1", &key, value).unwrap();

        // Get
        let retrieved = backend.get("test-db", "store1", &key).unwrap();
        assert_eq!(retrieved, Some(value.to_vec()));

        // Put (update)
        let new_value = b"value2";
        backend.put("test-db", "store1", &key, new_value).unwrap();

        let retrieved = backend.get("test-db", "store1", &key).unwrap();
        assert_eq!(retrieved, Some(new_value.to_vec()));

        // Delete
        backend.delete("test-db", "store1", &key).unwrap();
        let retrieved = backend.get("test-db", "store1", &key).unwrap();
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_add_duplicate_key_fails() {
        let mut backend = MemoryBackend::new();
        backend.open_database("test-db", 1).unwrap();
        backend
            .create_object_store("test-db", "store1", None, false)
            .unwrap();

        let key = IDBKey::String("key1".to_string());
        let value = b"value1";

        backend.add("test-db", "store1", &key, value).unwrap();
        let result = backend.add("test-db", "store1", &key, value);
        assert!(result.is_err());
    }
}
