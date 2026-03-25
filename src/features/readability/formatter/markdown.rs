// Markdown conversion and formatting

use anyhow::Result;
use scraper::{Html, Selector};

/// Convert HTML to markdown format
pub(super) fn to_markdown(html: &Html, base_url: &str) -> Result<String> {
    let mut markdown = String::new();

    // Find the root content element (body or main content area)
    let root_element = if let Ok(body_selector) = Selector::parse("body") {
        html.select(&body_selector)
            .next()
            .unwrap_or_else(|| html.root_element())
    } else {
        html.root_element()
    };

    // Process the content tree in document order
    super::html_processing::process_element_tree(&root_element, &mut markdown, base_url)?;

    // Clean up excessive whitespace
    let cleaned = super::text_extraction::remove_extra_newlines(&markdown);

    Ok(cleaned)
}

/// Format heading with appropriate level
pub(super) fn format_heading(level: u8, content: &str) -> String {
    let prefix = "#".repeat(level as usize);
    format!("{} {}\n\n", prefix, content)
}

/// Format list item (unordered)
pub(super) fn format_list_item(content: &str) -> String {
    format!("- {}\n", content)
}

/// Format list item (ordered)
pub(super) fn format_ordered_list_item(number: u32, content: &str) -> String {
    format!("{}. {}\n", number, content)
}

/// Format code block
pub(super) fn format_code_block(content: &str, language: Option<&str>) -> String {
    if let Some(lang) = language {
        format!("```{}\n{}\n```\n\n", lang, content)
    } else {
        format!("```\n{}\n```\n\n", content)
    }
}

/// Format inline code
pub(super) fn format_inline_code(content: &str) -> String {
    format!("`{}`", content)
}

/// Format bold text
pub(super) fn format_bold(content: &str) -> String {
    format!("**{}**", content)
}

/// Format italic text
pub(super) fn format_italic(content: &str) -> String {
    format!("*{}*", content)
}

/// Format link
pub(super) fn format_link(text: &str, url: &str) -> String {
    format!("[{}]({})", text, url)
}

/// Format image
pub(super) fn format_image(alt: &str, url: &str, title: Option<&str>) -> String {
    if let Some(t) = title {
        format!("![{}]({} \"{}\")", alt, url, t)
    } else {
        format!("![{}]({})", alt, url)
    }
}

/// Format blockquote
pub(super) fn format_blockquote(content: &str) -> String {
    format!("> {}\n\n", content)
}

/// Format horizontal rule
pub(super) fn format_horizontal_rule() -> String {
    "---\n\n".to_string()
}
