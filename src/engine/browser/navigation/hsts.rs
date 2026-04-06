//! HTTP Strict Transport Security (HSTS) implementation.
//!
//! Parses `Strict-Transport-Security` response headers and maintains a store
//! of domains that must be accessed over HTTPS. Before issuing an HTTP request
//! the caller can ask `should_upgrade` to transparently rewrite `http://` URLs
//! to `https://`.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A single HSTS entry for a domain.
#[derive(Debug, Clone)]
pub struct HstsEntry {
    pub max_age: Duration,
    pub include_subdomains: bool,
    pub added: Instant,
}

/// In-memory store of HSTS policies keyed by lowercase domain.
#[derive(Debug, Clone)]
pub struct HstsStore {
    entries: HashMap<String, HstsEntry>,
}

impl Default for HstsStore {
    fn default() -> Self {
        Self::new()
    }
}

impl HstsStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Returns `true` when the entry has exceeded its `max_age`.
    pub fn is_expired(entry: &HstsEntry) -> bool {
        entry.added.elapsed() >= entry.max_age
    }

    /// Parse a `Strict-Transport-Security` header value and store the policy.
    ///
    /// Header format example:
    /// `max-age=31536000; includeSubDomains; preload`
    ///
    /// A `max-age=0` directive signals that the host should be removed from the
    /// store (per RFC 6797 section 6.1.1).
    pub fn parse_header(&mut self, domain: &str, header_value: &str) {
        let domain = domain.to_ascii_lowercase();

        let mut max_age: Option<u64> = None;
        let mut include_subdomains = false;

        for directive in header_value.split(';') {
            let directive = directive.trim();
            let lower = directive.to_ascii_lowercase();

            if lower.starts_with("max-age") {
                // Extract the value after '='
                if let Some(val) = lower.split('=').nth(1)
                    && let Ok(seconds) = val.trim().parse::<u64>()
                {
                    max_age = Some(seconds);
                }
            } else if lower == "includesubdomains" {
                include_subdomains = true;
            }
            // "preload" is informational; we simply ignore it.
        }

        if let Some(seconds) = max_age {
            if seconds == 0 {
                // max-age=0 means "remove this host from the HSTS list"
                self.entries.remove(&domain);
            } else {
                self.entries.insert(
                    domain,
                    HstsEntry {
                        max_age: Duration::from_secs(seconds),
                        include_subdomains,
                        added: Instant::now(),
                    },
                );
            }
        }
    }

    /// Check whether the given URL should be upgraded from `http://` to `https://`.
    ///
    /// Returns `Some(upgraded_url)` when the URL should be upgraded, or `None`
    /// if no HSTS policy applies.
    pub fn should_upgrade(&self, url: &str) -> Option<String> {
        // Only http:// URLs are candidates for upgrade.
        if !url.starts_with("http://") {
            return None;
        }

        let domain = extract_domain(url)?;

        if self.matches_domain(&domain) {
            // Replace the scheme.
            Some(format!("https://{}", &url[7..]))
        } else {
            None
        }
    }

    /// Returns `true` if the domain (or a parent domain with `includeSubDomains`)
    /// has an active, non-expired HSTS entry.
    fn matches_domain(&self, domain: &str) -> bool {
        let domain = domain.to_ascii_lowercase();

        // Exact match.
        if let Some(entry) = self.entries.get(&domain)
            && !Self::is_expired(entry)
        {
            return true;
        }

        // Walk parent domains to check includeSubDomains.
        let mut remaining = domain.as_str();
        while let Some(dot_pos) = remaining.find('.') {
            remaining = &remaining[dot_pos + 1..];
            if let Some(entry) = self.entries.get(remaining)
                && entry.include_subdomains
                && !Self::is_expired(entry)
            {
                return true;
            }
        }

        false
    }
}

