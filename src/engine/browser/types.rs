use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapedData {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub links: Vec<Link>,
    pub images: Vec<Image>,
    pub metadata: HashMap<String, String>,
    pub extracted_data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    pub url: String,
    pub text: String,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub src: String,
    pub alt: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormField {
    pub name: String,
    pub field_type: String,
    pub value: Option<String>,
    pub required: bool,
    pub placeholder: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Form {
    pub action: String,
    pub method: String,
    pub fields: Vec<FormField>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InteractionResponse {
    pub success: bool,
    pub message: String,
    pub redirect_url: Option<String>,
    pub new_content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BrowserStorage {
    pub local_storage: HashMap<String, String>,
    pub session_storage: HashMap<String, String>,
}

impl Default for BrowserStorage {
    fn default() -> Self {
        Self {
            local_storage: HashMap::new(),
            session_storage: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub cookies: HashMap<String, String>,
    pub auth_headers: HashMap<String, String>,
    pub csrf_tokens: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct NavigationHistory {
    pub entries: Vec<HistoryEntry>,
    pub current_index: usize,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct StealthConfig {
    pub random_delays: bool,
    pub random_user_agents: bool,
    pub stealth_headers: bool,
    pub canvas_fingerprinting_protection: bool,
    pub webgl_fingerprinting_protection: bool,
    pub audio_fingerprinting_protection: bool,
    pub font_fingerprinting_protection: bool,
    pub screen_resolution_spoofing: bool,
    pub timezone_spoofing: bool,
    pub language_spoofing: bool,
    pub webrtc_protection: bool,
    pub battery_api_protection: bool,
    pub gamepad_api_protection: bool,
    pub request_timing: RequestTiming,
}

#[derive(Debug, Clone)]
pub struct RequestTiming {
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for StealthConfig {
    fn default() -> Self {
        Self {
            random_delays: true,
            random_user_agents: true,
            stealth_headers: true,
            canvas_fingerprinting_protection: true,
            webgl_fingerprinting_protection: true,
            audio_fingerprinting_protection: true,
            font_fingerprinting_protection: true,
            screen_resolution_spoofing: true,
            timezone_spoofing: true,
            language_spoofing: true,
            webrtc_protection: true,
            battery_api_protection: true,
            gamepad_api_protection: true,
            request_timing: RequestTiming {
                min_delay_ms: 100,
                max_delay_ms: 2000,
            },
        }
    }
}