//! Content Security Policy (CSP) parser and evaluator.
//!
//! Parses Content-Security-Policy headers and enforces script-src directives.
//! https://www.w3.org/TR/CSP3/

/// A parsed CSP source expression.
#[derive(Debug, Clone, PartialEq)]
pub enum SourceExpression {
    /// 'self' — matches the page's own origin
    OriginSelf,
    /// 'unsafe-inline' — allows inline scripts/styles
    UnsafeInline,
    /// 'unsafe-eval' — allows eval() and Function()
    UnsafeEval,
    /// 'nonce-<base64>' — matches scripts with a specific nonce
    Nonce(String),
    /// 'sha256-<base64>' / 'sha384-<base64>' / 'sha512-<base64>'
    Hash(String, String), // (algorithm, base64_hash)
    /// 'strict-dynamic' — trust scripts loaded by already-trusted scripts
    StrictDynamic,
    /// 'none' — blocks everything
    None,
    /// URL pattern (e.g., https://cdn.example.com, *.example.com, https:)
    Url(String),
}

/// Parsed CSP policy with relevant directives.
#[derive(Debug, Clone, Default)]
pub struct CspPolicy {
    pub script_src: Vec<SourceExpression>,
    pub default_src: Vec<SourceExpression>,
    pub style_src: Vec<SourceExpression>,
    pub img_src: Vec<SourceExpression>,
    pub connect_src: Vec<SourceExpression>,
    pub font_src: Vec<SourceExpression>,
    pub frame_src: Vec<SourceExpression>,
    pub media_src: Vec<SourceExpression>,
    pub object_src: Vec<SourceExpression>,
    pub worker_src: Vec<SourceExpression>,
    pub child_src: Vec<SourceExpression>,
    pub base_uri: Vec<SourceExpression>,
    pub form_action: Vec<SourceExpression>,
    pub frame_ancestors: Vec<SourceExpression>,
    /// report-uri / report-to endpoint
    pub report_uri: Option<String>,
}

impl CspPolicy {
    /// Parse a Content-Security-Policy header value.
    pub fn parse(header: &str) -> Self {
        let mut policy = CspPolicy::default();

        for directive in header.split(';') {
            let directive = directive.trim();
            if directive.is_empty() {
                continue;
            }

            let mut parts = directive.split_whitespace();
            let Some(name) = parts.next() else {
                continue;
            };

            let sources: Vec<SourceExpression> = parts
                .filter_map(|token| parse_source_expression(token))
                .collect();

            match name {
                "script-src" => policy.script_src = sources,
                "default-src" => policy.default_src = sources,
                "style-src" => policy.style_src = sources,
                "img-src" => policy.img_src = sources,
                "connect-src" => policy.connect_src = sources,
                "font-src" => policy.font_src = sources,
                "frame-src" => policy.frame_src = sources,
                "media-src" => policy.media_src = sources,
                "object-src" => policy.object_src = sources,
                "worker-src" => policy.worker_src = sources,
                "child-src" => policy.child_src = sources,
                "base-uri" => policy.base_uri = sources,
                "form-action" => policy.form_action = sources,
                "frame-ancestors" => policy.frame_ancestors = sources,
                "report-uri" => {
                    // report-uri takes a URI, not source expressions
                    let uri = directive.strip_prefix("report-uri").unwrap_or("").trim();
                    if !uri.is_empty() {
                        policy.report_uri = Some(uri.to_string());
                    }
                }
                "report-to" => {
                    // report-to takes a group name, store as report_uri for now
                    let group = directive.strip_prefix("report-to").unwrap_or("").trim();
                    if !group.is_empty() && policy.report_uri.is_none() {
                        policy.report_uri = Some(group.to_string());
                    }
                }
                _ => {}
            }
        }

        policy
    }

