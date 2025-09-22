// Core content extraction engine
//
// Orchestrates the entire readability extraction process by combining
// HTML cleaning, content scoring, and output formatting to produce
// clean, readable content from web pages.

use anyhow::{Result, Context};
use scraper::{Html, ElementRef};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use super::cleaner::HtmlCleaner;
use super::scorer::{ContentScorer, ContentScore, ScoringMetrics};
use super::formatter::{ContentFormatter, OutputFormat, FormattedContent, ContentMetadata};
use super::QualityMetrics;

/// Options for content extraction
#[derive(Debug, Clone)]
pub struct ExtractionOptions {
    /// Base URL for resolving relative links
    pub base_url: String,
    /// Include images in extracted content
    pub include_images: bool,
    /// Include metadata extraction
    pub include_metadata: bool,
    /// Output format preference
    pub output_format: OutputFormat,
    /// Minimum content score threshold
    pub min_content_score: f32,
    /// Maximum link density allowed
    pub max_link_density: f32,
    /// Minimum paragraph count
    pub min_paragraph_count: u32,
}

/// Complete extraction result with content and quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// Formatted content
    pub content: FormattedContent,
    /// Quality metrics for the extraction
    pub quality: QualityMetrics,
    /// Scoring metrics used during extraction
    pub scoring: ScoringMetrics,
    /// Processing time in milliseconds
    pub processing_time_ms: u32,
    /// Whether extraction was successful
    pub success: bool,
    /// Error message if extraction failed
    pub error: Option<String>,
}

/// Content extraction confidence levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtractionConfidence {
    High,    // >0.7 score, clear article structure
    Medium,  // 0.4-0.7 score, some content found
    Low,     // 0.2-0.4 score, minimal content
    Failed,  // <0.2 score, no viable content
}

/// Main readability extractor
pub struct ReadabilityExtractor {
    cleaner: HtmlCleaner,
    scorer: ContentScorer,
}

impl ReadabilityExtractor {
    /// Create a new readability extractor
    pub fn new() -> Self {
        Self {
            cleaner: HtmlCleaner::new(),
            scorer: ContentScorer::new(),
        }
    }

    /// Extract readable content from HTML document
    pub fn extract(&mut self, document: &Html, options: &ExtractionOptions) -> Result<ExtractionResult> {
        let start_time = Instant::now();

        // Step 1: Score content nodes to find the best content on original document
        // (We'll do selective cleaning later to preserve text flow)
        let best_content = self.scorer.find_best_content_node(document);
        let (best_element, best_score) = match best_content {
            Some((element, score)) => (element, score),
            None => {
                return Ok(ExtractionResult {
                    content: FormattedContent {
                        content: String::new(),
                        format: options.output_format.clone(),
                        metadata: ContentMetadata::default(),
                    },
                    quality: QualityMetrics {
                        readability_score: 0,
                        completeness: 0.0,
                        noise_level: 1.0,
                        structure_quality: 0.0,
                        word_count: 0,
                        reading_time_minutes: 0,
                    },
                    scoring: ScoringMetrics::default(),
                    processing_time_ms: start_time.elapsed().as_millis() as u32,
                    success: false,
                    error: Some("No viable content found".to_string()),
                });
            }
        };

        // Step 2: Validate the found content meets minimum requirements
        if !Self::meets_quality_requirements(&best_element, &best_score, options) {
            let scoring = self.scorer.get_metrics(document);
            return Ok(ExtractionResult {
                content: FormattedContent {
                    content: String::new(),
                    format: options.output_format.clone(),
                    metadata: ContentMetadata::default(),
                },
                quality: QualityMetrics {
                    readability_score: (best_score.final_score * 100.0) as u32,
                    completeness: 0.3,
                    noise_level: best_score.link_density,
                    structure_quality: best_score.text_density,
                    word_count: 0,
                    reading_time_minutes: 0,
                },
                scoring,
                processing_time_ms: start_time.elapsed().as_millis() as u32,
                success: false,
                error: Some("Content does not meet quality requirements".to_string()),
            });
        }

        // Step 3: Extract and expand the content area
        let content_html = Self::extract_content_area(&best_element)?;
        let content_document = Html::parse_document(&content_html);

        // Step 5: Format the content according to options
        let formatter = ContentFormatter::new(&options.base_url);
        let formatted_content = formatter.format(
            &content_document,
            &options.output_format,
            options.include_metadata
        ).context("Failed to format content")?;

        // Step 4: Calculate quality metrics
        let quality = Self::calculate_quality_metrics(
            &best_score,
            &formatted_content,
            document
        );

        let scoring = self.scorer.get_metrics(document);
        let processing_time = start_time.elapsed().as_millis() as u32;

        Ok(ExtractionResult {
            content: formatted_content,
            quality,
            scoring,
            processing_time_ms: processing_time,
            success: true,
            error: None,
        })
    }

    /// Check if content meets minimum quality requirements
    fn meets_quality_requirements(
        element: &ElementRef,
        score: &ContentScore,
        options: &ExtractionOptions
    ) -> bool {
        // Check minimum score threshold
        if score.final_score < options.min_content_score {
            return false;
        }

        // Check link density threshold (more lenient for Wikipedia)
        if score.link_density > 0.5 {
            return false;
        }

        // Check minimum paragraph count (more lenient)
        if score.paragraph_count < 1 {
            return false;
        }

        // Check minimum text length (reduced from 200 to be more permissive)
        let text_content = element.text().collect::<String>();
        if text_content.trim().len() < 100 {
            return false;
        }

        true
    }

