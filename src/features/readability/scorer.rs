// Content scoring algorithm for readability extraction
//
// Implements sophisticated heuristics to identify the main content area of a webpage
// by scoring DOM nodes based on text density, semantic HTML, and other factors.
// Based on Mozilla's Readability algorithm and similar implementations.

use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Individual content score components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentScore {
    /// Ratio of text to HTML tags (higher is better)
    pub text_density: f32,
    /// Number of paragraph elements (more indicates article content)
    pub paragraph_count: u32,
    /// Ratio of link text to total text (lower is better for content)
    pub link_density: f32,
    /// Bonus for semantic HTML5 elements
    pub semantic_weight: f32,
    /// Penalty for navigation/advertisement class names
    pub class_weight: f32,
    /// Final combined score (0.0 - 1.0)
    pub final_score: f32,
}

/// Scoring metrics for the entire extraction process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringMetrics {
    /// Number of nodes evaluated
    pub nodes_evaluated: u32,
    /// Best scoring node's score
    pub best_score: f32,
    /// Average score across all nodes
    pub average_score: f32,
    /// Number of nodes above minimum threshold
    pub viable_nodes: u32,
}

/// Content scoring engine that evaluates DOM nodes
pub struct ContentScorer {
    /// Minimum text length to consider for scoring
    min_text_length: usize,
    /// Weight factors for different scoring components
    weights: ScoringWeights,
    /// Cache for computed scores
    score_cache: HashMap<String, ContentScore>,
}

/// Weight configuration for scoring components
#[derive(Debug, Clone)]
struct ScoringWeights {
    text_density: f32,
    paragraph_count: f32,
    link_density: f32,
    semantic_weight: f32,
    class_weight: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            text_density: 0.25,
            paragraph_count: 0.30,
            link_density: -0.20, // Negative because high link density is bad
            semantic_weight: 0.15,
            class_weight: 0.10,
        }
    }
}

impl ContentScorer {
    /// Create a new content scorer with default settings
    pub fn new() -> Self {
        Self {
            min_text_length: 50,
            weights: ScoringWeights::default(),
            score_cache: HashMap::new(),
        }
    }

