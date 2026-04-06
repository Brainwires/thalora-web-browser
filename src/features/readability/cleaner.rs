// HTML cleaning and preprocessing utilities
//
// Removes noise elements and prepares HTML for content extraction.
// This includes removing scripts, styles, ads, navigation, and other
// non-content elements that would interfere with readability scoring.

use anyhow::{Context, Result};
use scraper::{ElementRef, Html, Selector};
use std::collections::HashSet;

/// HTML cleaner that removes unwanted elements and prepares content for extraction
pub struct HtmlCleaner {
    /// Elements to completely remove from the document
    remove_selectors: Vec<String>,
    /// Class/ID patterns that indicate unwanted content
    unwanted_patterns: HashSet<String>,
    /// Minimum text length to preserve an element
    min_element_text: usize,
}

impl HtmlCleaner {
    /// Create a new HTML cleaner with default rules
    pub fn new() -> Self {
        let remove_selectors = vec![
            // Scripts and styles
            "script".to_string(),
            "style".to_string(),
            "noscript".to_string(),
            // Common unwanted elements
            "nav".to_string(),
            "header".to_string(),
            "footer".to_string(),
            ".navigation".to_string(),
            ".navbar".to_string(),
            ".menu".to_string(),
            ".sidebar".to_string(),
            ".widget".to_string(),
            ".advertisement".to_string(),
            ".ad".to_string(),
            ".ads".to_string(),
            ".banner".to_string(),
            ".popup".to_string(),
            ".modal".to_string(),
            ".overlay".to_string(),
            // Social and sharing
            ".social".to_string(),
            ".share".to_string(),
            ".sharing".to_string(),
            ".comments".to_string(),
            ".comment".to_string(),
            // Navigation and metadata
            ".breadcrumb".to_string(),
            ".breadcrumbs".to_string(),
            ".pagination".to_string(),
            ".pager".to_string(),
            ".tags".to_string(),
            ".categories".to_string(),
            ".metadata".to_string(),
            // Related content
            ".related".to_string(),
            ".recommended".to_string(),
            ".more-stories".to_string(),
            ".other-stories".to_string(),
            // Forms and interactive elements (unless they're part of content)
            "form".to_string(),
            ".search".to_string(),
            ".newsletter".to_string(),
            ".subscribe".to_string(),
        ];

        let unwanted_patterns = [
            // Advertisement patterns
            "ad",
            "ads",
            "advertisement",
            "banner",
            "sponsored",
            "promo",
            // Navigation patterns
            "nav",
            "navigation",
            "menu",
            "navbar",
            "sidebar",
            "aside",
            // Social patterns
            "social",
            "share",
            "sharing",
            "tweet",
            "facebook",
            "twitter",
            "linkedin",
            "pinterest",
            "instagram",
            // Comment patterns
            "comment",
            "comments",
            "disqus",
            "livefyre",
            // Utility patterns
            "breadcrumb",
            "pagination",
            "pager",
            "tag",
            "category",
            "meta",
            "widget",
            "popup",
            "modal",
            "overlay",
            "toolbar",
            // Related content patterns
            "related",
            "recommended",
            "more",
            "other",
            "similar",
            // Header/footer patterns
            "header",
            "footer",
            "top",
            "bottom",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            remove_selectors,
            unwanted_patterns,
            min_element_text: 20,
        }
    }

    /// Clean HTML document by removing unwanted elements
    pub fn clean(&self, html: &str) -> Result<String> {
        let mut document = Html::parse_document(html);

        // Remove unwanted elements by selector
        self.remove_unwanted_elements(&mut document)?;

        // Remove elements with unwanted class/id patterns
        self.remove_elements_by_patterns(&mut document)?;

        // Remove empty containers
        self.remove_empty_containers(&mut document)?;

        // Clean up attributes
        let cleaned_html = self.clean_attributes(&document)?;

        Ok(cleaned_html)
    }