/// Extract the host (domain) portion from a URL string.
fn extract_domain(url: &str) -> Option<String> {
    // Strip scheme
    let after_scheme = if let Some(rest) = url.strip_prefix("http://") {
        rest
    } else if let Some(rest) = url.strip_prefix("https://") {
        rest
    } else {
        return None;
    };

    // Take everything before the first '/' or '?' or '#' or ':'
    let host = after_scheme.split(['/', '?', '#', ':']).next()?;

    if host.is_empty() {
        None
    } else {
        Some(host.to_ascii_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_header() {
        let mut store = HstsStore::new();
        store.parse_header("example.com", "max-age=31536000");
        let entry = store.entries.get("example.com").unwrap();
        assert_eq!(entry.max_age, Duration::from_secs(31536000));
        assert!(!entry.include_subdomains);
    }

    #[test]
    fn parse_header_with_include_subdomains() {
        let mut store = HstsStore::new();
        store.parse_header(
            "example.com",
            "max-age=31536000; includeSubDomains; preload",
        );
        let entry = store.entries.get("example.com").unwrap();
        assert_eq!(entry.max_age, Duration::from_secs(31536000));
        assert!(entry.include_subdomains);
    }

    #[test]
    fn parse_header_case_insensitive() {
        let mut store = HstsStore::new();
        store.parse_header("Example.COM", "Max-Age=600; IncludeSubDomains");
        let entry = store.entries.get("example.com").unwrap();
        assert_eq!(entry.max_age, Duration::from_secs(600));
        assert!(entry.include_subdomains);
    }

    #[test]
    fn max_age_zero_removes_entry() {
        let mut store = HstsStore::new();
        store.parse_header("example.com", "max-age=31536000");
        assert!(store.entries.contains_key("example.com"));
        store.parse_header("example.com", "max-age=0");
        assert!(!store.entries.contains_key("example.com"));
    }

    #[test]
    fn should_upgrade_http() {
        let mut store = HstsStore::new();
        store.parse_header("secure.example.com", "max-age=3600");
        let result = store.should_upgrade("http://secure.example.com/path?q=1");
        assert_eq!(
            result,
            Some("https://secure.example.com/path?q=1".to_string())
        );
    }

    #[test]
    fn should_not_upgrade_https() {
        let mut store = HstsStore::new();
        store.parse_header("secure.example.com", "max-age=3600");
        assert!(
            store
                .should_upgrade("https://secure.example.com/")
                .is_none()
        );
    }

    #[test]
    fn should_not_upgrade_unknown_domain() {
        let store = HstsStore::new();
        assert!(
            store
                .should_upgrade("http://unknown.example.com/")
                .is_none()
        );
    }

    #[test]
    fn subdomain_matching() {
        let mut store = HstsStore::new();
        store.parse_header("example.com", "max-age=3600; includeSubDomains");

        // Subdomain should be upgraded
        assert!(store.should_upgrade("http://sub.example.com/").is_some());
        // Deeper subdomain
        assert!(store.should_upgrade("http://a.b.example.com/").is_some());
        // Exact domain
        assert!(store.should_upgrade("http://example.com/").is_some());
    }

    #[test]
    fn subdomain_not_matched_without_flag() {
        let mut store = HstsStore::new();
        store.parse_header("example.com", "max-age=3600");

        // Without includeSubDomains, subdomains should NOT be upgraded
        assert!(store.should_upgrade("http://sub.example.com/").is_none());
        // Exact domain still works
        assert!(store.should_upgrade("http://example.com/").is_some());
    }

    #[test]
    fn expired_entry_not_matched() {
        let mut store = HstsStore::new();
        // Insert an entry that is already expired (max_age = 0 seconds but added in the past)
        store.entries.insert(
            "example.com".to_string(),
            HstsEntry {
                max_age: Duration::from_secs(0),
                include_subdomains: false,
                added: Instant::now() - Duration::from_secs(1),
            },
        );
        assert!(store.should_upgrade("http://example.com/").is_none());
    }

    #[test]
    fn extract_domain_works() {
        assert_eq!(
            extract_domain("http://example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            extract_domain("https://Example.COM:8080/"),
            Some("example.com".to_string())
        );
        assert_eq!(extract_domain("ftp://nope"), None);
    }
}
