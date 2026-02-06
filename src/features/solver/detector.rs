//! Challenge Detection Module
//!
//! Detects various types of web challenges including:
//! - Cloudflare JS Challenge
//! - Cloudflare Turnstile
//! - hCaptcha
//! - reCAPTCHA
//! - Generic challenge pages
//!
//! Detection is based on DOM patterns, script sources, and page structure.

use serde::{Deserialize, Serialize};

/// Types of web challenges that can be detected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeType {
    /// Cloudflare JavaScript challenge (automatic, no interaction needed)
    CloudflareJS,
    /// Cloudflare Turnstile (may require checkbox interaction)
    CloudflareTurnstile,
    /// hCaptcha challenge
    HCaptcha,
    /// Google reCAPTCHA v2
    ReCaptchaV2,
    /// Google reCAPTCHA v3 (invisible)
    ReCaptchaV3,
    /// Generic interstitial/loading page
    GenericInterstitial,
    /// Unknown challenge type
    Unknown,
}

/// Detected challenge with metadata
#[derive(Debug, Clone)]
pub struct DetectedChallenge {
    /// The type of challenge detected
    pub challenge_type: ChallengeType,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Selector for the challenge widget element (if applicable)
    pub widget_selector: Option<String>,
    /// Selector for the checkbox/button to click (if applicable)
    pub click_target_selector: Option<String>,
    /// Whether the challenge requires user interaction
    pub requires_interaction: bool,
    /// Expected resolution time in milliseconds
    pub expected_resolution_time_ms: u64,
    /// Detected markers that led to this classification
    pub detected_markers: Vec<String>,
}

impl DetectedChallenge {
    /// Create a new Cloudflare JS challenge detection
    pub fn cloudflare_js(confidence: f64, markers: Vec<String>) -> Self {
        Self {
            challenge_type: ChallengeType::CloudflareJS,
            confidence,
            widget_selector: None,
            click_target_selector: None,
            requires_interaction: false,
            expected_resolution_time_ms: 5000,
            detected_markers: markers,
        }
    }

    /// Create a new Cloudflare Turnstile detection
    pub fn cloudflare_turnstile(confidence: f64, markers: Vec<String>) -> Self {
        Self {
            challenge_type: ChallengeType::CloudflareTurnstile,
            confidence,
            widget_selector: Some("[data-cf-turnstile-widget], .cf-turnstile, iframe[src*='challenges.cloudflare.com']".to_string()),
            click_target_selector: Some("input[type='checkbox'], .cf-turnstile-wrapper".to_string()),
            requires_interaction: true, // Turnstile often needs a click
            expected_resolution_time_ms: 8000,
            detected_markers: markers,
        }
    }

    /// Create a new hCaptcha detection
    pub fn hcaptcha(confidence: f64) -> Self {
        Self {
            challenge_type: ChallengeType::HCaptcha,
            confidence,
            widget_selector: Some("[data-hcaptcha-widget-id], .h-captcha, iframe[src*='hcaptcha.com']".to_string()),
            click_target_selector: Some(".hcaptcha-checkbox, [id^='checkbox']".to_string()),
            requires_interaction: true,
            expected_resolution_time_ms: 10000,
            detected_markers: vec!["hcaptcha".to_string()],
        }
    }

    /// Create a new reCAPTCHA v2 detection
    pub fn recaptcha_v2(confidence: f64) -> Self {
        Self {
            challenge_type: ChallengeType::ReCaptchaV2,
            confidence,
            widget_selector: Some(".g-recaptcha, iframe[src*='recaptcha']".to_string()),
            click_target_selector: Some(".recaptcha-checkbox-border, .recaptcha-checkbox".to_string()),
            requires_interaction: true,
            expected_resolution_time_ms: 10000,
            detected_markers: vec!["recaptcha".to_string()],
        }
    }
}

/// Challenge detector that analyzes page content
pub struct ChallengeDetector {
    /// Cloudflare challenge markers
    cf_markers: Vec<&'static str>,
    /// Turnstile-specific markers
    turnstile_markers: Vec<&'static str>,
}

impl ChallengeDetector {
    /// Create a new challenge detector
    pub fn new() -> Self {
        Self {
            cf_markers: vec![
                // Strong indicators (these alone can identify a Cloudflare challenge)
                "challenges.cloudflare.com",
                "Just a moment...",
                "/cdn-cgi/challenge-platform/",  // Challenge script path
                "/cdn-cgi/challenge",            // General challenge path
                "_cf_chl_opt",                   // Current Cloudflare JS config variable
                "_cf_chl_",                      // General prefix (catches _cf_chl_rt_tk, _cf_chl_f_tk, etc.)
                "cf_chl_",                       // Alternative prefix variant
                // Moderate indicators
                "cf-browser-verification",
                "cf_chl_prog",
                "__cf_chl_rt_tk",
                "Checking your browser before accessing",
                "Please wait while we verify your browser",
                "cf-spinner-please-wait",
                "challenge-running",
                "challenge-form",
                "cf-im-under-attack-mode",
                "challenge-error-text",          // Current challenge error element
                "cRay",                          // Current ray ID format
                "ray-id",                        // Legacy ray ID format
            ],
            turnstile_markers: vec![
                "cf-turnstile",
                "data-cf-turnstile",
                "turnstile/v0/api.js",
                "challenges.cloudflare.com/turnstile",
            ],
        }
    }

