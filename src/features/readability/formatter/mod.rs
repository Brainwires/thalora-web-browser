// Output formatting for extracted content
//
// Converts cleaned HTML content into various formats suitable for AI consumption:
// - Markdown: Preserves structure with clean formatting
// - Plain text: Simple text with paragraph breaks
// - Structured JSON: Hierarchical data with metadata

use anyhow::{Result, Context};
use scraper::Html;
use serde::{Deserialize, Serialize};
use url::Url;

mod html_processing;
mod text_extraction;
mod markdown;
mod metadata;

// Re-export public types
pub use metadata::ContentMetadata;

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
        let mut metadata = if include_metadata {
            self.extract_metadata(html)?
        } else {
            ContentMetadata::default()
        };

        let content = match format {
            OutputFormat::Markdown => self.to_markdown(html)?,
            OutputFormat::Text => self.to_text(html)?,
            OutputFormat::Structured => self.to_structured(html)?,
        };

        // Calculate reading metrics if metadata is included
        if include_metadata {
            let text_content = if matches!(format, OutputFormat::Text) {
                content.clone()
            } else {
                self.to_text(html)?
            };
            metadata.word_count = text_content.split_whitespace().count() as u32;
            metadata.reading_time_minutes = (metadata.word_count / 200).max(1); // 200 WPM average
        }

        Ok(FormattedContent {
            content,
            format: format.clone(),
            metadata,
        })
    }

    /// Convert HTML to markdown format
    fn to_markdown(&self, html: &Html) -> Result<String> {
        markdown::to_markdown(html, &self.base_url)
    }

    /// Convert HTML to plain text
    fn to_text(&self, html: &Html) -> Result<String> {
        text_extraction::to_text(html)
    }

    /// Convert HTML to structured JSON
    fn to_structured(&self, html: &Html) -> Result<String> {
        html_processing::to_structured(html, &self.base_url)
    }

    /// Extract metadata from the HTML document
    fn extract_metadata(&self, html: &Html) -> Result<ContentMetadata> {
        metadata::extract_metadata(html, &self.base_url)
    }
}

/// Resolve relative URLs to absolute URLs
pub(crate) fn resolve_url(url: &str, base_url: &str) -> Result<String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(url.to_string())
    } else {
        let base = Url::parse(base_url)
            .context("Failed to parse base URL")?;
        let resolved = base.join(url)
            .context("Failed to resolve relative URL")?;
        Ok(resolved.to_string())
    }
}
