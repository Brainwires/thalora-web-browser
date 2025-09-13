use anyhow::{anyhow, Result};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use url::Url;

use crate::renderer::RustRenderer;

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

pub struct WebScraper {
    client: reqwest::Client,
    renderer: RustRenderer,
}

impl WebScraper {
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

        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .default_headers(headers)
            .build()
            .unwrap();

        Self {
            client,
            renderer: RustRenderer::new(),
        }
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
        
        let response = self.client.get(url).send().await?;
        
        // Check if response is successful
        if !response.status().is_success() {
            return Err(anyhow!("HTTP request failed with status: {}", response.status()));
        }
        
        // Get the response content with proper encoding handling
        let html_content = response.text().await?;

        let processed_html = if wait_for_js {
            self.renderer.render_with_js(&html_content, url).await?
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
}