use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};

/// Encryption key for password protection
const ENCRYPTION_KEY: &[u8] = b"thalora_ai_memory_key_2025";

/// Simple XOR encryption for passwords (not secure for production, but sufficient for dev cache)
pub(super) fn encrypt_password(password: &str) -> Result<String> {
    let encrypted: Vec<u8> = password
        .bytes()
        .enumerate()
        .map(|(i, b)| b ^ ENCRYPTION_KEY[i % ENCRYPTION_KEY.len()])
        .collect();
    Ok(general_purpose::STANDARD.encode(&encrypted))
}

/// Decrypt password from base64 XOR encoded string
pub(super) fn decrypt_password(encrypted_base64: &str) -> Result<String> {
    let encrypted = general_purpose::STANDARD.decode(encrypted_base64)?;
    let decrypted: Vec<u8> = encrypted
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ ENCRYPTION_KEY[i % ENCRYPTION_KEY.len()])
        .collect();
    Ok(String::from_utf8(decrypted)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original = "test_password_123";
        let encrypted = encrypt_password(original).unwrap();
        let decrypted = decrypt_password(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_encrypt_produces_base64() {
        let password = "mypassword";
        let encrypted = encrypt_password(password).unwrap();
        // Should be valid base64
        assert!(general_purpose::STANDARD.decode(&encrypted).is_ok());
    }
}
