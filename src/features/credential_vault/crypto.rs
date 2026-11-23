use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use anyhow::{Context, Result, anyhow};
use argon2::{
    Argon2, ParamsBuilder,
    password_hash::{PasswordHasher, SaltString},
};
use base64::{Engine as _, engine::general_purpose};
use std::env;
use zeroize::Zeroizing;

/// Version prefix for encrypted data format
const VERSION_V2: &str = "v2";

/// Salt size for Argon2 key derivation (16 bytes = 128 bits)
const SALT_SIZE: usize = 16;

/// Nonce size for AES-256-GCM (12 bytes = 96 bits, recommended for GCM)
const NONCE_SIZE: usize = 12;

/// Key size for AES-256 (32 bytes = 256 bits)
const KEY_SIZE: usize = 32;

/// Get master password from environment variable
/// SECURITY: Master password MUST be set via THALORA_MASTER_PASSWORD environment variable
/// SECURITY: Returns Zeroizing<String> to ensure password is zeroed from memory when dropped
fn get_master_password() -> Result<Zeroizing<String>> {
    env::var("THALORA_MASTER_PASSWORD")
        .map(Zeroizing::new)
        .context(
            "THALORA_MASTER_PASSWORD environment variable not set. \
            Please set a strong master password (min 32 characters) to encrypt credentials.",
        )
}

/// Derive encryption key from master password using Argon2id
/// SECURITY: Returns Zeroizing<[u8; KEY_SIZE]> to ensure key is zeroed from memory when dropped
fn derive_key(master_password: &str, salt: &[u8]) -> Result<Zeroizing<[u8; KEY_SIZE]>> {
    if salt.len() != SALT_SIZE {
        return Err(anyhow!(
            "Invalid salt size: expected {}, got {}",
            SALT_SIZE,
            salt.len()
        ));
    }

    // Configure Argon2id with secure parameters
    // - Memory cost: 64 MB (65536 KiB)
    // - Time cost: 3 iterations
    // - Parallelism: 4 threads
    let params = ParamsBuilder::new()
        .m_cost(65536) // 64 MB
        .t_cost(3) // 3 iterations
        .p_cost(4) // 4 parallel threads
        .output_len(KEY_SIZE)
        .build()
        .context("Failed to build Argon2 parameters")?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    // Derive key using Argon2id
    let salt_string =
        SaltString::encode_b64(salt).map_err(|e| anyhow!("Failed to encode salt: {}", e))?;

    let password_hash = argon2
        .hash_password(master_password.as_bytes(), &salt_string)
        .map_err(|e| anyhow!("Failed to derive key: {}", e))?;

    let hash_output = password_hash
        .hash
        .ok_or_else(|| anyhow!("No hash output from Argon2"))?;

    let hash_bytes = hash_output.as_bytes();
    if hash_bytes.len() != KEY_SIZE {
        return Err(anyhow!("Unexpected key length: {}", hash_bytes.len()));
    }

    // SECURITY: Use Zeroizing wrapper to ensure key is zeroed from memory when dropped
    let mut key = Zeroizing::new([0u8; KEY_SIZE]);
    key.copy_from_slice(hash_bytes);

    Ok(key)
}

/// Encrypt password using AES-256-GCM with Argon2id key derivation
///
/// Format: `v2:base64(salt):base64(nonce):base64(ciphertext+tag)`
///
/// Security features:
/// - AES-256-GCM authenticated encryption (detects tampering)
/// - Argon2id key derivation (resistant to GPU/ASIC attacks)
/// - Random salt per password (prevents rainbow tables)
/// - Random nonce per encryption (prevents pattern analysis)
/// - 128-bit authentication tag (prevents forgery)
pub fn encrypt_password(password: &str) -> Result<String> {
    // Get master password from environment (Zeroizing ensures cleanup on drop)
    let master_password = get_master_password()?;

    // Validate master password strength (minimum 32 characters)
    if master_password.len() < 32 {
        return Err(anyhow!(
            "Master password too weak. Minimum length: 32 characters. \
            Current length: {} characters. \
            Please set a stronger THALORA_MASTER_PASSWORD.",
            master_password.len()
        ));
    }

    // Generate random salt (16 bytes)
    let mut salt = [0u8; SALT_SIZE];
    use aes_gcm::aead::rand_core::RngCore;
    OsRng.fill_bytes(&mut salt);

    // Derive encryption key from master password + salt (Zeroizing ensures cleanup on drop)
    let key = derive_key(&master_password, &salt)?;

    // Create AES-256-GCM cipher (key is automatically zeroed when 'key' goes out of scope)
    let cipher =
        Aes256Gcm::new_from_slice(&*key).map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

    // Generate random nonce (12 bytes)
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt password (includes authentication tag)
    let ciphertext = cipher
        .encrypt(nonce, password.as_bytes())
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    // Encode result: v2:salt:nonce:ciphertext
    let encoded = format!(
        "{}:{}:{}:{}",
        VERSION_V2,
        general_purpose::STANDARD.encode(salt),
        general_purpose::STANDARD.encode(nonce_bytes),
        general_purpose::STANDARD.encode(&ciphertext)
    );

    Ok(encoded)
}

