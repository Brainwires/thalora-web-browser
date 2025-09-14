use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use rand::Rng;
use regex::Regex;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use tracing::debug;

/// Generate a hash of HTML content for caching purposes
pub fn html_hash(html: &str) -> String {
    let mut hasher = DefaultHasher::new();
    html.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Generate human-like delays between actions
pub fn human_delay() -> Duration {
    let mut rng = rand::thread_rng();
    let millis = rng.gen_range(100..2000); // 100ms to 2s
    Duration::from_millis(millis)
}

/// Generate a random user agent string
pub fn random_user_agent() -> &'static str {
    let user_agents = [
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/121.0",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/121.0",
    ];
    
    let mut rng = rand::thread_rng();
    user_agents[rng.gen_range(0..user_agents.len())]
}

/// Extract domain from URL
pub fn extract_domain(url: &str) -> Result<String> {
    let parsed = url::Url::parse(url)
        .map_err(|e| anyhow!("Failed to parse URL {}: {}", url, e))?;
    
    parsed.host_str()
        .map(|host| host.to_string())
        .ok_or_else(|| anyhow!("No host found in URL: {}", url))
}

/// Generate a fake canvas fingerprint
pub fn generate_canvas_fingerprint() -> String {
    let mut rng = rand::thread_rng();
    let mut fingerprint = String::new();
    
    for _ in 0..32 {
        fingerprint.push_str(&format!("{:02x}", rng.r#gen::<u8>()));
    }
    
    fingerprint
}

/// Generate a fake WebGL fingerprint
pub fn generate_webgl_fingerprint() -> HashMap<String, String> {
    let mut fingerprint = HashMap::new();
    
    fingerprint.insert("vendor".to_string(), "Google Inc.".to_string());
    fingerprint.insert("renderer".to_string(), "ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 Direct3D11 vs_5_0 ps_5_0, D3D11-30.0.15.1179)".to_string());
    fingerprint.insert("version".to_string(), "WebGL 1.0 (OpenGL ES 2.0 Chromium)".to_string());
    fingerprint.insert("shading_language_version".to_string(), "WebGL GLSL ES 1.0 (OpenGL ES GLSL ES 1.0 Chromium)".to_string());
    
    fingerprint
}

/// Extract JavaScript code from HTML
pub fn extract_javascript(html: &str) -> Vec<String> {
    let mut scripts = Vec::new();
    
    // Find all script tags and extract their content
    let script_regex = regex::Regex::new(r"<script[^>]*>(.*?)</script>").unwrap();
    for captures in script_regex.captures_iter(html) {
        if let Some(script_content) = captures.get(1) {
            let content = script_content.as_str().trim();
            if !content.is_empty() {
                scripts.push(content.to_string());
            }
        }
    }
    
    // Also look for inline event handlers
    let inline_regex = regex::Regex::new(r#"on\w+\s*=\s*["']([^"']+)["']"#).unwrap();
    for captures in inline_regex.captures_iter(html) {
        if let Some(handler_content) = captures.get(1) {
            scripts.push(handler_content.as_str().to_string());
        }
    }
    
    scripts
}

/// Check if JavaScript code contains dangerous patterns
pub fn is_dangerous_javascript(js_code: &str) -> bool {
    let dangerous_patterns = [
        // File system access
        "require('fs')",
        "require(\"fs\")",
        "import fs from",
        "readFile",
        "writeFile",
        
        // Network access
        "XMLHttpRequest",
        "fetch(",
        "import(",
        "require(",
        
        // Process/system access
        "require('child_process')",
        "spawn(",
        "exec(",
        "eval(",
        "Function(",
        
        // Global pollution
        "globalThis",
        "global.",
        "process.",
        
        // Sensitive data access
        "document.cookie",
        "localStorage",
        "sessionStorage",
        "indexedDB",
        
        // Dynamic code execution
        "new Function",
        "setTimeout(",
        "setInterval(",
    ];
    
    for pattern in &dangerous_patterns {
        if js_code.contains(pattern) {
            debug!("Detected dangerous pattern in JavaScript: {}", pattern);
            return true;
        }
    }
    
    false
}

/// Sanitize JavaScript code by removing dangerous patterns
pub fn sanitize_javascript(js_code: &str) -> String {
    let mut sanitized = js_code.to_string();
    
    // Replace dangerous functions with safe alternatives
    let replacements = [
        ("eval(", "// eval("),
        ("Function(", "// Function("),
        ("setTimeout(", "// setTimeout("),
        ("setInterval(", "// setInterval("),
        ("XMLHttpRequest", "// XMLHttpRequest"),
        ("fetch(", "// fetch("),
        ("document.cookie", "''"),
        ("localStorage", "{}"),
        ("sessionStorage", "{}"),
    ];
    
    for (dangerous, safe) in &replacements {
        sanitized = sanitized.replace(dangerous, safe);
    }
    
    sanitized
}

/// Generate a timestamp in milliseconds
pub fn timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Generate a random string of specified length
pub fn random_string(length: usize) -> String {
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

/// Generate a random hex string of specified length
pub fn random_hex_string(length: usize) -> String {
    let chars: Vec<char> = "0123456789abcdef".chars().collect();
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

/// Encode data to base64
pub fn encode_base64(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

/// Decode base64 data
pub fn decode_base64(data: &str) -> Result<Vec<u8>> {
    general_purpose::STANDARD.decode(data)
        .map_err(|e| anyhow!("Failed to decode base64: {}", e))
}

/// Generate browser-like headers for requests
pub fn generate_browser_headers(url: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    
    headers.insert("User-Agent".to_string(), random_user_agent().to_string());
    headers.insert("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8".to_string());
    headers.insert("Accept-Language".to_string(), "en-US,en;q=0.9".to_string());
    headers.insert("Accept-Encoding".to_string(), "gzip, deflate, br".to_string());
    headers.insert("Connection".to_string(), "keep-alive".to_string());
    headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string());
    headers.insert("Sec-Fetch-Dest".to_string(), "document".to_string());
    headers.insert("Sec-Fetch-Mode".to_string(), "navigate".to_string());
    headers.insert("Sec-Fetch-Site".to_string(), "none".to_string());
    headers.insert("Cache-Control".to_string(), "max-age=0".to_string());
    
    // Add referer if not the initial request
    if !url.is_empty() {
        if let Ok(domain) = extract_domain(url) {
            headers.insert("Referer".to_string(), format!("https://{}/", domain));
        }
    }
    
    headers
}

/// Parse form data from HTML
pub fn parse_form_data(html: &str) -> Vec<HashMap<String, String>> {
    let mut forms = Vec::new();
    
    let form_regex = Regex::new(r"(?s)<form[^>]*>(.*?)</form>").unwrap();
    for form_match in form_regex.captures_iter(html) {
        let mut form_data = HashMap::new();
        
        let form_tag = form_match.get(0).unwrap().as_str();
        let form_content = form_match.get(1).unwrap().as_str();
        
        // Extract form attributes
        if let Some(action) = extract_attribute(form_tag, "action") {
            form_data.insert("action".to_string(), action);
        }
        if let Some(method) = extract_attribute(form_tag, "method") {
            form_data.insert("method".to_string(), method);
        }
        
        // Extract input fields
        let input_regex = Regex::new(r#"<input[^>]*>"#).unwrap();
        for input_match in input_regex.find_iter(form_content) {
            let input_tag = input_match.as_str();
            
            if let Some(name) = extract_attribute(input_tag, "name") {
                let value = extract_attribute(input_tag, "value").unwrap_or_default();
                form_data.insert(name, value);
            }
        }
        
        forms.push(form_data);
    }
    
    forms
}

/// Extract attribute value from HTML tag
fn extract_attribute(tag: &str, attribute: &str) -> Option<String> {
    let regex = Regex::new(&format!(r#"{}=["']([^"']+)["']"#, attribute)).unwrap();
    regex.captures(tag)
        .and_then(|captures| captures.get(1))
        .map(|m| m.as_str().to_string())
}

/// Simulate mouse movement with random jitter
pub fn simulate_mouse_movement() -> (i32, i32) {
    let mut rng = rand::thread_rng();
    (rng.gen_range(100..1800), rng.gen_range(100..900))
}

/// Generate a realistic typing speed delay
pub fn typing_delay() -> Duration {
    let mut rng = rand::thread_rng();
    let millis = rng.gen_range(50..200); // 50-200ms between keystrokes
    Duration::from_millis(millis)
}

/// Calculate simple checksum for challenge verification
pub fn calculate_checksum(data: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

/// Generate a fake timezone offset
pub fn generate_timezone_offset() -> i32 {
    // Common timezone offsets (in minutes)
    let offsets = [-480, -420, -360, -300, -240, -180, -120, -60, 0, 60, 120, 180, 240, 300, 360, 420, 480, 540, 600, 660, 720];
    let mut rng = rand::thread_rng();
    offsets[rng.gen_range(0..offsets.len())]
}

/// Generate a list of common fonts
pub fn generate_font_list() -> Vec<String> {
    vec![
        "Arial".to_string(),
        "Arial Black".to_string(),
        "Arial Narrow".to_string(),
        "Calibri".to_string(),
        "Cambria".to_string(),
        "Comic Sans MS".to_string(),
        "Courier New".to_string(),
        "Georgia".to_string(),
        "Helvetica".to_string(),
        "Impact".to_string(),
        "Lucida Console".to_string(),
        "Lucida Sans Unicode".to_string(),
        "Microsoft Sans Serif".to_string(),
        "Palatino".to_string(),
        "Times".to_string(),
        "Times New Roman".to_string(),
        "Trebuchet MS".to_string(),
        "Verdana".to_string(),
    ]
}

/// Validate URL format
pub fn is_valid_url(url: &str) -> bool {
    url::Url::parse(url).is_ok() && (url.starts_with("http://") || url.starts_with("https://"))
}