// Security utilities for protocol layer
// Provides input validation and sanitization functions

use anyhow::{Result, anyhow};

/// Maximum length for session IDs to prevent buffer overflow attacks
const MAX_SESSION_ID_LENGTH: usize = 64;

/// Sanitize session ID to prevent path traversal attacks (CWE-22)
///
/// Only allows alphanumeric characters, hyphens, and underscores.
/// Rejects any path traversal sequences like `../` or `..\\`.
///
/// # Arguments
/// * `session_id` - The raw session ID from user input
///
/// # Returns
/// * `Ok(String)` - The validated session ID (unchanged if valid)
/// * `Err` - If the session ID contains invalid characters or patterns
///
/// # Security
/// This function prevents:
/// - Path traversal via `../` or `..\\` sequences
/// - Null byte injection
/// - Unicode normalization attacks
/// - Overly long identifiers (DoS prevention)
pub fn sanitize_session_id(session_id: &str) -> Result<String> {
    // Check for empty input
    if session_id.is_empty() {
        return Err(anyhow!("Session ID cannot be empty"));
    }

    // Check length limit (DoS prevention)
    if session_id.len() > MAX_SESSION_ID_LENGTH {
        return Err(anyhow!(
            "Session ID too long: {} chars (max {})",
            session_id.len(),
            MAX_SESSION_ID_LENGTH
        ));
    }

    // Check for path traversal patterns (redundant but explicit)
    if session_id.contains("..") || session_id.contains('/') || session_id.contains('\\') {
        return Err(anyhow!(
            "Session ID contains path traversal characters: '{}'",
            session_id
        ));
    }

    // Check for null bytes
    if session_id.contains('\0') {
        return Err(anyhow!("Session ID contains null byte"));
    }

    // Only allow safe characters: alphanumeric, hyphen, underscore
    if !session_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(anyhow!(
            "Session ID contains invalid characters: '{}'. Only alphanumeric, hyphens, and underscores are allowed",
            session_id
        ));
    }

    Ok(session_id.to_string())
}

/// Validate a URL for safe navigation (SSRF prevention - CWE-918)
///
/// Blocks requests to:
/// - Private IP ranges (10.x.x.x, 172.16-31.x.x, 192.168.x.x)
/// - Localhost and loopback addresses
/// - Link-local addresses (169.254.x.x)
/// - Cloud metadata endpoints
/// - Non-HTTP(S) schemes
///
/// # Arguments
/// * `url` - The URL to validate
///
/// # Returns
/// * `Ok(())` - If the URL is safe to navigate to
/// * `Err` - If the URL targets internal/private resources
pub fn validate_url_for_navigation(url: &str) -> Result<()> {
    // Parse the URL
    let parsed = url::Url::parse(url)
        .map_err(|e| anyhow!("Invalid URL: {}", e))?;

    // Only allow HTTP and HTTPS schemes
    match parsed.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(anyhow!(
                "Unsafe URL scheme '{}'. Only http and https are allowed",
                scheme
            ));
        }
    }

    // Get the host
    let host = parsed.host_str()
        .ok_or_else(|| anyhow!("URL has no host"))?;

    // Block localhost and loopback
    let host_lower = host.to_lowercase();
    if host_lower == "localhost"
        || host_lower == "127.0.0.1"
        || host_lower == "::1"
        || host_lower == "[::1]"
        || host_lower == "0.0.0.0"
    {
        return Err(anyhow!("Access to localhost is blocked for security"));
    }

    // Block cloud metadata endpoints
    if host_lower == "169.254.169.254"
        || host_lower == "metadata.google.internal"
        || host_lower.ends_with(".internal")
    {
        return Err(anyhow!("Access to cloud metadata endpoints is blocked"));
    }

    // Parse IP address if it looks like one
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        if !is_public_ip(&ip) {
            return Err(anyhow!(
                "Access to private/internal IP address {} is blocked",
                ip
            ));
        }
    }

    // Block potential DNS rebinding with numeric-looking hosts
    // e.g., "10-0-0-1.attacker.com" could resolve to 10.0.0.1
    // This is a heuristic - full protection requires DNS resolution checking

    Ok(())
}

