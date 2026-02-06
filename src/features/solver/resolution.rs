//! Challenge Resolution Module
//!
//! Handles waiting for challenges to resolve naturally after behavioral
//! simulation. This module monitors for signs that a challenge has been
//! passed, such as:
//! - Cookie changes (cf_clearance)
//! - DOM changes (challenge elements disappearing)
//! - Page content changes
//! - Token callbacks being fired

use std::time::Duration;
use super::detector::{DetectedChallenge, ChallengeType};

/// Configuration for challenge resolution
#[derive(Debug, Clone)]
pub struct ResolutionConfig {
    /// Default timeout for resolution (ms)
    pub default_timeout_ms: u64,
    /// Polling interval for checking resolution (ms)
    pub poll_interval_ms: u64,
    /// Extra wait time after apparent resolution (ms)
    pub post_resolution_delay_ms: u64,
    /// Whether to wait for cookies
    pub wait_for_cookies: bool,
    /// Whether to wait for DOM changes
    pub wait_for_dom_changes: bool,
}

impl Default for ResolutionConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: 15000,
            poll_interval_ms: 200,
            post_resolution_delay_ms: 500,
            wait_for_cookies: true,
            wait_for_dom_changes: true,
        }
    }
}

/// Result of challenge resolution
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    /// Whether the challenge was resolved successfully
    pub success: bool,
    /// How long it took to resolve (if successful)
    pub resolution_time: Option<Duration>,
    /// Any cookies that were set
    pub cookies_set: Vec<String>,
    /// Final page state
    pub final_state: ResolutionState,
}

/// State of the page after resolution attempt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionState {
    /// Challenge resolved, page content available
    Resolved,
    /// Challenge still running (timeout)
    StillRunning,
    /// Challenge failed (blocked)
    Failed,
    /// Unknown state
    Unknown,
}

/// Challenge resolver that monitors for challenge completion
pub struct ChallengeResolver {
    config: ResolutionConfig,
}

