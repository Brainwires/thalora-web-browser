use scraper::{Html, Selector};
use serde_json::Value;

/// Extract lists (ul, ol) from HTML content
pub fn extract(html: &str) -> Vec<Value> {
    let document = Html::parse_document(html);
    let mut lists = Vec::new();

    // Select all lists
    if let Ok(list_selector) = Selector::parse("ul, ol") {
        for list in document.select(&list_selector) {
            let list_type = list.value().name();
            let mut list_data = serde_json::json!({
                "type": list_type,
                "items": []
            });

            // Extract list items
            if let Ok(item_selector) = Selector::parse("li") {
                let items: Vec<String> = list.select(&item_selector)
                    .map(|li| {
                        // Get text content, handling nested lists
                        let mut text = String::new();
                        for node in li.children() {
                            if let Some(element) = node.value().as_element() {
                                if element.name() != "ul" && element.name() != "ol" {
                                    text.push_str(&scraper::ElementRef::wrap(node)
                                        .unwrap()
                                        .text()
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                        .trim());
                                    text.push(' ');
                                }
                            } else if let Some(text_node) = node.value().as_text() {
                                text.push_str(text_node.trim());
                                text.push(' ');
                            }
                        }
                        text.trim().to_string()
                    })
                    .filter(|item| !item.is_empty())
                    .collect();

                if !items.is_empty() {
                    list_data["items"] = Value::Array(
                        items.into_iter().map(Value::String).collect()
                    );
                    lists.push(list_data);
                }
            }
        }
    }

    lists
}
