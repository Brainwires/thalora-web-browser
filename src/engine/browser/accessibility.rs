//! Accessibility tree computation from HTML DOM.
//!
//! Maps HTML elements to ARIA roles and computes accessible names per:
//! - WAI-ARIA 1.2: https://www.w3.org/TR/wai-aria-1.2/
//! - HTML-AAM: https://www.w3.org/TR/html-aam-1.0/
//! - Accessible Name Computation: https://www.w3.org/TR/accname-1.2/

use scraper::{ElementRef, Html, Selector};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Compute the implicit ARIA role for an HTML element based on its tag and attributes.
fn implicit_role(tag: &str, attrs: &HashMap<&str, &str>) -> Option<&'static str> {
    match tag {
        "button" | "summary" => Some("button"),
        "a" if attrs.contains_key("href") => Some("link"),
        "area" if attrs.contains_key("href") => Some("link"),
        "nav" => Some("navigation"),
        "main" => Some("main"),
        "header" => Some("banner"),
        "footer" => Some("contentinfo"),
        "aside" => Some("complementary"),
        "form" if attrs.contains_key("aria-label") || attrs.contains_key("aria-labelledby") => {
            Some("form")
        }
        "form" => Some("form"),
        "search" => Some("search"),
        "section"
            if attrs.contains_key("aria-label") || attrs.contains_key("aria-labelledby") =>
        {
            Some("region")
        }
        "article" => Some("article"),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Some("heading"),
        "ul" | "ol" => Some("list"),
        "li" => Some("listitem"),
        "menu" => Some("list"),
        "table" => Some("table"),
        "thead" | "tbody" | "tfoot" => Some("rowgroup"),
        "tr" => Some("row"),
        "td" => Some("cell"),
        "th" => Some("columnheader"),
        "caption" => Some("caption"),
        "img" if attrs.get("alt").map(|a| !a.is_empty()).unwrap_or(false) => Some("img"),
        "img" if attrs.get("alt").map(|a| a.is_empty()).unwrap_or(false) => Some("presentation"),
        "img" => Some("img"),
        "figure" => Some("figure"),
        "figcaption" => None, // part of figure, no standalone role
        "hr" => Some("separator"),
        "output" => Some("status"),
        "progress" => Some("progressbar"),
        "meter" => Some("meter"),
        "dialog" => Some("dialog"),
        "details" => Some("group"),
        "fieldset" => Some("group"),
        "legend" => None,
        "textarea" => Some("textbox"),
        "select" if attrs.get("multiple").is_some() => Some("listbox"),
        "select" => Some("combobox"),
        "option" => Some("option"),
        "optgroup" => Some("group"),
        "input" => match attrs.get("type").copied() {
            Some("checkbox") => Some("checkbox"),
            Some("radio") => Some("radio"),
            Some("range") => Some("slider"),
            Some("search") => Some("searchbox"),
            Some("email") | Some("tel") | Some("url") | Some("text") | None => Some("textbox"),
            Some("number") => Some("spinbutton"),
            Some("submit") | Some("reset") | Some("button") | Some("image") => Some("button"),
            Some("hidden") => None,
            _ => Some("textbox"),
        },
        "label" => None,
        "address" => Some("group"),
        "blockquote" => Some("blockquote"),
        "code" => Some("code"),
        "pre" => None,
        "strong" | "b" => Some("strong"),
        "em" | "i" => Some("emphasis"),
        "del" | "s" => Some("deletion"),
        "ins" => Some("insertion"),
        "time" => Some("time"),
        "abbr" => None,
        _ => None,
    }
}