impl ChallengeResolver {
    /// Create a new resolver with default config
    pub fn new() -> Self {
        Self {
            config: ResolutionConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ResolutionConfig) -> Self {
        Self { config }
    }

    /// Get recommended wait time for a challenge type
    pub fn recommended_wait_time(&self, challenge: &DetectedChallenge) -> Duration {
        Duration::from_millis(challenge.expected_resolution_time_ms)
    }

    /// Generate JavaScript to wait for challenge resolution
    pub fn generate_wait_js(&self, challenge: &DetectedChallenge, timeout_ms: u64) -> String {
        match challenge.challenge_type {
            ChallengeType::CloudflareJS => self.generate_cloudflare_js_wait(timeout_ms),
            ChallengeType::CloudflareTurnstile => self.generate_turnstile_wait(timeout_ms),
            ChallengeType::HCaptcha => self.generate_hcaptcha_wait(timeout_ms),
            ChallengeType::ReCaptchaV2 | ChallengeType::ReCaptchaV3 => self.generate_recaptcha_wait(timeout_ms),
            ChallengeType::GenericInterstitial => self.generate_generic_wait(timeout_ms),
            ChallengeType::Unknown => self.generate_generic_wait(timeout_ms),
        }
    }

    /// Generate wait JavaScript for Cloudflare JS challenge
    /// Uses window._asyncResult and window._asyncComplete for result storage
    fn generate_cloudflare_js_wait(&self, timeout_ms: u64) -> String {
        format!(
            r#"(function() {{
    var startTime = Date.now();
    var timeout = {};
    var pollInterval = {};
    var postDelay = {};

    function completeWith(result) {{
        window._asyncResult = result;
        window._asyncComplete = true;
    }}

    function checkResolution() {{
        try {{
            // Check for cf_clearance cookie
            var cookies = document.cookie;
            if (cookies.indexOf('cf_clearance') !== -1) {{
                // Cookie found! Wait a bit then complete
                setTimeout(function() {{
                    completeWith({{ success: true, reason: 'cf_clearance cookie found' }});
                }}, postDelay);
                return;
            }}

            // Check if challenge elements are gone
            var challengeRunning = document.querySelector('.challenge-running');
            var spinner = document.querySelector('.cf-spinner-please-wait');

            if (!challengeRunning && !spinner && document.readyState === 'complete') {{
                // Challenge elements gone, check page title
                var title = document.title || '';
                if (title.indexOf('Just a moment') === -1) {{
                    setTimeout(function() {{
                        completeWith({{ success: true, reason: 'challenge elements removed' }});
                    }}, postDelay);
                    return;
                }}
            }}

            // Check for challenge success class
            var challengeSuccess = document.querySelector('.challenge-success');
            if (challengeSuccess) {{
                setTimeout(function() {{
                    completeWith({{ success: true, reason: 'challenge-success element found' }});
                }}, postDelay);
                return;
            }}

            // Check timeout
            if (Date.now() - startTime > timeout) {{
                completeWith({{ success: false, reason: 'timeout' }});
                return;
            }}

            // Poll again
            setTimeout(checkResolution, pollInterval);
        }} catch(e) {{
            completeWith({{ success: false, reason: 'error: ' + e.message }});
        }}
    }}

    checkResolution();
}})()"#,
            timeout_ms,
            self.config.poll_interval_ms,
            self.config.post_resolution_delay_ms
        )
    }

    /// Generate wait JavaScript for Turnstile challenge
    /// Uses window._asyncResult and window._asyncComplete for result storage
    fn generate_turnstile_wait(&self, timeout_ms: u64) -> String {
        format!(
            r#"(function() {{
    var startTime = Date.now();
    var timeout = {};
    var pollInterval = {};
    var postDelay = {};

    function completeWith(result) {{
        window._asyncResult = result;
        window._asyncComplete = true;
    }}

    function checkResolution() {{
        try {{
            // Check turnstile API directly for existing responses (most reliable)
            if (typeof turnstile !== 'undefined' && turnstile.getResponse) {{
                // Try to get response using widget IDs from data attributes
                try {{
                    var containers = document.querySelectorAll('.cf-turnstile, [data-cf-turnstile]');
                    for (var i = 0; i < containers.length; i++) {{
                        var container = containers[i];
                        var widgetId = container.getAttribute('data-cf-turnstile-widget-id') || container.id || '0';
                        try {{
                            var response = turnstile.getResponse(widgetId);
                            if (response) {{
                                setTimeout(function() {{
                                    completeWith({{ success: true, reason: 'turnstile API response', token: response }});
                                }}, postDelay);
                                return;
                            }}
                        }} catch (e) {{}}
                    }}
                }} catch (e) {{}}

                // Try default widget ID '0'
                try {{
                    var response = turnstile.getResponse('0');
                    if (response) {{
                        setTimeout(function() {{
                            completeWith({{ success: true, reason: 'turnstile default widget response', token: response }});
                        }}, postDelay);
                        return;
                    }}
                }} catch (e) {{}}
            }}

            // Check for stored token from callback hook
            if (window._turnstileSuccess && window._turnstileToken) {{
                setTimeout(function() {{
                    completeWith({{ success: true, reason: 'turnstile callback triggered', token: window._turnstileToken }});
                }}, postDelay);
                return;
            }}

            // Check for Turnstile token in form inputs (fallback for DOM-based detection)
            var tokenInput = document.querySelector('input[name="cf-turnstile-response"]');
            if (tokenInput && tokenInput.value) {{
                setTimeout(function() {{
                    completeWith({{ success: true, reason: 'turnstile token found', token: tokenInput.value }});
                }}, postDelay);
                return;
            }}

            // Check for cf_clearance cookie as fallback
            if (document.cookie.indexOf('cf_clearance') !== -1) {{
                setTimeout(function() {{
                    completeWith({{ success: true, reason: 'cf_clearance cookie found' }});
                }}, postDelay);
                return;
            }}

            // Check timeout
            if (Date.now() - startTime > timeout) {{
                completeWith({{ success: false, reason: 'timeout' }});
                return;
            }}

            // Poll again
            setTimeout(checkResolution, pollInterval);
        }} catch(e) {{
            completeWith({{ success: false, reason: 'error: ' + e.message }});
        }}
    }}

    // Set up callback hook for Turnstile render (catches future renders)
    if (typeof turnstile !== 'undefined' && !window._turnstileHooked) {{
        window._turnstileHooked = true;
        var originalRender = turnstile.render;
        if (originalRender) {{
            turnstile.render = function(element, options) {{
                options = options || {{}};
                var originalCallback = options.callback;
                options.callback = function(token) {{
                    window._turnstileSuccess = true;
                    window._turnstileToken = token;
                    if (originalCallback) originalCallback(token);
                }};
                return originalRender.call(this, element, options);
            }};
        }}
    }}

    checkResolution();
}})()"#,
            timeout_ms,
            self.config.poll_interval_ms,
            self.config.post_resolution_delay_ms
        )
    }

    /// Generate wait JavaScript for hCaptcha challenge
    /// Uses window._asyncResult and window._asyncComplete for result storage
    fn generate_hcaptcha_wait(&self, timeout_ms: u64) -> String {
        format!(
            r#"(function() {{
    var startTime = Date.now();
    var timeout = {};
    var pollInterval = {};
    var postDelay = {};

    function completeWith(result) {{
        window._asyncResult = result;
        window._asyncComplete = true;
    }}

    function checkResolution() {{
        try {{
            // Check for hCaptcha response in form
            var tokenInput = document.querySelector('[name="h-captcha-response"]');
            if (tokenInput && tokenInput.value) {{
                setTimeout(function() {{
                    completeWith({{ success: true, reason: 'hcaptcha token found' }});
                }}, postDelay);
                return;
            }}

            // Check for success callback
            if (window._hcaptchaSuccess) {{
                setTimeout(function() {{
                    completeWith({{ success: true, reason: 'hcaptcha callback triggered' }});
                }}, postDelay);
                return;
            }}

            // Check timeout
            if (Date.now() - startTime > timeout) {{
                completeWith({{ success: false, reason: 'timeout' }});
                return;
            }}

            setTimeout(checkResolution, pollInterval);
        }} catch(e) {{
            completeWith({{ success: false, reason: 'error: ' + e.message }});
        }}
    }}

    // Hook into hcaptcha if available
    if (typeof hcaptcha !== 'undefined' && !window._hcaptchaHooked) {{
        window._hcaptchaHooked = true;
        var originalRender = hcaptcha.render;
        hcaptcha.render = function(element, options) {{
            var originalCallback = options.callback;
            options.callback = function(token) {{
                window._hcaptchaSuccess = true;
                if (originalCallback) originalCallback(token);
            }};
            return originalRender.call(this, element, options);
        }};
    }}

    checkResolution();
}})()"#,
            timeout_ms,
            self.config.poll_interval_ms,
            self.config.post_resolution_delay_ms
        )
    }

    /// Generate wait JavaScript for reCAPTCHA challenge
    /// Uses window._asyncResult and window._asyncComplete for result storage
    fn generate_recaptcha_wait(&self, timeout_ms: u64) -> String {
        format!(
            r#"(function() {{
    var startTime = Date.now();
    var timeout = {};
    var pollInterval = {};
    var postDelay = {};

    function completeWith(result) {{
        window._asyncResult = result;
        window._asyncComplete = true;
    }}

    function checkResolution() {{
        try {{
            // Check for reCAPTCHA response
            var tokenInput = document.querySelector('[name="g-recaptcha-response"]');
            if (tokenInput && tokenInput.value) {{
                setTimeout(function() {{
                    completeWith({{ success: true, reason: 'recaptcha token found' }});
                }}, postDelay);
                return;
            }}

            // Check grecaptcha API
            if (typeof grecaptcha !== 'undefined') {{
                try {{
                    var response = grecaptcha.getResponse();
                    if (response) {{
                        setTimeout(function() {{
                            completeWith({{ success: true, reason: 'grecaptcha response found' }});
                        }}, postDelay);
                        return;
                    }}
                }} catch(e) {{}}
            }}

            // Check timeout
            if (Date.now() - startTime > timeout) {{
                completeWith({{ success: false, reason: 'timeout' }});
                return;
            }}

            setTimeout(checkResolution, pollInterval);
        }} catch(e) {{
            completeWith({{ success: false, reason: 'error: ' + e.message }});
        }}
    }}

    checkResolution();
}})()"#,
            timeout_ms,
            self.config.poll_interval_ms,
            self.config.post_resolution_delay_ms
        )
    }

    /// Generate wait JavaScript for generic interstitial
    /// Uses window._asyncResult and window._asyncComplete for result storage
    fn generate_generic_wait(&self, timeout_ms: u64) -> String {
        format!(
            r#"(function() {{
    var startTime = Date.now();
    var timeout = {};
    var pollInterval = {};
    var postDelay = {};
    var initialContent = document.body ? document.body.innerHTML.length : 0;

    function completeWith(result) {{
        window._asyncResult = result;
        window._asyncComplete = true;
    }}

    function checkResolution() {{
        try {{
            // Check if page content has significantly changed
            var currentContent = document.body ? document.body.innerHTML.length : 0;
            var contentChanged = Math.abs(currentContent - initialContent) > 1000;

            // Check if document is fully loaded and content changed
            if (document.readyState === 'complete' && contentChanged) {{
                setTimeout(function() {{
                    completeWith({{ success: true, reason: 'page content changed' }});
                }}, postDelay);
                return;
            }}

            // Check for common loading indicators gone
            var loadingElements = document.querySelectorAll('.loading, .spinner, [class*="loading"], [class*="spinner"]');
            var hasLoading = loadingElements.length > 0;

            if (!hasLoading && document.readyState === 'complete') {{
                setTimeout(function() {{
                    completeWith({{ success: true, reason: 'loading complete' }});
                }}, postDelay);
                return;
            }}

            // Check timeout
            if (Date.now() - startTime > timeout) {{
                completeWith({{ success: false, reason: 'timeout' }});
                return;
            }}

            setTimeout(checkResolution, pollInterval);
        }} catch(e) {{
            completeWith({{ success: false, reason: 'error: ' + e.message }});
        }}
    }}

    checkResolution();
}})()"#,
            timeout_ms,
            self.config.poll_interval_ms,
            self.config.post_resolution_delay_ms
        )
    }

    /// Check if content indicates challenge is resolved
    pub fn is_resolved(&self, html: &str, challenge: &DetectedChallenge) -> bool {
        let html_lower = html.to_lowercase();

        match challenge.challenge_type {
            ChallengeType::CloudflareJS | ChallengeType::CloudflareTurnstile => {
                // Not resolved if still has challenge markers
                !html.contains("Just a moment...")
                    && !html_lower.contains("challenge-running")
                    && !html_lower.contains("cf-spinner")
            }
            ChallengeType::HCaptcha => {
                // Check if hCaptcha widget is gone or has success
                !html_lower.contains("h-captcha-response=\"\"")
            }
            ChallengeType::ReCaptchaV2 | ChallengeType::ReCaptchaV3 => {
                !html_lower.contains("g-recaptcha-response=\"\"")
            }
            ChallengeType::GenericInterstitial | ChallengeType::Unknown => {
                // Generic: assume resolved if content is substantial
                html.len() > 5000
            }
        }
    }
}

