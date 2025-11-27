//! Browser module stub for WASM builds
//!
//! In WASM builds, the browser functionality is handled by the host browser environment.
//! This module provides placeholder types for API compatibility.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Authentication context for requests
#[derive(Debug, Clone, Default)]
pub struct AuthContext {
    pub cookies: HashMap<String, String>,
    pub bearer_token: Option<String>,
    pub basic_auth: Option<(String, String)>,
}

/// Browser storage (localStorage/sessionStorage simulation)
#[derive(Debug, Clone, Default)]
pub struct BrowserStorage {
    pub local: HashMap<String, String>,
    pub session: HashMap<String, String>,
}

/// Scraped page data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScrapedData {
    pub title: Option<String>,
    pub url: String,
    pub html: String,
    pub text: String,
    pub links: Vec<Link>,
    pub images: Vec<Image>,
    pub forms: Vec<Form>,
    pub metadata: HashMap<String, String>,
}

/// Link data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Link {
    pub url: String,
    pub text: String,
    pub title: Option<String>,
}

/// Image data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Image {
    pub src: String,
    pub alt: Option<String>,
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Form data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Form {
    pub action: String,
    pub method: String,
    pub fields: Vec<FormField>,
}

/// Form field data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FormField {
    pub name: String,
    pub field_type: String,
    pub value: Option<String>,
    pub required: bool,
}

/// Interaction response
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InteractionResponse {
    pub success: bool,
    pub message: String,
    pub new_url: Option<String>,
}

/// Headless browser stub for WASM builds
#[derive(Debug, Clone)]
pub struct HeadlessWebBrowser {
    pub current_url: Option<String>,
    pub storage: BrowserStorage,
    pub auth_context: AuthContext,
}

impl HeadlessWebBrowser {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            current_url: None,
            storage: BrowserStorage::default(),
            auth_context: AuthContext::default(),
        }))
    }

    pub fn new_with_engine(_engine_type: crate::engine::engine_trait::EngineType) -> Arc<Mutex<Self>> {
        Self::new()
    }

    pub async fn navigate(&mut self, _url: &str) -> Result<ScrapedData> {
        Err(anyhow::anyhow!("HeadlessWebBrowser.navigate() is not available in WASM. Use web-sys fetch instead."))
    }

    pub async fn scrape(&self) -> Result<ScrapedData> {
        Err(anyhow::anyhow!("HeadlessWebBrowser.scrape() is not available in WASM."))
    }

    pub fn get_current_url(&self) -> Option<&str> {
        self.current_url.as_deref()
    }

    pub fn set_cookie(&mut self, name: &str, value: &str) {
        self.auth_context.cookies.insert(name.to_string(), value.to_string());
    }

    pub fn get_cookie(&self, name: &str) -> Option<&str> {
        self.auth_context.cookies.get(name).map(|s| s.as_str())
    }

    pub fn set_local_storage(&mut self, key: &str, value: &str) {
        self.storage.local.insert(key.to_string(), value.to_string());
    }

    pub fn get_local_storage(&self, key: &str) -> Option<&str> {
        self.storage.local.get(key).map(|s| s.as_str())
    }

    pub fn set_session_storage(&mut self, key: &str, value: &str) {
        self.storage.session.insert(key.to_string(), value.to_string());
    }

    pub fn get_session_storage(&self, key: &str) -> Option<&str> {
        self.storage.session.get(key).map(|s| s.as_str())
    }
}

// Re-export for module compatibility
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
