use super::*;

#[test]
fn test_credential_manager_creation() {
    let manager = CredentialManager::new();
    assert_eq!(manager.get_all_credentials().len(), 0);
}

#[test]
fn test_store_and_retrieve_credential() {
    let manager = CredentialManager::new();

    let credential = StoredCredential {
        id: "test@example.com".to_string(),
        credential_type: CredentialType::Password,
        password: Some("test_password".to_string()),
        name: Some("Test User".to_string()),
        icon_url: None,
        origin: "https://example.com".to_string(),
        created_at: 1234567890,
    };

    manager.store_credential(credential.clone());

    let stored = manager.get_all_credentials();
    assert_eq!(stored.len(), 1);
    assert_eq!(
        stored.get("test@example.com").unwrap().password,
        credential.password
    );
}

#[test]
fn test_remove_credential() {
    let manager = CredentialManager::new();

    let credential = StoredCredential {
        id: "test@example.com".to_string(),
        credential_type: CredentialType::Password,
        password: Some("test_password".to_string()),
        name: Some("Test User".to_string()),
        icon_url: None,
        origin: "https://example.com".to_string(),
        created_at: 1234567890,
    };

    manager.store_credential(credential);
    assert_eq!(manager.get_all_credentials().len(), 1);

    let removed = manager.remove_credential("test@example.com");
    assert!(removed);
    assert_eq!(manager.get_all_credentials().len(), 0);
}

#[test]
fn test_get_credentials_for_origin() {
    let manager = CredentialManager::new();

    let credential1 = StoredCredential {
        id: "user1@example.com".to_string(),
        credential_type: CredentialType::Password,
        password: Some("password1".to_string()),
        name: Some("User 1".to_string()),
        icon_url: None,
        origin: "https://example.com".to_string(),
        created_at: 1234567890,
    };

    let credential2 = StoredCredential {
        id: "user2@other.com".to_string(),
        credential_type: CredentialType::Password,
        password: Some("password2".to_string()),
        name: Some("User 2".to_string()),
        icon_url: None,
        origin: "https://other.com".to_string(),
        created_at: 1234567891,
    };

    manager.store_credential(credential1);
    manager.store_credential(credential2);

    let example_creds = manager.get_credentials_for_origin("https://example.com");
    assert_eq!(example_creds.len(), 1);
    assert_eq!(example_creds[0].id, "user1@example.com");

    let other_creds = manager.get_credentials_for_origin("https://other.com");
    assert_eq!(other_creds.len(), 1);
    assert_eq!(other_creds[0].id, "user2@other.com");
}
