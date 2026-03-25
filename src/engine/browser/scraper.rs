use crate::engine::browser::types::{Form, FormField, Image, Link, ScrapedData};
use anyhow::Result;
use scraper::{Html, Selector};
use serde_json::{Map, Value};
use std::collections::HashMap;
use url::Url;

pub struct WebScraper;

impl WebScraper {
    pub fn new() -> Self {
        Self
    }

    pub fn scrape_page(&self, html_content: &str, base_url: &str) -> Result<ScrapedData> {
        let document = Html::parse_document(html_content);

        // Extract title
        let title = if let Ok(title_selector) = Selector::parse("title") {
            document
                .select(&title_selector)
                .next()
                .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .filter(|t| !t.is_empty())
        } else {
            None
        };

        // Extract links
        let links = self.extract_links(&document, base_url)?;

        // Extract images
        let images = self.extract_images(&document, base_url)?;

        // Extract metadata
        let metadata = self.extract_metadata(&document)?;

        // Get text content
        let content = self.extract_text_content(&document);

        // Try to extract structured data
        let extracted_data = self.extract_structured_data(&document)?;

        Ok(ScrapedData {
            url: base_url.to_string(),
            title,
            content,
            links,
            images,
            metadata,
            extracted_data,
        })
    }

    fn extract_links(&self, document: &Html, base_url: &str) -> Result<Vec<Link>> {
        let mut links = Vec::new();

        if let Ok(selector) = Selector::parse("a[href]") {
            for element in document.select(&selector) {
                if let Some(href) = element.value().attr("href") {
                    if let Ok(url) = self.resolve_url(base_url, href) {
                        let text = element
                            .text()
                            .collect::<Vec<_>>()
                            .join(" ")
                            .trim()
                            .to_string();
                        let title = element.value().attr("title").map(|s| s.to_string());

                        links.push(Link { url, text, title });
                    }
                }
            }
        }

        Ok(links)
    }

    fn extract_images(&self, document: &Html, base_url: &str) -> Result<Vec<Image>> {
        let mut images = Vec::new();

        if let Ok(selector) = Selector::parse("img[src]") {
            for element in document.select(&selector) {
                if let Some(src) = element.value().attr("src") {
                    if let Ok(url) = self.resolve_url(base_url, src) {
                        let alt = element.value().attr("alt").map(|s| s.to_string());
                        let title = element.value().attr("title").map(|s| s.to_string());

                        images.push(Image {
                            src: url,
                            alt,
                            title,
                        });
                    }
                }
            }
        }

        Ok(images)
    }

    fn extract_metadata(&self, document: &Html) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();

        // Extract meta tags
        if let Ok(meta_selector) = Selector::parse("meta") {
            for element in document.select(&meta_selector) {
                let attrs = element.value();

                // Handle different meta tag patterns
                if let (Some(name), Some(content)) = (attrs.attr("name"), attrs.attr("content")) {
                    metadata.insert(name.to_string(), content.to_string());
                } else if let (Some(property), Some(content)) =
                    (attrs.attr("property"), attrs.attr("content"))
                {
                    metadata.insert(property.to_string(), content.to_string());
                } else if let (Some(http_equiv), Some(content)) =
                    (attrs.attr("http-equiv"), attrs.attr("content"))
                {
                    metadata.insert(format!("http-equiv:{}", http_equiv), content.to_string());
                }
            }
        }

        // Extract canonical URL
        if let Ok(canonical_selector) = Selector::parse(r#"link[rel="canonical"]"#) {
            if let Some(element) = document.select(&canonical_selector).next() {
                if let Some(href) = element.value().attr("href") {
                    metadata.insert("canonical".to_string(), href.to_string());
                }
            }
        }

