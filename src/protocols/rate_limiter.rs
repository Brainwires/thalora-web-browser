// Rate limiting for MCP tool calls (DoS prevention)
// Uses token bucket algorithm for smooth rate limiting

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Rate limits for different tool categories (requests per minute)
#[derive(Clone, Copy)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            burst_size: 10,
        }
    }
}

/// Token bucket for rate limiting
struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    fn new(config: RateLimitConfig) -> Self {
        let max_tokens = config.burst_size as f64;
        let refill_rate = config.requests_per_minute as f64 / 60.0;

        Self {
            tokens: max_tokens, // Start full
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn try_consume(&mut self) -> Result<(), Duration> {
        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            Ok(())
        } else {
            // Calculate wait time until one token is available
            let tokens_needed = 1.0 - self.tokens;
            let wait_seconds = tokens_needed / self.refill_rate;
            Err(Duration::from_secs_f64(wait_seconds))
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
}

/// Rate limiter with per-category limits
pub struct RateLimiter {
    buckets: Mutex<HashMap<String, TokenBucket>>,
    configs: HashMap<String, RateLimitConfig>,
}

impl RateLimiter {
    /// Create a new rate limiter with default category limits
    pub fn new() -> Self {
        let mut configs = HashMap::new();

        // Navigation tools - most expensive (HTTP requests)
        configs.insert(
            "navigation".to_string(),
            RateLimitConfig {
                requests_per_minute: 10,
                burst_size: 3,
            },
        );

        // Search tools - external API calls
        configs.insert(
            "search".to_string(),
            RateLimitConfig {
                requests_per_minute: 30,
                burst_size: 5,
            },
        );

        // JavaScript execution - CPU intensive
        configs.insert(
            "javascript".to_string(),
            RateLimitConfig {
                requests_per_minute: 20,
                burst_size: 5,
            },
        );

        // Memory tools - filesystem I/O
        configs.insert(
            "memory".to_string(),
            RateLimitConfig {
                requests_per_minute: 60,
                burst_size: 10,
            },
        );

        // Snapshot tools - HTTP + processing
        configs.insert(
            "snapshot_url".to_string(),
            RateLimitConfig {
                requests_per_minute: 20,
                burst_size: 5,
            },
        );

        // Session management - lighter operations
        configs.insert(
            "session".to_string(),
            RateLimitConfig {
                requests_per_minute: 60,
                burst_size: 10,
            },
        );

        // CDP tools - moderate usage
        configs.insert(
            "cdp".to_string(),
            RateLimitConfig {
                requests_per_minute: 30,
                burst_size: 5,
            },
        );

        // WASM debug tools - moderate usage
        configs.insert(
            "wasm_debug".to_string(),
            RateLimitConfig {
                requests_per_minute: 30,
                burst_size: 5,
            },
        );

        // Default fallback
        configs.insert("default".to_string(), RateLimitConfig::default());

        Self {
            buckets: Mutex::new(HashMap::new()),
            configs,
        }
    }

    /// Check if a request is allowed for the given category
    /// Returns Ok(()) if allowed, Err(wait_duration) if rate limited
    pub fn check(&self, category: &str) -> Result<(), Duration> {
        let config = self
            .configs
            .get(category)
            .or_else(|| self.configs.get("default"))
            .copied()
            .unwrap_or_default();

        let mut buckets = self.buckets.lock().unwrap();

        let bucket = buckets
            .entry(category.to_string())
            .or_insert_with(|| TokenBucket::new(config));

        bucket.try_consume()
    }

    /// Map a tool name to its rate limit category
    pub fn tool_to_category(tool_name: &str) -> &'static str {
        match tool_name {
            // Navigation tools
            "browser_navigate_to"
            | "browser_navigate_back"
            | "browser_navigate_forward"
            | "browser_refresh_page" => "navigation",

            // Search tools
            "web_search" | "image_search" | "google_search" | "bing_search"
            | "duckduckgo_search" | "startpage_search" => "search",

            // JavaScript execution
            "cdp_runtime_evaluate" => "javascript",

            // Memory tools
            "ai_memory_store_research"
            | "ai_memory_get_research"
            | "ai_memory_search_research"
            | "ai_memory_store_credentials"
            | "ai_memory_get_credentials"
            | "ai_memory_store_bookmark"
            | "ai_memory_get_bookmarks"
            | "ai_memory_store_note"
            | "ai_memory_get_notes" => "memory",

            // Snapshot/scraping tools
            "snapshot_url" | "browse_readable_content" | "browser_get_page_content" => {
                "snapshot_url"
            }

            // Session management
            "browser_session_management" | "browser_validate_session" => "session",

            // CDP tools
            "cdp_dom_get_document"
            | "cdp_dom_query_selector"
            | "cdp_dom_get_attributes"
            | "cdp_dom_get_computed_style"
            | "cdp_network_get_cookies"
            | "cdp_network_set_cookie"
            | "cdp_console_get_messages"
            | "cdp_page_screenshot"
            | "cdp_page_reload" => "cdp",

            // Browser automation - treated as navigation (HTTP requests)
            "browser_click_element"
            | "browser_type_text"
            | "browser_fill_form"
            | "browser_wait_for_element"
            | "browser_prepare_form_submission" => "navigation",

            // WASM debug tools
            "wasm_debug_load_module"
            | "wasm_debug_unload_module"
            | "wasm_debug_list_modules"
            | "wasm_debug_validate"
            | "wasm_debug_inspect"
            | "wasm_debug_disassemble"
            | "wasm_debug_read_memory"
            | "wasm_debug_write_memory"
            | "wasm_debug_call_function"
            | "wasm_debug_profile_function" => "wasm_debug",

            // Default for unknown tools
            _ => "default",
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rate_limiter_allows_burst() {
        let limiter = RateLimiter::new();

        // Should allow burst size requests immediately
        for _ in 0..3 {
            assert!(limiter.check("navigation").is_ok());
        }
    }

    #[test]
    fn test_rate_limiter_blocks_after_burst() {
        let limiter = RateLimiter::new();

        // Exhaust burst
        for _ in 0..3 {
            let _ = limiter.check("navigation");
        }

        // Should be rate limited
        assert!(limiter.check("navigation").is_err());
    }

    #[test]
    fn test_rate_limiter_refills_over_time() {
        let limiter = RateLimiter::new();

        // Use up one token
        assert!(limiter.check("memory").is_ok());

        // Wait for refill (1 second should add 1 token for 60/min rate)
        thread::sleep(Duration::from_millis(100));

        // Should still have tokens
        assert!(limiter.check("memory").is_ok());
    }

    #[test]
    fn test_tool_to_category() {
        assert_eq!(RateLimiter::tool_to_category("web_search"), "search");
        assert_eq!(
            RateLimiter::tool_to_category("browser_navigate_to"),
            "navigation"
        );
        assert_eq!(
            RateLimiter::tool_to_category("ai_memory_store_research"),
            "memory"
        );
        assert_eq!(RateLimiter::tool_to_category("unknown_tool"), "default");
    }
}