/// Decrypt password from AES-256-GCM encrypted format
///
/// Supports both formats:
/// - v2: `v2:base64(salt):base64(nonce):base64(ciphertext+tag)` (current)
/// - legacy: base64-encoded XOR cipher (INSECURE, will fail with error)
pub fn decrypt_password(encrypted: &str) -> Result<String> {
    // Check if this is v2 format
    if encrypted.starts_with("v2:") {
        return decrypt_password_v2(encrypted);
    }

    // Legacy XOR format is no longer supported - force re-encryption
    Err(anyhow!(
        "Legacy XOR encryption detected. This encryption method is INSECURE and no longer supported. \
        All credentials must be re-encrypted with AES-256-GCM. \
        Please delete the old credential cache at ~/.cache/thalora/ai_memory/ and re-enter credentials."
    ))
}

/// Decrypt v2 format: `v2:base64(salt):base64(nonce):base64(ciphertext+tag)`
fn decrypt_password_v2(encrypted: &str) -> Result<String> {
    // Get master password from environment (Zeroizing ensures cleanup on drop)
    let master_password = get_master_password()?;

    // Parse format: v2:salt:nonce:ciphertext
    let parts: Vec<&str> = encrypted.split(':').collect();
    if parts.len() != 4 {
        return Err(anyhow!(
            "Invalid encrypted format: expected 4 parts, got {}",
            parts.len()
        ));
    }

    if parts[0] != VERSION_V2 {
        return Err(anyhow!("Unknown encryption version: {}", parts[0]));
    }

    // Decode salt
    let salt = general_purpose::STANDARD
        .decode(parts[1])
        .context("Failed to decode salt")?;

    if salt.len() != SALT_SIZE {
        return Err(anyhow!(
            "Invalid salt size: expected {}, got {}",
            SALT_SIZE,
            salt.len()
        ));
    }

    // Decode nonce
    let nonce_bytes = general_purpose::STANDARD
        .decode(parts[2])
        .context("Failed to decode nonce")?;

    if nonce_bytes.len() != NONCE_SIZE {
        return Err(anyhow!(
            "Invalid nonce size: expected {}, got {}",
            NONCE_SIZE,
            nonce_bytes.len()
        ));
    }

    let nonce = Nonce::from_slice(&nonce_bytes);

    // Decode ciphertext
    let ciphertext = general_purpose::STANDARD
        .decode(parts[3])
        .context("Failed to decode ciphertext")?;

    // Derive key from master password + salt (Zeroizing ensures cleanup on drop)
    let key = derive_key(&master_password, &salt)?;

    // Create cipher (key is automatically zeroed when 'key' goes out of scope)
    let cipher =
        Aes256Gcm::new_from_slice(&*key).map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

    // Decrypt and verify authentication tag
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|_| {
        anyhow!(
            "Decryption failed: either the master password is incorrect, \
            or the ciphertext has been tampered with"
        )
    })?;

    // Convert to string
    String::from_utf8(plaintext).context("Decrypted data is not valid UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    fn setup_test_password() {
        // Set a test master password (minimum 32 characters)
        // SAFETY: Tests run sequentially with --test-threads=1 or isolated
        unsafe {
            env::set_var(
                "THALORA_MASTER_PASSWORD",
                "test_master_password_min_32chars_secure",
            )
        };
    }

    #[test]
    #[serial]
    fn test_encrypt_decrypt_roundtrip() {
        setup_test_password();

        let original = "test_password_123!@#";
        let encrypted = encrypt_password(original).unwrap();
        let decrypted = decrypt_password(&encrypted).unwrap();

        assert_eq!(original, decrypted);
    }

    #[test]
    #[serial]
    fn test_encrypt_produces_v2_format() {
        setup_test_password();

        let password = "mypassword";
        let encrypted = encrypt_password(password).unwrap();

        // Should start with v2:
        assert!(encrypted.starts_with("v2:"));

        // Should have 4 colon-separated parts
        let parts: Vec<&str> = encrypted.split(':').collect();
        assert_eq!(parts.len(), 4);
    }

    #[test]
    #[serial]
    fn test_encrypt_different_each_time() {
        setup_test_password();

        let password = "same_password";
        let encrypted1 = encrypt_password(password).unwrap();
        let encrypted2 = encrypt_password(password).unwrap();

        // Due to random salt and nonce, should be different
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to same password
        assert_eq!(decrypt_password(&encrypted1).unwrap(), password);
        assert_eq!(decrypt_password(&encrypted2).unwrap(), password);
    }

    #[test]
    #[serial]
    fn test_tampered_ciphertext_rejected() {
        setup_test_password();

        let password = "test_password";
        let mut encrypted = encrypt_password(password).unwrap();

        // Tamper with the ciphertext part
        if let Some(last_colon) = encrypted.rfind(':') {
            encrypted.push_str("tampered");

            // Should fail to decrypt
            let result = decrypt_password(&encrypted);
            assert!(result.is_err());
        }
    }

    #[test]
    #[serial]
    fn test_wrong_master_password_rejected() {
        setup_test_password();

        let password = "test_password";
        let encrypted = encrypt_password(password).unwrap();

        // Change master password
        // SAFETY: Tests run sequentially with --test-threads=1 or isolated
        unsafe {
            env::set_var(
                "THALORA_MASTER_PASSWORD",
                "wrong_password_min_32chars_wrong!",
            )
        };

        // Should fail to decrypt
        let result = decrypt_password(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_weak_master_password_rejected() {
        // SAFETY: Tests run sequentially with --test-threads=1 or isolated
        unsafe { env::set_var("THALORA_MASTER_PASSWORD", "weak") }; // Less than 32 chars

        let password = "test";
        let result = encrypt_password(password);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too weak"));
    }

    #[test]
    #[serial]
    fn test_legacy_xor_format_rejected() {
        setup_test_password();

        // Simulate legacy XOR encrypted data (base64 encoded)
        let legacy_encrypted = general_purpose::STANDARD.encode(b"fake_xor_encrypted");

        let result = decrypt_password(&legacy_encrypted);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Legacy XOR"));
    }

    #[test]
    #[serial]
    fn test_master_password_required() {
        // SAFETY: Tests run sequentially with --test-threads=1 or isolated
        unsafe { env::remove_var("THALORA_MASTER_PASSWORD") };

        let password = "test";
        let result = encrypt_password(password);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("THALORA_MASTER_PASSWORD")
        );
    }

    #[test]
    #[serial]
    fn test_decrypt_invalid_format() {
        setup_test_password();

        // Invalid format - not enough parts
        let result = decrypt_password("v2:invalid");
        assert!(result.is_err());

        // Invalid version
        let result2 = decrypt_password("v1:a:b:c");
        assert!(result2.is_err());
    }

    #[test]
    fn test_key_derivation_with_different_salts() {
        let master_password = "test_master_password_min_32chars_secure";

        let salt1 = [0u8; SALT_SIZE];
        let salt2 = [1u8; SALT_SIZE];

        let key1 = derive_key(master_password, &salt1).unwrap();
        let key2 = derive_key(master_password, &salt2).unwrap();

        // Different salts should produce different keys
        // Dereference Zeroizing wrappers for comparison
        assert_ne!(*key1, *key2);
    }

    #[test]
    #[serial]
    fn test_unicode_password_support() {
        setup_test_password();

        let password = "пароль🔒日本語";
        let encrypted = encrypt_password(password).unwrap();
        let decrypted = decrypt_password(&encrypted).unwrap();

        assert_eq!(password, decrypted);
    }
}
