//! Shared CSP (Content Security Policy) state for cross-crate enforcement.
//!
//! The main crate sets the CSP policy after parsing HTTP headers, and
//! browser API implementations (fetch, eval, etc.) check it at runtime.
//!
//! Uses a static `LazyLock<Mutex<...>>` pattern consistent with other
//! shared state in this crate (e.g., `PREFLIGHT_CACHE`, `CLIPBOARD_STORAGE`).

use std::sync::{LazyLock, Mutex};

/// Lightweight CSP state shared across crate boundaries.
/// This mirrors the relevant fields from the main crate's `CspPolicy`
/// without creating a dependency cycle.
#[derive(Debug, Clone, Default)]
pub struct CspState {
    /// The page URL (origin) for 'self' checks.
    pub page_url: Option<String>,
    /// Whether any CSP policy is active.
    pub has_policy: bool,
    /// Whether eval() / new Function() is allowed.
    pub allows_eval: bool,
    /// connect-src directive sources (URL patterns and keywords).
    pub connect_src: Vec<CspSource>,
    /// img-src directive sources.
    pub img_src: Vec<CspSource>,
    /// style-src directive sources (for inline style checks).
    pub style_src: Vec<CspSource>,
    /// default-src directive sources (fallback).
    pub default_src: Vec<CspSource>,
}

/// A simplified CSP source expression for cross-crate use.
#[derive(Debug, Clone, PartialEq)]
pub enum CspSource {
    /// 'self' — matches the page's own origin
    OriginSelf,
    /// 'unsafe-inline' — allows inline scripts/styles
    UnsafeInline,
    /// 'unsafe-eval' — allows eval() and Function()
    UnsafeEval,
    /// 'none' — blocks everything
    None,
    /// URL pattern (e.g., https://cdn.example.com, *.example.com, https:)
    Url(String),
}

/// Global CSP state, set by the main crate before JS execution.
static CSP_STATE: LazyLock<Mutex<CspState>> = LazyLock::new(|| Mutex::new(CspState::default()));

/// Set the current CSP policy state. Called by the main crate before JS execution.
pub fn set_csp_state(state: CspState) {
    if let Ok(mut guard) = CSP_STATE.lock() {
        *guard = state;
    }
}

/// Clear the CSP state (e.g., on navigation to a new page).
pub fn clear_csp_state() {
    if let Ok(mut guard) = CSP_STATE.lock() {
        *guard = CspState::default();
    }
}

/// Check if a URL is allowed by connect-src (for fetch/XHR/WebSocket).
pub fn csp_allows_connect(url: &str) -> bool {
    let guard = match CSP_STATE.lock() {
        Ok(g) => g,
        Err(_) => return true, // Poisoned lock = allow
    };

    if !guard.has_policy {
        return true;
    }

    allows_url(
        &guard.connect_src,
        &guard.default_src,
        url,
        guard.page_url.as_deref(),
    )
}

/// Check if eval() / new Function() is allowed by CSP.
pub fn csp_allows_eval() -> bool {
    let guard = match CSP_STATE.lock() {
        Ok(g) => g,
        Err(_) => return true,
    };

    if !guard.has_policy {
        return true;
    }

    guard.allows_eval
}

/// Check if an image URL is allowed by img-src.
pub fn csp_allows_image(url: &str) -> bool {
    let guard = match CSP_STATE.lock() {
        Ok(g) => g,
        Err(_) => return true,
    };

    if !guard.has_policy {
        return true;
    }

    allows_url(
        &guard.img_src,
        &guard.default_src,
        url,
        guard.page_url.as_deref(),
    )
}

