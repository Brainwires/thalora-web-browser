// Article metadata extraction (title, author, date)

use anyhow::Result;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

/// Metadata extracted from the content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub publication_date: Option<String>,
    pub word_count: u32,
    pub reading_time_minutes: u32,
    pub main_image: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

impl Default for ContentMetadata {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            publication_date: None,
            word_count: 0,
            reading_time_minutes: 0,
            main_image: None,
            tags: Vec::new(),
            description: None,
        }
    }
}

/// Extract metadata from the HTML document
pub(super) fn extract_metadata(html: &Html, base_url: &str) -> Result<ContentMetadata> {
    let mut metadata = ContentMetadata::default();

    // Extract title
    metadata.title = extract_title(html);

    // Extract author
    metadata.author = extract_author(html);

    // Extract publication date
    metadata.publication_date = extract_publication_date(html);

    // Extract main image
    metadata.main_image = extract_main_image(html, base_url);

    // Extract description
    metadata.description = extract_description(html);

    // Extract tags/keywords
    metadata.tags = extract_tags(html);

    Ok(metadata)
}

/// Extract title from various sources
pub(super) fn extract_title(html: &Html) -> Option<String> {
    // Try h1 first
    if let Ok(h1_selector) = Selector::parse("h1") {
        if let Some(h1) = html.select(&h1_selector).next() {
            let title = get_text_content(&h1);
            if !title.is_empty() && title.len() > 10 && title.len() < 200 {
                return Some(title);
            }
        }
    }

    // Try title tag
    if let Ok(title_selector) = Selector::parse("title") {
        if let Some(title_elem) = html.select(&title_selector).next() {
            let title = get_text_content(&title_elem);
            if !title.is_empty() {
                return Some(title);
            }
        }
    }

    // Try Open Graph title
    if let Ok(og_title_selector) = Selector::parse(r#"meta[property="og:title"]"#) {
        if let Some(og_title) = html.select(&og_title_selector).next() {
            if let Some(content) = og_title.value().attr("content") {
                return Some(content.to_string());
            }
        }
    }

    None
}

/// Extract author information
pub(super) fn extract_author(html: &Html) -> Option<String> {
    // Try various author selectors
    let author_selectors = [
        r#"meta[name="author"]"#,
        r#"meta[property="article:author"]"#,
        r#".author"#,
        r#".byline"#,
        r#"[rel="author"]"#,
    ];

    for selector_str in &author_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = html.select(&selector).next() {
                if let Some(content) = element.value().attr("content") {
                    return Some(content.to_string());
                } else {
                    let text = get_text_content(&element);
                    if !text.is_empty() && text.len() < 100 {
                        return Some(text);
                    }
                }
            }
        }
    }

    None
}

/// Extract publication date
pub(super) fn extract_publication_date(html: &Html) -> Option<String> {
    let date_selectors = [
        r#"meta[property="article:published_time"]"#,
        r#"meta[name="publishdate"]"#,
        r#"time[datetime]"#,
        r#".date"#,
        r#".published"#,
    ];

    for selector_str in &date_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = html.select(&selector).next() {
                if let Some(datetime) = element.value().attr("datetime") {
                    return Some(datetime.to_string());
                } else if let Some(content) = element.value().attr("content") {
                    return Some(content.to_string());
                }
            }
        }
    }

    None
}

/// Extract main image
pub(super) fn extract_main_image(html: &Html, base_url: &str) -> Option<String> {
    // Try Open Graph image first
    if let Ok(og_image_selector) = Selector::parse(r#"meta[property="og:image"]"#) {
        if let Some(og_image) = html.select(&og_image_selector).next() {
            if let Some(content) = og_image.value().attr("content") {
                if let Ok(url) = super::resolve_url(content, base_url) {
                    return Some(url);
                }
            }
        }
    }

    // Try first significant image in content
    if let Ok(img_selector) = Selector::parse("img") {
        for img in html.select(&img_selector) {
            if let Some(src) = img.value().attr("src") {
                if let Ok(url) = super::resolve_url(src, base_url) {
                    // Basic size filtering to avoid icons
                    if let Some(width) = img.value().attr("width") {
                        if let Ok(w) = width.parse::<u32>() {
                            if w >= 200 {
                                return Some(url);
                            }
                        }
                    } else {
                        // No explicit width, assume it might be main image
                        return Some(url);
                    }
                }
            }
        }
    }

    None
}

/// Extract description/summary
pub(super) fn extract_description(html: &Html) -> Option<String> {
    // Try meta description
    if let Ok(desc_selector) = Selector::parse(r#"meta[name="description"]"#) {
        if let Some(desc) = html.select(&desc_selector).next() {
            if let Some(content) = desc.value().attr("content") {
                return Some(content.to_string());
            }
        }
    }

    // Try Open Graph description
    if let Ok(og_desc_selector) = Selector::parse(r#"meta[property="og:description"]"#) {
        if let Some(og_desc) = html.select(&og_desc_selector).next() {
            if let Some(content) = og_desc.value().attr("content") {
                return Some(content.to_string());
            }
        }
    }

    None
}

/// Extract tags/keywords
pub(super) fn extract_tags(html: &Html) -> Vec<String> {
    let mut tags = Vec::new();

    // Try meta keywords
    if let Ok(keywords_selector) = Selector::parse(r#"meta[name="keywords"]"#) {
        if let Some(keywords) = html.select(&keywords_selector).next() {
            if let Some(content) = keywords.value().attr("content") {
                tags.extend(content.split(',').map(|s| s.trim().to_string()));
            }
        }
    }

    // Try article tags
    if let Ok(tag_selector) = Selector::parse(r#"meta[property="article:tag"]"#) {
        for tag_elem in html.select(&tag_selector) {
            if let Some(content) = tag_elem.value().attr("content") {
                tags.push(content.to_string());
            }
        }
    }

    tags
}

/// Get clean text content from an element, preserving word boundaries
fn get_text_content(element: &ElementRef) -> String {
    get_text_with_spacing(element)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Get text content while preserving proper spacing around inline elements
fn get_text_with_spacing(element: &ElementRef) -> String {
    let mut result = String::new();

    for node in element.children() {
        if let Some(child_element) = ElementRef::wrap(node) {
            // For inline elements that might break words, ensure spacing
            match child_element.value().name() {
                "a" | "span" | "em" | "strong" | "i" | "b" | "code" => {
                    // Add space before if needed
                    if !result.is_empty() && !result.ends_with(' ') {
                        result.push(' ');
                    }
                    result.push_str(&get_text_with_spacing(&child_element));
                    // Add space after if needed
                    if !result.ends_with(' ') {
                        result.push(' ');
                    }
                }
                _ => {
                    result.push_str(&get_text_with_spacing(&child_element));
                }
            }
        } else if let Some(text) = node.value().as_text() {
            result.push_str(text);
        }
    }

    result
}
