use crate::engine::browser::types::{Form, FormField, Image, Link, ScrapedData};
use anyhow::Result;
use serde_json::{Map, Value};
use std::collections::HashMap;
use thalora_dom::{Document, NodeId};
use url::Url;

pub struct WebScraper;

/// `textContent` of `node` (empty if it has none).
fn text_of(doc: &Document, node: NodeId) -> String {
    doc.text_content(node).ok().flatten().unwrap_or_default()
}

/// The text nodes under `node` joined with single spaces.
pub(crate) fn joined_text(doc: &Document, node: NodeId) -> String {
    doc.descendant_texts(node)
        .map(|texts| texts.collect::<Vec<_>>().join(" "))
        .unwrap_or_default()
}

/// The elements of `doc` matching `selector` (a fixed, valid selector).
fn select(doc: &Document, selector: &str) -> Vec<NodeId> {
    doc.query_selector_all(doc.document(), selector)
        .unwrap_or_default()
}

/// Attribute `name` of element `node`.
fn attr<'a>(doc: &'a Document, node: NodeId, name: &str) -> Option<&'a str> {
    doc.as_element(node).and_then(|e| e.attr(name))
}

impl WebScraper {
    pub fn new() -> Self {
        Self
    }

    pub fn scrape_page(&self, document: &Document, base_url: &str) -> Result<ScrapedData> {
        // Extract title
        let title = select(document, "title")
            .first()
            .map(|&el| joined_text(document, el).trim().to_string())
            .filter(|t| !t.is_empty());

        // Extract links
        let links = self.extract_links(document, base_url)?;

        // Extract images
        let images = self.extract_images(document, base_url)?;

        // Extract metadata
        let metadata = self.extract_metadata(document)?;

        // Get text content
        let content = self.extract_text_content(document);

        // Try to extract structured data
        let extracted_data = self.extract_structured_data(document)?;

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

    fn extract_links(&self, document: &Document, base_url: &str) -> Result<Vec<Link>> {
        let mut links = Vec::new();

        for element in select(document, "a[href]") {
            if let Some(href) = attr(document, element, "href")
                && let Ok(url) = self.resolve_url(base_url, href)
            {
                let text = joined_text(document, element).trim().to_string();
                let title = attr(document, element, "title").map(|s| s.to_string());

                links.push(Link { url, text, title });
            }
        }

        Ok(links)
    }

    fn extract_images(&self, document: &Document, base_url: &str) -> Result<Vec<Image>> {
        let mut images = Vec::new();

        for element in select(document, "img[src]") {
            if let Some(src) = attr(document, element, "src")
                && let Ok(url) = self.resolve_url(base_url, src)
            {
                // CSP: Check img-src before including the image
                if !crate::engine::security::csp_state::csp_allows_image(&url) {
                    eprintln!("🔒 CSP: Image blocked by img-src: {}", url);
                    continue;
                }

                let alt = attr(document, element, "alt").map(|s| s.to_string());
                let title = attr(document, element, "title").map(|s| s.to_string());

                images.push(Image {
                    src: url,
                    alt,
                    title,
                });
            }
        }

        Ok(images)
    }

    fn extract_metadata(&self, document: &Document) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();

        // Extract meta tags
        for element in select(document, "meta") {
            let name = attr(document, element, "name");
            let property = attr(document, element, "property");
            let http_equiv = attr(document, element, "http-equiv");
            let content = attr(document, element, "content");

            // Handle different meta tag patterns
            if let (Some(name), Some(content)) = (name, content) {
                metadata.insert(name.to_string(), content.to_string());
            } else if let (Some(property), Some(content)) = (property, content) {
                metadata.insert(property.to_string(), content.to_string());
            } else if let (Some(http_equiv), Some(content)) = (http_equiv, content) {
                metadata.insert(format!("http-equiv:{}", http_equiv), content.to_string());
            }
        }

        // Extract canonical URL
        if let Some(&element) = select(document, r#"link[rel="canonical"]"#).first()
            && let Some(href) = attr(document, element, "href")
        {
            metadata.insert("canonical".to_string(), href.to_string());
        }

        Ok(metadata)
    }

    fn extract_text_content(&self, document: &Document) -> String {
        let mut content = String::new();

        if let Some(&body) = select(document, "body").first() {
            content = joined_text(document, body);
        }

        if content.is_empty()
            && let Some(root) = document.document_element()
        {
            content = joined_text(document, root);
        }

        // Clean up whitespace
        content
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    fn extract_structured_data(&self, document: &Document) -> Result<Option<Value>> {
        let mut structured_data = Map::new();

        // Extract JSON-LD
        for (i, element) in select(document, r#"script[type="application/ld+json"]"#)
            .into_iter()
            .enumerate()
        {
            let json_text = text_of(document, element);
            if let Ok(parsed) = serde_json::from_str::<Value>(&json_text) {
                structured_data.insert(format!("jsonld_{}", i), parsed);
            }
        }

        // Extract Open Graph data
        let mut og_data = Map::new();
        for element in select(document, r#"meta[property^="og:"]"#) {
            if let (Some(property), Some(content)) = (
                attr(document, element, "property"),
                attr(document, element, "content"),
            ) {
                og_data.insert(property.to_string(), Value::String(content.to_string()));
            }
        }
        if !og_data.is_empty() {
            structured_data.insert("opengraph".to_string(), Value::Object(og_data));
        }

        // Extract Twitter Card data
        let mut twitter_data = Map::new();
        for element in select(document, r#"meta[name^="twitter:"]"#) {
            if let (Some(name), Some(content)) = (
                attr(document, element, "name"),
                attr(document, element, "content"),
            ) {
                twitter_data.insert(name.to_string(), Value::String(content.to_string()));
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

    pub fn extract_forms(&self, document: &Document, base_url: &str) -> Result<Vec<Form>> {
        let mut forms = Vec::new();

        for form_element in select(document, "form") {
            let action = attr(document, form_element, "action")
                .map(|a| {
                    self.resolve_url(base_url, a)
                        .unwrap_or_else(|_| a.to_string())
                })
                .unwrap_or_else(|| base_url.to_string());

            let method = attr(document, form_element, "method")
                .unwrap_or("GET")
                .to_uppercase();

            let mut fields = Vec::new();

            // Extract input fields
            for input in document.query_selector_all(form_element, "input, textarea, select")? {
                let Some(el) = document.as_element(input) else {
                    continue;
                };
                let name = el.attr("name").unwrap_or("").to_string();
                if name.is_empty() {
                    continue;
                }

                let field_type = el
                    .attr("type")
                    .unwrap_or_else(|| match &**el.local_name() {
                        "textarea" => "textarea",
                        "select" => "select",
                        _ => "text",
                    })
                    .to_string();

                let value = el.attr("value").map(|s| s.to_string());
                let required = el.has_attr("required");
                let placeholder = el.attr("placeholder").map(|s| s.to_string());

                fields.push(FormField {
                    name,
                    field_type,
                    value,
                    required,
                    placeholder,
                });
            }

            forms.push(Form {
                action,
                method,
                fields,
            });
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
