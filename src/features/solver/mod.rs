pub mod types;
pub mod patterns;
pub mod solvers;
pub mod browser_globals;
pub mod utils;

use anyhow::Result;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info, warn};

pub use types::{ChallengeSolver, ChallengeResult, ChallengeType, SolverConfig, ChallengeInfo, JsExecutionResult, ChallengeFormData, BrowserFingerprint};
pub use patterns::{ChallengePatterns, detect_challenge_type, extract_challenge_info};
pub use solvers::*;
pub use browser_globals::*;
pub use utils::*;

impl ChallengeSolver {
    /// Create a new challenge solver
    pub fn new() -> Self {
        let mut context = boa_engine::Context::default();
        browser_globals::setup_browser_globals(&mut context);
        
        Self {
            context,
            patterns: ChallengePatterns::default(),
            cache: HashMap::new(),
            config: SolverConfig::default(),
        }
    }

    /// Create a challenge solver with custom configuration
    pub fn with_config(config: SolverConfig) -> Self {
        let mut solver = Self::new();
        solver.config = config;
        solver
    }

    /// Detect and solve challenges in HTML content
    pub async fn solve_challenges(&mut self, html: &str, url: &str) -> Result<ChallengeResult> {
        let start_time = Instant::now();
        
        // Check cache first
        let cache_key = format!("{}:{}", url, utils::html_hash(html));
        if self.config.enable_caching {
            if let Some(cached) = self.cache.get(&cache_key) {
                debug!("Using cached challenge solution for {}", url);
                return Ok(cached.clone());
            }
        }

        // Detect challenge type
        let challenge_type = self.detect_challenge_type(html, url);
        info!("Detected challenge type: {:?} for {}", challenge_type, url);

        // Solve the specific challenge
        let mut result = match challenge_type.clone() {
            ChallengeType::GoogleAntiBot => solvers::solve_google_antibot(self, html, url).await?,
            ChallengeType::GoogleRecaptchaV3 => solvers::solve_recaptcha_v3(self, html, url).await?,
            ChallengeType::CloudflareJsChallenge => solvers::solve_cloudflare_js_challenge(self, html, url).await?,
            ChallengeType::CloudflareTurnstile => solvers::solve_cloudflare_turnstile(self, html, url).await?,
            ChallengeType::Generic => solvers::solve_generic_challenge(self, html, url).await?,
            _ => {
                warn!("Unsupported challenge type: {:?}", challenge_type);
                ChallengeResult {
                    solved: false,
                    solution_data: serde_json::Map::new(),
                    solve_time: start_time.elapsed(),
                    challenge_type: challenge_type.clone(),
                    metadata: HashMap::new(),
                }
            }
        };

        result.solve_time = start_time.elapsed();
        result.challenge_type = challenge_type;

        // Cache the result
        if self.config.enable_caching && result.solved {
            self.cache.insert(cache_key, result.clone());
        }

        Ok(result)
    }

    /// Detect the type of challenge present in the HTML
    pub fn detect_challenge_type(&self, html: &str, url: &str) -> ChallengeType {
        patterns::detect_challenge_type(&self.patterns, html, url)
    }

    /// Get statistics about cached solutions
    pub fn get_cache_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        for result in self.cache.values() {
            let key = format!("{:?}", result.challenge_type);
            *stats.entry(key).or_insert(0) += 1;
        }
        
        stats.insert("total".to_string(), self.cache.len());
        stats
    }

    /// Clear the solution cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}