    /// Remove elements matching unwanted selectors
    fn remove_unwanted_elements(&self, document: &mut Html) -> Result<()> {
        for selector_str in &self.remove_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                // Note: scraper crate doesn't support DOM modification
                // We'll handle this in the string processing step
            }
        }
        Ok(())
    }

    /// Remove elements with unwanted class or ID patterns
    fn remove_elements_by_patterns(&self, document: &mut Html) -> Result<()> {
        // This will be implemented as part of the filtering during extraction
        // since scraper doesn't support DOM modification
        Ok(())
    }

    /// Remove containers that become empty after cleaning
    fn remove_empty_containers(&self, document: &mut Html) -> Result<()> {
        // This will be handled during the extraction phase
        Ok(())
    }

    /// Clean up HTML by removing unwanted selectors and generating clean markup
    fn clean_attributes(&self, document: &Html) -> Result<String> {
        let mut cleaned_parts = Vec::new();

        // Process body content, filtering out unwanted elements
        if let Ok(body_selector) = Selector::parse("body") {
            if let Some(body) = document.select(&body_selector).next() {
                self.process_element_for_cleaning(&body, &mut cleaned_parts)?;
            } else {
                // No body tag, process the whole document
                self.process_element_for_cleaning(&document.root_element(), &mut cleaned_parts)?;
            }
        }

        let cleaned_html = format!("<html><body>{}</body></html>", cleaned_parts.join(""));
        Ok(cleaned_html)
    }

    /// Recursively process an element and its children, filtering unwanted content
    fn process_element_for_cleaning(
        &self,
        element: &ElementRef,
        output: &mut Vec<String>,
    ) -> Result<()> {
        let tag_name = element.value().name();

        // Skip unwanted elements completely
        if self.should_remove_element(element) {
            return Ok(());
        }

        // For text nodes, just add the text
        if tag_name == "text" {
            let text = element.text().collect::<String>();
            if !text.trim().is_empty() {
                output.push(text);
            }
            return Ok(());
        }

        // Check if this element has enough text content to be worthwhile
        let text_content = element.text().collect::<String>();
        if text_content.trim().len() < self.min_element_text
            && !self.is_structural_element(tag_name)
        {
            return Ok(());
        }

        // Build opening tag with cleaned attributes
        let mut tag_start = format!("<{}", tag_name);

        // Only preserve certain attributes
        if let Some(id) = element.value().attr("id")
            && !self.is_unwanted_attribute_value(id)
        {
            tag_start.push_str(&format!(" id=\"{}\"", id));
        }

        if let Some(class) = element.value().attr("class") {
            let cleaned_classes = self.clean_class_attribute(class);
            if !cleaned_classes.is_empty() {
                tag_start.push_str(&format!(" class=\"{}\"", cleaned_classes));
            }
        }

        // Preserve href for links
        if tag_name == "a"
            && let Some(href) = element.value().attr("href")
        {
            tag_start.push_str(&format!(" href=\"{}\"", href));
        }

        // Preserve src and alt for images
        if tag_name == "img" {
            if let Some(src) = element.value().attr("src") {
                tag_start.push_str(&format!(" src=\"{}\"", src));
            }
            if let Some(alt) = element.value().attr("alt") {
                tag_start.push_str(&format!(" alt=\"{}\"", alt));
            }
        }

        tag_start.push('>');
        output.push(tag_start);

        // Process children
        for child in element.children() {
            if let Some(child_element) = ElementRef::wrap(child) {
                self.process_element_for_cleaning(&child_element, output)?;
            } else if let Some(text) = child.value().as_text() {
                let text_content = text.trim();
                if !text_content.is_empty() {
                    output.push(text_content.to_string());
                }
            }
        }

        // Closing tag
        if !self.is_self_closing_tag(tag_name) {
            output.push(format!("</{}>", tag_name));
        }

        Ok(())
    }

    /// Check if an element should be completely removed
    fn should_remove_element(&self, element: &ElementRef) -> bool {
        let tag_name = element.value().name();

        // Remove script/style tags
        if matches!(tag_name, "script" | "style" | "noscript") {
            return true;
        }

        // Check if element has unwanted class or ID
        if let Some(id) = element.value().attr("id")
            && self.is_unwanted_attribute_value(id)
        {
            return true;
        }

        if let Some(class) = element.value().attr("class")
            && self.is_unwanted_attribute_value(class)
        {
            return true;
        }

        // Remove elements with very high link density (likely navigation)
        let total_text = element.text().collect::<String>();
        if !total_text.is_empty()
            && let Ok(link_selector) = Selector::parse("a")
        {
            let link_text: String = element
                .select(&link_selector)
                .map(|link| link.text().collect::<String>())
                .collect();

            let link_density = link_text.len() as f32 / total_text.len() as f32;
            if link_density > 0.8 && total_text.len() > 50 {
                return true;
            }
        }

        false
    }

    /// Check if an attribute value contains unwanted patterns
    fn is_unwanted_attribute_value(&self, value: &str) -> bool {
        let value_lower = value.to_lowercase();

        self.unwanted_patterns
            .iter()
            .any(|pattern| value_lower.contains(pattern))
    }

    /// Clean class attribute by removing unwanted classes
    fn clean_class_attribute(&self, class_value: &str) -> String {
        class_value
            .split_whitespace()
            .filter(|class| !self.is_unwanted_attribute_value(class))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Check if a tag is structural and should be preserved even with little text
    fn is_structural_element(&self, tag_name: &str) -> bool {
        matches!(
            tag_name,
            "html"
                | "head"
                | "body"
                | "div"
                | "section"
                | "article"
                | "main"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "p"
                | "blockquote"
                | "ul"
                | "ol"
                | "li"
                | "table"
                | "tr"
                | "td"
                | "th"
        )
    }

    /// Check if a tag is self-closing
    fn is_self_closing_tag(&self, tag_name: &str) -> bool {
        matches!(
            tag_name,
            "img"
                | "br"
                | "hr"
                | "input"
                | "meta"
                | "link"
                | "area"
                | "base"
                | "col"
                | "embed"
                | "source"
                | "track"
                | "wbr"
        )
    }

    /// Additional cleaning pass to remove low-quality content
    pub fn remove_low_quality_content<'a>(&self, document: &'a Html) -> Vec<ElementRef<'a>> {
        let mut quality_elements = Vec::new();

        // Find elements with good content indicators
        if let Ok(candidate_selector) = Selector::parse("div, article, section, main, p") {
            for element in document.select(&candidate_selector) {
                if self.is_quality_content(&element) {
                    quality_elements.push(element);
                }
            }
        }

        quality_elements
    }

    /// Determine if an element contains quality content
    fn is_quality_content(&self, element: &ElementRef) -> bool {
        let text_content = element.text().collect::<String>();
        let text_length = text_content.trim().len();

        // Must have sufficient text
        if text_length < self.min_element_text {
            return false;
        }

        // Check paragraph count
        let paragraph_count = if let Ok(p_selector) = Selector::parse("p") {
            element.select(&p_selector).count()
        } else {
            0
        };

        // Quality indicators
        let has_paragraphs = paragraph_count > 0;
        let good_length = text_length > 100;
        let not_navigation = !self.should_remove_element(element);

        has_paragraphs && good_length && not_navigation
    }

    /// Preserve important structural elements
    pub fn preserve_structure(&self, html: &str) -> Result<String> {
        // This method ensures we keep important structural HTML
        // while removing only the noise elements

        let document = Html::parse_document(html);
        let mut preserved_html = String::new();

        // Keep the essential structure
        preserved_html.push_str("<!DOCTYPE html><html><head>");

        // Preserve title
        if let Ok(title_selector) = Selector::parse("title")
            && let Some(title) = document.select(&title_selector).next()
        {
            preserved_html.push_str(&format!(
                "<title>{}</title>",
                title.text().collect::<String>()
            ));
        }

        // Preserve meta description
        if let Ok(meta_selector) = Selector::parse(r#"meta[name="description"]"#)
            && let Some(meta) = document.select(&meta_selector).next()
            && let Some(content) = meta.value().attr("content")
        {
            preserved_html.push_str(&format!(
                r#"<meta name="description" content="{}">"#,
                content
            ));
        }

        preserved_html.push_str("</head><body>");

        // Add cleaned body content
        let cleaned_content = self.clean(html)?;
        let body_document = Html::parse_document(&cleaned_content);

        if let Ok(body_selector) = Selector::parse("body")
            && let Some(body) = body_document.select(&body_selector).next()
        {
            preserved_html.push_str(&body.inner_html());
        }

        preserved_html.push_str("</body></html>");

        Ok(preserved_html)
    }
}

impl Default for HtmlCleaner {
    fn default() -> Self {
        Self::new()
    }
}
