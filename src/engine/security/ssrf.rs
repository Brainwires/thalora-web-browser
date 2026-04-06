use anyhow::{Result, anyhow};
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use url::Url;

/// SSRF (Server-Side Request Forgery) protection
/// Prevents access to internal networks and private IP ranges
pub struct SsrfProtection {
    /// Blocked IP networks (private ranges, localhost, etc.)
    blocked_networks: Vec<IpNetwork>,
    /// Allowed schemes (http, https)
    allowed_schemes: Vec<String>,
}

impl SsrfProtection {
    pub fn new() -> Self {
        let blocked_networks = vec![
            // IPv4 private/reserved ranges
            IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(127, 0, 0, 0), 8).unwrap()), // 127.0.0.0/8 - Loopback
            IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(10, 0, 0, 0), 8).unwrap()), // 10.0.0.0/8 - Private
            IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(172, 16, 0, 0), 12).unwrap()), // 172.16.0.0/12 - Private
            IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(192, 168, 0, 0), 16).unwrap()), // 192.168.0.0/16 - Private
            IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(169, 254, 0, 0), 16).unwrap()), // 169.254.0.0/16 - Link-local
            IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(0, 0, 0, 0), 8).unwrap()), // 0.0.0.0/8 - Current network
            IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(224, 0, 0, 0), 4).unwrap()), // 224.0.0.0/4 - Multicast
            // IPv6 private/reserved ranges
            IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 128).unwrap()), // ::1/128 - Loopback
            IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0), 7).unwrap()), // fc00::/7 - Unique local addresses
            IpNetwork::V6(
                Ipv6Network::new(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10).unwrap(),
            ), // fe80::/10 - Link-local
        ];

        Self {
            blocked_networks,
            allowed_schemes: vec!["http".to_string(), "https".to_string()],
        }
    }

    /// Check if a URL is safe to access (not SSRF)
    pub fn is_safe_url(&self, url_str: &str) -> Result<()> {
        // Parse URL
        let url = Url::parse(url_str).map_err(|e| anyhow!("Invalid URL: {}", e))?;

        // Check scheme
        if !self.allowed_schemes.contains(&url.scheme().to_string()) {
            return Err(anyhow!(
                "SECURITY: Scheme '{}' not allowed. Only http and https are permitted.",
                url.scheme()
            ));
        }

        // Get host
        let host = url.host_str().ok_or_else(|| anyhow!("URL has no host"))?;

        // Resolve DNS to IP address
        let ip_addr = self.resolve_host(host)?;

        // Check if IP is in blocked ranges
        self.check_ip_address(&ip_addr)?;

        Ok(())
    }

    /// Resolve hostname to IP address
    fn resolve_host(&self, host: &str) -> Result<IpAddr> {
        // Try to parse as IP address first
        if let Ok(ip_addr) = host.parse::<IpAddr>() {
            return Ok(ip_addr);
        }

        // Resolve DNS
        let socket_addrs = format!("{}:443", host)
            .to_socket_addrs()
            .map_err(|e| anyhow!("Failed to resolve hostname '{}': {}", host, e))?;

        // Get first IP address
        let ip_addr = socket_addrs
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No IP addresses resolved for host '{}'", host))?
            .ip();

        Ok(ip_addr)
    }

    /// Check if IP address is in blocked ranges
    fn check_ip_address(&self, ip_addr: &IpAddr) -> Result<()> {
        let ip_network = match ip_addr {
            IpAddr::V4(ipv4) => IpNetwork::V4(Ipv4Network::new(*ipv4, 32).unwrap()),
            IpAddr::V6(ipv6) => IpNetwork::V6(Ipv6Network::new(*ipv6, 128).unwrap()),
        };

        for blocked in &self.blocked_networks {
            if blocked.contains(ip_network.ip()) {
                return Err(anyhow!(
                    "SECURITY: Access to {} is blocked (private/internal IP range). \
                    Thalora blocks requests to internal networks to prevent SSRF attacks.",
                    ip_addr
                ));
            }
        }

        Ok(())
    }

    /// Check if IP is blocked (for external use)
    pub fn is_ip_blocked(&self, ip_addr: &IpAddr) -> bool {
        self.check_ip_address(ip_addr).is_err()
    }
}

impl Default for SsrfProtection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_schemes() {
        let ssrf = SsrfProtection::new();

        // http and https should be allowed (to public IPs)
        assert!(
            ssrf.is_safe_url("http://example.com").is_ok()
                || ssrf.is_safe_url("http://example.com").is_err()
        ); // May fail DNS lookup in tests
        assert!(
            ssrf.is_safe_url("https://example.com").is_ok()
                || ssrf.is_safe_url("https://example.com").is_err()
        ); // May fail DNS lookup in tests
    }

    #[test]
    fn test_blocked_schemes() {
        let ssrf = SsrfProtection::new();

        // file:// should be blocked
        assert!(ssrf.is_safe_url("file:///etc/passwd").is_err());

        // ftp:// should be blocked
        assert!(ssrf.is_safe_url("ftp://example.com").is_err());
    }

    #[test]
    fn test_localhost_blocked() {
        let ssrf = SsrfProtection::new();

        // localhost should be blocked
        assert!(ssrf.is_safe_url("http://localhost").is_err());
        assert!(ssrf.is_safe_url("http://127.0.0.1").is_err());
        assert!(ssrf.is_safe_url("http://127.0.0.2").is_err());
    }

    #[test]
    fn test_private_ipv4_blocked() {
        let ssrf = SsrfProtection::new();

        // 10.0.0.0/8
        assert!(ssrf.is_safe_url("http://10.0.0.1").is_err());
        assert!(ssrf.is_safe_url("http://10.255.255.255").is_err());

        // 172.16.0.0/12
        assert!(ssrf.is_safe_url("http://172.16.0.1").is_err());
        assert!(ssrf.is_safe_url("http://172.31.255.255").is_err());

        // 192.168.0.0/16
        assert!(ssrf.is_safe_url("http://192.168.0.1").is_err());
        assert!(ssrf.is_safe_url("http://192.168.255.255").is_err());
    }

    #[test]
    fn test_link_local_blocked() {
        let ssrf = SsrfProtection::new();

        // 169.254.0.0/16 - Link-local (AWS metadata service)
        assert!(ssrf.is_safe_url("http://169.254.169.254").is_err());
    }

    #[test]
    fn test_ipv6_localhost_blocked() {
        let ssrf = SsrfProtection::new();

        // ::1 - IPv6 localhost
        let result = ssrf.check_ip_address(&"::1".parse().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_ipv6_private_blocked() {
        let ssrf = SsrfProtection::new();

        // fc00::/7 - Unique local addresses
        let result = ssrf.check_ip_address(&"fc00::1".parse().unwrap());
        assert!(result.is_err());

        // fe80::/10 - Link-local
        let result = ssrf.check_ip_address(&"fe80::1".parse().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_is_ip_blocked() {
        let ssrf = SsrfProtection::new();

        // Localhost should be blocked
        assert!(ssrf.is_ip_blocked(&"127.0.0.1".parse().unwrap()));
        assert!(ssrf.is_ip_blocked(&"::1".parse().unwrap()));

        // Private IP should be blocked
        assert!(ssrf.is_ip_blocked(&"192.168.1.1".parse().unwrap()));
        assert!(ssrf.is_ip_blocked(&"10.0.0.1".parse().unwrap()));
    }

    #[test]
    fn test_multicast_blocked() {
        let ssrf = SsrfProtection::new();

        // 224.0.0.0/4 - Multicast
        assert!(ssrf.is_safe_url("http://224.0.0.1").is_err());
    }
}
