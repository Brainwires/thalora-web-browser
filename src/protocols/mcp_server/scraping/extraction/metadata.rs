use scraper::{Html, Selector};
use serde_json::Value;

/// Extract article metadata (author, publish date, tags) from HTML content
pub fn extract(html: &str) -> Value {
    let document = Html::parse_document(html);
    let mut metadata = serde_json::json!({
        "title": null,
        "author": null,
        "publish_date": null,
        "tags": [],
        "description": null,
        "canonical_url": null
    });

    // Extract title
    if let Ok(title_selector) = Selector::parse("title, h1, .title, .article-title, [property='og:title'], [name='twitter:title']") {
        if let Some(title_element) = document.select(&title_selector).next() {
            let title = if title_element.value().name() == "meta" {
                title_element.value().attr("content").unwrap_or("").to_string()
            } else {
                title_element.text().collect::<Vec<_>>().join(" ").trim().to_string()
            };
            if !title.is_empty() {
                metadata["title"] = Value::String(title);
            }
        }
    }

    // Extract author
    let author_selectors = [
        "[name='author']", "[property='article:author']", "[rel='author']",
        ".author", ".byline", ".article-author", ".post-author"
    ];

    for selector_str in &author_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(author_element) = document.select(&selector).next() {
                let author = if author_element.value().name() == "meta" {
                    author_element.value().attr("content").unwrap_or("").to_string()
                } else {
                    author_element.text().collect::<Vec<_>>().join(" ").trim().to_string()
                };
                if !author.is_empty() {
                    metadata["author"] = Value::String(author);
                    break;
                }
            }
        }
    }

    // Extract publish date
    let date_selectors = [
        "[property='article:published_time']", "[name='publish_date']",
        "[name='date']", "time[datetime]", ".publish-date", ".date",
        ".article-date", ".post-date"
    ];

    for selector_str in &date_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(date_element) = document.select(&selector).next() {
                let date = if let Some(datetime) = date_element.value().attr("datetime") {
                    datetime.to_string()
                } else if let Some(content) = date_element.value().attr("content") {
                    content.to_string()
                } else {
                    date_element.text().collect::<Vec<_>>().join(" ").trim().to_string()
                };
                if !date.is_empty() {
                    metadata["publish_date"] = Value::String(date);
                    break;
                }
            }
        }
    }

    // Extract tags/keywords
    let tag_selectors = [
        "[name='keywords']", "[property='article:tag']",
        ".tags a", ".tag", ".article-tags a", ".post-tags a"
    ];

    let mut tags = Vec::new();
    for selector_str in &tag_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for tag_element in document.select(&selector) {
                let tag = if tag_element.value().name() == "meta" {
                    tag_element.value().attr("content").unwrap_or("").to_string()
                } else {
                    tag_element.text().collect::<Vec<_>>().join(" ").trim().to_string()
                };

                if !tag.is_empty() {
                    // Split comma-separated keywords
                    if tag.contains(',') {
                        for t in tag.split(',') {
                            let clean_tag = t.trim().to_string();
                            if !clean_tag.is_empty() && !tags.contains(&clean_tag) {
                                tags.push(clean_tag);
                            }
                        }
                    } else if !tags.contains(&tag) {
                        tags.push(tag);
                    }
                }
            }
        }
    }

    if !tags.is_empty() {
        metadata["tags"] = Value::Array(
            tags.into_iter().map(Value::String).collect()
        );
    }

    // Extract description
    let desc_selectors = [
        "[name='description']", "[property='og:description']",
        "[name='twitter:description']", ".description", ".summary"
    ];

    for selector_str in &desc_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(desc_element) = document.select(&selector).next() {
                let description = if desc_element.value().name() == "meta" {
                    desc_element.value().attr("content").unwrap_or("").to_string()
                } else {
                    desc_element.text().collect::<Vec<_>>().join(" ").trim().to_string()
                };
                if !description.is_empty() {
                    metadata["description"] = Value::String(description);
                    break;
                }
            }
        }
    }

    // Extract canonical URL
    if let Ok(canonical_selector) = Selector::parse("[rel='canonical']") {
        if let Some(canonical_element) = document.select(&canonical_selector).next() {
            if let Some(href) = canonical_element.value().attr("href") {
                metadata["canonical_url"] = Value::String(href.to_string());
            }
        }
    }

    metadata
}