/// Check if inline styles are allowed by style-src.
pub fn csp_allows_inline_style() -> bool {
    let guard = match CSP_STATE.lock() {
        Ok(g) => g,
        Err(_) => return true,
    };

    if !guard.has_policy {
        return true;
    }

    let sources = if !guard.style_src.is_empty() {
        &guard.style_src
    } else if !guard.default_src.is_empty() {
        &guard.default_src
    } else {
        return true; // No applicable policy
    };

    sources.iter().any(|s| matches!(s, CspSource::UnsafeInline))
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
                if let Some(page) = page_url {
                    if same_origin(page, url) {
                        return true;
                    }
                }
            }
            CspSource::Url(pattern) => {
                if url_matches_pattern(url, pattern) {
                    return true;
                }
            }
            CspSource::UnsafeInline | CspSource::UnsafeEval => continue,
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
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                return host.ends_with(domain_suffix) || host == &domain_suffix[1..];
            }
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

    #[test]
    fn test_no_policy_allows_all() {
        clear_csp_state();
        assert!(csp_allows_connect("https://any.com/api"));
        assert!(csp_allows_eval());
        assert!(csp_allows_image("https://any.com/img.png"));
        assert!(csp_allows_inline_style());
    }

    #[test]
    fn test_connect_src_blocks_disallowed() {
        set_csp_state(CspState {
            has_policy: true,
            page_url: Some("https://example.com".to_string()),
            allows_eval: true,
            connect_src: vec![CspSource::OriginSelf],
            default_src: vec![],
            img_src: vec![],
            style_src: vec![],
        });

        assert!(csp_allows_connect("https://example.com/api"));
        assert!(!csp_allows_connect("https://evil.com/api"));

        // Clean up
        clear_csp_state();
    }

    #[test]
    fn test_connect_src_none_blocks_all() {
        set_csp_state(CspState {
            has_policy: true,
            page_url: Some("https://example.com".to_string()),
            allows_eval: true,
            connect_src: vec![CspSource::None],
            default_src: vec![],
            img_src: vec![],
            style_src: vec![],
        });

        assert!(!csp_allows_connect("https://example.com/api"));

        clear_csp_state();
    }

    #[test]
    fn test_eval_blocked() {
        set_csp_state(CspState {
            has_policy: true,
            page_url: None,
            allows_eval: false,
            connect_src: vec![],
            default_src: vec![],
            img_src: vec![],
            style_src: vec![],
        });

        assert!(!csp_allows_eval());

        clear_csp_state();
    }

    #[test]
    fn test_img_src_url_pattern() {
        set_csp_state(CspState {
            has_policy: true,
            page_url: Some("https://example.com".to_string()),
            allows_eval: true,
            connect_src: vec![],
            default_src: vec![],
            img_src: vec![
                CspSource::OriginSelf,
                CspSource::Url("https://cdn.example.com".to_string()),
            ],
            style_src: vec![],
        });

        assert!(csp_allows_image("https://example.com/logo.png"));
        assert!(csp_allows_image("https://cdn.example.com/photo.jpg"));
        assert!(!csp_allows_image("https://evil.com/tracker.gif"));

        clear_csp_state();
    }

    #[test]
    fn test_inline_style_blocked_without_unsafe_inline() {
        set_csp_state(CspState {
            has_policy: true,
            page_url: None,
            allows_eval: true,
            connect_src: vec![],
            default_src: vec![],
            img_src: vec![],
            style_src: vec![CspSource::OriginSelf],
        });

        assert!(!csp_allows_inline_style());

        clear_csp_state();
    }

    #[test]
    fn test_inline_style_allowed_with_unsafe_inline() {
        set_csp_state(CspState {
            has_policy: true,
            page_url: None,
            allows_eval: true,
            connect_src: vec![],
            default_src: vec![],
            img_src: vec![],
            style_src: vec![CspSource::OriginSelf, CspSource::UnsafeInline],
        });

        assert!(csp_allows_inline_style());

        clear_csp_state();
    }

    #[test]
    fn test_default_src_fallback() {
        set_csp_state(CspState {
            has_policy: true,
            page_url: Some("https://example.com".to_string()),
            allows_eval: true,
            connect_src: vec![], // empty = fallback to default-src
            default_src: vec![CspSource::OriginSelf],
            img_src: vec![],
            style_src: vec![],
        });

        assert!(csp_allows_connect("https://example.com/api"));
        assert!(!csp_allows_connect("https://other.com/api"));

        clear_csp_state();
    }
}