    /// Detect challenges in the given HTML content
    pub fn detect(&self, html: &str, _url: &str) -> Option<DetectedChallenge> {
        let html_lower = html.to_lowercase();

        // Check for Turnstile first (more specific)
        let turnstile_matches: Vec<String> = self.turnstile_markers
            .iter()
            .filter(|marker| html_lower.contains(&marker.to_lowercase()))
            .map(|s| s.to_string())
            .collect();

        if turnstile_matches.len() >= 1 {
            let confidence = (turnstile_matches.len() as f64 / self.turnstile_markers.len() as f64).min(1.0);
            return Some(DetectedChallenge::cloudflare_turnstile(confidence.max(0.7), turnstile_matches));
        }

        // Check for Cloudflare JS challenge
        let cf_matches: Vec<String> = self.cf_markers
            .iter()
            .filter(|marker| html_lower.contains(&marker.to_lowercase()))
            .map(|s| s.to_string())
            .collect();

        // Strong indicators that alone can identify a Cloudflare challenge
        let has_strong_indicator = cf_matches.iter().any(|m| {
            let m_lower = m.to_lowercase();
            m_lower.contains("just a moment")
                || m_lower.contains("_cf_chl")
                || m_lower.contains("cf_chl_")
                || m_lower.contains("/cdn-cgi/challenge")
                || m_lower.contains("challenges.cloudflare.com")
        });

        // If we have a strong indicator, only need 1 match for detection
        if has_strong_indicator && !cf_matches.is_empty() {
            let confidence = (cf_matches.len() as f64 / 3.0).min(1.0).max(0.8);
            return Some(DetectedChallenge::cloudflare_js(confidence, cf_matches));
        }

        // For weaker markers, require 2+ matches
        if cf_matches.len() >= 2 {
            let confidence = (cf_matches.len() as f64 / 5.0).min(1.0);
            return Some(DetectedChallenge::cloudflare_js(confidence, cf_matches));
        }

        // Check for specific Cloudflare page structure
        if html.contains("<title>Just a moment...</title>")
            || (html_lower.contains("cf-browser-verification") && html_lower.contains("challenge-running"))
        {
            return Some(DetectedChallenge::cloudflare_js(0.9, vec!["title-match".to_string()]));
        }

        // Check for hCaptcha
        if html_lower.contains("hcaptcha.com") || html_lower.contains("h-captcha") || html_lower.contains("data-hcaptcha") {
            return Some(DetectedChallenge::hcaptcha(0.9));
        }

        // Check for reCAPTCHA
        if html_lower.contains("recaptcha") && (html_lower.contains("g-recaptcha") || html_lower.contains("grecaptcha")) {
            return Some(DetectedChallenge::recaptcha_v2(0.9));
        }

        // Check for generic interstitial pages
        if self.is_generic_interstitial(&html_lower) {
            return Some(DetectedChallenge {
                challenge_type: ChallengeType::GenericInterstitial,
                confidence: 0.5,
                widget_selector: None,
                click_target_selector: None,
                requires_interaction: false,
                expected_resolution_time_ms: 3000,
                detected_markers: vec!["generic-loading".to_string()],
            });
        }

        None
    }

    /// Check for generic interstitial/loading pages
    fn is_generic_interstitial(&self, html_lower: &str) -> bool {
        let loading_indicators = [
            "please wait",
            "loading...",
            "redirecting",
            "processing your request",
            "verifying",
        ];

        // Need multiple indicators and short page length to be a true interstitial
        let indicator_count = loading_indicators.iter()
            .filter(|ind| html_lower.contains(*ind))
            .count();

        // Generic interstitials are usually short pages
        let is_short_page = html_lower.len() < 5000;

        indicator_count >= 1 && is_short_page
    }

    /// Get Cloudflare markers for external use
    pub fn cloudflare_markers(&self) -> &[&str] {
        &self.cf_markers
    }

    /// Get Turnstile markers for external use
    pub fn turnstile_markers(&self) -> &[&str] {
        &self.turnstile_markers
    }
}

impl Default for ChallengeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloudflare_js_detection() {
        let detector = ChallengeDetector::new();

        let cf_html = r#"
            <html>
            <head><title>Just a moment...</title></head>
            <body>
                <div class="cf-browser-verification">
                    <div class="challenge-running">
                        <span class="ray-id">Ray ID: abc123</span>
                    </div>
                </div>
            </body>
            </html>
        "#;

