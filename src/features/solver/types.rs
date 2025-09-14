use boa_engine::Context;
use serde_json::Map;
use std::collections::HashMap;
use std::time::Duration;

/// Types of challenges that can be detected and solved
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChallengeType {
    /// Google's anti-bot detection system
    GoogleAntiBot,
    /// Google reCAPTCHA v3 invisible challenges
    GoogleRecaptchaV3,
    /// Google reCAPTCHA v2 checkbox/image challenges
    GoogleRecaptchaV2,
    /// Cloudflare JavaScript computation challenges
    CloudflareJsChallenge,
    /// Cloudflare Turnstile CAPTCHA system
    CloudflareTurnstile,
    /// Cloudflare Browser Integrity Check
    CloudflareBic,
    /// Generic bot detection (custom implementations)
    Generic,
    /// hCaptcha challenges
    HCaptcha,
    /// DataDome bot protection
    DataDome,
    /// PerimeterX bot detection
    PerimeterX,
    /// Shape Security challenges
    ShapeSecurity,
    /// Akamai Bot Manager
    AkamaiBotManager,
    /// Unknown or unrecognized challenge
    Unknown,
}

impl Default for ChallengeType {
    fn default() -> Self {
        ChallengeType::Unknown
    }
}

/// Configuration for the challenge solver
#[derive(Debug, Clone)]
pub struct SolverConfig {
    /// Maximum time to spend solving a challenge
    pub timeout: Duration,
    /// Whether to cache solved challenges
    pub enable_caching: bool,
    /// Whether to add human-like delays
    pub human_delays: bool,
    /// Aggressiveness level (1-10, higher = more attempts)
    pub aggressiveness: u8,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            enable_caching: true,
            human_delays: true,
            aggressiveness: 5,
        }
    }
}

/// Result of a challenge solving attempt
#[derive(Debug, Clone)]
pub struct ChallengeResult {
    /// Whether the challenge was successfully solved
    pub solved: bool,
    /// Solution data (tokens, cookies, form data, etc.)
    pub solution_data: Map<String, serde_json::Value>,
    /// Time taken to solve the challenge
    pub solve_time: Duration,
    /// Type of challenge that was solved
    pub challenge_type: ChallengeType,
    /// Additional metadata about the solving process
    pub metadata: HashMap<String, String>,
}

impl Default for ChallengeResult {
    fn default() -> Self {
        Self {
            solved: false,
            solution_data: Map::new(),
            solve_time: Duration::from_secs(0),
            challenge_type: ChallengeType::Unknown,
            metadata: HashMap::new(),
        }
    }
}

/// Main challenge solver struct
pub struct ChallengeSolver {
    /// JavaScript execution context
    pub context: Context,
    /// Challenge detection patterns
    pub patterns: crate::challenge_solver::ChallengePatterns,
    /// Cache for solved challenges
    pub cache: HashMap<String, ChallengeResult>,
    /// Solver configuration
    pub config: SolverConfig,
}

/// Information about a detected challenge
#[derive(Debug, Clone)]
pub struct ChallengeInfo {
    /// Type of challenge detected
    pub challenge_type: ChallengeType,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,
    /// Detected parameters/tokens
    pub parameters: HashMap<String, String>,
    /// Challenge-specific metadata
    pub metadata: HashMap<String, String>,
}

/// JavaScript execution result for challenges
#[derive(Debug, Clone)]
pub struct JsExecutionResult {
    /// Whether execution was successful
    pub success: bool,
    /// Execution result value
    pub result: Option<String>,
    /// Any error that occurred
    pub error: Option<String>,
    /// Execution time
    pub duration: Duration,
}

/// Form submission data for challenges
#[derive(Debug, Clone)]
pub struct ChallengeFormData {
    /// Form action URL
    pub action: String,
    /// Form method (GET/POST)
    pub method: String,
    /// Form fields and values
    pub fields: HashMap<String, String>,
    /// Additional headers to send
    pub headers: HashMap<String, String>,
}

/// Browser fingerprint data for challenges
#[derive(Debug, Clone)]
pub struct BrowserFingerprint {
    /// User agent string
    pub user_agent: String,
    /// Screen resolution
    pub screen_resolution: String,
    /// Timezone offset
    pub timezone_offset: i32,
    /// Available fonts
    pub fonts: Vec<String>,
    /// WebGL renderer info
    pub webgl_info: String,
    /// Canvas fingerprint
    pub canvas_fingerprint: String,
}