pub mod origin;
pub mod ssrf;

pub use origin::Origin;
pub use ssrf::SsrfProtection;

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