    /// Extract the content area, potentially expanding to include surrounding content
    fn extract_content_area(best_element: &ElementRef) -> Result<String> {
        // Start with the best element
        let mut content_html = best_element.html();

        // Try to expand to include sibling content that might be part of the article
        if let Some(parent) = best_element.parent() {
            if let Some(parent_element) = ElementRef::wrap(parent) {
                content_html = Self::expand_content_area(&parent_element, best_element)?;
            }
        }

        Ok(content_html)
    }

    /// Expand content area to include related sibling elements
    fn expand_content_area(parent: &ElementRef, original: &ElementRef) -> Result<String> {
        let mut expanded_elements = Vec::new();
        let original_html = original.html();

        // Collect elements that seem to be part of the same content
        for child in parent.children() {
            if let Some(child_element) = ElementRef::wrap(child) {
                if Self::is_related_content(&child_element, original) {
                    expanded_elements.push(child_element.html());
                }
            }
        }

        // If we found additional content, combine it
        if expanded_elements.len() > 1 {
            Ok(format!("<div>{}</div>", expanded_elements.join("")))
        } else {
            Ok(original_html)
        }
    }

    /// Check if an element is related content that should be included
    fn is_related_content(element: &ElementRef, reference: &ElementRef) -> bool {
        let tag_name = element.value().name();

        // Include if it's the same element
        if element.html() == reference.html() {
            return true;
        }

        // Include content-like elements
        if matches!(tag_name, "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "blockquote" | "div") {
            let text_content = element.text().collect::<String>();
            let text_length = text_content.trim().len();

            // Must have reasonable text content
            if text_length > 50 {
                // Check if it's not navigation or ads
                if let Some(class) = element.value().attr("class") {
                    let class_lower = class.to_lowercase();
                    if class_lower.contains("nav") || class_lower.contains("ad") ||
                       class_lower.contains("sidebar") || class_lower.contains("footer") {
                        return false;
                    }
                }

                return true;
            }
        }

        false
    }

    /// Calculate comprehensive quality metrics
    fn calculate_quality_metrics(
        score: &ContentScore,
        content: &FormattedContent,
        original_document: &Html
    ) -> QualityMetrics {
        // Basic readability score from content scoring
        let readability_score = (score.final_score * 100.0) as u32;

        // Estimate completeness by comparing extracted text to total text
        let original_text = original_document.root_element().text().collect::<String>();
        let extracted_text = &content.content;

        let completeness = if !original_text.is_empty() {
            (extracted_text.len() as f32 / original_text.len() as f32).min(1.0)
        } else {
            0.0
        };

        // Noise level is primarily based on link density
        let noise_level = score.link_density;

        // Structure quality based on text density and semantic elements
        let structure_quality = (score.text_density + score.semantic_weight) / 2.0;

        // Reading time from metadata
        let reading_time_minutes = content.metadata.reading_time_minutes;
        let word_count = content.metadata.word_count;

        QualityMetrics {
            readability_score,
            completeness,
            noise_level,
            structure_quality,
            word_count,
            reading_time_minutes,
        }
    }

    /// Get extraction confidence level based on score
    pub fn get_confidence_level(score: f32) -> ExtractionConfidence {
        match score {
            s if s >= 0.7 => ExtractionConfidence::High,
            s if s >= 0.4 => ExtractionConfidence::Medium,
            s if s >= 0.2 => ExtractionConfidence::Low,
            _ => ExtractionConfidence::Failed,
        }
    }

    /// Quick extraction method with default options
    pub fn extract_quick(&mut self, html: &str, url: &str) -> Result<ExtractionResult> {
        let document = Html::parse_document(html);
        let options = ExtractionOptions {
            base_url: url.to_string(),
            include_images: true,
            include_metadata: true,
            output_format: OutputFormat::Markdown,
            min_content_score: 0.3,
            max_link_density: 0.25,
            min_paragraph_count: 2,
        };

        self.extract(&document, &options)
    }

    /// Extract only text content without formatting
    pub fn extract_text_only(&mut self, html: &str, url: &str) -> Result<String> {
        let options = ExtractionOptions {
            base_url: url.to_string(),
            include_images: false,
            include_metadata: false,
            output_format: OutputFormat::Text,
            min_content_score: 0.2, // Lower threshold for text-only
            max_link_density: 0.5,
            min_paragraph_count: 1,
        };

        let document = Html::parse_document(html);
        let result = self.extract(&document, &options)?;

        if result.success {
            Ok(result.content.content)
        } else {
            Ok(String::new())
        }
    }

    /// Clear internal caches (useful when processing multiple documents)
    pub fn clear_caches(&mut self) {
        self.scorer.clear_cache();
    }
}

impl Default for ReadabilityExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ExtractionOptions {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            include_images: true,
            include_metadata: true,
            output_format: OutputFormat::Markdown,
            min_content_score: 0.3,
            max_link_density: 0.25,
            min_paragraph_count: 3,
        }
    }
}