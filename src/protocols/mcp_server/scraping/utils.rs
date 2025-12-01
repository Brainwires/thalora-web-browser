use scraper::{ElementRef, Selector};
use url::Url;

/// Extract title from an element using multiple selector strategies
pub fn extract_generic_title(element: &ElementRef, selectors: &[&str]) -> String {
    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(title_element) = element.select(&selector).next() {
                let title = title_element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !title.is_empty() && title.len() > 3 {
                    return title;
                }
            }
        }
    }

    // Fallback: look for any heading in the element
    let fallback_selectors = ["h1", "h2", "h3", "h4", "h5", "h6"];
    for selector_str in &fallback_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(title_element) = element.select(&selector).next() {
                let title = title_element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !title.is_empty() && title.len() > 3 {
                    return title;
                }
            }
        }
    }

    String::new()
}

/// Extract URL from an element using multiple selector strategies
pub fn extract_generic_url(element: &ElementRef, selectors: &[&str]) -> String {
    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(link_element) = element.select(&selector).next() {
                if let Some(href) = link_element.value().attr("href") {
                    let cleaned_url = clean_url(href);
                    if !cleaned_url.is_empty() && cleaned_url.starts_with("http") {
                        return cleaned_url;
                    }
                }
            }
        }
    }

    // Fallback: look for any link in the element
    if let Ok(selector) = Selector::parse("a[href]") {
        if let Some(link_element) = element.select(&selector).next() {
            if let Some(href) = link_element.value().attr("href") {
                let cleaned_url = clean_url(href);
                if !cleaned_url.is_empty() && cleaned_url.starts_with("http") {
                    return cleaned_url;
                }
            }
        }
    }

    String::new()
}

/// Extract snippet from an element using multiple selector strategies
pub fn extract_generic_snippet(element: &ElementRef, selectors: &[&str]) -> String {
    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(snippet_element) = element.select(&selector).next() {
                let snippet = snippet_element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !snippet.is_empty() && snippet.len() > 10 && snippet.len() < 500 {
                    return snippet;
                }
            }
        }
    }

    // Fallback: look for paragraph text
    if let Ok(selector) = Selector::parse("p") {
        if let Some(snippet_element) = element.select(&selector).next() {
            let snippet = snippet_element.text().collect::<Vec<_>>().join(" ").trim().to_string();
            if !snippet.is_empty() && snippet.len() > 10 && snippet.len() < 500 {
                return snippet;
            }
        }
    }

    String::new()
}

/// Clean various URL redirect patterns
pub fn clean_url(url: &str) -> String {
    // Handle various redirect patterns
    if url.starts_with("/url?q=") {
        // Google-style redirect
        if let Ok(parsed_url) = Url::parse(&format!("https://google.com{}", url)) {
            if let Some(query) = parsed_url.query() {
                for pair in query.split('&') {
                    if let Some(q_url) = pair.strip_prefix("q=") {
                        return urlencoding::decode(q_url)
                            .unwrap_or_else(|_| q_url.into())
                            .into_owned();
                    }
                }
            }
        }
    } else if url.starts_with("/l/?u=") {
        // Some redirect patterns
        if let Some(u_param) = url.strip_prefix("/l/?u=") {
            return urlencoding::decode(u_param)
                .unwrap_or_else(|_| u_param.into())
                .into_owned();
        }
    } else if url.starts_with("http") {
        return url.to_string();
    } else if url.starts_with("//") {
        return format!("https:{}", url);
    } else if url.starts_with("/") {
        // Relative URL - would need base URL to resolve properly
        return String::new();
    }

    url.to_string()
}

/// Clean Google-specific redirect URLs
pub fn clean_google_url(url: &str) -> String {
    if url.starts_with("/url?q=") {
        // Extract the actual URL from Google's redirect URL
        if let Ok(parsed_url) = Url::parse(&format!("https://google.com{}", url)) {
            if let Some(query) = parsed_url.query() {
                for pair in query.split('&') {
                    if let Some(q_url) = pair.strip_prefix("q=") {
                        return urlencoding::decode(q_url)
                            .unwrap_or_else(|_| q_url.into())
                            .into_owned();
                    }
                }
            }
        }
    } else if url.starts_with("http") {
        return url.to_string();
    }

    url.to_string()
}
