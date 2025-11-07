// Text extraction, whitespace normalization

use scraper::{Html, Selector, ElementRef};
use anyhow::Result;

/// Convert HTML to plain text
pub(super) fn to_text(html: &Html) -> Result<String> {
    let mut text = String::new();

    if let Ok(selector) = Selector::parse("h1, h2, h3, h4, h5, h6, p, blockquote") {
        for element in html.select(&selector) {
            let content = get_text_content(&element);
            if !content.trim().is_empty() {
                text.push_str(&format!("{}\n\n", content));
            }
        }
    }

    Ok(text.trim().to_string())
}

/// Get clean text content from an element, preserving word boundaries
pub(super) fn get_text_content(element: &ElementRef) -> String {
    get_text_with_spacing(element)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Get text content while preserving proper spacing around inline elements
pub(super) fn get_text_with_spacing(element: &ElementRef) -> String {
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
                },
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

/// Normalize whitespace in text
pub(super) fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Remove excessive newlines from text
pub(super) fn remove_extra_newlines(text: &str) -> String {
    text.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .replace("\n\n\n", "\n\n")
        .trim()
        .to_string()
}
