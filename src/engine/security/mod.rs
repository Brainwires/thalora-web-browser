pub mod origin;
pub mod ssrf;

pub use origin::Origin;
pub use ssrf::SsrfProtection;

/// Whether navigation to loopback addresses (127.0.0.0/8, ::1, localhost) is
/// allowed, so tests can drive the browser against a local fixture server.
///
/// Requires `THALORA_ALLOW_LOOPBACK=1` *and* a debug build or the
/// `test-hooks` feature; release builds without the feature never allow it.
/// Only loopback is affected — private and link-local ranges stay blocked.
pub fn loopback_override_enabled() -> bool {
    cfg!(any(debug_assertions, feature = "test-hooks"))
        && std::env::var("THALORA_ALLOW_LOOPBACK").is_ok_and(|v| v == "1" || v == "true")
}

/// True for loopback addresses, including IPv4-mapped IPv6 loopback.
pub fn is_loopback_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => v4.is_loopback(),
        std::net::IpAddr::V6(v6) => {
            v6.is_loopback() || v6.to_ipv4_mapped().is_some_and(|v4| v4.is_loopback())
        }
    }
}

/// Security configuration for the browser
pub struct SecurityConfig {
    /// Enable SSRF protection
    pub ssrf_protection_enabled: bool,
    /// Enable origin isolation
    pub origin_isolation_enabled: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            ssrf_protection_enabled: true,
            origin_isolation_enabled: true,
        }
    }
}