/// Compute the heading level from an h1-h6 tag.
fn heading_level(tag: &str) -> Option<u8> {
    match tag {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
}

/// Compute the accessible name for an element per the Accessible Name Computation spec.
fn compute_accessible_name(element: &ElementRef, doc: &Html) -> String {
    // 1. aria-label takes highest priority
    if let Some(label) = element.value().attr("aria-label") {
        if !label.trim().is_empty() {
            return label.trim().to_string();
        }
    }

    // 2. aria-labelledby — resolve referenced element IDs
    if let Some(labelledby) = element.value().attr("aria-labelledby") {
        let mut parts = Vec::new();
        for id in labelledby.split_whitespace() {
            let selector_str = format!("#{}", id);
            if let Ok(sel) = Selector::parse(&selector_str) {
                if let Some(referenced) = doc.select(&sel).next() {
                    let text: String = referenced.text().collect();
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        parts.push(text);
                    }
                }
            }
        }
        if !parts.is_empty() {
            return parts.join(" ");
        }
    }

    let tag = element.value().name();

    // 3. Element-specific fallbacks
    match tag {
        "img" | "area" => {
            if let Some(alt) = element.value().attr("alt") {
                if !alt.is_empty() {
                    return alt.to_string();
                }
            }
            if let Some(title) = element.value().attr("title") {
                return title.to_string();
            }
        }
        "input" | "textarea" | "select" => {
            // Check for associated label
            if let Some(id) = element.value().attr("id") {
                let label_sel = format!("label[for=\"{}\"]", id);
                if let Ok(sel) = Selector::parse(&label_sel) {
                    if let Some(label) = doc.select(&sel).next() {
                        let text: String = label.text().collect();
                        let text = text.trim().to_string();
                        if !text.is_empty() {
                            return text;
                        }
                    }
                }
            }
            if let Some(placeholder) = element.value().attr("placeholder") {
                if !placeholder.is_empty() {
                    return placeholder.to_string();
                }
            }
            if let Some(title) = element.value().attr("title") {
                return title.to_string();
            }
        }
        "a" | "button" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "th" | "caption"
        | "legend" | "summary" | "option" | "label" | "li" | "td" => {
            let text: String = element.text().collect();
            let text = text.trim().to_string();
            if !text.is_empty() {
                // Truncate long names
                if text.len() > 200 {
                    return format!("{}...", &text[..197]);
                }
                return text;
            }
        }
        _ => {}
    }

    // 4. title attribute as last resort
    if let Some(title) = element.value().attr("title") {
        if !title.is_empty() {
            return title.to_string();
        }
    }

    String::new()
}

