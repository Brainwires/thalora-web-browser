// Tests for AiMemoryHeap credential management

use thalora::features::AiMemoryHeap;
use std::collections::HashMap;
use tempfile::TempDir;

fn create_test_memory() -> (AiMemoryHeap, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_file = temp_dir.path().join("test_memory.json");
    let memory = AiMemoryHeap::new(&cache_file).expect("Failed to create memory heap");
    (memory, temp_dir)
}

/// Set up test master password for credential encryption
/// SAFETY: Tests should run with --test-threads=1 to avoid env var conflicts
fn setup_test_password() {
    // Set a test master password (minimum 32 characters required by crypto module)
    unsafe { std::env::set_var("THALORA_MASTER_PASSWORD", "test_master_password_min_32chars_secure") };
}

#[test]
fn test_store_and_get_credentials() {
    setup_test_password();
    let (mut memory, _temp) = create_test_memory();

    let mut additional = HashMap::new();
    additional.insert("api_key".to_string(), "sk-test-123".to_string());

    memory.store_credentials(
        "github_creds",
        "github.com",
        "testuser",
        "secretpassword123",
        additional.clone(),
    ).expect("Failed to store credentials");

    let result = memory.get_credentials("github_creds").expect("Failed to get credentials");
    assert!(result.is_some(), "Credentials should be retrievable");

    let (service, username, password, data) = result.unwrap();
    assert_eq!(service, "github.com");
    assert_eq!(username, "testuser");
    assert_eq!(password, "secretpassword123");
    assert_eq!(data.get("api_key"), Some(&"sk-test-123".to_string()));
}

#[test]
fn test_get_nonexistent_credentials() {
    setup_test_password();
    let (mut memory, _temp) = create_test_memory();

    let result = memory.get_credentials("nonexistent").expect("Should not error");
    assert!(result.is_none(), "Should return None for nonexistent credentials");
}

#[test]
fn test_credentials_encryption() {
    setup_test_password();
    let (mut memory, _temp) = create_test_memory();

    // Store sensitive password
    memory.store_credentials(
        "secure_creds",
        "secure-service.com",
        "admin",
        "super_secret_password!@#$%",
        HashMap::new(),
    ).expect("Failed to store");

    // Retrieve and verify password is decrypted correctly
    let result = memory.get_credentials("secure_creds")
        .expect("Failed to get")
        .expect("Should exist");

    assert_eq!(result.2, "super_secret_password!@#$%", "Password should be decrypted correctly");
}

#[test]
fn test_list_credential_keys() {
    setup_test_password();
    let (mut memory, _temp) = create_test_memory();

    // Store multiple credentials
    memory.store_credentials("cred1", "service1.com", "user1", "pass1", HashMap::new())
        .expect("Failed to store");
    memory.store_credentials("cred2", "service2.com", "user2", "pass2", HashMap::new())
        .expect("Failed to store");
    memory.store_credentials("cred3", "service3.com", "user3", "pass3", HashMap::new())
        .expect("Failed to store");

    let keys = memory.list_credential_keys();
    assert_eq!(keys.len(), 3, "Should list 3 credential keys");
}

#[test]
fn test_credentials_with_special_characters() {
    setup_test_password();
    let (mut memory, _temp) = create_test_memory();

    let special_password = r#"p@$$w0rd!@#$%^&*()_+-=[]{}|;':",.<>?/`~"#;

    memory.store_credentials(
        "special_creds",
        "special-service.com",
        "special_user",
        special_password,
        HashMap::new(),
    ).expect("Failed to store credentials with special chars");

    let result = memory.get_credentials("special_creds")
        .expect("Failed to get")
        .expect("Should exist");

    assert_eq!(result.2, special_password, "Special characters should be preserved");
}

#[test]
fn test_credentials_with_unicode() {
    setup_test_password();
    let (mut memory, _temp) = create_test_memory();

    let unicode_password = "密码🔐пароль";
    let unicode_username = "用户名_пользователь";

    memory.store_credentials(
        "unicode_creds",
        "unicode-service.com",
        unicode_username,
        unicode_password,
        HashMap::new(),
    ).expect("Failed to store unicode credentials");

    let result = memory.get_credentials("unicode_creds")
        .expect("Failed to get")
        .expect("Should exist");

    assert_eq!(result.1, unicode_username);
    assert_eq!(result.2, unicode_password);
}

