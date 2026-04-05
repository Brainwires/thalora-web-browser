//! Mixed Content Blocking
//!
//! Implements the W3C Mixed Content specification to prevent loading
//! insecure (HTTP) subresources on secure (HTTPS) pages.
//! https://www.w3.org/TR/mixed-content/

/// Resource types that are always blocked when loaded over HTTP on an HTTPS page
/// (a.k.a. "blockable" or "active" mixed content per the spec).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Script,
    Stylesheet,
    Font,
    Iframe,
    Xhr,
    Fetch,
    /// Images, audio, video are "optionally blockable" (passive mixed content).
    /// We block them by default for stronger security.
    Image,
    Media,
    /// Unknown resource type - block by default
    Other,
}

/// Check whether a resource URL should be blocked due to mixed content policy.
///
/// Returns `true` if the resource should be blocked (HTTP resource on HTTPS page).
/// Returns `false` if the resource is safe to load.
///
/// If `upgrade_insecure` is true, the caller should rewrite the URL from http:// to https://
/// instead of blocking (per `upgrade-insecure-requests` CSP directive).
pub fn should_block_mixed_content(
    page_url: &str,
    resource_url: &str,
    _resource_type: ResourceType,
) -> MixedContentResult {
    // Only enforce when the page is loaded over HTTPS
    if !page_url.starts_with("https://") {
        return MixedContentResult::Allow;
    }

    // Check if the resource URL is HTTP
    if resource_url.starts_with("http://") {
        return MixedContentResult::Block;
    }

    // Protocol-relative URLs (//example.com/...) inherit the page protocol
    // so they're fine on HTTPS pages
    // Data URLs, blob URLs, etc. are also fine
    MixedContentResult::Allow
}

/// Attempt to upgrade an HTTP URL to HTTPS.
///
/// Used when `upgrade-insecure-requests` CSP directive is present.
/// Returns the upgraded URL, or None if the URL can't be upgraded.
pub fn upgrade_url_to_https(url: &str) -> Option<String> {
    if url.starts_with("http://") {
        Some(format!("https://{}", &url[7..]))
    } else {
        None
    }
}

/// Result of mixed content check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixedContentResult {
    /// Resource is safe to load
    Allow,
    /// Resource should be blocked (HTTP on HTTPS page)
    Block,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_https_page_http_script_blocked() {
        assert_eq!(
            should_block_mixed_content("https://example.com", "http://cdn.example.com/script.js", ResourceType::Script),
            MixedContentResult::Block
        );
    }

    #[test]
    fn test_https_page_https_script_allowed() {
        assert_eq!(
            should_block_mixed_content("https://example.com", "https://cdn.example.com/script.js", ResourceType::Script),
            MixedContentResult::Allow
        );
    }

    #[test]
    fn test_http_page_http_script_allowed() {
        // No mixed content on HTTP pages
        assert_eq!(
            should_block_mixed_content("http://example.com", "http://cdn.example.com/script.js", ResourceType::Script),
            MixedContentResult::Allow
        );
    }

    #[test]
    fn test_https_page_http_stylesheet_blocked() {
        assert_eq!(
            should_block_mixed_content("https://example.com", "http://cdn.example.com/style.css", ResourceType::Stylesheet),
            MixedContentResult::Block
        );
    }

    #[test]
    fn test_https_page_http_image_blocked() {
        assert_eq!(
            should_block_mixed_content("https://example.com", "http://cdn.example.com/image.png", ResourceType::Image),
            MixedContentResult::Block
        );
    }

    #[test]
    fn test_https_page_data_url_allowed() {
        assert_eq!(
            should_block_mixed_content("https://example.com", "data:text/html,<h1>Hello</h1>", ResourceType::Iframe),
            MixedContentResult::Allow
        );
    }

    #[test]
    fn test_upgrade_http_to_https() {
        assert_eq!(
            upgrade_url_to_https("http://example.com/style.css"),
            Some("https://example.com/style.css".to_string())
        );
    }

    #[test]
    fn test_upgrade_already_https() {
        assert_eq!(
            upgrade_url_to_https("https://example.com/style.css"),
            None
        );
    }

    #[test]
    fn test_upgrade_data_url() {
        assert_eq!(
            upgrade_url_to_https("data:text/html,hello"),
            None
        );
    }
}