        Ok(metadata)
    }

    fn extract_text_content(&self, document: &Html) -> String {
        // Remove script and style elements
        let mut content = String::new();

        if let Ok(body_selector) = Selector::parse("body") {
            if let Some(body) = document.select(&body_selector).next() {
                content = body.text().collect::<Vec<_>>().join(" ");
            }
        }

        if content.is_empty() {
            content = document.root_element().text().collect::<Vec<_>>().join(" ");
        }

        // Clean up whitespace
        content
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    fn extract_structured_data(&self, document: &Html) -> Result<Option<Value>> {
        let mut structured_data = Map::new();

        // Extract JSON-LD
        if let Ok(jsonld_selector) = Selector::parse(r#"script[type="application/ld+json"]"#) {
            for (i, element) in document.select(&jsonld_selector).enumerate() {
                let json_text = element.text().collect::<String>();
                if let Ok(parsed) = serde_json::from_str::<Value>(&json_text) {
                    structured_data.insert(format!("jsonld_{}", i), parsed);
                }
            }
        }

        // Extract Open Graph data
        let mut og_data = Map::new();
        if let Ok(og_selector) = Selector::parse(r#"meta[property^="og:"]"#) {
            for element in document.select(&og_selector) {
                if let (Some(property), Some(content)) = (
                    element.value().attr("property"),
                    element.value().attr("content"),
                ) {
                    og_data.insert(property.to_string(), Value::String(content.to_string()));
                }
            }
        }
        if !og_data.is_empty() {
            structured_data.insert("opengraph".to_string(), Value::Object(og_data));
        }

        // Extract Twitter Card data
        let mut twitter_data = Map::new();
        if let Ok(twitter_selector) = Selector::parse(r#"meta[name^="twitter:"]"#) {
            for element in document.select(&twitter_selector) {
                if let (Some(name), Some(content)) = (
                    element.value().attr("name"),
                    element.value().attr("content"),
                ) {
                    twitter_data.insert(name.to_string(), Value::String(content.to_string()));
                }
            }
        }
        if !twitter_data.is_empty() {
            structured_data.insert("twitter".to_string(), Value::Object(twitter_data));
        }

        if structured_data.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Value::Object(structured_data)))
        }
    }

    pub fn extract_forms(&self, document: &Html, base_url: &str) -> Result<Vec<Form>> {
        let mut forms = Vec::new();

        if let Ok(form_selector) = Selector::parse("form") {
            for form_element in document.select(&form_selector) {
                let action = form_element
                    .value()
                    .attr("action")
                    .map(|a| {
                        self.resolve_url(base_url, a)
                            .unwrap_or_else(|_| a.to_string())
                    })
                    .unwrap_or_else(|| base_url.to_string());

                let method = form_element
                    .value()
                    .attr("method")
                    .unwrap_or("GET")
                    .to_uppercase();

                let mut fields = Vec::new();

                // Extract input fields
                if let Ok(input_selector) = Selector::parse("input, textarea, select") {
                    for input in form_element.select(&input_selector) {
                        let name = input.value().attr("name").unwrap_or("").to_string();
                        if name.is_empty() {
                            continue;
                        }

                        let field_type = input
                            .value()
                            .attr("type")
                            .unwrap_or_else(|| match input.value().name() {
                                "textarea" => "textarea",
                                "select" => "select",
                                _ => "text",
                            })
                            .to_string();

                        let value = input.value().attr("value").map(|s| s.to_string());
                        let required = input.value().attr("required").is_some();
                        let placeholder = input.value().attr("placeholder").map(|s| s.to_string());

                        fields.push(FormField {
                            name,
                            field_type,
                            value,
                            required,
                            placeholder,
                        });
                    }
                }

                forms.push(Form {
                    action,
                    method,
                    fields,
                });
            }
        }

        Ok(forms)
    }

    fn resolve_url(&self, base: &str, relative: &str) -> Result<String> {
        let base_url = Url::parse(base)?;
        let resolved = base_url.join(relative)?;
        Ok(resolved.to_string())
    }
}

impl Default for WebScraper {
    fn default() -> Self {
        Self::new()
    }
}