    /// Get the effective script-src directive (falls back to default-src).
    fn effective_script_src(&self) -> &[SourceExpression] {
        if !self.script_src.is_empty() {
            &self.script_src
        } else if !self.default_src.is_empty() {
            &self.default_src
        } else {
            &[] // No policy = allow all
        }
    }

    /// Check if an inline script is allowed.
    /// `nonce` is the script element's nonce attribute (if any).
    /// `hash` is the sha256 hash of the script content (if computed).
    pub fn allows_inline_script(&self, nonce: Option<&str>, content_hash: Option<&str>) -> bool {
        let sources = self.effective_script_src();

        // No policy = allow all
        if sources.is_empty() {
            return true;
        }

        for source in sources {
            match source {
                SourceExpression::UnsafeInline => return true,
                SourceExpression::Nonce(expected_nonce) => {
                    if let Some(nonce) = nonce {
                        if nonce == expected_nonce {
                            return true;
                        }
                    }
                }
                SourceExpression::Hash(algo, expected_hash) => {
                    if algo == "sha256" {
                        if let Some(hash) = content_hash {
                            if hash == expected_hash {
                                return true;
                            }
                        }
                    }
                }
                SourceExpression::None => return false,
                SourceExpression::StrictDynamic => {
                    // strict-dynamic trusts scripts loaded by trusted scripts,
                    // but blocks inline scripts without nonce/hash
                    continue;
                }
                _ => {}
            }
        }

        false
    }

    /// Check if an external script URL is allowed.
    pub fn allows_external_script(&self, script_url: &str, page_url: Option<&str>) -> bool {
        let sources = self.effective_script_src();

        // No policy = allow all
        if sources.is_empty() {
            return true;
        }

        for source in sources {
            match source {
                SourceExpression::None => return false,
                SourceExpression::OriginSelf => {
                    if let Some(page) = page_url {
                        if same_origin(page, script_url) {
                            return true;
                        }
                    }
                }
                SourceExpression::Url(pattern) => {
                    if url_matches_pattern(script_url, pattern) {
                        return true;
                    }
                }
                SourceExpression::StrictDynamic => {
                    // With strict-dynamic, URL-based allowlists are ignored
                    // Scripts must be loaded by already-trusted scripts
                    return true; // In headless context, allow for compatibility
                }
                SourceExpression::Nonce(_) | SourceExpression::Hash(_, _) => {
                    // Nonce/hash can also allow external scripts
                    // (handled at the call site with the script's nonce/integrity)
                    continue;
                }
                _ => {}
            }
        }

        false
    }

    /// Check if eval() is allowed by CSP.
    pub fn allows_eval(&self) -> bool {
        let sources = self.effective_script_src();
        if sources.is_empty() {
            return true;
        }
        sources.iter().any(|s| matches!(s, SourceExpression::UnsafeEval))
    }

    /// Check if a URL is allowed by a specific directive, falling back to default-src.
    fn allows_url(&self, directive: &[SourceExpression], url: &str, page_url: Option<&str>) -> bool {
        let sources = if !directive.is_empty() {
            directive
        } else if !self.default_src.is_empty() {
            &self.default_src
        } else {
            return true; // No policy = allow all
        };

        for source in sources {
            match source {
                SourceExpression::None => return false,
                SourceExpression::OriginSelf => {
                    if let Some(page) = page_url {
                        if same_origin(page, url) {
                            return true;
                        }
                    }
                }
                SourceExpression::Url(pattern) => {
                    if url_matches_pattern(url, pattern) {
                        return true;
                    }
                }
                SourceExpression::UnsafeInline => continue,
                SourceExpression::UnsafeEval => continue,
                _ => continue,
            }
        }

        false
    }

    /// Check if a style resource URL is allowed.
    pub fn allows_style(&self, url: &str, page_url: Option<&str>) -> bool {
        self.allows_url(&self.style_src, url, page_url)
    }

    /// Check if an image URL is allowed.
    pub fn allows_image(&self, url: &str, page_url: Option<&str>) -> bool {
        self.allows_url(&self.img_src, url, page_url)
    }

