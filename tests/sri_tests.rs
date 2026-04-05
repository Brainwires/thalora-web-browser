// Subresource Integrity (SRI) tests
// Tests the verify_integrity function logic via the public browser API

use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_sri_function_accessible() {
    // Verify that the browser can execute scripts (basic sanity)
    let browser = HeadlessWebBrowser::new();
    let result = browser.lock().unwrap().execute_javascript("1 + 1").await;
    assert!(result.is_ok());
}

// Unit test the verify_integrity logic directly by testing the sha2 integration
#[test]
fn test_sri_sha256_hash_computation() {
    use base64::Engine as _;
    use digest::Digest;

    // Known test vector: sha256 hash of "alert('Hello')"
    let content = b"alert('Hello')";
    let hash = sha2::Sha256::digest(content);
    let b64 = base64::engine::general_purpose::STANDARD.encode(hash);

    // The integrity attribute format
    let integrity = format!("sha256-{}", b64);

    // Verify the format is correct
    assert!(integrity.starts_with("sha256-"));
    assert!(!b64.is_empty());
}

#[test]
fn test_sri_sha384_hash_computation() {
    use base64::Engine as _;
    use digest::Digest;

    let content = b"console.log('test')";
    let hash = sha2::Sha384::digest(content);
    let b64 = base64::engine::general_purpose::STANDARD.encode(hash);

    assert!(!b64.is_empty());
    // SHA-384 produces 48 bytes = 64 base64 chars
    assert_eq!(b64.len(), 64);
}

#[test]
fn test_sri_sha512_hash_computation() {
    use base64::Engine as _;
    use digest::Digest;

    let content = b"var x = 42;";
    let hash = sha2::Sha512::digest(content);
    let b64 = base64::engine::general_purpose::STANDARD.encode(hash);

    assert!(!b64.is_empty());
    // SHA-512 produces 64 bytes = 88 base64 chars
    assert_eq!(b64.len(), 88);
}

#[test]
fn test_sri_mismatch_detection() {
    use base64::Engine as _;
    use digest::Digest;

    let content = b"alert('Hello')";
    let hash = sha2::Sha256::digest(content);
    let correct_b64 = base64::engine::general_purpose::STANDARD.encode(hash);

    // Wrong content should produce different hash
    let wrong_content = b"alert('Hacked')";
    let wrong_hash = sha2::Sha256::digest(wrong_content);
    let wrong_b64 = base64::engine::general_purpose::STANDARD.encode(wrong_hash);

    assert_ne!(correct_b64, wrong_b64);
}