impl Default for ChallengeResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::solver::detector::ChallengeType;

    fn create_test_challenge(challenge_type: ChallengeType) -> DetectedChallenge {
        DetectedChallenge {
            challenge_type,
            confidence: 0.9,
            widget_selector: None,
            click_target_selector: None,
            requires_interaction: false,
            expected_resolution_time_ms: 5000,
            detected_markers: vec!["test".to_string()],
        }
    }

    #[test]
    fn test_resolver_creation() {
        let resolver = ChallengeResolver::new();
        assert!(resolver.config.wait_for_cookies);
        assert!(resolver.config.wait_for_dom_changes);
    }

    #[test]
    fn test_generate_cloudflare_wait_js() {
        let resolver = ChallengeResolver::new();
        let challenge = create_test_challenge(ChallengeType::CloudflareJS);

        let js = resolver.generate_wait_js(&challenge, 10000);

        assert!(js.contains("cf_clearance"));
        assert!(js.contains("challenge-running"));
        assert!(js.contains("_asyncComplete"));
    }

    #[test]
    fn test_generate_turnstile_wait_js() {
        let resolver = ChallengeResolver::new();
        let challenge = create_test_challenge(ChallengeType::CloudflareTurnstile);

        let js = resolver.generate_wait_js(&challenge, 10000);

        assert!(js.contains("cf-turnstile-response"));
        assert!(js.contains("turnstile"));
    }

    #[test]
    fn test_is_resolved() {
        let resolver = ChallengeResolver::new();
        let challenge = create_test_challenge(ChallengeType::CloudflareJS);

        // Challenge page should not be resolved
        let challenge_html = "<title>Just a moment...</title><div class='challenge-running'></div>";
        assert!(!resolver.is_resolved(challenge_html, &challenge));

        // Normal page should be resolved
        let normal_html = "<html><body><h1>Welcome</h1><p>Lots of content here...</p></body></html>";
        assert!(resolver.is_resolved(normal_html, &challenge));
    }

    #[test]
    fn test_recommended_wait_time() {
        let resolver = ChallengeResolver::new();
        let challenge = DetectedChallenge {
            challenge_type: ChallengeType::CloudflareJS,
            confidence: 0.9,
            widget_selector: None,
            click_target_selector: None,
            requires_interaction: false,
            expected_resolution_time_ms: 8000,
            detected_markers: vec![],
        };

        let wait_time = resolver.recommended_wait_time(&challenge);
        assert_eq!(wait_time, Duration::from_millis(8000));
    }
}
