//! Challenge Solver Module
//!
//! This module provides proper browser behavior for handling web challenges
//! like Cloudflare and Turnstile. The approach is NOT to "hack" or "bypass"
//! these challenges, but to behave like a real browser with a real user.
//!
//! Philosophy: A real browser doesn't need to "solve" Cloudflare - it just
//! works because it IS a real browser with a real user interacting with it.
//!
//! The key insight is that Cloudflare detection is based on behavioral
//! fingerprinting. A headless browser fails not because it's technically
//! different, but because it doesn't exhibit human behavior patterns.

pub mod detector;
pub mod behavioral;
pub mod resolution;

// Re-export main types
pub use detector::{ChallengeType, ChallengeDetector, DetectedChallenge};
pub use behavioral::{BehavioralSimulator, BehavioralConfig, InteractionResult};
pub use resolution::{ChallengeResolver, ResolutionConfig, ResolutionResult};

use anyhow::Result;
use std::time::Duration;

/// Main challenge handler that orchestrates detection, behavioral simulation,
/// and resolution for web challenges.
///
/// This is the high-level API that integrates all components.
pub struct ChallengeSolver {
    /// Challenge detector
    detector: ChallengeDetector,
    /// Behavioral simulator
    simulator: BehavioralSimulator,
    /// Challenge resolver
    resolver: ChallengeResolver,
}

impl ChallengeSolver {
    /// Create a new challenge solver with default configuration
    pub fn new() -> Self {
        Self {
            detector: ChallengeDetector::new(),
            simulator: BehavioralSimulator::new(),
            resolver: ChallengeResolver::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        behavioral_config: BehavioralConfig,
        resolution_config: ResolutionConfig,
    ) -> Self {
        Self {
            detector: ChallengeDetector::new(),
            simulator: BehavioralSimulator::with_config(behavioral_config),
            resolver: ChallengeResolver::with_config(resolution_config),
        }
    }

    /// Detect what type of challenge (if any) is present on the page
    pub fn detect_challenge(&self, html: &str, url: &str) -> Option<DetectedChallenge> {
        self.detector.detect(html, url)
    }

    /// Generate a behavioral interaction sequence for the given challenge
    /// This creates mouse movements, timing delays, and event sequences
    /// that mimic real human interaction with the page.
    pub fn generate_interaction_sequence(
        &self,
        challenge: &DetectedChallenge,
        viewport_width: f64,
        viewport_height: f64,
    ) -> Result<InteractionResult> {
        self.simulator.generate_interaction(challenge, viewport_width, viewport_height)
    }

    /// Generate JavaScript code to execute the behavioral simulation
    /// This returns JS that dispatches proper mouse events with realistic timing
    ///
    /// For Turnstile and other WASM-rendered widgets, this uses smart widget detection
    /// to find the actual click target coordinates at runtime.
    pub fn generate_interaction_js(
        &self,
        challenge: &DetectedChallenge,
        viewport_width: f64,
        viewport_height: f64,
    ) -> Result<String> {
        let interaction = self.generate_interaction_sequence(challenge, viewport_width, viewport_height)?;

        // Use smart widget detection for Turnstile (WASM-rendered checkbox)
        match challenge.challenge_type {
            detector::ChallengeType::CloudflareTurnstile |
            detector::ChallengeType::HCaptcha |
            detector::ChallengeType::ReCaptchaV2 => {
                // These challenges have WASM/iframe-rendered checkboxes
                // Use widget detection to find real coordinates
                Ok(self.simulator.to_javascript_with_widget_detection(&interaction, challenge))
            }
            _ => {
                // For other challenge types, use the standard method
                Ok(self.simulator.to_javascript(&interaction))
            }
        }
    }

    /// Get the recommended wait time for challenge resolution
    pub fn get_resolution_wait_time(&self, challenge: &DetectedChallenge) -> Duration {
        self.resolver.recommended_wait_time(challenge)
    }

    /// Generate JavaScript to wait for challenge resolution
    pub fn generate_wait_for_resolution_js(&self, challenge: &DetectedChallenge, timeout_ms: u64) -> String {
        self.resolver.generate_wait_js(challenge, timeout_ms)
    }

    /// Generate a complete challenge handling sequence as JavaScript
    /// This combines behavioral simulation with resolution waiting
    pub fn generate_complete_handling_js(
        &self,
        challenge: &DetectedChallenge,
        viewport_width: f64,
        viewport_height: f64,
        timeout_ms: u64,
    ) -> Result<String> {
        let interaction_js = self.generate_interaction_js(challenge, viewport_width, viewport_height)?;
        let wait_js = self.generate_wait_for_resolution_js(challenge, timeout_ms);

        Ok(format!(
            r#"
(async function() {{
    // Phase 1: Behavioral simulation (act like a real user)
    {}

    // Phase 2: Wait for challenge resolution
    return {};
}})()
"#,
            interaction_js, wait_js
        ))
    }
}

impl Default for ChallengeSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_challenge_solver_creation() {
        let solver = ChallengeSolver::new();

        // Test that no challenge is detected on normal HTML
        let html = "<html><body><h1>Hello World</h1></body></html>";
        let challenge = solver.detect_challenge(html, "https://example.com");
        assert!(challenge.is_none());
    }

    #[test]
    fn test_cloudflare_detection() {
        let solver = ChallengeSolver::new();

        let cf_html = r#"
            <html>
            <head><title>Just a moment...</title></head>
            <body>
                <div class="cf-browser-verification">
                    <div class="challenge-running">
                        <div class="cf-spinner-please-wait">Please wait...</div>
                    </div>
                </div>
            </body>
            </html>
        "#;

        let challenge = solver.detect_challenge(cf_html, "https://example.com");
        assert!(challenge.is_some());

        let detected = challenge.unwrap();
        assert!(matches!(detected.challenge_type, ChallengeType::CloudflareJS | ChallengeType::CloudflareTurnstile));
    }
}