    /// Check if a connect (fetch/XHR/WebSocket) URL is allowed.
    pub fn allows_connect(&self, url: &str, page_url: Option<&str>) -> bool {
        self.allows_url(&self.connect_src, url, page_url)
    }

    /// Check if a font URL is allowed.
    pub fn allows_font(&self, url: &str, page_url: Option<&str>) -> bool {
        self.allows_url(&self.font_src, url, page_url)
    }

    /// Check if a frame URL is allowed.
    pub fn allows_frame(&self, url: &str, page_url: Option<&str>) -> bool {
        // frame-src falls back to child-src, then default-src
        let sources = if !self.frame_src.is_empty() {
            &self.frame_src
        } else if !self.child_src.is_empty() {
            &self.child_src
        } else {
            &self.default_src
        };
        self.allows_url(sources, url, page_url)
    }

    /// Check if a worker URL is allowed.
    pub fn allows_worker(&self, url: &str, page_url: Option<&str>) -> bool {
        // worker-src falls back to child-src, then script-src, then default-src
        let sources = if !self.worker_src.is_empty() {
            &self.worker_src
        } else if !self.child_src.is_empty() {
            &self.child_src
        } else if !self.script_src.is_empty() {
            &self.script_src
        } else {
            &self.default_src
        };
        self.allows_url(sources, url, page_url)
    }

    /// Check if an inline style is allowed.
    pub fn allows_inline_style(&self) -> bool {
        let sources = if !self.style_src.is_empty() {
            &self.style_src
        } else if !self.default_src.is_empty() {
            &self.default_src
        } else {
            return true;
        };
        sources
            .iter()
            .any(|s| matches!(s, SourceExpression::UnsafeInline))
    }

    /// Returns true if any CSP policy is set.
    pub fn has_policy(&self) -> bool {
        !self.script_src.is_empty()
            || !self.default_src.is_empty()
            || !self.style_src.is_empty()
            || !self.connect_src.is_empty()
            || !self.img_src.is_empty()
            || !self.font_src.is_empty()
            || !self.frame_src.is_empty()
            || !self.worker_src.is_empty()
    }
}