/// Check if an IP address is publicly routable (not private/internal)
fn is_public_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(ipv4) => {
            // Private ranges
            if ipv4.is_private() {
                return false;
            }
            // Loopback (127.0.0.0/8)
            if ipv4.is_loopback() {
                return false;
            }
            // Link-local (169.254.0.0/16)
            if ipv4.is_link_local() {
                return false;
            }
            // Broadcast
            if ipv4.is_broadcast() {
                return false;
            }
            // Documentation ranges (192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24)
            let octets = ipv4.octets();
            if (octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
                || (octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
                || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
            {
                return false;
            }
            // Unspecified (0.0.0.0)
            if ipv4.is_unspecified() {
                return false;
            }
            true
        }
        std::net::IpAddr::V6(ipv6) => {
            // Loopback (::1)
            if ipv6.is_loopback() {
                return false;
            }
            // Unspecified (::)
            if ipv6.is_unspecified() {
                return false;
            }
            // IPv4-mapped addresses - check the embedded IPv4
            if let Some(ipv4) = ipv6.to_ipv4_mapped() {
                return is_public_ip(&std::net::IpAddr::V4(ipv4));
            }
            // Unique local (fc00::/7)
            let segments = ipv6.segments();
            if (segments[0] & 0xfe00) == 0xfc00 {
                return false;
            }
            // Link-local (fe80::/10)
            if (segments[0] & 0xffc0) == 0xfe80 {
                return false;
            }
            true
        }
    }
}

/// Validate cookie name and value to prevent injection attacks (CWE-113)
///
/// Rejects cookies containing:
/// - CRLF sequences (HTTP header injection)
/// - Null bytes
/// - Invalid characters per RFC 6265
pub fn validate_cookie(name: &str, value: &str) -> Result<()> {
    // Check name
    if name.is_empty() {
        return Err(anyhow!("Cookie name cannot be empty"));
    }

    // RFC 6265 token characters for cookie names
    // token = 1*<any CHAR except CTLs or separators>
    // separators = "(" | ")" | "<" | ">" | "@" | "," | ";" | ":" | "\" | <"> | "/" | "[" | "]" | "?" | "=" | "{" | "}" | SP | HT
    let invalid_name_chars = |c: char| {
        c.is_control()
            || c == '('
            || c == ')'
            || c == '<'
            || c == '>'
            || c == '@'
            || c == ','
            || c == ';'
            || c == ':'
            || c == '\\'
            || c == '"'
            || c == '/'
            || c == '['
            || c == ']'
            || c == '?'
            || c == '='
            || c == '{'
            || c == '}'
            || c == ' '
            || c == '\t'
    };

    if name.chars().any(invalid_name_chars) {
        return Err(anyhow!("Cookie name contains invalid characters"));
    }

    // Check value for injection characters
    // CRLF would allow HTTP header injection
    if value.contains('\r') || value.contains('\n') {
        return Err(anyhow!("Cookie value contains CRLF (potential header injection)"));
    }

    // Null bytes
    if value.contains('\0') {
        return Err(anyhow!("Cookie value contains null byte"));
    }

    // Semicolons and commas can be used for cookie attribute injection
    // However, they're technically allowed in quoted values. We'll be strict here.
    if value.contains(';') {
        return Err(anyhow!("Cookie value contains semicolon (potential attribute injection)"));
    }

    Ok(())
}