#[test]
fn test_credentials_additional_data() {
    setup_test_password();
    let (mut memory, _temp) = create_test_memory();

    let mut additional = HashMap::new();
    additional.insert("api_key".to_string(), "key123".to_string());
    additional.insert("secret".to_string(), "secret456".to_string());
    additional.insert("region".to_string(), "us-east-1".to_string());
    additional.insert("account_id".to_string(), "123456789".to_string());

    memory.store_credentials(
        "aws_creds",
        "aws.amazon.com",
        "iam_user",
        "aws_password",
        additional,
    ).expect("Failed to store");

    let result = memory.get_credentials("aws_creds")
        .expect("Failed to get")
        .expect("Should exist");

    let (_, _, _, data) = result;
    assert_eq!(data.len(), 4);
    assert_eq!(data.get("api_key"), Some(&"key123".to_string()));
    assert_eq!(data.get("region"), Some(&"us-east-1".to_string()));
}

#[test]
fn test_credentials_overwrite() {
    setup_test_password();
    let (mut memory, _temp) = create_test_memory();

    // Store initial credentials
    memory.store_credentials("overwrite_test", "service.com", "user1", "pass1", HashMap::new())
        .expect("Failed to store initial");

    // Overwrite with new credentials
    memory.store_credentials("overwrite_test", "service.com", "user2", "pass2", HashMap::new())
        .expect("Failed to overwrite");

    let result = memory.get_credentials("overwrite_test")
        .expect("Failed to get")
        .expect("Should exist");

    assert_eq!(result.1, "user2", "Username should be updated");
    assert_eq!(result.2, "pass2", "Password should be updated");
}

#[test]
fn test_credentials_persistence() {
    setup_test_password();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_file = temp_dir.path().join("cred_persist.json");

    // Store credentials
    {
        let mut memory = AiMemoryHeap::new(&cache_file).expect("Failed to create");
        memory.store_credentials(
            "persist_creds",
            "persist-service.com",
            "persist_user",
            "persist_password",
            HashMap::new(),
        ).expect("Failed to store");
    }

    // Reload and verify
    {
        let mut memory = AiMemoryHeap::new(&cache_file).expect("Failed to reload");
        let result = memory.get_credentials("persist_creds")
            .expect("Failed to get")
            .expect("Should persist");

        assert_eq!(result.0, "persist-service.com");
        assert_eq!(result.1, "persist_user");
        assert_eq!(result.2, "persist_password");
    }
}

#[test]
fn test_empty_password() {
    setup_test_password();
    let (mut memory, _temp) = create_test_memory();

    // Some services might have empty passwords (token-based auth)
    memory.store_credentials("empty_pass", "token-service.com", "user", "", HashMap::new())
        .expect("Failed to store empty password");

    let result = memory.get_credentials("empty_pass")
        .expect("Failed to get")
        .expect("Should exist");

    assert_eq!(result.2, "", "Empty password should be preserved");
}

#[test]
fn test_credentials_statistics() {
    setup_test_password();
    let (mut memory, _temp) = create_test_memory();

    memory.store_credentials("cred1", "s1.com", "u1", "p1", HashMap::new()).unwrap();
    memory.store_credentials("cred2", "s2.com", "u2", "p2", HashMap::new()).unwrap();
    memory.store_credentials("cred3", "s3.com", "u3", "p3", HashMap::new()).unwrap();

    let stats = memory.get_statistics();
    assert_eq!(stats.credential_count, 3);
}

#[test]
fn test_long_password() {
    setup_test_password();
    let (mut memory, _temp) = create_test_memory();

    // Test with a very long password
    let long_password = "a".repeat(10000);

    memory.store_credentials(
        "long_pass",
        "service.com",
        "user",
        &long_password,
        HashMap::new(),
    ).expect("Failed to store long password");

    let result = memory.get_credentials("long_pass")
        .expect("Failed to get")
        .expect("Should exist");

    assert_eq!(result.2.len(), 10000);
    assert_eq!(result.2, long_password);
}

#[test]
fn test_multiple_credential_access() {
    setup_test_password();
    let (mut memory, _temp) = create_test_memory();

    memory.store_credentials("multi_access", "service.com", "user", "pass", HashMap::new())
        .expect("Failed to store");

    // Access multiple times - should work consistently
    for i in 0..5 {
        let result = memory.get_credentials("multi_access")
            .expect(&format!("Failed to get on access {}", i))
            .expect("Should exist");
        assert_eq!(result.2, "pass");
    }
}
