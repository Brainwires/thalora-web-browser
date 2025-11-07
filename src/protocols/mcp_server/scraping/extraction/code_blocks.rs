use scraper::{Html, Selector};
use serde_json::Value;

/// Extract code blocks from HTML content
pub fn extract(html: &str) -> Vec<Value> {
    let document = Html::parse_document(html);
    let mut code_blocks = Vec::new();

    // Extract pre/code blocks
    let selectors = [
        "pre", "code", "pre code", ".highlight", ".code",
        ".sourceCode", ".language-*", "[class*='lang-']"
    ];

    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let code_text = element.text().collect::<Vec<_>>().join("\n").trim().to_string();

                if !code_text.is_empty() && code_text.len() > 10 {
                    // Try to detect language from class attributes
                    let mut language = None;
                    if let Some(class_attr) = element.value().attr("class") {
                        for class in class_attr.split_whitespace() {
                            if class.starts_with("language-") {
                                language = Some(class.strip_prefix("language-").unwrap().to_string());
                                break;
                            } else if class.starts_with("lang-") {
                                language = Some(class.strip_prefix("lang-").unwrap().to_string());
                                break;
                            }
                        }
                    }

                    let code_block = serde_json::json!({
                        "code": code_text,
                        "language": language,
                        "element_type": element.value().name()
                    });

                    // Avoid duplicates by checking if we already have this exact code
                    let is_duplicate = code_blocks.iter().any(|existing: &Value| {
                        existing["code"].as_str() == Some(code_text.as_str())
                    });

                    if !is_duplicate {
                        code_blocks.push(code_block);
                    }
                }
            }
        }
    }

    code_blocks
}
