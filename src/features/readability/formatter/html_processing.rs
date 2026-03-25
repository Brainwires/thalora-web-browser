// HTML parsing, DOM traversal, tag handling

use anyhow::Result;
use scraper::{ElementRef, Html, Selector};

/// Convert HTML to structured JSON
pub(super) fn to_structured(html: &Html, base_url: &str) -> Result<String> {
    let mut sections = Vec::new();

    if let Ok(selector) = Selector::parse("h1, h2, h3, h4, h5, h6, p, blockquote, img") {
        for element in html.select(&selector) {
            match element.value().name() {
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    let level = element
                        .value()
                        .name()
                        .chars()
                        .nth(1)
                        .unwrap()
                        .to_digit(10)
                        .unwrap() as u8;
                    sections.push(serde_json::json!({
                        "type": "heading",
                        "level": level,
                        "content": super::text_extraction::get_text_content(&element)
                    }));
                }
                "p" => {
                    let content = super::text_extraction::get_text_content(&element);
                    if !content.trim().is_empty() {
                        sections.push(serde_json::json!({
                            "type": "paragraph",
                            "content": content
                        }));
                    }
                }
                "blockquote" => {
                    sections.push(serde_json::json!({
                        "type": "quote",
                        "content": super::text_extraction::get_text_content(&element)
                    }));
                }
                "img" => {
                    if let Some(src) = element.value().attr("src") {
                        if let Ok(resolved_url) = super::resolve_url(src, base_url) {
                            sections.push(serde_json::json!({
                                "type": "image",
                                "src": resolved_url,
                                "alt": element.value().attr("alt").unwrap_or(""),
                                "title": element.value().attr("title").unwrap_or("")
                            }));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let structured = serde_json::json!({
        "sections": sections
    });

    serde_json::to_string_pretty(&structured)
        .map_err(|e| anyhow::anyhow!("Failed to serialize structured content: {}", e))
}

/// Recursively process element tree to maintain proper document structure
pub(super) fn process_element_tree(
    element: &ElementRef,
    output: &mut String,
    base_url: &str,
) -> Result<()> {
    let tag_name = element.value().name();

    match tag_name {
        "h1" => {
            let content = super::text_extraction::get_text_content(element);
            if !content.trim().is_empty() {
                output.push_str(&format!("# {}\n\n", content));
            }
        }
        "h2" => {
            let content = super::text_extraction::get_text_content(element);
            if !content.trim().is_empty() {
                output.push_str(&format!("## {}\n\n", content));
            }
        }
        "h3" => {
            let content = super::text_extraction::get_text_content(element);
            if !content.trim().is_empty() {
                output.push_str(&format!("### {}\n\n", content));
            }
        }
        "h4" => {
            let content = super::text_extraction::get_text_content(element);
            if !content.trim().is_empty() {
                output.push_str(&format!("#### {}\n\n", content));
            }
        }
        "h5" => {
            let content = super::text_extraction::get_text_content(element);
            if !content.trim().is_empty() {
                output.push_str(&format!("##### {}\n\n", content));
            }
        }
        "h6" => {
            let content = super::text_extraction::get_text_content(element);
            if !content.trim().is_empty() {
                output.push_str(&format!("###### {}\n\n", content));
            }
        }
        "p" => {
            let content = process_paragraph(element, base_url)?;
            if !content.trim().is_empty() {
                output.push_str(&format!("{}\n\n", content));
            }
        }
        "blockquote" => {
            let content = super::text_extraction::get_text_content(element);
            if !content.trim().is_empty() {
                output.push_str(&format!("> {}\n\n", content));
            }
        }
        "pre" => {
            let content = super::text_extraction::get_text_content(element);
            if !content.trim().is_empty() {
                output.push_str(&format!("```\n{}\n```\n\n", content));
            }
        }
        "img" => {
            if let Some(img_md) = format_image(element, base_url)? {
                output.push_str(&format!("{}\n\n", img_md));
            }
        }
        "ul" => {
            for child in element.children() {
                if let Some(child_element) = ElementRef::wrap(child) {
                    if child_element.value().name() == "li" {
                        let content = process_paragraph(&child_element, base_url)?;
                        if !content.trim().is_empty() {
                            output.push_str(&format!("- {}\n", content));
                        }
                    }
                }
            }
            output.push('\n');
        }
        "ol" => {
            let mut counter = 1;
            for child in element.children() {
                if let Some(child_element) = ElementRef::wrap(child) {
                    if child_element.value().name() == "li" {
                        let content = process_paragraph(&child_element, base_url)?;
                        if !content.trim().is_empty() {
                            output.push_str(&format!("{}. {}\n", counter, content));
                            counter += 1;
                        }
                    }
                }
            }
            output.push('\n');
        }
        // For container elements, process children
        "div" | "section" | "article" | "main" | "aside" | "nav" | "header" | "footer" | "body"
        | "html" => {
            for child in element.children() {
                if let Some(child_element) = ElementRef::wrap(child) {
                    process_element_tree(&child_element, output, base_url)?;
                }
            }
        }
        // Skip other elements but don't process their children
        _ => {}
    }

    Ok(())
}

/// Process a paragraph element, handling inline formatting
fn process_paragraph(element: &ElementRef, base_url: &str) -> Result<String> {
    let mut content = String::new();

    // Recursively process all child nodes to handle nested formatting
    process_inline_content(element, &mut content, base_url)?;

    // Clean up whitespace and return
    Ok(content.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// Process inline content recursively to handle nested formatting properly
fn process_inline_content(element: &ElementRef, output: &mut String, base_url: &str) -> Result<()> {
    for node in element.children() {
        if let Some(child_element) = ElementRef::wrap(node) {
            match child_element.value().name() {
                "strong" | "b" => {
                    output.push_str("**");
                    process_inline_content(&child_element, output, base_url)?;
                    output.push_str("**");
                }
                "em" | "i" => {
                    output.push_str("*");
                    process_inline_content(&child_element, output, base_url)?;
                    output.push_str("*");
                }
                "code" => {
                    output.push('`');
                    output.push_str(&super::text_extraction::get_text_content(&child_element));
                    output.push('`');
                }
                "a" => {
                    let text = super::text_extraction::get_text_content(&child_element);
                    if let Some(href) = child_element.value().attr("href") {
                        if let Ok(url) = super::resolve_url(href, base_url) {
                            output.push_str(&format!("[{}]({})", text, url));
                        } else {
                            output.push_str(&text);
                        }
                    } else {
                        output.push_str(&text);
                    }
                }
                "br" => {
                    output.push(' ');
                }
                "span" | "div" => {
                    // Process content of span/div elements inline
                    process_inline_content(&child_element, output, base_url)?;
                }
                _ => {
                    // For other inline elements, just include the text content
                    output.push_str(&super::text_extraction::get_text_content(&child_element));
                }
            }
        } else if let Some(text) = node.value().as_text() {
            output.push_str(text);
        }
    }

    Ok(())
}

/// Format an image element for markdown
fn format_image(element: &ElementRef, base_url: &str) -> Result<Option<String>> {
    if let Some(src) = element.value().attr("src") {
        if let Ok(resolved_url) = super::resolve_url(src, base_url) {
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