/// Build an accessibility tree from HTML source.
///
/// Returns a JSON value representing the tree structure with roles, names, and states.
pub fn build_accessibility_tree(html: &str, max_depth: usize) -> Value {
    let doc = Html::parse_document(html);
    let all = Selector::parse("*").unwrap();

    let mut nodes = Vec::new();

    for element in doc.select(&all) {
        let tag = element.value().name();

        // Collect attributes
        let attrs: HashMap<&str, &str> = element.value().attrs().collect();

        // Check for explicit role override
        let role = if let Some(explicit_role) = attrs.get("role") {
            if !explicit_role.is_empty() {
                Some(*explicit_role)
            } else {
                implicit_role(tag, &attrs)
            }
        } else {
            implicit_role(tag, &attrs)
        };

        // Skip elements with no semantic role (div, span, etc.)
        let Some(role) = role else {
            continue;
        };

        // Skip hidden elements
        if attrs.get("aria-hidden") == Some(&"true") {
            continue;
        }
        if attrs.get("hidden").is_some() {
            continue;
        }

        let name = compute_accessible_name(&element, &doc);

        let mut node = json!({
            "role": role,
            "tag": tag,
        });

        if !name.is_empty() {
            node["name"] = json!(name);
        }

        // Add heading level
        if let Some(level) = heading_level(tag) {
            node["level"] = json!(level);
        }

        // Add id if present
        if let Some(id) = attrs.get("id") {
            if !id.is_empty() {
                node["id"] = json!(id);
            }
        }

        // Add states
        let mut states = Vec::new();
        if attrs.get("disabled").is_some() {
            states.push("disabled");
        }
        if attrs.get("required").is_some() {
            states.push("required");
        }
        if attrs.get("readonly").is_some() {
            states.push("readonly");
        }
        if attrs.get("checked").is_some() {
            states.push("checked");
        }
        if attrs.get("selected").is_some() {
            states.push("selected");
        }
        if let Some(expanded) = attrs.get("aria-expanded") {
            if *expanded == "true" {
                states.push("expanded");
            } else {
                states.push("collapsed");
            }
        }
        if attrs.get("aria-pressed") == Some(&"true") {
            states.push("pressed");
        }
        if attrs.get("aria-current") == Some(&"true") || attrs.get("aria-current") == Some(&"page")
        {
            states.push("current");
        }
        if !states.is_empty() {
            node["states"] = json!(states);
        }

        // Add description if present
        if let Some(desc) = attrs.get("aria-describedby") {
            node["describedBy"] = json!(desc);
        }

        // Add value for inputs
        if let Some(value) = attrs.get("value") {
            if matches!(tag, "input" | "textarea" | "select" | "progress" | "meter") {
                node["value"] = json!(value);
            }
        }

        nodes.push(node);

        // Respect max_depth (approximate — flat list, not recursive tree)
        if nodes.len() >= max_depth * 100 {
            break;
        }
    }

    json!({
        "nodeCount": nodes.len(),
        "nodes": nodes
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_role() {
        let tree = build_accessibility_tree("<button>Click me</button>", 10);
        let nodes = tree["nodes"].as_array().unwrap();
        let button = nodes.iter().find(|n| n["role"] == "button").unwrap();
        assert_eq!(button["name"], "Click me");
    }

    #[test]
    fn test_link_role() {
        let tree = build_accessibility_tree("<a href='/about'>About</a>", 10);
        let nodes = tree["nodes"].as_array().unwrap();
        let link = nodes.iter().find(|n| n["role"] == "link").unwrap();
        assert_eq!(link["name"], "About");
    }

    #[test]
    fn test_heading_with_level() {
        let tree = build_accessibility_tree("<h2>Section Title</h2>", 10);
        let nodes = tree["nodes"].as_array().unwrap();
        let heading = nodes.iter().find(|n| n["role"] == "heading").unwrap();
        assert_eq!(heading["name"], "Section Title");
        assert_eq!(heading["level"], 2);
    }

    #[test]
    fn test_nav_landmark() {
        let tree = build_accessibility_tree("<nav><a href='/'>Home</a></nav>", 10);
        let nodes = tree["nodes"].as_array().unwrap();
        assert!(nodes.iter().any(|n| n["role"] == "navigation"));
    }

    #[test]
    fn test_img_with_alt() {
        let tree = build_accessibility_tree("<img src='photo.jpg' alt='A sunset'>", 10);
        let nodes = tree["nodes"].as_array().unwrap();
        let img = nodes.iter().find(|n| n["role"] == "img").unwrap();
        assert_eq!(img["name"], "A sunset");
    }

    #[test]
    fn test_input_with_label() {
        let tree = build_accessibility_tree(
            "<label for='email'>Email</label><input id='email' type='text'>",
            10,
        );
        let nodes = tree["nodes"].as_array().unwrap();
        let input = nodes.iter().find(|n| n["role"] == "textbox").unwrap();
        assert_eq!(input["name"], "Email");
    }

    #[test]
    fn test_aria_label_override() {
        let tree = build_accessibility_tree(
            "<button aria-label='Close dialog'>X</button>",
            10,
        );
        let nodes = tree["nodes"].as_array().unwrap();
        let button = nodes.iter().find(|n| n["role"] == "button").unwrap();
        assert_eq!(button["name"], "Close dialog");
    }

    #[test]
    fn test_explicit_role_override() {
        let tree = build_accessibility_tree("<div role='alert'>Error occurred</div>", 10);
        let nodes = tree["nodes"].as_array().unwrap();
        assert!(nodes.iter().any(|n| n["role"] == "alert"));
    }

    #[test]
    fn test_hidden_elements_excluded() {
        let tree = build_accessibility_tree(
            "<button>Visible</button><button aria-hidden='true'>Hidden</button>",
            10,
        );
        let nodes = tree["nodes"].as_array().unwrap();
        let buttons: Vec<_> = nodes.iter().filter(|n| n["role"] == "button").collect();
        assert_eq!(buttons.len(), 1);
        assert_eq!(buttons[0]["name"], "Visible");
    }

    #[test]
    fn test_disabled_state() {
        let tree = build_accessibility_tree("<button disabled>Submit</button>", 10);
        let nodes = tree["nodes"].as_array().unwrap();
        let button = nodes.iter().find(|n| n["role"] == "button").unwrap();
        let states = button["states"].as_array().unwrap();
        assert!(states.iter().any(|s| s == "disabled"));
    }

    #[test]
    fn test_checkbox_checked_state() {
        let tree = build_accessibility_tree("<input type='checkbox' checked>", 10);
        let nodes = tree["nodes"].as_array().unwrap();
        let cb = nodes.iter().find(|n| n["role"] == "checkbox").unwrap();
        let states = cb["states"].as_array().unwrap();
        assert!(states.iter().any(|s| s == "checked"));
    }

    #[test]
    fn test_div_span_skipped() {
        let tree = build_accessibility_tree("<div><span>text</span></div>", 10);
        let nodes = tree["nodes"].as_array().unwrap();
        // div and span have no implicit role, should be skipped
        assert!(nodes.is_empty());
    }
}
