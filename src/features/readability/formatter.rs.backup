// Output formatting for extracted content
//
// Converts cleaned HTML content into various formats suitable for AI consumption:
// - Markdown: Preserves structure with clean formatting
// - Plain text: Simple text with paragraph breaks
// - Structured JSON: Hierarchical data with metadata

use anyhow::{Result, Context};
use scraper::{Html, Selector, ElementRef};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// Supported output formats for extracted content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Markdown,
    Text,
    Structured,
}

/// Formatted content with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedContent {
    pub content: String,
    pub format: OutputFormat,
    pub metadata: ContentMetadata,
}

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

/// Content formatter that converts HTML to various formats
pub struct ContentFormatter {
    base_url: String,
}

impl ContentFormatter {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }

    /// Format content according to the specified format
    pub fn format(&self, html: &Html, format: &OutputFormat, include_metadata: bool) -> Result<FormattedContent> {
        let metadata = if include_metadata {
            self.extract_metadata(html)?
        } else {
            ContentMetadata::default()
        };

        let content = match format {
            OutputFormat::Markdown => self.to_markdown(html)?,
            OutputFormat::Text => self.to_text(html)?,
            OutputFormat::Structured => self.to_structured(html)?,
        };

        Ok(FormattedContent {
            content,
            format: format.clone(),
            metadata,
        })
    }

    /// Convert HTML to markdown format
    fn to_markdown(&self, html: &Html) -> Result<String> {
        let mut markdown = String::new();

        // Find the root content element (body or main content area)
        let root_element = if let Ok(body_selector) = Selector::parse("body") {
            html.select(&body_selector).next().unwrap_or_else(|| html.root_element())
        } else {
            html.root_element()
        };

        // Process the content tree in document order
        self.process_element_tree(&root_element, &mut markdown)?;

        // Clean up excessive whitespace
        let cleaned = markdown
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
            .replace("\n\n\n", "\n\n")
            .trim()
            .to_string();

        Ok(cleaned)
    }

    /// Recursively process element tree to maintain proper document structure
    fn process_element_tree(&self, element: &ElementRef, output: &mut String) -> Result<()> {
        let tag_name = element.value().name();

        match tag_name {
            "h1" => {
                let content = self.get_text_content(element);
                if !content.trim().is_empty() {
                    output.push_str(&format!("# {}\n\n", content));
                }
            },
            "h2" => {
                let content = self.get_text_content(element);
                if !content.trim().is_empty() {
                    output.push_str(&format!("## {}\n\n", content));
                }
            },
            "h3" => {
                let content = self.get_text_content(element);
                if !content.trim().is_empty() {
                    output.push_str(&format!("### {}\n\n", content));
                }
            },
            "h4" => {
                let content = self.get_text_content(element);
                if !content.trim().is_empty() {
                    output.push_str(&format!("#### {}\n\n", content));
                }
            },
            "h5" => {
                let content = self.get_text_content(element);
                if !content.trim().is_empty() {
                    output.push_str(&format!("##### {}\n\n", content));
                }
            },
            "h6" => {
                let content = self.get_text_content(element);
                if !content.trim().is_empty() {
                    output.push_str(&format!("###### {}\n\n", content));
                }
            },
            "p" => {
                let content = self.process_paragraph(element)?;
                if !content.trim().is_empty() {
                    output.push_str(&format!("{}\n\n", content));
                }
            },
            "blockquote" => {
                let content = self.get_text_content(element);
                if !content.trim().is_empty() {
                    output.push_str(&format!("> {}\n\n", content));
                }
            },
            "pre" => {
                let content = self.get_text_content(element);
                if !content.trim().is_empty() {
                    output.push_str(&format!("```\n{}\n```\n\n", content));
                }
            },
            "img" => {
                if let Some(img_md) = self.format_image(element)? {
                    output.push_str(&format!("{}\n\n", img_md));
                }
            },
            "ul" => {
                for child in element.children() {
                    if let Some(child_element) = ElementRef::wrap(child) {
                        if child_element.value().name() == "li" {
                            let content = self.process_paragraph(&child_element)?;
                            if !content.trim().is_empty() {
                                output.push_str(&format!("- {}\n", content));
                            }
                        }
                    }
                }
                output.push('\n');
            },
            "ol" => {
                let mut counter = 1;
                for child in element.children() {
                    if let Some(child_element) = ElementRef::wrap(child) {
                        if child_element.value().name() == "li" {
                            let content = self.process_paragraph(&child_element)?;
                            if !content.trim().is_empty() {
                                output.push_str(&format!("{}. {}\n", counter, content));
                                counter += 1;
                            }
                        }
                    }
                }
                output.push('\n');
            },
            // For container elements, process children
            "div" | "section" | "article" | "main" | "aside" | "nav" | "header" | "footer" | "body" | "html" => {
                for child in element.children() {
                    if let Some(child_element) = ElementRef::wrap(child) {
                        self.process_element_tree(&child_element, output)?;
                    }
                }
            },
            // Skip other elements but don't process their children
            _ => {}
        }

        Ok(())
    }

    /// Convert HTML to plain text
    fn to_text(&self, html: &Html) -> Result<String> {
        let mut text = String::new();

        if let Ok(selector) = Selector::parse("h1, h2, h3, h4, h5, h6, p, blockquote") {
            for element in html.select(&selector) {
                let content = self.get_text_content(&element);
                if !content.trim().is_empty() {
                    text.push_str(&format!("{}\n\n", content));
                }
            }
        }

        Ok(text.trim().to_string())
    }

    /// Convert HTML to structured JSON
    fn to_structured(&self, html: &Html) -> Result<String> {
        let mut sections = Vec::new();

        if let Ok(selector) = Selector::parse("h1, h2, h3, h4, h5, h6, p, blockquote, img") {
            for element in html.select(&selector) {
                match element.value().name() {
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        let level = element.value().name().chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
                        sections.push(serde_json::json!({
                            "type": "heading",
                            "level": level,
                            "content": self.get_text_content(&element)
                        }));
                    },
                    "p" => {
                        let content = self.get_text_content(&element);
                        if !content.trim().is_empty() {
                            sections.push(serde_json::json!({
                                "type": "paragraph",
                                "content": content
                            }));
                        }
                    },
                    "blockquote" => {
                        sections.push(serde_json::json!({
                            "type": "quote",
                            "content": self.get_text_content(&element)
                        }));
                    },
                    "img" => {
                        if let Some(src) = element.value().attr("src") {
                            if let Ok(resolved_url) = self.resolve_url(src) {
                                sections.push(serde_json::json!({
                                    "type": "image",
                                    "src": resolved_url,
                                    "alt": element.value().attr("alt").unwrap_or(""),
                                    "title": element.value().attr("title").unwrap_or("")
                                }));
                            }
                        }
                    },
                    _ => {}
                }
            }
        }

        let structured = serde_json::json!({
            "sections": sections
        });

        serde_json::to_string_pretty(&structured)
            .context("Failed to serialize structured content")
    }

    /// Process a paragraph element, handling inline formatting
    fn process_paragraph(&self, element: &ElementRef) -> Result<String> {
        let mut content = String::new();

        // Recursively process all child nodes to handle nested formatting
        self.process_inline_content(element, &mut content)?;

        // Clean up whitespace and return
        Ok(content.split_whitespace().collect::<Vec<_>>().join(" "))
    }

    /// Process inline content recursively to handle nested formatting properly
    fn process_inline_content(&self, element: &ElementRef, output: &mut String) -> Result<()> {
        for node in element.children() {
            if let Some(child_element) = ElementRef::wrap(node) {
                match child_element.value().name() {
                    "strong" | "b" => {
                        output.push_str("**");
                        self.process_inline_content(&child_element, output)?;
                        output.push_str("**");
                    },
                    "em" | "i" => {
                        output.push_str("*");
                        self.process_inline_content(&child_element, output)?;
                        output.push_str("*");
                    },
                    "code" => {
                        output.push('`');
                        output.push_str(&self.get_text_content(&child_element));
                        output.push('`');
                    },
                    "a" => {
                        let text = self.get_text_content(&child_element);
                        if let Some(href) = child_element.value().attr("href") {
                            if let Ok(url) = self.resolve_url(href) {
                                output.push_str(&format!("[{}]({})", text, url));
                            } else {
                                output.push_str(&text);
                            }
                        } else {
                            output.push_str(&text);
                        }
                    },
                    "br" => {
                        output.push(' ');
                    },
                    "span" | "div" => {
                        // Process content of span/div elements inline
                        self.process_inline_content(&child_element, output)?;
                    },
                    _ => {
                        // For other inline elements, just include the text content
                        output.push_str(&self.get_text_content(&child_element));
                    }
                }
            } else if let Some(text) = node.value().as_text() {
                output.push_str(text);
            }
        }

        Ok(())
    }

    /// Format an image element for markdown
    fn format_image(&self, element: &ElementRef) -> Result<Option<String>> {
        if let Some(src) = element.value().attr("src") {
            if let Ok(resolved_url) = self.resolve_url(src) {
                let alt = element.value().attr("alt").unwrap_or("");
                let title = element.value().attr("title").unwrap_or("");

                let markdown = if !title.is_empty() {
                    format!("![{}]({} \"{}\")", alt, resolved_url, title)
                } else {
                    format!("![{}]({})", alt, resolved_url)
                };

                Ok(Some(markdown))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Extract metadata from the HTML document
    fn extract_metadata(&self, html: &Html) -> Result<ContentMetadata> {
        let mut metadata = ContentMetadata::default();

        // Extract title
        metadata.title = self.extract_title(html);

        // Extract author
        metadata.author = self.extract_author(html);

        // Extract publication date
        metadata.publication_date = self.extract_publication_date(html);

        // Extract main image
        metadata.main_image = self.extract_main_image(html);

        // Extract description
        metadata.description = self.extract_description(html);

        // Extract tags/keywords
        metadata.tags = self.extract_tags(html);

        // Calculate reading metrics
        let text_content = self.to_text(html)?;
        metadata.word_count = text_content.split_whitespace().count() as u32;
        metadata.reading_time_minutes = (metadata.word_count / 200).max(1); // 200 WPM average

        Ok(metadata)
    }

    /// Extract title from various sources
    fn extract_title(&self, html: &Html) -> Option<String> {
        // Try h1 first
        if let Ok(h1_selector) = Selector::parse("h1") {
            if let Some(h1) = html.select(&h1_selector).next() {
                let title = self.get_text_content(&h1);
                if !title.is_empty() && title.len() > 10 && title.len() < 200 {
                    return Some(title);
                }
            }
        }

        // Try title tag
        if let Ok(title_selector) = Selector::parse("title") {
            if let Some(title_elem) = html.select(&title_selector).next() {
                let title = self.get_text_content(&title_elem);
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
    fn extract_author(&self, html: &Html) -> Option<String> {
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
                        let text = self.get_text_content(&element);
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
    fn extract_publication_date(&self, html: &Html) -> Option<String> {
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
    fn extract_main_image(&self, html: &Html) -> Option<String> {
        // Try Open Graph image first
        if let Ok(og_image_selector) = Selector::parse(r#"meta[property="og:image"]"#) {
            if let Some(og_image) = html.select(&og_image_selector).next() {
                if let Some(content) = og_image.value().attr("content") {
                    if let Ok(url) = self.resolve_url(content) {
                        return Some(url);
                    }
                }
            }
        }

        // Try first significant image in content
        if let Ok(img_selector) = Selector::parse("img") {
            for img in html.select(&img_selector) {
                if let Some(src) = img.value().attr("src") {
                    if let Ok(url) = self.resolve_url(src) {
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
    fn extract_description(&self, html: &Html) -> Option<String> {
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
    fn extract_tags(&self, html: &Html) -> Vec<String> {
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
    fn get_text_content(&self, element: &ElementRef) -> String {
        self.get_text_with_spacing(element)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get text content while preserving proper spacing around inline elements
    fn get_text_with_spacing(&self, element: &ElementRef) -> String {
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
                        result.push_str(&self.get_text_with_spacing(&child_element));
                        // Add space after if needed
                        if !result.ends_with(' ') {
                            result.push(' ');
                        }
                    },
                    _ => {
                        result.push_str(&self.get_text_with_spacing(&child_element));
                    }
                }
            } else if let Some(text) = node.value().as_text() {
                result.push_str(text);
            }
        }

        result
    }

    /// Resolve relative URLs to absolute URLs
    fn resolve_url(&self, url: &str) -> Result<String> {
        if url.starts_with("http://") || url.starts_with("https://") {
            Ok(url.to_string())
        } else {
            let base = Url::parse(&self.base_url)
                .context("Failed to parse base URL")?;
            let resolved = base.join(url)
                .context("Failed to resolve relative URL")?;
            Ok(resolved.to_string())
        }
    }
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