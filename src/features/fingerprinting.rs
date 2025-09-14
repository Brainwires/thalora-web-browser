use anyhow::{Result, anyhow};
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Browser fingerprinting resistance module
/// Provides realistic browser signatures to avoid detection

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserFingerprint {
    /// User agent string
    pub user_agent: String,
    /// Accept headers
    pub accept_headers: HashMap<String, String>,
    /// TLS/SSL fingerprint
    pub tls_fingerprint: TlsFingerprint,
    /// HTTP/2 settings
    pub http2_settings: Http2Settings,
    /// Browser-specific headers
    pub browser_headers: HashMap<String, String>,
    /// Canvas fingerprint simulation
    pub canvas_fingerprint: String,
    /// WebGL fingerprint simulation
    pub webgl_fingerprint: HashMap<String, String>,
    /// Screen resolution and display info
    pub screen_info: ScreenInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsFingerprint {
    /// TLS version (e.g., "1.3")
    pub version: String,
    /// Supported cipher suites
    pub cipher_suites: Vec<String>,
    /// TLS extensions
    pub extensions: Vec<String>,
    /// Elliptic curves
    pub curves: Vec<String>,
    /// Signature algorithms
    pub signature_algorithms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Http2Settings {
    /// HTTP/2 window size
    pub window_size: u32,
    /// Max frame size
    pub max_frame_size: u32,
    /// Header table size
    pub header_table_size: u32,
    /// Max concurrent streams
    pub max_concurrent_streams: u32,
    /// Push enabled
    pub enable_push: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenInfo {
    /// Screen width
    pub width: u32,
    /// Screen height
    pub height: u32,
    /// Color depth (bits)
    pub color_depth: u8,
    /// Pixel density ratio
    pub device_pixel_ratio: f32,
    /// Available screen area (minus taskbar)
    pub available_width: u32,
    /// Available screen height
    pub available_height: u32,
}

/// Browser types for fingerprint generation
#[derive(Debug, Clone)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Edge,
}

impl BrowserFingerprint {
    /// Generate a realistic browser fingerprint
    pub fn generate(browser_type: BrowserType) -> Self {
        match browser_type {
            BrowserType::Chrome => Self::generate_chrome(),
            BrowserType::Firefox => Self::generate_firefox(),
            BrowserType::Safari => Self::generate_safari(),
            BrowserType::Edge => Self::generate_edge(),
        }
    }

    /// Generate Chrome fingerprint (most common, good for general use)
    fn generate_chrome() -> Self {
        let mut rng = rand::thread_rng();
        let version = rng.gen_range(115..121); // Recent Chrome versions
        let build = rng.gen_range(1000..9999);
        
        let user_agent = format!(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.{}.88 Safari/537.36",
            version, build
        );

        let mut accept_headers = HashMap::new();
        accept_headers.insert("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string());
        accept_headers.insert("Accept-Language".to_string(), "en-US,en;q=0.9".to_string());
        accept_headers.insert("Accept-Encoding".to_string(), "gzip, deflate, br".to_string());
        accept_headers.insert("Sec-Fetch-Dest".to_string(), "document".to_string());
        accept_headers.insert("Sec-Fetch-Mode".to_string(), "navigate".to_string());
        accept_headers.insert("Sec-Fetch-Site".to_string(), "none".to_string());
        accept_headers.insert("Sec-Fetch-User".to_string(), "?1".to_string());
        accept_headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string());

        let mut browser_headers = HashMap::new();
        browser_headers.insert("sec-ch-ua".to_string(), format!(r#""Google Chrome";v="{}", "Chromium";v="{}", "Not_A Brand";v="8""#, version, version));
        browser_headers.insert("sec-ch-ua-mobile".to_string(), "?0".to_string());
        browser_headers.insert("sec-ch-ua-platform".to_string(), r#""Windows""#.to_string());

        Self {
            user_agent,
            accept_headers,
            tls_fingerprint: Self::generate_chrome_tls(),
            http2_settings: Self::generate_chrome_http2(),
            browser_headers,
            canvas_fingerprint: Self::generate_chrome_canvas(),
            webgl_fingerprint: Self::generate_chrome_webgl(),
            screen_info: Self::generate_common_screen(),
        }
    }

    /// Generate Firefox fingerprint
    fn generate_firefox() -> Self {
        let mut rng = rand::thread_rng();
        let version = rng.gen_range(110..122); // Recent Firefox versions
        
        let user_agent = format!(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:{}.0) Gecko/20100101 Firefox/{}.0",
            version, version
        );

        let mut accept_headers = HashMap::new();
        accept_headers.insert("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8".to_string());
        accept_headers.insert("Accept-Language".to_string(), "en-US,en;q=0.5".to_string());
        accept_headers.insert("Accept-Encoding".to_string(), "gzip, deflate, br".to_string());
        accept_headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string());

        Self {
            user_agent,
            accept_headers,
            tls_fingerprint: Self::generate_firefox_tls(),
            http2_settings: Self::generate_firefox_http2(),
            browser_headers: HashMap::new(), // Firefox doesn't send sec-ch-ua
            canvas_fingerprint: Self::generate_firefox_canvas(),
            webgl_fingerprint: Self::generate_firefox_webgl(),
            screen_info: Self::generate_common_screen(),
        }
    }

    /// Generate Safari fingerprint
    fn generate_safari() -> Self {
        let mut rng = rand::thread_rng();
        let version = rng.gen_range(14..17); // Recent Safari versions
        let build = rng.gen_range(600..700);
        
        let user_agent = format!(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/{}.1.15 (KHTML, like Gecko) Version/{}.1.2 Safari/{}.1.15",
            build, version, build
        );

        let mut accept_headers = HashMap::new();
        accept_headers.insert("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string());
        accept_headers.insert("Accept-Language".to_string(), "en-US,en;q=0.9".to_string());
        accept_headers.insert("Accept-Encoding".to_string(), "gzip, deflate, br".to_string());

        Self {
            user_agent,
            accept_headers,
            tls_fingerprint: Self::generate_safari_tls(),
            http2_settings: Self::generate_safari_http2(),
            browser_headers: HashMap::new(),
            canvas_fingerprint: Self::generate_safari_canvas(),
            webgl_fingerprint: Self::generate_safari_webgl(),
            screen_info: ScreenInfo {
                width: 1440,
                height: 900,
                color_depth: 24,
                device_pixel_ratio: 2.0,
                available_width: 1440,
                available_height: 877,
            },
        }
    }

    /// Generate Edge fingerprint
    fn generate_edge() -> Self {
        let mut rng = rand::thread_rng();
        let version = rng.gen_range(110..120); // Recent Edge versions
        let build = rng.gen_range(1000..9999);
        
        let user_agent = format!(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.{}.88 Safari/537.36 Edg/{}.0.{}.62",
            version, build, version, build
        );

        let mut accept_headers = HashMap::new();
        accept_headers.insert("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string());
        accept_headers.insert("Accept-Language".to_string(), "en-US,en;q=0.9".to_string());
        accept_headers.insert("Accept-Encoding".to_string(), "gzip, deflate, br".to_string());

        let mut browser_headers = HashMap::new();
        browser_headers.insert("sec-ch-ua".to_string(), format!(r#""Microsoft Edge";v="{}", "Chromium";v="{}", "Not_A Brand";v="8""#, version, version));
        browser_headers.insert("sec-ch-ua-mobile".to_string(), "?0".to_string());
        browser_headers.insert("sec-ch-ua-platform".to_string(), r#""Windows""#.to_string());

        Self {
            user_agent,
            accept_headers,
            tls_fingerprint: Self::generate_chrome_tls(), // Edge uses similar TLS to Chrome
            http2_settings: Self::generate_chrome_http2(),
            browser_headers,
            canvas_fingerprint: Self::generate_chrome_canvas(),
            webgl_fingerprint: Self::generate_chrome_webgl(),
            screen_info: Self::generate_common_screen(),
        }
    }

    // TLS fingerprint generators
    fn generate_chrome_tls() -> TlsFingerprint {
        TlsFingerprint {
            version: "1.3".to_string(),
            cipher_suites: vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                "ECDHE-ECDSA-AES128-GCM-SHA256".to_string(),
                "ECDHE-RSA-AES128-GCM-SHA256".to_string(),
            ],
            extensions: vec![
                "server_name".to_string(),
                "extended_master_secret".to_string(),
                "renegotiation_info".to_string(),
                "supported_groups".to_string(),
                "ec_point_formats".to_string(),
                "application_layer_protocol_negotiation".to_string(),
            ],
            curves: vec![
                "X25519".to_string(),
                "secp256r1".to_string(),
                "secp384r1".to_string(),
            ],
            signature_algorithms: vec![
                "ecdsa_secp256r1_sha256".to_string(),
                "rsa_pss_rsae_sha256".to_string(),
                "rsa_pkcs1_sha256".to_string(),
            ],
        }
    }

    fn generate_firefox_tls() -> TlsFingerprint {
        TlsFingerprint {
            version: "1.3".to_string(),
            cipher_suites: vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "ECDHE-ECDSA-AES128-GCM-SHA256".to_string(),
            ],
            extensions: vec![
                "server_name".to_string(),
                "extended_master_secret".to_string(),
                "supported_groups".to_string(),
                "signature_algorithms".to_string(),
                "application_layer_protocol_negotiation".to_string(),
            ],
            curves: vec![
                "X25519".to_string(),
                "secp256r1".to_string(),
                "secp384r1".to_string(),
                "secp521r1".to_string(),
            ],
            signature_algorithms: vec![
                "ecdsa_secp256r1_sha256".to_string(),
                "ecdsa_secp384r1_sha384".to_string(),
                "rsa_pss_rsae_sha256".to_string(),
            ],
        }
    }

    fn generate_safari_tls() -> TlsFingerprint {
        TlsFingerprint {
            version: "1.3".to_string(),
            cipher_suites: vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "ECDHE-ECDSA-AES256-GCM-SHA384".to_string(),
                "ECDHE-ECDSA-AES128-GCM-SHA256".to_string(),
            ],
            extensions: vec![
                "server_name".to_string(),
                "supported_groups".to_string(),
                "signature_algorithms".to_string(),
                "application_layer_protocol_negotiation".to_string(),
            ],
            curves: vec![
                "secp256r1".to_string(),
                "X25519".to_string(),
                "secp384r1".to_string(),
            ],
            signature_algorithms: vec![
                "ecdsa_secp256r1_sha256".to_string(),
                "rsa_pss_rsae_sha256".to_string(),
            ],
        }
    }

    // HTTP/2 settings generators
    fn generate_chrome_http2() -> Http2Settings {
        Http2Settings {
            window_size: 6291456,
            max_frame_size: 16384,
            header_table_size: 4096,
            max_concurrent_streams: 1000,
            enable_push: false,
        }
    }

    fn generate_firefox_http2() -> Http2Settings {
        Http2Settings {
            window_size: 131072,
            max_frame_size: 16384,
            header_table_size: 4096,
            max_concurrent_streams: 100,
            enable_push: false,
        }
    }

    fn generate_safari_http2() -> Http2Settings {
        Http2Settings {
            window_size: 65536,
            max_frame_size: 16384,
            header_table_size: 4096,
            max_concurrent_streams: 100,
            enable_push: true,
        }
    }

    // Canvas fingerprint generators
    fn generate_chrome_canvas() -> String {
        format!("chrome_canvas_{}", rand::thread_rng().r#gen::<u32>())
    }

    fn generate_firefox_canvas() -> String {
        format!("firefox_canvas_{}", rand::thread_rng().r#gen::<u32>())
    }

    fn generate_safari_canvas() -> String {
        format!("safari_canvas_{}", rand::thread_rng().r#gen::<u32>())
    }

    // WebGL fingerprint generators
    fn generate_chrome_webgl() -> HashMap<String, String> {
        let mut webgl = HashMap::new();
        webgl.insert("vendor".to_string(), "Google Inc. (NVIDIA)".to_string());
        webgl.insert("renderer".to_string(), "ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 Direct3D11 vs_5_0 ps_5_0, D3D11)".to_string());
        webgl.insert("version".to_string(), "WebGL 1.0 (OpenGL ES 2.0 Chromium)".to_string());
        webgl.insert("shading_language_version".to_string(), "WebGL GLSL ES 1.0 (OpenGL ES GLSL ES 1.0 Chromium)".to_string());
        webgl
    }

    fn generate_firefox_webgl() -> HashMap<String, String> {
        let mut webgl = HashMap::new();
        webgl.insert("vendor".to_string(), "Mozilla".to_string());
        webgl.insert("renderer".to_string(), "Mozilla -- NVIDIA GeForce RTX 3080/PCIe/SSE2".to_string());
        webgl.insert("version".to_string(), "WebGL 1.0".to_string());
        webgl.insert("shading_language_version".to_string(), "WebGL GLSL ES 1.0".to_string());
        webgl
    }

    fn generate_safari_webgl() -> HashMap<String, String> {
        let mut webgl = HashMap::new();
        webgl.insert("vendor".to_string(), "Apple Inc.".to_string());
        webgl.insert("renderer".to_string(), "Apple GPU".to_string());
        webgl.insert("version".to_string(), "WebGL 1.0 (OpenGL ES 2.0 Apple-1.0)".to_string());
        webgl.insert("shading_language_version".to_string(), "WebGL GLSL ES 1.0 (OpenGL ES GLSL ES 1.0 Apple-1.0)".to_string());
        webgl
    }

    fn generate_common_screen() -> ScreenInfo {
        let screens = vec![
            (1920, 1080), (1366, 768), (1440, 900), (1536, 864), (1280, 720)
        ];
        let mut rng = rand::thread_rng();
        let (width, height) = screens[rng.gen_range(0..screens.len())];
        
        ScreenInfo {
            width,
            height,
            color_depth: 24,
            device_pixel_ratio: 1.0,
            available_width: width,
            available_height: height - 40, // Subtract taskbar
        }
    }

    /// Apply fingerprint to reqwest client builder
    pub fn apply_to_client_builder(&self, builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
        let mut default_headers = reqwest::header::HeaderMap::new();
        
        // Add User-Agent
        default_headers.insert(
            reqwest::header::USER_AGENT,
            self.user_agent.parse().unwrap(),
        );

        // Add accept headers
        for (key, value) in &self.accept_headers {
            if let (Ok(header_name), Ok(header_value)) = (
                reqwest::header::HeaderName::from_lowercase(key.to_lowercase().as_bytes()),
                reqwest::header::HeaderValue::from_str(value)
            ) {
                default_headers.insert(header_name, header_value);
            }
        }

        // Add browser-specific headers
        for (key, value) in &self.browser_headers {
            if let (Ok(header_name), Ok(header_value)) = (
                reqwest::header::HeaderName::from_lowercase(key.to_lowercase().as_bytes()),
                reqwest::header::HeaderValue::from_str(value)
            ) {
                default_headers.insert(header_name, header_value);
            }
        }

        builder.default_headers(default_headers)
    }
}

/// Fingerprint manager for rotating browser signatures
pub struct FingerprintManager {
    fingerprints: Vec<BrowserFingerprint>,
    current_index: usize,
}

impl FingerprintManager {
    /// Create a new fingerprint manager with diverse browser signatures
    pub fn new() -> Self {
        let fingerprints = vec![
            BrowserFingerprint::generate(BrowserType::Chrome),
            BrowserFingerprint::generate(BrowserType::Firefox),
            BrowserFingerprint::generate(BrowserType::Safari),
            BrowserFingerprint::generate(BrowserType::Edge),
        ];

        Self {
            fingerprints,
            current_index: 0,
        }
    }

    /// Get current fingerprint
    pub fn current(&self) -> &BrowserFingerprint {
        &self.fingerprints[self.current_index]
    }

    /// Rotate to next fingerprint
    pub fn rotate(&mut self) -> &BrowserFingerprint {
        self.current_index = (self.current_index + 1) % self.fingerprints.len();
        debug!("Rotated to fingerprint {}: {}", self.current_index, 
               &self.fingerprints[self.current_index].user_agent[..50]);
        &self.fingerprints[self.current_index]
    }

    /// Get a random fingerprint
    pub fn random(&self) -> &BrowserFingerprint {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.fingerprints.len());
        &self.fingerprints[index]
    }
}

impl Default for FingerprintManager {
    fn default() -> Self {
        Self::new()
    }
}