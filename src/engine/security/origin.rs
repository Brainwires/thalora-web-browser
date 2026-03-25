use anyhow::{Result, anyhow};
use std::fmt;
use url::Url;

/// Represents a web origin (scheme, host, port)
/// Used for implementing Same-Origin Policy
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Origin {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
}

impl Origin {
    /// Parse origin from URL string
    pub fn from_url(url_str: &str) -> Result<Self> {
        let url = Url::parse(url_str).map_err(|e| anyhow!("Invalid URL: {}", e))?;

        // Ensure URL has a host
        let host = url
            .host_str()
            .ok_or_else(|| anyhow!("URL has no host: {}", url_str))?
            .to_string();

        Ok(Self {
            scheme: url.scheme().to_string(),
            host,
            port: url.port(),
        })
    }

    /// Check if this origin matches another origin (Same-Origin Policy)
    pub fn matches(&self, other: &Origin) -> bool {
        self.scheme == other.scheme && self.host == other.host && self.port == other.port
    }

    /// Get the canonical origin string (scheme://host:port)
    pub fn to_string(&self) -> String {
        if let Some(port) = self.port {
            format!("{}://{}:{}", self.scheme, self.host, port)
        } else {
            format!("{}://{}", self.scheme, self.host)
        }
    }

    /// Check if origin is secure (HTTPS)
    pub fn is_secure(&self) -> bool {
        self.scheme == "https" || self.scheme == "wss"
    }

    /// Check if origin is localhost
    pub fn is_localhost(&self) -> bool {
        self.host == "localhost" || self.host == "127.0.0.1" || self.host == "::1"
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_parsing() {
        let origin = Origin::from_url("https://example.com/path").unwrap();
        assert_eq!(origin.scheme, "https");
        assert_eq!(origin.host, "example.com");
        assert_eq!(origin.port, None);
    }

    #[test]
    fn test_origin_with_port() {
        let origin = Origin::from_url("https://example.com:8080/path").unwrap();
        assert_eq!(origin.scheme, "https");
        assert_eq!(origin.host, "example.com");
        assert_eq!(origin.port, Some(8080));
    }

    #[test]
    fn test_same_origin() {
        let origin1 = Origin::from_url("https://example.com/page1").unwrap();
        let origin2 = Origin::from_url("https://example.com/page2").unwrap();
        assert!(origin1.matches(&origin2));
    }

    #[test]
    fn test_different_origin_scheme() {
        let origin1 = Origin::from_url("https://example.com").unwrap();
        let origin2 = Origin::from_url("http://example.com").unwrap();
        assert!(!origin1.matches(&origin2));
    }

    #[test]
    fn test_different_origin_host() {
        let origin1 = Origin::from_url("https://example.com").unwrap();
        let origin2 = Origin::from_url("https://other.com").unwrap();
        assert!(!origin1.matches(&origin2));
    }

    #[test]
    fn test_different_origin_port() {
        let origin1 = Origin::from_url("https://example.com:8080").unwrap();
        let origin2 = Origin::from_url("https://example.com:9090").unwrap();
        assert!(!origin1.matches(&origin2));
    }

    #[test]
    fn test_is_secure() {
        let https_origin = Origin::from_url("https://example.com").unwrap();
        assert!(https_origin.is_secure());

        let http_origin = Origin::from_url("http://example.com").unwrap();
        assert!(!http_origin.is_secure());
    }

    #[test]
    fn test_is_localhost() {
        let localhost = Origin::from_url("http://localhost").unwrap();
        assert!(localhost.is_localhost());

        let localhost_ip = Origin::from_url("http://127.0.0.1").unwrap();
        assert!(localhost_ip.is_localhost());

        let remote = Origin::from_url("http://example.com").unwrap();
        assert!(!remote.is_localhost());
    }

    #[test]
    fn test_to_string() {
        let origin = Origin::from_url("https://example.com:8080/path").unwrap();
        assert_eq!(origin.to_string(), "https://example.com:8080");

        let origin_no_port = Origin::from_url("https://example.com/path").unwrap();
        assert_eq!(origin_no_port.to_string(), "https://example.com");
    }
}
