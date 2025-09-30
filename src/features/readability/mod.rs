// Reading Mode Feature - Clean, readable content extraction from web pages
//
// This module implements sophisticated content extraction algorithms similar to
// Chrome's reading mode, providing AI models with clean, focused content without
// the noise and distractions of modern web pages.
//
// Key capabilities:
// - Intelligent content detection using scoring algorithms
// - Removal of navigation, ads, sidebars, and other boilerplate
// - Support for multiple output formats (markdown, text, structured JSON)
// - Multi-page article handling with session management
// - Quality metrics and readability scoring

pub mod extractor;
pub mod scorer;
pub mod cleaner;
pub mod formatter;

// Re-export main types for easy access
pub use extractor::{ReadabilityExtractor, ExtractionResult, ExtractionOptions};
pub use scorer::{ContentScore, ScoringMetrics};
pub use formatter::{OutputFormat, FormattedContent};

use anyhow::Result;
use scraper::Html;
use serde::{Deserialize, Serialize};

/// Configuration options for content extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadabilityConfig {
    /// Minimum content score threshold (0.0 - 1.0)
    pub min_content_score: f32,
    /// Maximum link density allowed in content (0.0 - 1.0)
    pub max_link_density: f32,
    /// Minimum paragraph count for valid content
    pub min_paragraph_count: u32,
    /// Include images in extracted content
    pub include_images: bool,
    /// Include metadata (author, date, etc.)
    pub include_metadata: bool,
    /// Output format preference
    pub output_format: OutputFormat,
}

impl Default for ReadabilityConfig {
    fn default() -> Self {
        Self {
            min_content_score: 0.3,
            max_link_density: 0.25,
            min_paragraph_count: 3,
            include_images: true,
            include_metadata: true,
            output_format: OutputFormat::Markdown,
        }
    }
}

/// Quality metrics for extracted content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Overall readability score (0-100)
    pub readability_score: u32,
    /// Content completeness indicator (0.0-1.0)
    pub completeness: f32,
    /// Noise level in extracted content (0.0-1.0)
    pub noise_level: f32,
    /// Structure preservation quality (0.0-1.0)
    pub structure_quality: f32,
    /// Word count of extracted content
    pub word_count: u32,
    /// Estimated reading time in minutes
    pub reading_time_minutes: u32,
}

/// Main interface for content extraction
pub struct ReadabilityEngine {
    config: ReadabilityConfig,
    extractor: ReadabilityExtractor,
}

impl ReadabilityEngine {
    /// Create a new readability engine with default configuration
    pub fn new() -> Self {
        Self {
            config: ReadabilityConfig::default(),
            extractor: ReadabilityExtractor::new(),
        }
    }

    /// Create a new readability engine with custom configuration
    pub fn with_config(config: ReadabilityConfig) -> Self {
        Self {
            config,
            extractor: ReadabilityExtractor::new(),
        }
    }

    /// Extract readable content from HTML
    pub fn extract(&mut self, html: &str, url: &str) -> Result<ExtractionResult> {
        let document = Html::parse_document(html);
        let options = ExtractionOptions {
            base_url: url.to_string(),
            include_images: self.config.include_images,
            include_metadata: self.config.include_metadata,
            output_format: self.config.output_format.clone(),
            min_content_score: self.config.min_content_score,
            max_link_density: self.config.max_link_density,
            min_paragraph_count: self.config.min_paragraph_count,
        };

        self.extractor.extract(&document, &options)
    }

    /// Get current configuration
    pub fn config(&self) -> &ReadabilityConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: ReadabilityConfig) {
        self.config = config;
    }
}

impl Default for ReadabilityEngine {
    fn default() -> Self {
        Self::new()
    }
}