//! IndexedDB Integration Tests
//!
//! Comprehensive tests for the full IndexedDB implementation

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::initialize_browser_apis;
    use boa_engine::{Context, Source};

    /// Helper to create a test context with IndexedDB
    fn create_test_context() -> Context {
        let mut context = Context::default();
        initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
        context
    }

    /// Helper to execute JavaScript and check result
    fn eval_js(context: &mut Context, code: &str) -> String {
        match context.eval(Source::from_bytes(code)) {
            Ok(result) => result.to_string(context).unwrap().to_std_string_escaped(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[test]
    fn test_indexeddb_global_exists() {
        let mut context = create_test_context();

        let result = eval_js(&mut context, "typeof indexedDB");
        eprintln!("typeof indexedDB = {}", result);
        assert_eq!(result, "object");

        let result = eval_js(&mut context, "typeof window");
        eprintln!("typeof window = {}", result);

        let result = eval_js(&mut context, "typeof window.localStorage");
        eprintln!("typeof window.localStorage = {}", result);

        let result = eval_js(&mut context, "Object.keys(window).join(',')");
        eprintln!("window keys = {}", result);

        let result = eval_js(&mut context, "typeof window.indexedDB");
        eprintln!("typeof window.indexedDB = {}", result);
        assert_eq!(result, "object");
    }

    #[test]
    fn test_indexeddb_factory_methods() {
        let mut context = create_test_context();

        // Check methods exist
        let result = eval_js(&mut context, "typeof indexedDB.open");
        assert_eq!(result, "function");

        let result = eval_js(&mut context, "typeof indexedDB.deleteDatabase");
        assert_eq!(result, "function");

        let result = eval_js(&mut context, "typeof indexedDB.databases");
        assert_eq!(result, "function");

        let result = eval_js(&mut context, "typeof indexedDB.cmp");
        assert_eq!(result, "function");
    }

    #[test]
    fn test_indexeddb_open_database() {
        let mut context = create_test_context();

        let code = r#"
            let openRequest = indexedDB.open("testDB", 1);
            typeof openRequest === "object"
        "#;

        let result = eval_js(&mut context, code);
        assert_eq!(result, "true");
    }

    #[test]
    fn test_indexeddb_key_comparison() {
        let mut context = create_test_context();

        // Test cmp method
        let result = eval_js(&mut context, "indexedDB.cmp(1, 2)");
        assert_eq!(result, "-1");

        let result = eval_js(&mut context, "indexedDB.cmp(2, 1)");
        assert_eq!(result, "1");

        let result = eval_js(&mut context, "indexedDB.cmp(5, 5)");
        assert_eq!(result, "0");

        // String comparison
        let result = eval_js(&mut context, "indexedDB.cmp('apple', 'banana')");
        assert_eq!(result, "-1");
    }

    #[test]
    fn test_indexeddb_keyrange_static_methods() {
        let mut context = create_test_context();

        // Test that IDBKeyRange constructor exists
        let result = eval_js(&mut context, "typeof IDBKeyRange");
        assert_eq!(result, "function");

        // Test static methods exist
        let result = eval_js(&mut context, "typeof IDBKeyRange.only");
        assert_eq!(result, "function");

        let result = eval_js(&mut context, "typeof IDBKeyRange.lowerBound");
        assert_eq!(result, "function");

        let result = eval_js(&mut context, "typeof IDBKeyRange.upperBound");
        assert_eq!(result, "function");

        let result = eval_js(&mut context, "typeof IDBKeyRange.bound");
        assert_eq!(result, "function");
    }

    #[test]
    fn test_idb_key_from_js_conversion() {
        use super::super::key::IDBKey;
        use boa_engine::JsValue;

        let mut context = create_test_context();

        // Test number conversion
        let js_num = JsValue::from(42.0);
        let key = IDBKey::from_js_value(&js_num, &mut context).unwrap();
        assert!(matches!(key, IDBKey::Number(n) if (n - 42.0).abs() < f64::EPSILON));

        // Test string conversion
        let js_str = JsValue::from(boa_engine::js_string!("test"));
        let key = IDBKey::from_js_value(&js_str, &mut context).unwrap();
        assert!(matches!(key, IDBKey::String(ref s) if s == "test"));
    }

    #[test]
    fn test_idb_key_ordering() {
        use super::super::key::IDBKey;

        // Test key ordering
        let key1 = IDBKey::Number(1.0);
        let key2 = IDBKey::Number(2.0);
        let key3 = IDBKey::String("a".to_string());
        let key4 = IDBKey::String("b".to_string());

        // Numbers come before strings
        assert!(key1 < key3);
        assert!(key2 < key4);

        // Within same type
        assert!(key1 < key2);
        assert!(key3 < key4);
    }

    #[test]
    fn test_idb_keyrange_includes() {
        use super::super::key::IDBKey;
        use super::super::key_range::IDBKeyRange;

        // Test range.includes()
        let range = IDBKeyRange::new(
            Some(IDBKey::Number(1.0)),
            Some(IDBKey::Number(10.0)),
            false,
            false,
        )
        .unwrap();

        assert!(range.includes(&IDBKey::Number(5.0)));
        assert!(range.includes(&IDBKey::Number(1.0)));
        assert!(range.includes(&IDBKey::Number(10.0)));
        assert!(!range.includes(&IDBKey::Number(0.0)));
        assert!(!range.includes(&IDBKey::Number(11.0)));

        // Test open bounds
        let open_range = IDBKeyRange::new(
            Some(IDBKey::Number(1.0)),
            Some(IDBKey::Number(10.0)),
            true,
            true,
        )
        .unwrap();

        assert!(open_range.includes(&IDBKey::Number(5.0)));
        assert!(!open_range.includes(&IDBKey::Number(1.0)));
        assert!(!open_range.includes(&IDBKey::Number(10.0)));
    }

    #[test]
    fn test_memory_backend_operations() {
        use super::super::backend::StorageBackend;
        use super::super::backend::memory::MemoryBackend;
        use super::super::key::IDBKey;

        let mut backend = MemoryBackend::new();

        // Open database
        let handle = backend.open_database("testDB", 1).unwrap();
        assert_eq!(handle.name, "testDB");
        assert_eq!(handle.version, 1);

        // Create object store
        backend
            .create_object_store("testDB", "users", Some("id".to_string()), false)
            .unwrap();

        // Add data
        let key = IDBKey::Number(1.0);
        let value = b"test data";
        backend.add("testDB", "users", &key, value).unwrap();

        // Get data
        let retrieved = backend.get("testDB", "users", &key).unwrap();
        assert_eq!(retrieved, Some(value.to_vec()));

        // Delete data
        backend.delete("testDB", "users", &key).unwrap();
        let after_delete = backend.get("testDB", "users", &key).unwrap();
        assert_eq!(after_delete, None);
    }

    #[test]
    fn test_memory_backend_index_operations() {
        use super::super::backend::StorageBackend;
        use super::super::backend::memory::MemoryBackend;
        use super::super::key::IDBKey;

        let mut backend = MemoryBackend::new();

        // Setup
        backend.open_database("testDB", 1).unwrap();
        backend
            .create_object_store("testDB", "users", Some("id".to_string()), false)
            .unwrap();
        backend
            .create_index("testDB", "users", "email_idx", "email", true, false)
            .unwrap();

        // Query by index (note: indexing values requires proper JSON structure in real usage)
        let index_key = IDBKey::String("test@example.com".to_string());
        let result = backend.get_by_index("testDB", "users", "email_idx", &index_key);
        // Index lookups may return None if no values indexed yet
        assert!(result.is_ok());
    }

    #[test]
    fn test_idb_request_state_management() {
        let mut context = create_test_context();

        // Test IDBRequest through JavaScript API
        // We'll test this by opening a database and checking the request object
        let code = r#"
            let request = indexedDB.open('testdb', 1);
            typeof request.readyState;
        "#;
        let result = eval_js(&mut context, code);
        assert_eq!(result, "string");
    }

    #[test]
    fn test_sled_backend_persistence() {
        use super::super::backend::StorageBackend;
        use super::super::backend::sled_backend::SledBackend;
        use super::super::key::IDBKey;
        use std::path::PathBuf;

        let temp_dir =
            std::env::temp_dir().join(format!("thalora_idb_test_{}", std::process::id()));

        // Create backend and add data
        {
            let mut backend = SledBackend::new(temp_dir.clone()).unwrap();
            backend.open_database("persistDB", 1).unwrap();
            backend
                .create_object_store("persistDB", "data", None, false)
                .unwrap();

            let key = IDBKey::String("test".to_string());
            let value = b"persistent data";
            backend.add("persistDB", "data", &key, value).unwrap();
        }

        // Reopen and verify data persists
        {
            let backend = SledBackend::new(temp_dir.clone()).unwrap();
            let key = IDBKey::String("test".to_string());
            let result = backend.get("persistDB", "data", &key).unwrap();
            assert_eq!(result, Some(b"persistent data".to_vec()));
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_auto_increment_keys() {
        use super::super::backend::StorageBackend;
        use super::super::backend::memory::MemoryBackend;
        use super::super::key::IDBKey;

        let mut backend = MemoryBackend::new();
        backend.open_database("testDB", 1).unwrap();
        backend
            .create_object_store("testDB", "items", None, true)
            .unwrap();

        // Add with auto-increment
        let key1 = backend
            .add("testDB", "items", &IDBKey::Number(0.0), b"item1")
            .unwrap();
        let key2 = backend
            .add("testDB", "items", &IDBKey::Number(0.0), b"item2")
            .unwrap();

        // Keys should be auto-incremented
        assert!(matches!(key1, IDBKey::Number(n) if n > 0.0));
        assert!(matches!(key2, IDBKey::Number(n) if n > 0.0));

        // key2 should be greater than key1
        assert!(key2 > key1);
    }

    #[test]
    fn test_range_queries() {
        use super::super::backend::StorageBackend;
        use super::super::backend::memory::MemoryBackend;
        use super::super::key::IDBKey;
        use super::super::key_range::IDBKeyRange;

        let mut backend = MemoryBackend::new();
        backend.open_database("testDB", 1).unwrap();
        backend
            .create_object_store("testDB", "numbers", None, false)
            .unwrap();

        // Add data
        for i in 1..=10 {
            backend
                .add("testDB", "numbers", &IDBKey::Number(i as f64), &[i])
                .unwrap();
        }

        // Query range [3, 7]
        let range = IDBKeyRange::new(
            Some(IDBKey::Number(3.0)),
            Some(IDBKey::Number(7.0)),
            false,
            false,
        )
        .unwrap();

        let results = backend
            .get_all("testDB", "numbers", Some(&range), None)
            .unwrap();
        assert_eq!(results.len(), 5); // 3, 4, 5, 6, 7
    }

    #[test]
    fn test_transaction_lifecycle() {
        use super::super::backend::memory::MemoryBackend;
        use super::super::backend::{StorageBackend, TransactionMode};

        let mut backend = MemoryBackend::new();
        backend.open_database("testDB", 1).unwrap();

        // Begin transaction
        let tx = backend
            .begin_transaction(&["store1".to_string()], TransactionMode::ReadWrite)
            .unwrap();
        assert!(tx.id > 0);

        // Commit transaction
        backend.commit_transaction(tx.id).unwrap();

        // Begin another and abort
        let tx2 = backend
            .begin_transaction(&["store2".to_string()], TransactionMode::ReadOnly)
            .unwrap();
        backend.abort_transaction(tx2.id).unwrap();
    }

    #[test]
    fn test_count_operations() {
        use super::super::backend::StorageBackend;
        use super::super::backend::memory::MemoryBackend;
        use super::super::key::IDBKey;

        let mut backend = MemoryBackend::new();
        backend.open_database("testDB", 1).unwrap();
        backend
            .create_object_store("testDB", "items", None, false)
            .unwrap();

        // Initially empty
        let count = backend.count("testDB", "items", None).unwrap();
        assert_eq!(count, 0);

        // Add items
        for i in 1..=5 {
            backend
                .add("testDB", "items", &IDBKey::Number(i as f64), &[i])
                .unwrap();
        }

        let count = backend.count("testDB", "items", None).unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_key_serialization() {
        use super::super::key::IDBKey;

        // Test number key
        let num_key = IDBKey::Number(42.0);
        let bytes = num_key.to_bytes();
        let restored = IDBKey::from_bytes(&bytes).unwrap();
        assert_eq!(num_key, restored);

        // Test string key
        let str_key = IDBKey::String("test".to_string());
        let bytes = str_key.to_bytes();
        let restored = IDBKey::from_bytes(&bytes).unwrap();
        assert_eq!(str_key, restored);

        // Test date key
        let date_key = IDBKey::Date(1609459200000.0); // 2021-01-01
        let bytes = date_key.to_bytes();
        let restored = IDBKey::from_bytes(&bytes).unwrap();
        assert_eq!(date_key, restored);
    }

    #[test]
    fn test_put_vs_add() {
        use super::super::backend::StorageBackend;
        use super::super::backend::memory::MemoryBackend;
        use super::super::key::IDBKey;

        let mut backend = MemoryBackend::new();
        backend.open_database("testDB", 1).unwrap();
        backend
            .create_object_store("testDB", "store", None, false)
            .unwrap();

        let key = IDBKey::Number(1.0);

        // Add first time - should succeed
        backend.add("testDB", "store", &key, b"first").unwrap();

        // Add again - should fail (key already exists)
        let result = backend.add("testDB", "store", &key, b"second");
        assert!(result.is_err());

        // Put should succeed (updates existing)
        backend.put("testDB", "store", &key, b"updated").unwrap();

        let value = backend.get("testDB", "store", &key).unwrap();
        assert_eq!(value, Some(b"updated".to_vec()));
    }

    #[test]
    fn test_clear_object_store() {
        use super::super::backend::StorageBackend;
        use super::super::backend::memory::MemoryBackend;
        use super::super::key::IDBKey;

        let mut backend = MemoryBackend::new();
        backend.open_database("testDB", 1).unwrap();
        backend
            .create_object_store("testDB", "store", None, false)
            .unwrap();

        // Add multiple items
        for i in 1..=5 {
            backend
                .add("testDB", "store", &IDBKey::Number(i as f64), &[i])
                .unwrap();
        }

        let count_before = backend.count("testDB", "store", None).unwrap();
        assert_eq!(count_before, 5);

        // Clear store
        backend.clear("testDB", "store").unwrap();

        let count_after = backend.count("testDB", "store", None).unwrap();
        assert_eq!(count_after, 0);
    }

    #[test]
    fn test_get_all_keys() {
        use super::super::backend::StorageBackend;
        use super::super::backend::memory::MemoryBackend;
        use super::super::key::IDBKey;

        let mut backend = MemoryBackend::new();
        backend.open_database("testDB", 1).unwrap();
        backend
            .create_object_store("testDB", "store", None, false)
            .unwrap();

        // Add data
        for i in 1..=5 {
            backend
                .add("testDB", "store", &IDBKey::Number(i as f64), &[i])
                .unwrap();
        }

        // Get all keys
        let keys = backend.get_all_keys("testDB", "store", None, None).unwrap();
        assert_eq!(keys.len(), 5);

        // Verify keys are in order
        for (i, key) in keys.iter().enumerate() {
            assert_eq!(*key, IDBKey::Number((i + 1) as f64));
        }
    }

    #[test]
    fn test_index_count() {
        use super::super::backend::StorageBackend;
        use super::super::backend::memory::MemoryBackend;
        use super::super::key::IDBKey;

        let mut backend = MemoryBackend::new();
        backend.open_database("testDB", 1).unwrap();
        backend
            .create_object_store("testDB", "users", None, false)
            .unwrap();
        backend
            .create_index("testDB", "users", "age_idx", "age", false, false)
            .unwrap();

        let count = backend
            .count_index("testDB", "users", "age_idx", None)
            .unwrap();
        assert_eq!(count, 0); // No entries yet
    }
}
