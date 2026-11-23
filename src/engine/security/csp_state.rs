//! Process-wide CSP (Content Security Policy) state for static resources.
//!
//! Navigation parses the page's `Content-Security-Policy` header and stores
//! the parts that apply without a JavaScript engine (`img-src`, `style-src`
//! and their `default-src` fallback) here, so the scraper and the layout
//! engine can check images and inline styles while processing a page.

use std::sync::{LazyLock, Mutex};

/// The subset of a page's CSP that static HTML/CSS processing enforces.
#[derive(Debug, Clone, Default)]
pub struct CspState {
    /// The page URL (origin) for 'self' checks.
    pub page_url: Option<String>,
    /// Whether any CSP policy is active.
    pub has_policy: bool,
    /// img-src directive sources.
    pub img_src: Vec<CspSource>,
    /// style-src directive sources (for inline style checks).
    pub style_src: Vec<CspSource>,
    /// default-src directive sources (fallback).
    pub default_src: Vec<CspSource>,
}

/// A simplified CSP source expression.
#[derive(Debug, Clone, PartialEq)]
pub enum CspSource {
    /// 'self' — matches the page's own origin
    OriginSelf,
    /// 'unsafe-inline' — allows inline styles
    UnsafeInline,
    /// 'none' — blocks everything
    None,
    /// URL pattern (e.g., `https://cdn.example.com`, `*.example.com`, `https:`)
    Url(String),
}

impl CspState {
    /// Check if an image URL is allowed by img-src.
    pub fn allows_image(&self, url: &str) -> bool {
        // Relative URLs (no scheme) are always same-origin — always allowed.
        // This covers paths like "/static/images/..." and "//cdn.example.com/..."
        if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("data:")
        {
            return true;
        }
        if !self.has_policy {
            return true;
        }
        allows_url(
            &self.img_src,
            &self.default_src,
            url,
            self.page_url.as_deref(),
        )
    }

    /// Check if inline styles are allowed by style-src.
    pub fn allows_inline_style(&self) -> bool {
        if !self.has_policy {
            return true;
        }
        let sources = if !self.style_src.is_empty() {
            &self.style_src
        } else if !self.default_src.is_empty() {
            &self.default_src
        } else {
            return true; // No applicable policy
        };
        sources.iter().any(|s| matches!(s, CspSource::UnsafeInline))
    }
}

/// CSP state of the page being processed, set by navigation.
static CSP_STATE: LazyLock<Mutex<CspState>> = LazyLock::new(|| Mutex::new(CspState::default()));

/// Set the current CSP policy state. Called by navigation after parsing headers.
pub fn set_csp_state(state: CspState) {
    if let Ok(mut guard) = CSP_STATE.lock() {
        *guard = state;
    }
}

/// Clear the CSP state (e.g., on navigation to a page without a policy).
pub fn clear_csp_state() {
    if let Ok(mut guard) = CSP_STATE.lock() {
        *guard = CspState::default();
    }
}

/// Check if an image URL is allowed by the current page's img-src.
pub fn csp_allows_image(url: &str) -> bool {
    match CSP_STATE.lock() {
        Ok(guard) => guard.allows_image(url),
        Err(_) => true, // Poisoned lock = allow
    }
}

/// Check if inline styles are allowed by the current page's style-src.
pub fn csp_allows_inline_style() -> bool {
    match CSP_STATE.lock() {
        Ok(guard) => guard.allows_inline_style(),
        Err(_) => true,
    }
}

/// Check if a URL is allowed by a directive, with default-src fallback.
fn allows_url(
    directive: &[CspSource],
    default_src: &[CspSource],
    url: &str,
    page_url: Option<&str>,
) -> bool {
    let sources = if !directive.is_empty() {
        directive
    } else if !default_src.is_empty() {
        default_src
    } else {
        return true; // No policy = allow all
    };

    for source in sources {
        match source {
            CspSource::None => return false,
            CspSource::OriginSelf => {
                if let Some(page) = page_url
                    && same_origin(page, url)
                {
                    return true;
                }
            }
            CspSource::Url(pattern) => {
                if url_matches_pattern(url, pattern) {
                    return true;
                }
            }
            CspSource::UnsafeInline => continue,
        }
    }

    false
}

