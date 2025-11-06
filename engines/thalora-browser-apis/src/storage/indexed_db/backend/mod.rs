//! Storage Backend Trait and Implementations
//!
//! Provides an abstraction layer for IndexedDB storage backends.
//! Supports in-memory (testing) and persistent (Sled) backends.

use super::key::IDBKey;
use super::key_range::IDBKeyRange;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod memory;
pub mod sled_backend;

// Re-export backend implementations
pub use memory::MemoryBackend;
pub use sled_backend::SledBackend;

/// Transaction mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionMode {
    ReadOnly,
    ReadWrite,
    VersionChange,
}

/// Cursor direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorDirection {
    Next,
    NextUnique,
    Prev,
    PrevUnique,
}

/// Database handle returned after opening
#[derive(Debug, Clone)]
pub struct DatabaseHandle {
    pub name: String,
    pub version: u32,
    pub old_version: u32,
    pub needs_upgrade: bool,
    pub object_stores: Vec<String>,
}

/// Object store metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStoreMetadata {
    pub name: String,
    pub key_path: Option<String>,
    pub auto_increment: bool,
    pub indexes: Vec<IndexMetadata>,
}

/// Index metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub name: String,
    pub key_path: String,
    pub unique: bool,
    pub multi_entry: bool,
}

/// Transaction handle
#[derive(Debug, Clone)]
pub struct TransactionHandle {
    pub id: u64,
    pub mode: TransactionMode,
    pub stores: Vec<String>,
}

/// Storage backend trait
///
/// All IndexedDB implementations must implement this trait to provide
/// the underlying storage mechanism.
pub trait StorageBackend: Send + Sync {
    /// Open or create a database
    ///
    /// Returns a database handle with upgrade information if needed
    fn open_database(&mut self, name: &str, version: u32) -> Result<DatabaseHandle, String>;

    /// Delete a database
    fn delete_database(&mut self, name: &str) -> Result<(), String>;

    /// List all databases
    fn databases(&self) -> Result<Vec<(String, u32)>, String>;

    /// Get database version
    fn get_version(&self, name: &str) -> Result<Option<u32>, String>;

    /// Create object store (only in versionchange transaction)
    fn create_object_store(
        &mut self,
        db: &str,
        store: &str,
        key_path: Option<String>,
        auto_increment: bool,
    ) -> Result<(), String>;

    /// Delete object store (only in versionchange transaction)
    fn delete_object_store(&mut self, db: &str, store: &str) -> Result<(), String>;

    /// Get object store metadata
    fn get_object_store_metadata(&self, db: &str, store: &str) -> Result<ObjectStoreMetadata, String>;

    /// List object stores in a database
    fn list_object_stores(&self, db: &str) -> Result<Vec<String>, String>;

    /// Add record (fail if key exists)
    fn add(&mut self, db: &str, store: &str, key: &IDBKey, value: &[u8]) -> Result<IDBKey, String>;

    /// Put record (overwrite if exists)
    fn put(&mut self, db: &str, store: &str, key: &IDBKey, value: &[u8]) -> Result<IDBKey, String>;

    /// Get record by key
    fn get(&self, db: &str, store: &str, key: &IDBKey) -> Result<Option<Vec<u8>>, String>;

    /// Delete record by key
    fn delete(&mut self, db: &str, store: &str, key: &IDBKey) -> Result<(), String>;

    /// Clear all records in store
    fn clear(&mut self, db: &str, store: &str) -> Result<(), String>;

    /// Count records in range
    fn count(&self, db: &str, store: &str, range: Option<&IDBKeyRange>) -> Result<u64, String>;

    /// Get all keys in range
    fn get_all_keys(
        &self,
        db: &str,
        store: &str,
        range: Option<&IDBKeyRange>,
        count: Option<u32>,
    ) -> Result<Vec<IDBKey>, String>;

    /// Get all values in range
    fn get_all(
        &self,
        db: &str,
        store: &str,
        range: Option<&IDBKeyRange>,
        count: Option<u32>,
    ) -> Result<Vec<Vec<u8>>, String>;

    /// Create index on object store
    fn create_index(
        &mut self,
        db: &str,
        store: &str,
        index_name: &str,
        key_path: &str,
        unique: bool,
        multi_entry: bool,
    ) -> Result<(), String>;

    /// Delete index from object store
    fn delete_index(&mut self, db: &str, store: &str, index_name: &str) -> Result<(), String>;

    /// Get values by index
    fn get_by_index(
        &self,
        db: &str,
        store: &str,
        index_name: &str,
        key: &IDBKey,
    ) -> Result<Option<Vec<u8>>, String>;

    /// Get primary key from index key
    fn get_key_from_index(
        &self,
        db: &str,
        store: &str,
        index_name: &str,
        key: &IDBKey,
    ) -> Result<Option<IDBKey>, String>;

    /// Get all values from index in range
    fn get_all_from_index(
        &self,
        db: &str,
        store: &str,
        index_name: &str,
        range: Option<&IDBKeyRange>,
        count: Option<u32>,
    ) -> Result<Vec<Vec<u8>>, String>;

    /// Get all keys from index in range
    fn get_all_keys_from_index(
        &self,
        db: &str,
        store: &str,
        index_name: &str,
        range: Option<&IDBKeyRange>,
        count: Option<u32>,
    ) -> Result<Vec<IDBKey>, String>;

    /// Count records in index
    fn count_index(
        &self,
        db: &str,
        store: &str,
        index_name: &str,
        range: Option<&IDBKeyRange>,
    ) -> Result<u32, String>;

    /// Begin transaction
    fn begin_transaction(&mut self, stores: &[String], mode: TransactionMode) -> Result<TransactionHandle, String>;

    /// Commit transaction
    fn commit_transaction(&mut self, transaction_id: u64) -> Result<(), String>;

    /// Abort transaction
    fn abort_transaction(&mut self, transaction_id: u64) -> Result<(), String>;
}