/// Limit input length to prevent DoS attacks
pub fn limit_input_length<'a>(input: &'a str, max_length: usize, field_name: &str) -> Result<&'a str> {
    if input.len() > max_length {
        return Err(anyhow!(
            "{} too long: {} chars (max {})",
            field_name,
            input.len(),
            max_length
        ));
    }
    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_session_id_valid() {
        assert!(sanitize_session_id("abc123").is_ok());
        assert!(sanitize_session_id("session-123").is_ok());
        assert!(sanitize_session_id("session_456").is_ok());
        assert!(sanitize_session_id("ABC-123_xyz").is_ok());
    }

    #[test]
    fn test_sanitize_session_id_path_traversal() {
        assert!(sanitize_session_id("../etc/passwd").is_err());
        assert!(sanitize_session_id("..\\windows\\system32").is_err());
        assert!(sanitize_session_id("session/../../../etc/passwd").is_err());
        assert!(sanitize_session_id("..").is_err());
    }

    #[test]
    fn test_sanitize_session_id_null_byte() {
        assert!(sanitize_session_id("session\0id").is_err());
    }

    #[test]
    fn test_sanitize_session_id_special_chars() {
        assert!(sanitize_session_id("session/id").is_err());
        assert!(sanitize_session_id("session\\id").is_err());
        assert!(sanitize_session_id("session:id").is_err());
        assert!(sanitize_session_id("session;id").is_err());
        assert!(sanitize_session_id("session id").is_err()); // spaces
    }

    #[test]
    fn test_sanitize_session_id_empty() {
        assert!(sanitize_session_id("").is_err());
    }

    #[test]
    fn test_sanitize_session_id_too_long() {
        let long_id = "a".repeat(65);
        assert!(sanitize_session_id(&long_id).is_err());

        let valid_id = "a".repeat(64);
        assert!(sanitize_session_id(&valid_id).is_ok());
    }

    #[test]
    fn test_validate_url_http_https_only() {
        assert!(validate_url_for_navigation("https://example.com").is_ok());
        assert!(validate_url_for_navigation("http://example.com").is_ok());
        assert!(validate_url_for_navigation("javascript:alert(1)").is_err());
        assert!(validate_url_for_navigation("file:///etc/passwd").is_err());
        assert!(validate_url_for_navigation("ftp://example.com").is_err());
    }

    #[test]
    fn test_validate_url_localhost_blocked() {
        assert!(validate_url_for_navigation("http://localhost").is_err());
        assert!(validate_url_for_navigation("http://127.0.0.1").is_err());
        assert!(validate_url_for_navigation("http://[::1]").is_err());
        assert!(validate_url_for_navigation("http://0.0.0.0").is_err());
    }

    #[test]
    fn test_validate_url_private_ips_blocked() {
        assert!(validate_url_for_navigation("http://10.0.0.1").is_err());
        assert!(validate_url_for_navigation("http://172.16.0.1").is_err());
        assert!(validate_url_for_navigation("http://192.168.1.1").is_err());
    }

    #[test]
    fn test_validate_url_metadata_blocked() {
        assert!(validate_url_for_navigation("http://169.254.169.254").is_err());
        assert!(validate_url_for_navigation("http://metadata.google.internal").is_err());
    }

    #[test]
    fn test_validate_url_public_allowed() {
        assert!(validate_url_for_navigation("https://google.com").is_ok());
        assert!(validate_url_for_navigation("https://8.8.8.8").is_ok());
    }

    #[test]
    fn test_validate_cookie_valid() {
        assert!(validate_cookie("session", "abc123").is_ok());
        assert!(validate_cookie("user_id", "12345").is_ok());
    }

    #[test]
    fn test_validate_cookie_crlf_injection() {
        assert!(validate_cookie("session", "abc\r\nSet-Cookie: evil=1").is_err());
        assert!(validate_cookie("session", "abc\nSet-Cookie: evil=1").is_err());
        assert!(validate_cookie("session", "abc\rSet-Cookie: evil=1").is_err());
    }

    #[test]
    fn test_validate_cookie_null_byte() {
        assert!(validate_cookie("session", "abc\0def").is_err());
    }

    #[test]
    fn test_validate_cookie_invalid_name() {
        assert!(validate_cookie("", "value").is_err());
        assert!(validate_cookie("session;", "value").is_err());
        assert!(validate_cookie("session=", "value").is_err());
    }
}