/// Simple same-origin check: compare scheme + host + port.
fn same_origin(url_a: &str, url_b: &str) -> bool {
    let parse_origin = |u: &str| -> Option<(String, String, u16)> {
        let parsed = url::Url::parse(u).ok()?;
        let scheme = parsed.scheme().to_string();
        let host = parsed.host_str()?.to_string();
        let port = parsed
            .port_or_known_default()
            .unwrap_or(if scheme == "https" { 443 } else { 80 });
        Some((scheme, host, port))
    };

    match (parse_origin(url_a), parse_origin(url_b)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// Check if a URL matches a CSP URL pattern.
fn url_matches_pattern(url: &str, pattern: &str) -> bool {
    // Scheme-only patterns (e.g., "https:")
    if pattern.ends_with(':') && !pattern.contains("//") {
        return url.starts_with(pattern);
    }

    // Wildcard subdomain (e.g., "*.example.com")
    if pattern.starts_with("*.") {
        let domain_suffix = &pattern[1..]; // ".example.com"
        if let Ok(parsed) = url::Url::parse(url)
            && let Some(host) = parsed.host_str()
        {
            return host.ends_with(domain_suffix) || host == &domain_suffix[1..];
        }
        return false;
    }

    // Exact host or host+path match
    if let (Ok(url_parsed), Ok(pattern_parsed)) = (url::Url::parse(url), url::Url::parse(pattern)) {
        return url_parsed.scheme() == pattern_parsed.scheme()
            && url_parsed.host_str() == pattern_parsed.host_str()
            && url_parsed.port_or_known_default() == pattern_parsed.port_or_known_default();
    }

    // Simple string prefix match as fallback
    url.starts_with(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    // These exercise `CspState` values directly rather than the process-wide
    // state, which other tests running in parallel read during layout.

    fn policy() -> CspState {
        CspState {
            has_policy: true,
            page_url: Some("https://example.com".to_string()),
            ..CspState::default()
        }
    }

    #[test]
    fn test_no_policy_allows_all() {
        let state = CspState::default();
        assert!(state.allows_image("https://any.com/img.png"));
        assert!(state.allows_inline_style());
    }

    #[test]
    fn test_img_src_url_pattern() {
        let state = CspState {
            img_src: vec![
                CspSource::OriginSelf,
                CspSource::Url("https://cdn.example.com".to_string()),
            ],
            ..policy()
        };
        assert!(state.allows_image("https://example.com/logo.png"));
        assert!(state.allows_image("https://cdn.example.com/photo.jpg"));
        assert!(!state.allows_image("https://evil.com/tracker.gif"));
        assert!(state.allows_image("/relative/logo.png"));
    }

    #[test]
    fn test_img_src_none_blocks_all() {
        let state = CspState {
            img_src: vec![CspSource::None],
            ..policy()
        };
        assert!(!state.allows_image("https://example.com/logo.png"));
    }

    #[test]
    fn test_inline_style_blocked_without_unsafe_inline() {
        let state = CspState {
            style_src: vec![CspSource::OriginSelf],
            ..policy()
        };
        assert!(!state.allows_inline_style());
    }

    #[test]
    fn test_inline_style_allowed_with_unsafe_inline() {
        let state = CspState {
            style_src: vec![CspSource::OriginSelf, CspSource::UnsafeInline],
            ..policy()
        };
        assert!(state.allows_inline_style());
    }

    #[test]
    fn test_default_src_fallback() {
        let state = CspState {
            default_src: vec![CspSource::OriginSelf],
            ..policy()
        };
        assert!(state.allows_image("https://example.com/logo.png"));
        assert!(!state.allows_image("https://other.com/logo.png"));
        assert!(!state.allows_inline_style());
    }
}
