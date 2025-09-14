// Tests for src/features/fingerprinting.rs
#[cfg(test)]
mod fingerprinting_tests {
    use synaptic::features::fingerprinting::*;

    #[test]
    fn test_fingerprint_manager_creation() {
        let manager = FingerprintManager::new();
        let fingerprint = manager.generate_fingerprint(BrowserType::Chrome);

        assert!(!fingerprint.user_agent.is_empty());
        assert!(!fingerprint.webgl_vendor.is_empty());
        assert!(!fingerprint.webgl_renderer.is_empty());
    }

    #[test]
    fn test_different_browser_types() {
        let manager = FingerprintManager::new();

        let chrome_fp = manager.generate_fingerprint(BrowserType::Chrome);
        let firefox_fp = manager.generate_fingerprint(BrowserType::Firefox);
        let safari_fp = manager.generate_fingerprint(BrowserType::Safari);

        // Each browser type should generate different user agents
        assert_ne!(chrome_fp.user_agent, firefox_fp.user_agent);
        assert_ne!(chrome_fp.user_agent, safari_fp.user_agent);
        assert_ne!(firefox_fp.user_agent, safari_fp.user_agent);
    }
}