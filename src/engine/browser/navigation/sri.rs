//! Subresource Integrity (SRI) verification
//!
//! Implements the W3C SRI specification for verifying that fetched resources
//! match their expected cryptographic hashes.
//! https://www.w3.org/TR/SRI/

/// Verify Subresource Integrity (SRI) hash for fetched resources.
///
/// The `integrity` attribute contains one or more `<algorithm>-<base64hash>` tokens.
/// Returns true if ANY token matches (per spec, any match = pass).
///
/// Supported algorithms: sha256, sha384, sha512
pub fn verify_integrity(content: &[u8], integrity_attr: &str) -> bool {
    use base64::Engine as _;
    use digest::Digest;

    for token in integrity_attr.split_whitespace() {
        let Some((algo, expected_b64)) = token.split_once('-') else {
            continue;
        };

        let computed = match algo {
            "sha256" => {
                let hash = sha2::Sha256::digest(content);
                base64::engine::general_purpose::STANDARD.encode(hash)
            }
            "sha384" => {
                let hash = sha2::Sha384::digest(content);
                base64::engine::general_purpose::STANDARD.encode(hash)
            }
            "sha512" => {
                let hash = sha2::Sha512::digest(content);
                base64::engine::general_purpose::STANDARD.encode(hash)
            }
            _ => continue, // Unknown algorithm, skip
        };

        if computed == expected_b64 {
            return true; // Any match passes
        }
    }

    false // No tokens matched
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sri_sha256_match() {
        let content = b"alert('Hello, world.');";
        // Precomputed SHA-256 hash
        let hash = {
            use base64::Engine as _;
            use digest::Digest;
            let h = sha2::Sha256::digest(content);
            base64::engine::general_purpose::STANDARD.encode(h)
        };
        let integrity = format!("sha256-{}", hash);
        assert!(verify_integrity(content, &integrity));
    }

    #[test]
    fn test_sri_sha256_mismatch() {
        let content = b"alert('Hello, world.');";
        assert!(!verify_integrity(content, "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="));
    }

    #[test]
    fn test_sri_multiple_tokens_any_match() {
        let content = b"test content";
        let hash = {
            use base64::Engine as _;
            use digest::Digest;
            let h = sha2::Sha256::digest(content);
            base64::engine::general_purpose::STANDARD.encode(h)
        };
        // First token wrong, second correct
        let integrity = format!("sha256-wrong sha256-{}", hash);
        assert!(verify_integrity(content, &integrity));
    }

    #[test]
    fn test_sri_empty_integrity() {
        assert!(!verify_integrity(b"content", ""));
    }

    #[test]
    fn test_sri_unknown_algorithm() {
        assert!(!verify_integrity(b"content", "md5-abc123"));
    }

    #[test]
    fn test_sri_sha384() {
        let content = b"test";
        let hash = {
            use base64::Engine as _;
            use digest::Digest;
            let h = sha2::Sha384::digest(content);
            base64::engine::general_purpose::STANDARD.encode(h)
        };
        assert!(verify_integrity(content, &format!("sha384-{}", hash)));
    }

    #[test]
    fn test_sri_sha512() {
        let content = b"test";
        let hash = {
            use base64::Engine as _;
            use digest::Digest;
            let h = sha2::Sha512::digest(content);
            base64::engine::general_purpose::STANDARD.encode(h)
        };
        assert!(verify_integrity(content, &format!("sha512-{}", hash)));
    }
}