    /// Score all potential content nodes in the document
    pub fn score_nodes<'a>(
        &mut self,
        html: &'a Html,
    ) -> (Vec<(ElementRef<'a>, ContentScore)>, ScoringMetrics) {
        let mut scored_nodes = Vec::new();
        let mut total_score = 0.0;
        let mut viable_count = 0;
        let mut best_score: f32 = 0.0;

        // Get candidate elements - focus on containers that might hold content
        if let Ok(candidate_selector) = Selector::parse("div, article, section, main, aside, p") {
            for element in html.select(&candidate_selector) {
                let score = self.score_element(&element);

                if score.final_score > 0.1 {
                    // Minimum threshold
                    viable_count += 1;
                }

                total_score += score.final_score;
                best_score = best_score.max(score.final_score);

                scored_nodes.push((element, score));
            }
        }

        let node_count = scored_nodes.len() as u32;
        let average_score = if node_count > 0 {
            total_score / node_count as f32
        } else {
            0.0
        };

        // Sort by score (highest first)
        scored_nodes.sort_by(|a, b| b.1.final_score.partial_cmp(&a.1.final_score).unwrap());

        let metrics = ScoringMetrics {
            nodes_evaluated: node_count,
            best_score,
            average_score,
            viable_nodes: viable_count,
        };

        (scored_nodes, metrics)
    }

    /// Score a single element based on multiple factors
    fn score_element(&mut self, element: &ElementRef) -> ContentScore {
        // Generate cache key based on element position and tag
        let cache_key = format!(
            "{}_{}",
            element.value().name(),
            element.text().collect::<String>().len()
        );

        if let Some(cached_score) = self.score_cache.get(&cache_key) {
            return cached_score.clone();
        }

        let text_content = element.text().collect::<String>();

        // Skip if text is too short
        if text_content.len() < self.min_text_length {
            let score = ContentScore {
                text_density: 0.0,
                paragraph_count: 0,
                link_density: 1.0, // Maximum penalty
                semantic_weight: 0.0,
                class_weight: 0.0,
                final_score: 0.0,
            };
            self.score_cache.insert(cache_key, score.clone());
            return score;
        }

        let text_density = self.calculate_text_density(element);
        let paragraph_count = self.count_paragraphs(element);
        let link_density = self.calculate_link_density(element);
        let semantic_weight = self.calculate_semantic_weight(element);
        let class_weight = self.calculate_class_weight(element);

        // Combine scores with weights
        let final_score = (text_density * self.weights.text_density
            + (paragraph_count as f32 / 10.0).min(1.0) * self.weights.paragraph_count
            + link_density * self.weights.link_density
            + semantic_weight * self.weights.semantic_weight
            + class_weight * self.weights.class_weight)
            .max(0.0)
            .min(1.0);

        let score = ContentScore {
            text_density,
            paragraph_count,
            link_density,
            semantic_weight,
            class_weight,
            final_score,
        };

        self.score_cache.insert(cache_key, score.clone());
        score
    }

    /// Calculate text density: ratio of text characters to HTML characters
    fn calculate_text_density(&self, element: &ElementRef) -> f32 {
        let text_content = element.text().collect::<String>();
        let html_content = element.html();

        let text_length = text_content.len() as f32;
        let html_length = html_content.len() as f32;

        if html_length == 0.0 {
            return 0.0;
        }

        (text_length / html_length).min(1.0)
    }

    /// Count paragraph elements within this element
    fn count_paragraphs(&self, element: &ElementRef) -> u32 {
        if let Ok(p_selector) = Selector::parse("p") {
            element.select(&p_selector).count() as u32
        } else {
            0
        }
    }

    /// Calculate link density: ratio of link text to total text
    fn calculate_link_density(&self, element: &ElementRef) -> f32 {
        let total_text = element.text().collect::<String>();
        let total_text_len = total_text.len() as f32;

        if total_text_len == 0.0 {
            return 0.0;
        }

        let mut link_text_len = 0.0;
        if let Ok(link_selector) = Selector::parse("a") {
            for link in element.select(&link_selector) {
                link_text_len += link.text().collect::<String>().len() as f32;
            }
        }

        link_text_len / total_text_len
    }

    /// Calculate semantic weight based on HTML5 semantic elements
    fn calculate_semantic_weight(&self, element: &ElementRef) -> f32 {
        match element.value().name() {
            "article" => 0.8,
            "main" => 0.7,
            "section" => 0.5,
            "div" => {
                // Check for content-indicating ID/class
                if let Some(id) = element.value().attr("id") {
                    if self.is_content_indicator(id) {
                        return 0.6;
                    }
                }
                if let Some(class) = element.value().attr("class") {
                    if self.is_content_indicator(class) {
                        return 0.6;
                    }
                }
                0.2
            }
            "aside" => 0.1, // Usually sidebar content
            "nav" => 0.0,   // Navigation
            "header" => 0.1,
            "footer" => 0.1,
            _ => 0.3,
        }
    }

    /// Calculate class weight based on class and ID names
    fn calculate_class_weight(&self, element: &ElementRef) -> f32 {
        let mut weight = 0.0;

        // Check ID attribute
        if let Some(id) = element.value().attr("id") {
            weight += self.classify_attribute_value(id);
        }

        // Check class attribute
        if let Some(class) = element.value().attr("class") {
            weight += self.classify_attribute_value(class);
        }

        weight.max(-1.0).min(1.0)
    }

    /// Classify an attribute value as positive (content) or negative (not content)
    fn classify_attribute_value(&self, value: &str) -> f32 {
        let value_lower = value.to_lowercase();

        // Positive indicators (content)
        let positive_keywords = [
            "content",
            "article",
            "main",
            "body",
            "text",
            "story",
            "post",
            "entry",
            "description",
            "summary",
            "detail",
        ];

        // Negative indicators (not content)
        let negative_keywords = [
            "nav",
            "navigation",
            "menu",
            "sidebar",
            "aside",
            "footer",
            "header",
            "ad",
            "ads",
            "advertisement",
            "banner",
            "popup",
            "modal",
            "overlay",
            "comment",
            "social",
            "share",
            "related",
            "recommended",
            "widget",
            "toolbar",
            "breadcrumb",
            "pagination",
            "tag",
            "category",
            "meta",
        ];

        let mut score = 0.0;

        for keyword in &positive_keywords {
            if value_lower.contains(keyword) {
                score += 0.2;
            }
        }

        for keyword in &negative_keywords {
            if value_lower.contains(keyword) {
                score -= 0.3;
            }
        }

        score
    }

    /// Check if an attribute value indicates content
    fn is_content_indicator(&self, value: &str) -> bool {
        let value_lower = value.to_lowercase();
        let content_indicators = [
            "content", "article", "main", "body", "text", "story", "post", "entry",
        ];

        content_indicators
            .iter()
            .any(|&indicator| value_lower.contains(indicator))
    }

    /// Find the best content node
    pub fn find_best_content_node<'a>(
        &mut self,
        html: &'a Html,
    ) -> Option<(ElementRef<'a>, ContentScore)> {
        let (scored_nodes, _) = self.score_nodes(html);

        if scored_nodes.is_empty() {
            return None;
        }

        // Get the highest scoring node - for now, we'll use this directly
        // Parent expansion can be handled in the extractor if needed
        scored_nodes.first().cloned()
    }

    /// Calculate score for an element without caching (helper method)
    fn calculate_element_score(&self, element: &ElementRef) -> ContentScore {
        let text_content = element.text().collect::<String>();

        // Skip if text is too short
        if text_content.len() < self.min_text_length {
            return ContentScore {
                text_density: 0.0,
                paragraph_count: 0,
                link_density: 1.0, // Maximum penalty
                semantic_weight: 0.0,
                class_weight: 0.0,
                final_score: 0.0,
            };
        }

        let text_density = self.calculate_text_density(element);
        let paragraph_count = self.count_paragraphs(element);
        let link_density = self.calculate_link_density(element);
        let semantic_weight = self.calculate_semantic_weight(element);
        let class_weight = self.calculate_class_weight(element);

        // Combine scores with weights
        let final_score = (text_density * self.weights.text_density
            + (paragraph_count as f32 / 10.0).min(1.0) * self.weights.paragraph_count
            + link_density * self.weights.link_density
            + semantic_weight * self.weights.semantic_weight
            + class_weight * self.weights.class_weight)
            .max(0.0)
            .min(1.0);

        ContentScore {
            text_density,
            paragraph_count,
            link_density,
            semantic_weight,
            class_weight,
            final_score,
        }
    }

    /// Get scoring metrics for analysis
    pub fn get_metrics<'a>(&mut self, html: &'a Html) -> ScoringMetrics {
        let (_, metrics) = self.score_nodes(html);
        metrics
    }

    /// Clear the score cache (useful when processing multiple documents)
    pub fn clear_cache(&mut self) {
        self.score_cache.clear();
    }
}

impl Default for ContentScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ContentScore {
    fn default() -> Self {
        Self {
            text_density: 0.0,
            paragraph_count: 0,
            link_density: 0.0,
            semantic_weight: 0.0,
            class_weight: 0.0,
            final_score: 0.0,
        }
    }
}

impl Default for ScoringMetrics {
    fn default() -> Self {
        Self {
            nodes_evaluated: 0,
            best_score: 0.0,
            average_score: 0.0,
            viable_nodes: 0,
        }
    }
}
