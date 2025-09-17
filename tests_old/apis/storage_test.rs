// Tests for src/apis/storage.rs
#[cfg(test)]
mod storage_tests {
    use thalora::apis::storage::*;

    #[test]
    fn test_web_storage_creation() {
        let storage = WebStorage::new();
        let data = storage.get_local_storage_data();
        assert!(data.is_empty());
    }

    #[test]
    fn test_storage_operations() {
        let storage = WebStorage::new();
        let session_data = storage.get_session_storage_data();
        assert!(session_data.is_empty());
    }
}