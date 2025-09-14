use super::types::{ChallengeType, ChallengeInfo};
use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, info};

/// Challenge detection patterns and rules
#[derive(Debug)]
pub struct ChallengePatterns {
    /// Compiled regex patterns for different challenges
    patterns: HashMap<ChallengeType, Vec<Regex>>,
    /// URL patterns that indicate specific challenges
    url_patterns: HashMap<ChallengeType, Vec<Regex>>,
    /// HTML element patterns for detection
    element_patterns: HashMap<ChallengeType, Vec<String>>,
    /// JavaScript code patterns
    js_patterns: HashMap<ChallengeType, Vec<String>>,
}

impl Default for ChallengePatterns {
    fn default() -> Self {
        let mut patterns = Self {
            patterns: HashMap::new(),
            url_patterns: HashMap::new(),
            element_patterns: HashMap::new(),
            js_patterns: HashMap::new(),
        };
        patterns.initialize_patterns();
        patterns
    }
}

impl ChallengePatterns {
    /// Initialize all challenge detection patterns
    fn initialize_patterns(&mut self) {
        self.init_google_patterns();
        self.init_cloudflare_patterns();
        self.init_recaptcha_patterns();
        self.init_generic_patterns();
        self.init_other_protection_patterns();
    }

    /// Initialize Google-specific challenge patterns
    fn init_google_patterns(&mut self) {
        // Google Anti-Bot patterns
        let google_regexes = vec![
            Regex::new(r"httpservice/retry/enablejs").unwrap(),
            Regex::new(r#"google\.tick\(["']load["'],\s*["']pbsst["']\)"#).unwrap(),
            Regex::new(r"meta.*refresh.*httpservice").unwrap(),
            Regex::new(r"Please click.*if you are not redirected").unwrap(),
        ];
        self.patterns.insert(ChallengeType::GoogleAntiBot, google_regexes);

        let google_elements = vec![
            r#"meta[http-equiv="refresh"][content*="httpservice"]"#.to_string(),
            r#"div:contains("Please click here if you are not redirected")"#.to_string(),
        ];
        self.element_patterns.insert(ChallengeType::GoogleAntiBot, google_elements);

        let google_js = vec![
            "google.tick(".to_string(),
            "httpservice/retry/enablejs".to_string(),
        ];
        self.js_patterns.insert(ChallengeType::GoogleAntiBot, google_js);
    }

    /// Initialize Cloudflare-specific challenge patterns
    fn init_cloudflare_patterns(&mut self) {
        // Cloudflare JS Challenge patterns
        let cf_js_regexes = vec![
            Regex::new(r"cf-challenge").unwrap(),
            Regex::new(r"cf-browser-verification").unwrap(),
            Regex::new(r"Checking your browser").unwrap(),
            Regex::new(r"/cdn-cgi/l/chk_captcha").unwrap(),
            Regex::new(r"jschl_vc").unwrap(),
        ];
        self.patterns.insert(ChallengeType::CloudflareJsChallenge, cf_js_regexes);

        // Cloudflare Turnstile patterns
        let cf_turnstile_regexes = vec![
            Regex::new(r"challenges\.cloudflare\.com/turnstile").unwrap(),
            Regex::new(r"cf-turnstile").unwrap(),
            Regex::new(r"turnstile\.render").unwrap(),
        ];
        self.patterns.insert(ChallengeType::CloudflareTurnstile, cf_turnstile_regexes);

        let cf_elements = vec![
            ".cf-challenge".to_string(),
            ".cf-browser-verification".to_string(),
            ".cf-turnstile".to_string(),
            r#"form[action*="/cdn-cgi/l/chk_captcha"]"#.to_string(),
        ];
        self.element_patterns.insert(ChallengeType::CloudflareJsChallenge, cf_elements.clone());
        self.element_patterns.insert(ChallengeType::CloudflareTurnstile, cf_elements);
    }

    /// Initialize reCAPTCHA patterns
    fn init_recaptcha_patterns(&mut self) {
        // reCAPTCHA v3 patterns (higher weight v3-specific indicators)
        let recaptcha_v3_regexes = vec![
            Regex::new(r"www\.google\.com/recaptcha/api\.js.*render=").unwrap(),
            Regex::new(r"grecaptcha\.execute").unwrap(),
            Regex::new(r"grecaptcha\.ready").unwrap(), // v3 specific
            Regex::new(r#"g-recaptcha".*?data-sitekey"#).unwrap(),
        ];
        self.patterns.insert(ChallengeType::GoogleRecaptchaV3, recaptcha_v3_regexes);

        // reCAPTCHA v2 patterns
        let recaptcha_v2_regexes = vec![
            Regex::new(r"www\.google\.com/recaptcha/api\.js").unwrap(),
            Regex::new(r"g-recaptcha").unwrap(),
            Regex::new(r"grecaptcha\.render").unwrap(),
        ];
        self.patterns.insert(ChallengeType::GoogleRecaptchaV2, recaptcha_v2_regexes);

        let recaptcha_elements = vec![
            ".g-recaptcha".to_string(),
            r#"div[data-sitekey]"#.to_string(),
            r#"script[src*="recaptcha/api.js"]"#.to_string(),
        ];
        self.element_patterns.insert(ChallengeType::GoogleRecaptchaV3, recaptcha_elements.clone());
        self.element_patterns.insert(ChallengeType::GoogleRecaptchaV2, recaptcha_elements);
    }

    /// Initialize generic bot detection patterns
    fn init_generic_patterns(&mut self) {
        let generic_regexes = vec![
            Regex::new(r"(?i)bot.{0,10}detect").unwrap(),
            Regex::new(r"(?i)verify.{0,10}human").unwrap(),
            Regex::new(r"(?i)access.{0,10}denied").unwrap(),
            Regex::new(r"(?i)suspicious.{0,10}activity").unwrap(),
            Regex::new(r"(?i)rate.{0,10}limit").unwrap(),
        ];
        self.patterns.insert(ChallengeType::Generic, generic_regexes);

        let generic_elements = vec![
            r#"div:contains("Bot detected")"#.to_string(),
            r#"div:contains("Verify you are human")"#.to_string(),
            r#"div:contains("Access denied")"#.to_string(),
        ];
        self.element_patterns.insert(ChallengeType::Generic, generic_elements);
    }

    /// Initialize other protection service patterns
    fn init_other_protection_patterns(&mut self) {
        // hCaptcha patterns
        let hcaptcha_regexes = vec![
            Regex::new(r"hcaptcha\.com").unwrap(),
            Regex::new(r"h-captcha").unwrap(),
        ];
        self.patterns.insert(ChallengeType::HCaptcha, hcaptcha_regexes);

        // DataDome patterns
        let datadome_regexes = vec![
            Regex::new(r"datadome").unwrap(),
            Regex::new(r"dd_cookie").unwrap(),
        ];
        self.patterns.insert(ChallengeType::DataDome, datadome_regexes);

        // PerimeterX patterns
        let perimeterx_regexes = vec![
            Regex::new(r"perimeterx").unwrap(),
            Regex::new(r"_px").unwrap(),
        ];
        self.patterns.insert(ChallengeType::PerimeterX, perimeterx_regexes);
    }
}

/// Detect the type of challenge present in HTML content
pub fn detect_challenge_type(patterns: &ChallengePatterns, html: &str, url: &str) -> ChallengeType {
    debug!("Detecting challenge type for URL: {}", url);
    
    let mut detections = HashMap::new();
    
    // Check each challenge type
    for (challenge_type, regex_patterns) in &patterns.patterns {
        let mut score = 0.0;
        
        // Check regex patterns
        for regex in regex_patterns {
            if regex.is_match(html) {
                score += 1.0;
                debug!("Pattern match for {:?}: {}", challenge_type, regex.as_str());
            }
        }
        
        // Check URL patterns if any
        if let Some(url_patterns) = patterns.url_patterns.get(challenge_type) {
            for regex in url_patterns {
                if regex.is_match(url) {
                    score += 0.5;
                    debug!("URL pattern match for {:?}: {}", challenge_type, regex.as_str());
                }
            }
        }
        
        // Add element-based detection
        if let Some(element_patterns) = patterns.element_patterns.get(challenge_type) {
            for pattern in element_patterns {
                if html.contains(&pattern.replace(r#"""#, "")) {
                    score += 0.5;
                }
            }
        }
        
        // Add JS-based detection
        if let Some(js_patterns) = patterns.js_patterns.get(challenge_type) {
            for pattern in js_patterns {
                if html.contains(pattern) {
                    score += 0.7;
                }
            }
        }
        
        if score > 0.0 {
            detections.insert(challenge_type.clone(), score);
        }
    }
    
    // Return the challenge type with the highest score
    if let Some((challenge_type, score)) = detections.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()) {
        info!("Detected challenge type: {:?} with score: {}", challenge_type, score);
        challenge_type.clone()
    } else {
        debug!("No specific challenge detected, returning Unknown");
        ChallengeType::Unknown
    }
}

/// Extract challenge-specific information from HTML
pub fn extract_challenge_info(challenge_type: &ChallengeType, html: &str, url: &str) -> ChallengeInfo {
    let mut parameters = HashMap::new();
    let mut metadata = HashMap::new();
    
    match challenge_type {
        ChallengeType::GoogleRecaptchaV3 | ChallengeType::GoogleRecaptchaV2 => {
            // Extract site key
            if let Some(site_key) = extract_recaptcha_site_key(html) {
                parameters.insert("site_key".to_string(), site_key);
            }
        },
        ChallengeType::CloudflareJsChallenge => {
            // Extract Cloudflare challenge parameters
            extract_cloudflare_params(html, &mut parameters);
        },
        ChallengeType::CloudflareTurnstile => {
            // Extract Turnstile site key
            if let Some(site_key) = extract_turnstile_site_key(html) {
                parameters.insert("site_key".to_string(), site_key);
            }
        },
        _ => {}
    }
    
    metadata.insert("url".to_string(), url.to_string());
    
    ChallengeInfo {
        challenge_type: challenge_type.clone(),
        confidence: 0.8, // Default confidence
        parameters,
        metadata,
    }
}

/// Extract reCAPTCHA site key from HTML
fn extract_recaptcha_site_key(html: &str) -> Option<String> {
    let site_key_regex = Regex::new(r#"data-sitekey=["']([^"']+)["']"#).unwrap();
    if let Some(captures) = site_key_regex.captures(html) {
        return Some(captures[1].to_string());
    }
    
    let render_regex = Regex::new(r#"render=([^&"']+)"#).unwrap();
    if let Some(captures) = render_regex.captures(html) {
        return Some(captures[1].to_string());
    }
    
    None
}

/// Extract Cloudflare challenge parameters
fn extract_cloudflare_params(html: &str, parameters: &mut HashMap<String, String>) {
    // Extract challenge parameters
    let s_regex = Regex::new(r#"name="s" value="([^"]+)""#).unwrap();
    if let Some(captures) = s_regex.captures(html) {
        parameters.insert("s".to_string(), captures[1].to_string());
    }
    
    let jschl_vc_regex = Regex::new(r#"name="jschl_vc" value="([^"]+)""#).unwrap();
    if let Some(captures) = jschl_vc_regex.captures(html) {
        parameters.insert("jschl_vc".to_string(), captures[1].to_string());
    }
    
    let pass_regex = Regex::new(r#"name="pass" value="([^"]+)""#).unwrap();
    if let Some(captures) = pass_regex.captures(html) {
        parameters.insert("pass".to_string(), captures[1].to_string());
    }
}

/// Extract Turnstile site key from HTML
fn extract_turnstile_site_key(html: &str) -> Option<String> {
    let site_key_regex = Regex::new(r#"data-sitekey=["']([^"']+)["']"#).unwrap();
    if let Some(captures) = site_key_regex.captures(html) {
        return Some(captures[1].to_string());
    }
    None
}