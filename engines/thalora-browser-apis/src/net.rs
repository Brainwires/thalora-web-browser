//! Outbound request safety for page-initiated requests (fetch, XHR, …).
//!
//! Pages are untrusted: a page (or a redirect it triggers) must not be able
//! to reach the host's private network, loopback services or cloud metadata
//! endpoints (SSRF). Every address a hostname resolves to must be public,
//! and redirects are re-checked hop by hop.

use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};

/// Whether loopback destinations are allowed (test fixture servers).
///
/// Requires `THALORA_ALLOW_LOOPBACK=1` *and* a debug build or the
/// `test-hooks` feature. Private and link-local ranges stay blocked.
pub fn loopback_override_enabled() -> bool {
    cfg!(any(debug_assertions, feature = "test-hooks"))
        && std::env::var("THALORA_ALLOW_LOOPBACK").is_ok_and(|v| v == "1" || v == "true")
}

/// True for loopback addresses, including IPv4-mapped IPv6 loopback.
pub fn is_loopback(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_loopback(),
        IpAddr::V6(v6) => {
            v6.is_loopback() || v6.to_ipv4_mapped().is_some_and(|v4| v4.is_loopback())
        }
    }
}

/// True for addresses that must not be reachable from page content.
pub fn is_internal_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_internal_v4(v4),
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_internal_v4(&v4);
            }
            let first = v6.segments()[0];
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || (first & 0xfe00) == 0xfc00 // unique local fc00::/7
                || (first & 0xffc0) == 0xfe80 // link-local fe80::/10
        }
    }
}

fn is_internal_v4(v4: &Ipv4Addr) -> bool {
    let [a, b, ..] = v4.octets();
    v4.is_private()
        || v4.is_loopback()
        || v4.is_link_local() // includes 169.254.169.254 metadata
        || v4.is_broadcast()
        || v4.is_unspecified()
        || v4.is_multicast()
        || a == 0 // "this network"
        || (a == 100 && (64..128).contains(&b)) // carrier-grade NAT 100.64/10
}

fn is_allowed(ip: &IpAddr) -> bool {
    !is_internal_ip(ip) || (is_loopback(ip) && loopback_override_enabled())
}

/// Check that `url` is http(s) and that every address its host resolves to
/// is public. Resolves DNS (blocking).
pub fn check_url(url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("invalid URL: {e}"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(format!("scheme '{}' is not allowed", parsed.scheme()));
    }
    let port = parsed.port_or_known_default().unwrap_or(80);
    let addresses: Vec<IpAddr> = match parsed.host().ok_or("URL has no host")? {
        url::Host::Ipv4(v4) => vec![IpAddr::V4(v4)],
        url::Host::Ipv6(v6) => vec![IpAddr::V6(v6)],
        url::Host::Domain(domain) => {
            let lower = domain.to_ascii_lowercase();
            if lower == "localhost" || lower.ends_with(".localhost") {
                vec![IpAddr::V4(Ipv4Addr::LOCALHOST)]
            } else {
                (domain, port)
                    .to_socket_addrs()
                    .map_err(|e| format!("could not resolve {domain}: {e}"))?
                    .map(|addr| addr.ip())
                    .collect()
            }
        }
    };
    if addresses.is_empty() {
        return Err("host did not resolve".to_string());
    }
    match addresses.iter().find(|ip| !is_allowed(ip)) {
        Some(ip) => Err(format!(
            "request to internal address {ip} blocked (SSRF protection)"
        )),
        None => Ok(()),
    }
}

/// Redirect policy that re-checks every hop with [`check_url`] (max 10).
pub fn redirect_policy() -> reqwest::redirect::Policy {
    reqwest::redirect::Policy::custom(|attempt| {
        if attempt.previous().len() >= 10 {
            return attempt.error("too many redirects");
        }
        match check_url(attempt.url().as_str()) {
            Ok(()) => attempt.follow(),
            Err(reason) => attempt.error(reason),
        }
    })
}

/// An async client for page requests with the redirect checks applied.
/// Run a network future (a reqwest send or body read) on Thalora's shared
/// network runtime and return a future for its output.
///
/// Page JS runs on a single-threaded runtime whose event loop is pumped
/// synchronously (`ThaloraJobExecutor::pump`); while it pumps, that
/// runtime's IO driver cannot run, so reqwest futures polled there would
/// never complete. The returned future only checks the spawned task for
/// completion, which works under any polling.
pub fn io<F>(future: F) -> impl std::future::Future<Output = F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    let runtime = RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("thalora-net")
            .enable_all()
            .build()
            .expect("failed to start the network runtime")
    });
    let task = runtime.spawn(future);
    async move {
        match task.await {
            Ok(output) => output,
            Err(e) => std::panic::resume_unwind(e.into_panic()),
        }
    }
}

pub fn page_client() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(redirect_policy())
        .build()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_addresses_are_recognised() {
        for ip in [
            "10.0.0.1",
            "172.16.5.4",
            "192.168.1.1",
            "127.0.0.1",
            "169.254.169.254",
            "0.0.0.0",
            "100.64.0.1",
            "::1",
            "fd00::1",
            "fe80::1",
            "::ffff:10.0.0.1",
        ] {
            let ip: IpAddr = ip.parse().unwrap();
            assert!(is_internal_ip(&ip), "{ip} should be internal");
        }
        for ip in ["8.8.8.8", "1.1.1.1", "2606:4700:4700::1111"] {
            let ip: IpAddr = ip.parse().unwrap();
            assert!(!is_internal_ip(&ip), "{ip} should be public");
        }
    }

    #[test]
    fn urls_with_internal_literals_are_blocked() {
        assert!(check_url("http://169.254.169.254/latest/meta-data").is_err());
        assert!(check_url("http://[fd00::1]/").is_err());
        assert!(check_url("http://[::ffff:192.168.0.1]/").is_err());
        assert!(check_url("file:///etc/passwd").is_err());
        assert!(check_url("http://8.8.8.8/").is_ok());
    }
}