/// Parse a single CSP source expression token.
fn parse_source_expression(token: &str) -> Option<SourceExpression> {
    match token {
        "'self'" => Some(SourceExpression::OriginSelf),
        "'unsafe-inline'" => Some(SourceExpression::UnsafeInline),
        "'unsafe-eval'" => Some(SourceExpression::UnsafeEval),
        "'strict-dynamic'" => Some(SourceExpression::StrictDynamic),
        "'none'" => Some(SourceExpression::None),
        _ if token.starts_with("'nonce-") && token.ends_with('\'') => {
            let nonce = &token[7..token.len() - 1];
            Some(SourceExpression::Nonce(nonce.to_string()))
        }
        _ if token.starts_with("'sha256-") && token.ends_with('\'') => {
            let hash = &token[8..token.len() - 1];
            Some(SourceExpression::Hash("sha256".to_string(), hash.to_string()))
        }
        _ if token.starts_with("'sha384-") && token.ends_with('\'') => {
            let hash = &token[8..token.len() - 1];
            Some(SourceExpression::Hash("sha384".to_string(), hash.to_string()))
        }
        _ if token.starts_with("'sha512-") && token.ends_with('\'') => {
            let hash = &token[8..token.len() - 1];
            Some(SourceExpression::Hash("sha512".to_string(), hash.to_string()))
        }
        _ => {
            // Treat as URL pattern
            Some(SourceExpression::Url(token.to_string()))
        }
    }
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
    if let (Ok(url_parsed), Ok(pattern_parsed)) =
        (url::Url::parse(url), url::Url::parse(pattern))
    {
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
    fn test_parse_basic_csp() {
        let policy = CspPolicy::parse("script-src 'self' 'unsafe-inline'");
        assert_eq!(policy.script_src.len(), 2);
        assert_eq!(policy.script_src[0], SourceExpression::OriginSelf);
        assert_eq!(policy.script_src[1], SourceExpression::UnsafeInline);
    }

    #[test]
    fn test_parse_nonce() {
        let policy = CspPolicy::parse("script-src 'nonce-abc123'");
        assert_eq!(policy.script_src.len(), 1);
        assert_eq!(
            policy.script_src[0],
            SourceExpression::Nonce("abc123".to_string())
        );
    }

    #[test]
    fn test_parse_multiple_directives() {
        let policy = CspPolicy::parse(
            "default-src 'self'; script-src 'self' https://cdn.example.com 'unsafe-eval'",
        );
        assert_eq!(policy.default_src.len(), 1);
        assert_eq!(policy.script_src.len(), 3);
    }

    #[test]
    fn test_allows_inline_with_unsafe_inline() {
        let policy = CspPolicy::parse("script-src 'self' 'unsafe-inline'");
        assert!(policy.allows_inline_script(None, None));
    }

    #[test]
    fn test_blocks_inline_without_unsafe_inline() {
        let policy = CspPolicy::parse("script-src 'self'");
        assert!(!policy.allows_inline_script(None, None));
    }

    #[test]
    fn test_allows_inline_with_nonce() {
        let policy = CspPolicy::parse("script-src 'nonce-abc123'");
        assert!(policy.allows_inline_script(Some("abc123"), None));
        assert!(!policy.allows_inline_script(Some("wrong"), None));
        assert!(!policy.allows_inline_script(None, None));
    }

    #[test]
    fn test_allows_eval() {
        let policy_with = CspPolicy::parse("script-src 'self' 'unsafe-eval'");
        assert!(policy_with.allows_eval());

        let policy_without = CspPolicy::parse("script-src 'self'");
        assert!(!policy_without.allows_eval());
    }

    #[test]
    fn test_allows_external_self_origin() {
        let policy = CspPolicy::parse("script-src 'self'");
        assert!(policy.allows_external_script(
            "https://example.com/script.js",
            Some("https://example.com/page")
        ));
        assert!(!policy.allows_external_script(
            "https://evil.com/script.js",
            Some("https://example.com/page")
        ));
    }

    #[test]
    fn test_allows_external_url_pattern() {
        let policy = CspPolicy::parse("script-src https://cdn.example.com");
        assert!(policy.allows_external_script(
            "https://cdn.example.com/lib.js",
            None
        ));
        assert!(!policy.allows_external_script(
            "https://evil.com/lib.js",
            None
        ));
    }

    #[test]
    fn test_none_blocks_everything() {
        let policy = CspPolicy::parse("script-src 'none'");
        assert!(!policy.allows_inline_script(None, None));
        assert!(!policy.allows_external_script("https://any.com/s.js", None));
    }

    #[test]
    fn test_default_src_fallback() {
        let policy = CspPolicy::parse("default-src 'self' 'unsafe-inline'");
        // No script-src, falls back to default-src
        assert!(policy.allows_inline_script(None, None));
    }

    #[test]
    fn test_no_policy_allows_all() {
        let policy = CspPolicy::parse("");
        assert!(policy.allows_inline_script(None, None));
        assert!(policy.allows_external_script("https://any.com/s.js", None));
        assert!(policy.allows_eval());
    }

    #[test]
    fn test_same_origin() {
        assert!(same_origin("https://example.com/a", "https://example.com/b"));
        assert!(!same_origin("https://example.com", "https://other.com"));
        assert!(!same_origin("http://example.com", "https://example.com"));
    }

    #[test]
    fn test_wildcard_subdomain() {
        let policy = CspPolicy::parse("script-src *.example.com");
        assert!(policy.allows_external_script("https://cdn.example.com/s.js", None));
        assert!(policy.allows_external_script("https://api.example.com/s.js", None));
        assert!(!policy.allows_external_script("https://evil.com/s.js", None));
    }
}
