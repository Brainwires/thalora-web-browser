use rand::prelude::*;
use rquest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE, ACCEPT_ENCODING, CONNECTION, UPGRADE_INSECURE_REQUESTS};
use crate::engine::browser::types::StealthConfig;
use thalora_constants::USER_AGENT as SHARED_USER_AGENT;

pub struct StealthManager {
    config: StealthConfig,
}

impl StealthManager {
    pub fn new(config: StealthConfig) -> Self {
        Self { config }
    }

    pub fn get_user_agent(&self) -> &'static str {
        // Always use shared USER_AGENT constant - single source of truth!
        // Random user agents cause fingerprint inconsistency and make us look like a bot
        SHARED_USER_AGENT
    }

    pub fn create_stealth_headers(&self, url: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();

        if self.config.stealth_headers {
            // User-Agent - use shared constant
            headers.insert(USER_AGENT, HeaderValue::from_static(self.get_user_agent()));

            // Standard browser headers
            headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"));
            headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
            headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br"));
            headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
            headers.insert(UPGRADE_INSECURE_REQUESTS, HeaderValue::from_static("1"));

            // Sec-Fetch headers (modern browsers) - inserted by name to support older reqwest
            headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
            headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
            headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
            headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));

            // Client hints - must match thalora_constants (Chrome 131, Windows)
            headers.insert("sec-ch-ua", HeaderValue::from_static(thalora_constants::SEC_CH_UA));
            headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
            headers.insert("sec-ch-ua-platform", HeaderValue::from_static(r#""Windows""#));

            // DNT header (Do Not Track)
            headers.insert("dnt", HeaderValue::from_static("1"));

            // Cache control
            headers.insert("cache-control", HeaderValue::from_static("max-age=0"));
        }

        headers
    }

    pub async fn apply_random_delay(&self) {
        if self.config.random_delays {
            let mut rng = thread_rng();
            let delay_ms = rng.gen_range(
                self.config.request_timing.min_delay_ms..=self.config.request_timing.max_delay_ms
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        }
    }

    pub fn get_stealth_viewport_size(&self) -> (u32, u32) {
        if !self.config.screen_resolution_spoofing {
            return (1920, 1080); // Default
        }

        let resolutions = [
            (1920, 1080),
            (1366, 768),
            (1536, 864),
            (1440, 900),
            (1280, 720),
            (1600, 900),
        ];

        let mut rng = thread_rng();
        resolutions[rng.gen_range(0..resolutions.len())]
    }

    pub fn get_stealth_timezone(&self) -> String {
        if !self.config.timezone_spoofing {
            return "America/New_York".to_string();
        }

        let timezones = [
            "America/New_York",
            "America/Los_Angeles",
            "Europe/London",
            "Europe/Berlin",
            "Asia/Tokyo",
            "America/Chicago",
        ];

        let mut rng = thread_rng();
        timezones[rng.gen_range(0..timezones.len())].to_string()
    }

    pub fn get_stealth_language(&self) -> String {
        if !self.config.language_spoofing {
            return "en-US".to_string();
        }

        let languages = [
            "en-US",
            "en-GB",
            "de-DE",
            "fr-FR",
            "es-ES",
            "ja-JP",
        ];

        let mut rng = thread_rng();
        languages[rng.gen_range(0..languages.len())].to_string()
    }
}