use anyhow::{anyhow, Result};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use url::Url;
use reqwest::header::CONTENT_TYPE;
use reqwest::cookie::CookieStore;
use rand::{thread_rng, Rng};

use crate::engine::renderer::RustRenderer;
// use crate::{ChallengeSolver, ChallengeType};

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
    pub submit_buttons: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InteractionResponse {
    pub url: String,
    pub status_code: u16,
    pub content: String,
    pub cookies: HashMap<String, String>,
    pub scraped_data: Option<ScrapedData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserStorage {
    pub local_storage: HashMap<String, String>,
    pub session_storage: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthContext {
    pub bearer_token: Option<String>,
    pub csrf_token: Option<String>,
    pub custom_headers: HashMap<String, String>,
    pub storage: BrowserStorage,
}

pub struct HeadlessWebBrowser {
    client: reqwest::Client,
    renderer: RustRenderer,
    cookie_jar: Arc<reqwest::cookie::Jar>,
    storage: Arc<Mutex<BrowserStorage>>,
    auth_context: Arc<Mutex<AuthContext>>,
    pub stealth_config: StealthConfig,
    request_history: Arc<Mutex<Vec<RequestTiming>>>,
    // challenge_solver: Arc<Mutex<ChallengeSolver>>,
}

#[derive(Debug, Clone)]
pub struct StealthConfig {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub device_pixel_ratio: f32,
    pub languages: Vec<String>,
    pub timezone: String,
    pub webgl_vendor: String,
    pub webgl_renderer: String,
    pub platform: String,
    pub hardware_concurrency: u8,
    pub memory: u32,
    pub screen_width: u32,
    pub screen_height: u32,
    pub color_depth: u8,
    pub random_delays: bool,
    pub mouse_movements: bool,
    pub keyboard_patterns: bool,
}

#[derive(Debug, Clone)]
pub struct RequestTiming {
    pub url: String,
    pub timestamp: Instant,
    pub duration: Duration,
    pub user_agent_used: String,
}

impl Default for StealthConfig {
    fn default() -> Self {
        Self {
            viewport_width: 1920,
            viewport_height: 1080,
            device_pixel_ratio: 1.0,
            languages: vec!["en-US".to_string(), "en".to_string()],
            timezone: "America/New_York".to_string(),
            webgl_vendor: "Google Inc. (Apple)".to_string(),
            webgl_renderer: "ANGLE (Apple, Apple M1 Pro, OpenGL 4.1)".to_string(),
            platform: "MacIntel".to_string(),
            hardware_concurrency: 8,
            memory: 8,
            screen_width: 1920,
            screen_height: 1080,
            color_depth: 24,
            random_delays: true,
            mouse_movements: false, // Disabled for headless
            keyboard_patterns: false, // Disabled for headless
        }
    }
}

impl HeadlessWebBrowser {
    pub fn new() -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        
        // Chrome-like headers to appear as a real browser
        headers.insert("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".parse().unwrap());
        headers.insert("Accept-Language", "en-US,en;q=0.9".parse().unwrap());
        headers.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
        headers.insert("Cache-Control", "no-cache".parse().unwrap());
        headers.insert("Pragma", "no-cache".parse().unwrap());
        headers.insert("Sec-Ch-Ua", "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"".parse().unwrap());
        headers.insert("Sec-Ch-Ua-Mobile", "?0".parse().unwrap());
        headers.insert("Sec-Ch-Ua-Platform", "\"macOS\"".parse().unwrap());
        headers.insert("Sec-Fetch-Dest", "document".parse().unwrap());
        headers.insert("Sec-Fetch-Mode", "navigate".parse().unwrap());
        headers.insert("Sec-Fetch-Site", "none".parse().unwrap());
        headers.insert("Sec-Fetch-User", "?1".parse().unwrap());
        headers.insert("Upgrade-Insecure-Requests", "1".parse().unwrap());

        let cookie_jar = Arc::new(reqwest::cookie::Jar::default());
        
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .default_headers(headers)
            .cookie_provider(Arc::clone(&cookie_jar))
            .tls_sni(true)
            .use_rustls_tls()
            .https_only(false)
            .redirect(reqwest::redirect::Policy::limited(10))
            .timeout(std::time::Duration::from_secs(30))
            // Remove http2_prior_knowledge() to allow ALPN negotiation
            .http1_title_case_headers()
            .http2_adaptive_window(true)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .pool_max_idle_per_host(32)
            .build()
            .unwrap();

        let storage = Arc::new(Mutex::new(BrowserStorage {
            local_storage: HashMap::new(),
            session_storage: HashMap::new(),
        }));

        let auth_context = Arc::new(Mutex::new(AuthContext {
            bearer_token: None,
            csrf_token: None,
            custom_headers: HashMap::new(),
            storage: BrowserStorage {
                local_storage: HashMap::new(),
                session_storage: HashMap::new(),
            },
        }));

        Self {
            client,
            renderer: RustRenderer::new(),
            cookie_jar,
            storage,
            auth_context,
            stealth_config: StealthConfig::default(),
            request_history: Arc::new(Mutex::new(Vec::new())),
            // challenge_solver: Arc::new(Mutex::new(ChallengeSolver::new())),
        }
    }

    // Anti-detection methods for modern browser support
    async fn add_human_timing_delay(&self) -> Result<()> {
        if self.stealth_config.random_delays {
            let mut rng = thread_rng();
            // Simulate human-like delays between requests (100ms to 2s)
            let delay_ms = rng.gen_range(100..2000);
            sleep(Duration::from_millis(delay_ms)).await;
        }
        Ok(())
    }

    fn generate_realistic_user_agents(&self) -> Vec<String> {
        vec![
            // Chrome variants (most common)
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".to_string(),
            // Firefox variants
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:131.0) Gecko/20100101 Firefox/131.0".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:131.0) Gecko/20100101 Firefox/131.0".to_string(),
            // Safari variants
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1.2 Safari/605.1.15".to_string(),
            // Edge variants
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0".to_string(),
        ]
    }

    pub fn get_random_user_agent(&self) -> String {
        let user_agents = self.generate_realistic_user_agents();
        let mut rng = thread_rng();
        user_agents[rng.gen_range(0..user_agents.len())].clone()
    }

    pub fn create_stealth_headers(&self, url: &str) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        let mut rng = thread_rng();

        // Randomize header order and values slightly for more realism
        headers.insert("Accept", 
            if rng.gen_bool(0.8) {
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8".parse().unwrap()
            } else {
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".parse().unwrap()
            }
        );

        // Vary Accept-Language slightly
        let languages = vec![
            "en-US,en;q=0.9",
            "en-US,en;q=0.8,fr;q=0.6",
            "en-US,en;q=0.9,es;q=0.8",
            "en-US,en;q=0.7"
        ];
        headers.insert("Accept-Language", languages[rng.gen_range(0..languages.len())].parse().unwrap());

        headers.insert("Accept-Encoding", "gzip, deflate, br, zstd".parse().unwrap());
        headers.insert("Cache-Control", "max-age=0".parse().unwrap());
        
        // Modern Chrome security headers with slight randomization
        let chrome_versions = vec!["131", "130", "129"];
        let chrome_version = chrome_versions[rng.gen_range(0..chrome_versions.len())];
        
        headers.insert("Sec-Ch-Ua", 
            format!("\"Google Chrome\";v=\"{}\", \"Chromium\";v=\"{}\", \"Not_A Brand\";v=\"24\"", 
                chrome_version, chrome_version).parse().unwrap()
        );
        headers.insert("Sec-Ch-Ua-Mobile", "?0".parse().unwrap());
        headers.insert("Sec-Ch-Ua-Platform", "\"macOS\"".parse().unwrap());
        
        // Dynamic Sec-Fetch headers based on request context
        if url.contains("api") || url.contains("json") {
            headers.insert("Sec-Fetch-Dest", "empty".parse().unwrap());
            headers.insert("Sec-Fetch-Mode", "cors".parse().unwrap());
        } else {
            headers.insert("Sec-Fetch-Dest", "document".parse().unwrap());
            headers.insert("Sec-Fetch-Mode", "navigate".parse().unwrap());
        }
        
        headers.insert("Sec-Fetch-Site", "cross-site".parse().unwrap());
        headers.insert("Sec-Fetch-User", "?1".parse().unwrap());
        headers.insert("Upgrade-Insecure-Requests", "1".parse().unwrap());
        
        // Add DNT header randomly (some users have it, some don't)
        if rng.gen_bool(0.3) {
            headers.insert("DNT", "1".parse().unwrap());
        }

        headers
    }

    pub fn simulate_canvas_fingerprint(&self) -> String {
        // Simulate a realistic canvas fingerprint that varies slightly
        let mut rng = thread_rng();
        let base_fingerprint = "a1b2c3d4e5f6g7h8i9j0";
        let variance: u32 = rng.gen_range(1000..9999);
        format!("{}{}", base_fingerprint, variance)
    }

    pub fn simulate_webgl_fingerprint(&self) -> (String, String) {
        // Return realistic WebGL vendor and renderer info
        let vendors = vec![
            ("Google Inc. (Apple)", "ANGLE (Apple, Apple M1 Pro, OpenGL 4.1)"),
            ("Google Inc. (Intel)", "ANGLE (Intel, Intel(R) Iris(TM) Xe Graphics, OpenGL 4.1)"),
            ("Google Inc. (NVIDIA)", "ANGLE (NVIDIA, NVIDIA GeForce RTX 3080, OpenGL 4.1)"),
        ];
        let mut rng = thread_rng();
        let (vendor, renderer) = vendors[rng.gen_range(0..vendors.len())];
        (vendor.to_string(), renderer.to_string())
    }

    async fn track_request_timing(&self, url: &str, duration: Duration) -> Result<()> {
        let mut history = self.request_history.lock().unwrap();
        history.push(RequestTiming {
            url: url.to_string(),
            timestamp: Instant::now(),
            duration,
            user_agent_used: self.get_random_user_agent(),
        });

        // Keep only last 100 requests to prevent memory bloat
        if history.len() > 100 {
            history.drain(0..50);
        }
        Ok(())
    }

    pub fn detect_automation_evasion_needed(&self, html: &str) -> bool {
        // Detect common automation detection patterns
        let detection_patterns = vec![
            "webdriver",
            "selenium",
            "navigator.webdriver",
            "window.chrome",
            "__nightmare",
            "_phantomjs",
            "callPhantomjs",
            "_selenium",
            "webdriver-evaluate",
            "webdriverCommand",
            "bot-detected",
            "automation-detected",
            "please enable javascript",
            "captcha",
            "recaptcha",
            "hcaptcha",
            "cloudflare",
            "challenge-platform"
        ];

        let html_lower = html.to_lowercase();
        detection_patterns.iter().any(|pattern| html_lower.contains(pattern))
    }

    pub async fn scrape(
        &mut self,
        url: &str,
        wait_for_js: bool,
        selector: Option<&str>,
        extract_links: bool,
        extract_images: bool,
    ) -> Result<ScrapedData> {
        let parsed_url = Url::parse(url)?;
        let start_time = Instant::now();
        
        // Add human-like delays to avoid detection
        self.add_human_timing_delay().await?;
        
        // Use stealth headers and random user agent for enhanced evasion
        let stealth_headers = self.create_stealth_headers(url);
        let random_user_agent = self.get_random_user_agent();
        
        let response = self.client.get(url)
            .headers(stealth_headers)
            .header("User-Agent", random_user_agent.clone())
            .send()
            .await?;
        
        // Check if response is successful
        if !response.status().is_success() {
            return Err(anyhow!("HTTP request failed with status: {}", response.status()));
        }
        
        // Get the response content with proper encoding handling
        let html_content = response.text().await?;

        let processed_html = if wait_for_js || self.is_challenge_page(&html_content) {
            // Handle JavaScript challenges automatically
            self.handle_challenge_page(&html_content, url).await?
        } else {
            html_content
        };

        let document = Html::parse_document(&processed_html);
        
        let title = document
            .select(&Selector::parse("title").unwrap())
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string());

        let content = if let Some(sel) = selector {
            let selector = Selector::parse(sel)
                .map_err(|e| anyhow!("Invalid CSS selector: {}", e))?;
            document
                .select(&selector)
                .map(|el| el.text().collect::<Vec<_>>().join(" "))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            document
                .select(&Selector::parse("body").unwrap())
                .next()
                .map(|el| el.text().collect::<Vec<_>>().join(" "))
                .unwrap_or_default()
        };

        let links = if extract_links {
            self.extract_links(&document, &parsed_url)?
        } else {
            Vec::new()
        };

        let images = if extract_images {
            self.extract_images(&document, &parsed_url)?
        } else {
            Vec::new()
        };

        let metadata = self.extract_metadata(&document)?;

        // Track request timing for behavioral analysis
        let total_duration = start_time.elapsed();
        self.track_request_timing(url, total_duration).await?;

        Ok(ScrapedData {
            url: url.to_string(),
            title,
            content: content.trim().to_string(),
            links,
            images,
            metadata,
            extracted_data: None,
        })
    }

    pub async fn extract_data(
        &self,
        html: &str,
        selectors: &Map<String, Value>,
    ) -> Result<Value> {
        let document = Html::parse_document(html);
        let mut result = serde_json::Map::new();

        for (field_name, selector_value) in selectors {
            let selector_str = selector_value
                .as_str()
                .ok_or_else(|| anyhow!("Selector for field '{}' must be a string", field_name))?;

            let selector = Selector::parse(selector_str)
                .map_err(|e| anyhow!("Invalid CSS selector for field '{}': {}", field_name, e))?;

            let values: Vec<String> = document
                .select(&selector)
                .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();

            let field_value = match values.len() {
                0 => Value::Null,
                1 => Value::String(values[0].clone()),
                _ => Value::Array(values.into_iter().map(Value::String).collect()),
            };

            result.insert(field_name.clone(), field_value);
        }

        Ok(Value::Object(result))
    }

    fn extract_links(&self, document: &Html, base_url: &Url) -> Result<Vec<Link>> {
        let link_selector = Selector::parse("a[href]").unwrap();
        let mut links = Vec::new();

        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(absolute_url) = base_url.join(href) {
                    let text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    let title = element.value().attr("title").map(|s| s.to_string());

                    links.push(Link {
                        url: absolute_url.to_string(),
                        text,
                        title,
                    });
                }
            }
        }

        Ok(links)
    }

    fn extract_images(&self, document: &Html, base_url: &Url) -> Result<Vec<Image>> {
        let img_selector = Selector::parse("img[src]").unwrap();
        let mut images = Vec::new();

        for element in document.select(&img_selector) {
            if let Some(src) = element.value().attr("src") {
                if let Ok(absolute_url) = base_url.join(src) {
                    let alt = element.value().attr("alt").map(|s| s.to_string());
                    let title = element.value().attr("title").map(|s| s.to_string());

                    images.push(Image {
                        src: absolute_url.to_string(),
                        alt,
                        title,
                    });
                }
            }
        }

        Ok(images)
    }

    fn extract_metadata(&self, document: &Html) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();

        let meta_selector = Selector::parse("meta").unwrap();
        for element in document.select(&meta_selector) {
            let attrs = element.value();
            
            if let Some(name) = attrs.attr("name") {
                if let Some(content) = attrs.attr("content") {
                    metadata.insert(name.to_string(), content.to_string());
                }
            }
            
            if let Some(property) = attrs.attr("property") {
                if let Some(content) = attrs.attr("content") {
                    metadata.insert(property.to_string(), content.to_string());
                }
            }
        }

        let description_selector = Selector::parse("meta[name='description']").unwrap();
        if let Some(desc) = document.select(&description_selector).next() {
            if let Some(content) = desc.value().attr("content") {
                metadata.insert("description".to_string(), content.to_string());
            }
        }

        let keywords_selector = Selector::parse("meta[name='keywords']").unwrap();
        if let Some(keywords) = document.select(&keywords_selector).next() {
            if let Some(content) = keywords.value().attr("content") {
                metadata.insert("keywords".to_string(), content.to_string());
            }
        }

        Ok(metadata)
    }

    pub fn extract_forms(&self, html: &str, base_url: &Url) -> Result<Vec<Form>> {
        let document = Html::parse_document(html);
        let form_selector = Selector::parse("form").unwrap();
        let mut forms = Vec::new();

        for form_element in document.select(&form_selector) {
            let action = form_element
                .value()
                .attr("action")
                .unwrap_or("")
                .to_string();
            
            let absolute_action = if action.is_empty() {
                base_url.to_string()
            } else {
                base_url.join(&action).map(|u| u.to_string()).unwrap_or(action)
            };

            let method = form_element
                .value()
                .attr("method")
                .unwrap_or("get")
                .to_lowercase();

            let mut fields = Vec::new();
            let mut submit_buttons = Vec::new();

            // Extract input fields
            let input_selector = Selector::parse("input").unwrap();
            for input in form_element.select(&input_selector) {
                let input_type = input.value().attr("type").unwrap_or("text").to_lowercase();
                let name = input.value().attr("name").unwrap_or("").to_string();
                
                if input_type == "submit" || input_type == "button" {
                    let value = input.value().attr("value").unwrap_or("Submit").to_string();
                    submit_buttons.push(value);
                } else if !name.is_empty() {
                    fields.push(FormField {
                        name,
                        field_type: input_type,
                        value: input.value().attr("value").map(|s| s.to_string()),
                        required: input.value().attr("required").is_some(),
                        placeholder: input.value().attr("placeholder").map(|s| s.to_string()),
                    });
                }
            }

            // Extract textarea fields
            let textarea_selector = Selector::parse("textarea").unwrap();
            for textarea in form_element.select(&textarea_selector) {
                if let Some(name) = textarea.value().attr("name") {
                    fields.push(FormField {
                        name: name.to_string(),
                        field_type: "textarea".to_string(),
                        value: Some(textarea.text().collect::<String>()),
                        required: textarea.value().attr("required").is_some(),
                        placeholder: textarea.value().attr("placeholder").map(|s| s.to_string()),
                    });
                }
            }

            // Extract select fields
            let select_selector = Selector::parse("select").unwrap();
            for select in form_element.select(&select_selector) {
                if let Some(name) = select.value().attr("name") {
                    let option_selector = Selector::parse("option").unwrap();
                    let selected_value = select
                        .select(&option_selector)
                        .find(|opt| opt.value().attr("selected").is_some())
                        .and_then(|opt| opt.value().attr("value"))
                        .map(|s| s.to_string());

                    fields.push(FormField {
                        name: name.to_string(),
                        field_type: "select".to_string(),
                        value: selected_value,
                        required: select.value().attr("required").is_some(),
                        placeholder: None,
                    });
                }
            }

            // Extract button elements (for <button> tags)
            let button_selector = Selector::parse("button").unwrap();
            for button in form_element.select(&button_selector) {
                let button_type = button.value().attr("type").unwrap_or("submit").to_lowercase();
                if button_type == "submit" || button_type == "button" {
                    let value = if let Some(attr_value) = button.value().attr("value") {
                        attr_value.to_string()
                    } else {
                        button.text().collect::<String>().trim().to_string()
                    };
                    submit_buttons.push(value);
                }
            }

            forms.push(Form {
                action: absolute_action,
                method,
                fields,
                submit_buttons,
            });
        }

        Ok(forms)
    }

    pub async fn submit_form(
        &mut self,
        form: &Form,
        form_data: HashMap<String, String>,
        wait_for_js: bool,
    ) -> Result<InteractionResponse> {
        let url = Url::parse(&form.action)?;
        
        let response = if form.method.to_lowercase() == "post" {
            // Create form-encoded data
            let mut form_params = Vec::new();
            for (key, value) in &form_data {
                form_params.push((key.as_str(), value.as_str()));
            }

            self.client
                .post(&form.action)
                .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .form(&form_params)
                .send()
                .await?
        } else {
            // GET request with query parameters
            let mut url_with_params = url.clone();
            {
                let mut query_pairs = url_with_params.query_pairs_mut();
                for (key, value) in &form_data {
                    query_pairs.append_pair(key, value);
                }
            }

            self.client
                .get(url_with_params)
                .send()
                .await?
        };

        let status_code = response.status().as_u16();
        let final_url = response.url().to_string();
        let html_content = response.text().await?;

        // Process with JavaScript if requested
        let processed_html = if wait_for_js {
            self.renderer.render_with_js(&html_content, &final_url).await?
        } else {
            html_content.clone()
        };

        // Extract cookies
        let mut cookies = HashMap::new();
        if let Ok(cookie_url) = Url::parse(&final_url) {
            if let Some(cookie_header) = self.cookie_jar.cookies(&cookie_url) {
                if let Ok(cookie_str) = cookie_header.to_str() {
                    for cookie in cookie_str.split(';') {
                        if let Some((key, value)) = cookie.trim().split_once('=') {
                            cookies.insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
        }

        // Generate scraped data
        let scraped_data = if processed_html != html_content || !processed_html.is_empty() {
            let document = Html::parse_document(&processed_html);
            let parsed_url = Url::parse(&final_url)?;
            
            let title = document
                .select(&Selector::parse("title").unwrap())
                .next()
                .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string());

            let content = document
                .select(&Selector::parse("body").unwrap())
                .next()
                .map(|el| el.text().collect::<Vec<_>>().join(" "))
                .unwrap_or_default();

            let links = self.extract_links(&document, &parsed_url)?;
            let images = self.extract_images(&document, &parsed_url)?;
            let metadata = self.extract_metadata(&document)?;

            Some(ScrapedData {
                url: final_url.clone(),
                title,
                content: content.trim().to_string(),
                links,
                images,
                metadata,
                extracted_data: None,
            })
        } else {
            None
        };

        Ok(InteractionResponse {
            url: final_url,
            status_code,
            content: processed_html,
            cookies,
            scraped_data,
        })
    }

    pub async fn click_link(&mut self, base_url: &str, link_selector: &str, wait_for_js: bool) -> Result<InteractionResponse> {
        // First scrape the page to find the link
        let scraped = self.scrape(base_url, false, None, true, false).await?;
        let document = Html::parse_document(&scraped.content);
        
        let selector = Selector::parse(link_selector)
            .map_err(|e| anyhow!("Invalid CSS selector: {}", e))?;
        
        let link_element = document
            .select(&selector)
            .next()
            .ok_or_else(|| anyhow!("Link not found with selector: {}", link_selector))?;

        let href = link_element
            .value()
            .attr("href")
            .ok_or_else(|| anyhow!("Link has no href attribute"))?;

        let base = Url::parse(base_url)?;
        let target_url = base.join(href)?;

        // Navigate to the link
        let response = self.client.get(target_url.as_str()).send().await?;
        let status_code = response.status().as_u16();
        let final_url = response.url().to_string();
        let html_content = response.text().await?;

        let processed_html = if wait_for_js {
            self.renderer.render_with_js(&html_content, &final_url).await?
        } else {
            html_content.clone()
        };

        // Extract cookies
        let mut cookies = HashMap::new();
        if let Ok(cookie_url) = Url::parse(&final_url) {
            if let Some(cookie_header) = self.cookie_jar.cookies(&cookie_url) {
                if let Ok(cookie_str) = cookie_header.to_str() {
                    for cookie in cookie_str.split(';') {
                        if let Some((key, value)) = cookie.trim().split_once('=') {
                            cookies.insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
        }

        Ok(InteractionResponse {
            url: final_url,
            status_code,
            content: processed_html,
            cookies,
            scraped_data: None,
        })
    }

    pub fn get_cookies(&self, url: &str) -> Result<HashMap<String, String>> {
        let mut cookies = HashMap::new();
        if let Ok(parsed_url) = Url::parse(url) {
            if let Some(cookie_header) = self.cookie_jar.cookies(&parsed_url) {
                if let Ok(cookie_str) = cookie_header.to_str() {
                    for cookie in cookie_str.split(';') {
                        if let Some((key, value)) = cookie.trim().split_once('=') {
                            cookies.insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
        }
        Ok(cookies)
    }

    // localStorage/sessionStorage methods
    pub fn set_local_storage(&self, key: &str, value: &str) -> Result<()> {
        let mut storage = self.storage.lock().map_err(|e| anyhow!("Storage lock error: {}", e))?;
        storage.local_storage.insert(key.to_string(), value.to_string());
        
        // Also update auth context storage
        let mut auth = self.auth_context.lock().map_err(|e| anyhow!("Auth context lock error: {}", e))?;
        auth.storage.local_storage.insert(key.to_string(), value.to_string());
        
        Ok(())
    }

    pub fn get_local_storage(&self, key: &str) -> Result<Option<String>> {
        let storage = self.storage.lock().map_err(|e| anyhow!("Storage lock error: {}", e))?;
        Ok(storage.local_storage.get(key).cloned())
    }

    pub fn set_session_storage(&self, key: &str, value: &str) -> Result<()> {
        let mut storage = self.storage.lock().map_err(|e| anyhow!("Storage lock error: {}", e))?;
        storage.session_storage.insert(key.to_string(), value.to_string());
        
        // Also update auth context storage
        let mut auth = self.auth_context.lock().map_err(|e| anyhow!("Auth context lock error: {}", e))?;
        auth.storage.session_storage.insert(key.to_string(), value.to_string());
        
        Ok(())
    }

    pub fn get_session_storage(&self, key: &str) -> Result<Option<String>> {
        let storage = self.storage.lock().map_err(|e| anyhow!("Storage lock error: {}", e))?;
        Ok(storage.session_storage.get(key).cloned())
    }

    pub fn clear_session_storage(&self) -> Result<()> {
        let mut storage = self.storage.lock().map_err(|e| anyhow!("Storage lock error: {}", e))?;
        storage.session_storage.clear();
        
        let mut auth = self.auth_context.lock().map_err(|e| anyhow!("Auth context lock error: {}", e))?;
        auth.storage.session_storage.clear();
        
        Ok(())
    }

    // Authentication methods
    pub fn set_bearer_token(&self, token: &str) -> Result<()> {
        let mut auth = self.auth_context.lock().map_err(|e| anyhow!("Auth context lock error: {}", e))?;
        auth.bearer_token = Some(token.to_string());
        Ok(())
    }

    pub fn get_bearer_token(&self) -> Result<Option<String>> {
        let auth = self.auth_context.lock().map_err(|e| anyhow!("Auth context lock error: {}", e))?;
        Ok(auth.bearer_token.clone())
    }

    pub fn set_csrf_token(&self, token: &str) -> Result<()> {
        let mut auth = self.auth_context.lock().map_err(|e| anyhow!("Auth context lock error: {}", e))?;
        auth.csrf_token = Some(token.to_string());
        Ok(())
    }

    pub fn get_csrf_token(&self) -> Result<Option<String>> {
        let auth = self.auth_context.lock().map_err(|e| anyhow!("Auth context lock error: {}", e))?;
        Ok(auth.csrf_token.clone())
    }

    pub fn set_custom_header(&self, name: &str, value: &str) -> Result<()> {
        let mut auth = self.auth_context.lock().map_err(|e| anyhow!("Auth context lock error: {}", e))?;
        auth.custom_headers.insert(name.to_string(), value.to_string());
        Ok(())
    }

    pub fn get_custom_headers(&self) -> Result<HashMap<String, String>> {
        let auth = self.auth_context.lock().map_err(|e| anyhow!("Auth context lock error: {}", e))?;
        Ok(auth.custom_headers.clone())
    }

    pub fn extract_csrf_token(&self, html: &str) -> Result<Option<String>> {
        let document = Html::parse_document(html);
        
        // Common CSRF token patterns
        let selectors = [
            "meta[name='csrf-token']",
            "meta[name='_token']",
            "input[name='_token']",
            "input[name='csrf_token']",
            "input[name='authenticity_token']",
        ];

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    if let Some(content) = element.value().attr("content").or_else(|| element.value().attr("value")) {
                        return Ok(Some(content.to_string()));
                    }
                }
            }
        }

        Ok(None)
    }

    pub async fn submit_json(&mut self, url: &str, json_data: &Value) -> Result<InteractionResponse> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::CONTENT_TYPE, "application/json".parse().unwrap());
        
        // Add auth headers
        if let Ok(Some(token)) = self.get_bearer_token() {
            headers.insert(reqwest::header::AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
        }

        // Add custom headers
        if let Ok(custom_headers) = self.get_custom_headers() {
            for (name, value) in custom_headers {
                if let (Ok(header_name), Ok(header_value)) = (
                    name.parse::<reqwest::header::HeaderName>(), 
                    value.parse::<reqwest::header::HeaderValue>()
                ) {
                    headers.insert(header_name, header_value);
                }
            }
        }

        let response = self.client
            .post(url)
            .headers(headers)
            .json(json_data)
            .send()
            .await?;

        let status_code = response.status().as_u16();
        let final_url = response.url().to_string();
        let content = response.text().await?;

        // Extract and store any new tokens
        if let Ok(Some(csrf_token)) = self.extract_csrf_token(&content) {
            let _ = self.set_csrf_token(&csrf_token);
        }

        // Extract cookies
        let mut cookies = HashMap::new();
        if let Ok(cookie_url) = Url::parse(&final_url) {
            if let Some(cookie_header) = self.cookie_jar.cookies(&cookie_url) {
                if let Ok(cookie_str) = cookie_header.to_str() {
                    for cookie in cookie_str.split(';') {
                        if let Some((key, value)) = cookie.trim().split_once('=') {
                            cookies.insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
        }

        Ok(InteractionResponse {
            url: final_url,
            status_code,
            content,
            cookies,
            scraped_data: None,
        })
    }

    pub fn get_storage_state(&self) -> Result<BrowserStorage> {
        let storage = self.storage.lock().map_err(|e| anyhow!("Storage lock error: {}", e))?;
        Ok(storage.clone())
    }

    pub fn restore_storage_state(&self, storage_state: BrowserStorage) -> Result<()> {
        let mut storage = self.storage.lock().map_err(|e| anyhow!("Storage lock error: {}", e))?;
        *storage = storage_state.clone();
        
        let mut auth = self.auth_context.lock().map_err(|e| anyhow!("Auth context lock error: {}", e))?;
        auth.storage = storage_state;
        
        Ok(())
    }

    fn is_challenge_page(&self, html: &str) -> bool {
        let challenge_indicators = [
            "httpservice/retry/enablejs",
            "Please click here if you are not redirected",
            "google.tick",
            "trustedTypes",
            "createPolicy",
            "sctm&&google.tick",
        ];

        let html_lower = html.to_lowercase();
        for indicator in &challenge_indicators {
            if html_lower.contains(&indicator.to_lowercase()) {
                return true;
            }
        }

        false
    }

    async fn handle_challenge_page(&mut self, html: &str, url: &str) -> Result<String> {
        tracing::info!("🧩 Detected JavaScript challenge page, attempting advanced solving...");

        // // Use advanced challenge solver - temporarily disabled
        // let challenge_result = {
        //     let mut solver = self.challenge_solver.lock().unwrap();
        //     solver.solve_challenges(html, url).await?
        // };
        // 
        // if challenge_result.solved {
        //     tracing::info!("✅ Challenge solved successfully! Type: {:?}, Time: {:?}", 
        //         challenge_result.challenge_type, challenge_result.solve_time);
        //     
        //     // Apply any solution data (cookies, tokens, etc.)
        //     if let Some(challenge_cookies) = challenge_result.solution_data.get("challenge_cookies") {
        //         // Apply challenge cookies to our cookie jar
        //         if let Ok(cookie_url) = Url::parse(url) {
        //             if let Value::Object(cookies) = challenge_cookies {
        //                 for (name, value) in cookies {
        //                     if let Value::String(cookie_value) = value {
        //                         let cookie_str = format!("{}={}", name, cookie_value);
        //                         self.cookie_jar.add_cookie_str(&cookie_str, &cookie_url);
        //                     }
        //                 }
        //             }
        //         }
        //     }
        //     
        //     // If we have reCAPTCHA tokens, we might need to submit a form
        //     if challenge_result.challenge_type == ChallengeType::GoogleRecaptchaV3 {
        //         if let Some(Value::String(token)) = challenge_result.solution_data.get("recaptcha_token") {
        //             tracing::info!("🎯 Generated reCAPTCHA token, submitting...");
        //             // Use the existing renderer to execute JavaScript with the token
        //             let token_js = format!(r#"
        //                 window.recaptcha_token = "{}";
        //                 if (window.grecaptcha) {{
        //                     window.grecaptcha.ready(function() {{
        //                         window.grecaptcha.execute('{}', {{action: 'homepage'}}).then(function(token) {{
        //                             console.log('reCAPTCHA completed');
        //                         }});
        //                     }});
        //                 }}
        //             "#, token, challenge_result.solution_data.get("site_key").and_then(|v| v.as_str()).unwrap_or(""));
        //             
        //             return self.renderer.render_with_js(&format!("{}<script>{}</script>", html, token_js), url).await;
        //         }
        //     }
        // } else {
        //     tracing::warn!("⚠️ Advanced challenge solver failed, falling back to basic JavaScript execution");
        // }

        // Fallback: First, execute any JavaScript in the page
        let processed_html = self.renderer.render_with_js(html, url).await?;
        
        // Look for meta refresh redirect
        let document = Html::parse_document(&processed_html);
        
        if let Some(meta_refresh) = self.extract_meta_refresh(&document) {
            tracing::info!("🔄 Following challenge redirect: {}", meta_refresh);
            
            // Follow the redirect with a realistic delay (500-1000ms)
            let redirect_delay = 500 + (rand::random::<u64>() % 500);
            tokio::time::sleep(tokio::time::Duration::from_millis(redirect_delay)).await;
            
            let redirect_url = if meta_refresh.starts_with('/') {
                let base = Url::parse(url)?;
                base.join(&meta_refresh)?.to_string()
            } else if meta_refresh.starts_with("http") {
                meta_refresh
            } else {
                let base = Url::parse(url)?;
                base.join(&meta_refresh)?.to_string()
            };

            // Make the redirect request
            let response = self.client.get(&redirect_url).send().await?;
            let redirect_content = response.text().await?;
            
            // Process JavaScript on the redirect page too
            return self.renderer.render_with_js(&redirect_content, &redirect_url).await;
        }
        
        Ok(processed_html)
    }

    fn extract_meta_refresh(&self, document: &Html) -> Option<String> {
        let meta_selector = Selector::parse("meta[http-equiv='refresh']").unwrap();
        
        for element in document.select(&meta_selector) {
            if let Some(content) = element.value().attr("content") {
                // Parse content like "0;url=/httpservice/retry/enablejs?sei=..."
                if let Some(url_part) = content.split(';').nth(1) {
                    if let Some(url) = url_part.strip_prefix("url=") {
                        return Some(url.to_string());
                    }
                }
            }
        }
        
        None
    }
}