        let result = detector.detect(cf_html, "https://example.com");
        assert!(result.is_some());

        let challenge = result.unwrap();
        assert_eq!(challenge.challenge_type, ChallengeType::CloudflareJS);
        assert!(challenge.confidence >= 0.6);
        assert!(!challenge.requires_interaction);
    }

    #[test]
    fn test_turnstile_detection() {
        let detector = ChallengeDetector::new();

        let turnstile_html = r#"
            <html>
            <body>
                <div class="cf-turnstile" data-sitekey="xxx"></div>
                <script src="https://challenges.cloudflare.com/turnstile/v0/api.js"></script>
            </body>
            </html>
        "#;

        let result = detector.detect(turnstile_html, "https://example.com");
        assert!(result.is_some());

        let challenge = result.unwrap();
        assert_eq!(challenge.challenge_type, ChallengeType::CloudflareTurnstile);
        assert!(challenge.requires_interaction);
        assert!(challenge.click_target_selector.is_some());
    }

    #[test]
    fn test_no_challenge() {
        let detector = ChallengeDetector::new();

        let normal_html = r#"
            <html>
            <head><title>Welcome to my site</title></head>
            <body>
                <h1>Hello World!</h1>
                <p>This is a normal page with no challenges.</p>
            </body>
            </html>
        "#;

        let result = detector.detect(normal_html, "https://example.com");
        assert!(result.is_none());
    }

    #[test]
    fn test_hcaptcha_detection() {
        let detector = ChallengeDetector::new();

        let hcaptcha_html = r#"
            <html>
            <body>
                <div class="h-captcha" data-sitekey="xxx"></div>
                <script src="https://hcaptcha.com/1/api.js"></script>
            </body>
            </html>
        "#;

        let result = detector.detect(hcaptcha_html, "https://example.com");
        assert!(result.is_some());

        let challenge = result.unwrap();
        assert_eq!(challenge.challenge_type, ChallengeType::HCaptcha);
    }

    /// Test detection of current Cloudflare challenge format (2024+)
    /// This tests the updated markers that match current Cloudflare HTML patterns
    #[test]
    fn test_current_cloudflare_format_detection() {
        let detector = ChallengeDetector::new();

        // Simulated current Cloudflare challenge page HTML (based on actual observed patterns)
        let cf_html = r#"
            <!DOCTYPE html>
            <html lang="en-US">
            <head>
                <title>Just a moment...</title>
                <script>
                    window._cf_chl_opt={
                        cvId: '3',
                        cZone: 'example.com',
                        cType: 'managed',
                        cRay: 'abc123def456',
                    };
                </script>
            </head>
            <body>
                <div id="challenge-error-text">
                    Enable JavaScript and cookies to continue
                </div>
                <script src="/cdn-cgi/challenge-platform/scripts/abc123.js"></script>
            </body>
            </html>
        "#;

        let result = detector.detect(cf_html, "https://example.com");
        assert!(result.is_some(), "Should detect current Cloudflare challenge format");

        let challenge = result.unwrap();
        assert_eq!(challenge.challenge_type, ChallengeType::CloudflareJS);
        assert!(challenge.confidence >= 0.8, "Confidence should be high for strong indicators");
    }

    /// Test that a page with only _cf_chl_opt is detected (strong indicator)
    #[test]
    fn test_cf_chl_opt_strong_indicator() {
        let detector = ChallengeDetector::new();

        let cf_html = r#"
            <html>
            <head><script>window._cf_chl_opt={cvId:'3'};</script></head>
            <body></body>
            </html>
        "#;

        let result = detector.detect(cf_html, "https://example.com");
        assert!(result.is_some(), "_cf_chl_opt alone should be detected as a challenge");

        let challenge = result.unwrap();
        assert_eq!(challenge.challenge_type, ChallengeType::CloudflareJS);
    }

    /// Test that /cdn-cgi/challenge path is detected (strong indicator)
    #[test]
    fn test_cdn_cgi_challenge_path() {
        let detector = ChallengeDetector::new();

        let cf_html = r#"
            <html>
            <body>
                <script src="/cdn-cgi/challenge-platform/h/g/scripts/something.js"></script>
            </body>
            </html>
        "#;

        let result = detector.detect(cf_html, "https://example.com");
        assert!(result.is_some(), "/cdn-cgi/challenge path should be detected as a challenge");
    }

    /// Test that "Just a moment..." title alone is detected (strong indicator)
    #[test]
    fn test_just_a_moment_title_strong_indicator() {
        let detector = ChallengeDetector::new();

        let cf_html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Just a moment...</title></head>
            <body></body>
            </html>
        "#;

        let result = detector.detect(cf_html, "https://example.com");
        assert!(result.is_some(), "'Just a moment...' alone should be detected as a challenge");
    }
}
