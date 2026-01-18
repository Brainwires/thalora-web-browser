use scraper::{Html, Selector};
use serde_json::Value;

use crate::protocols::mcp::McpResponse;
use crate::protocols::mcp_server::core::McpServer;
use crate::protocols::security::{
    validate_url_for_navigation, sanitize_session_id, limit_input_length,
    MAX_URL_LENGTH, MAX_SELECTOR_LENGTH,
};

use super::extraction;

impl McpServer {
    /// Unified scraping function that combines all scraping capabilities
    pub(in crate::protocols::mcp_server) async fn scrape_unified(&mut self, arguments: Value) -> McpResponse {
        let url = arguments["url"].as_str();
        let session_id = arguments.get("session_id").and_then(|v| v.as_str());

        // Validate that we have either URL or session_id
        if url.is_none() && session_id.is_none() {
            return McpResponse::error(-1, "Either 'url' or 'session_id' parameter is required".to_string());
        }

        // SECURITY: Validate input lengths to prevent DoS attacks
        if let Some(url_str) = url {
            if let Err(e) = limit_input_length(url_str, MAX_URL_LENGTH, "URL") {
                return McpResponse::error(-32602, format!("Input validation failed: {}", e));
            }
        }
        if let Some(sid) = session_id {
            if let Err(e) = sanitize_session_id(sid) {
                return McpResponse::error(-32602, format!("Session ID validation failed: {}", e));
            }
        }

        // Session & Navigation options
        let wait_for_js = arguments.get("wait_for_js").and_then(|v| v.as_bool()).unwrap_or(false);
        let _wait_timeout = arguments.get("wait_timeout").and_then(|v| v.as_u64()).unwrap_or(5000);

        // What to extract (all default to true for comprehensive extraction)
        let extract_basic = arguments.get("extract_basic").and_then(|v| v.as_bool()).unwrap_or(true);
        let extract_readable = arguments.get("extract_readable").and_then(|v| v.as_bool()).unwrap_or(false);
        let extract_structured = arguments.get("extract_structured").and_then(|v| v.as_bool()).unwrap_or(false);
        let extract_by_selectors = arguments.get("selectors").and_then(|v| v.as_object()).cloned();

        // Readability options
        let readability_format = arguments.get("format").and_then(|v| v.as_str()).unwrap_or("markdown");
        let include_images = arguments.get("include_images").and_then(|v| v.as_bool()).unwrap_or(true);
        let include_metadata = arguments.get("include_metadata").and_then(|v| v.as_bool()).unwrap_or(true);
        let min_content_score = arguments.get("min_content_score").and_then(|v| v.as_f64()).unwrap_or(0.3) as f32;

        // Structured content options
        let content_types = arguments.get("content_types")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["tables", "lists", "code_blocks", "metadata"]);

        // Pagination options (for future implementation)
        let _follow_pagination = arguments.get("follow_pagination").and_then(|v| v.as_bool()).unwrap_or(false);
        let _max_pages = arguments.get("max_pages").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

        // Navigate to URL if provided (or use existing session)
        let html_content = if let Some(url_str) = url {
            // SECURITY: Validate URL to prevent SSRF attacks (CWE-918)
            if let Err(e) = validate_url_for_navigation(url_str) {
                return McpResponse::error(-32602, format!("URL blocked for security: {}", e));
            }

            eprintln!("🔍 SCRAPE: Starting navigation to URL: {}", url_str);
            // Create temporary browser or use session
            let temp_browser = if let Some(sid) = session_id {
                // Try to get existing session browser
                if let Some(session_browser) = self.browser_tools.get_session_browser(sid) {
                    eprintln!("🔍 SCRAPE: Using existing session browser for session: {}", sid);
                    session_browser
                } else {
                    eprintln!("🔍 SCRAPE: Session {} not found, creating new browser", sid);
                    crate::engine::browser::HeadlessWebBrowser::new()
                }
            } else {
                eprintln!("🔍 SCRAPE: Creating temporary browser");
                crate::engine::browser::HeadlessWebBrowser::new()
            };

            eprintln!("🔍 SCRAPE: Browser created");

            // Navigate to URL
            {
                eprintln!("🔍 SCRAPE: Acquiring browser lock for navigation");
                let mut browser = match temp_browser.lock() {
                    Ok(b) => {
                        eprintln!("🔍 SCRAPE: Browser lock acquired");
                        b
                    }
                    Err(_) => {
                        eprintln!("🔍 SCRAPE: Failed to acquire browser lock");
                        return McpResponse::error(-1, "Failed to acquire browser lock".to_string());
                    }
                };

                eprintln!("🔍 SCRAPE: Calling navigate_to_with_options");
                match browser.navigate_to_with_options(url_str, wait_for_js).await {
                    Ok(_) => {
                        eprintln!("🔍 SCRAPE: Navigation successful");
                    },
                    Err(e) => {
                        eprintln!("🔍 SCRAPE: Navigation failed: {}", e);
                        return McpResponse::error(-1, format!("Failed to navigate to URL: {}", e));
                    }
                }
            }

            eprintln!("🔍 SCRAPE: Getting HTML content");
            // Get HTML content
            let html = {
                let browser = match temp_browser.lock() {
                    Ok(b) => b,
                    Err(_) => {
                        eprintln!("🔍 SCRAPE: Failed to acquire browser lock for content");
                        return McpResponse::error(-1, "Failed to acquire browser lock".to_string());
                    }
                };
                browser.get_current_content()
            };

            eprintln!("🔍 SCRAPE: HTML content retrieved, dropping browser");
            // Explicitly drop browser after getting content (Drop impl will handle cleanup)
            drop(temp_browser);
            eprintln!("🔍 SCRAPE: Browser dropped");

            html
        } else {
            // Get content from existing session
            let session_id_str = session_id.unwrap(); // We know it exists from earlier check
            eprintln!("🔍 SCRAPE: Getting content from session: {}", session_id_str);

            match self.browser_tools.get_session_browser(session_id_str) {
                Some(browser) => {
                    match browser.lock() {
                        Ok(browser_guard) => {
                            let content = browser_guard.get_current_content();
                            if content.is_empty() {
                                return McpResponse::error(
                                    -1,
                                    format!("Session '{}' has no content. Navigate to a URL first.", session_id_str)
                                );
                            }
                            eprintln!("🔍 SCRAPE: Got {} chars from session", content.len());
                            content
                        }
                        Err(_) => {
                            return McpResponse::error(-1, "Failed to acquire session browser lock".to_string());
                        }
                    }
                }
                None => {
                    return McpResponse::error(
                        -1,
                        format!("Session '{}' not found. Create a session first using browser_session_management.", session_id_str)
                    );
                }
            }
        };

        // Build unified response
        let mut result = serde_json::json!({
            "url": url.unwrap_or(""),
            "scraping_options": {
                "extract_basic": extract_basic,
                "extract_readable": extract_readable,
                "extract_structured": extract_structured,
                "has_custom_selectors": extract_by_selectors.is_some()
            }
        });

        // 1. Extract basic content (links, images, metadata)
        if extract_basic {
            let document = Html::parse_document(&html_content);

            // Extract links
            let mut links = Vec::new();
            if let Ok(link_selector) = Selector::parse("a[href]") {
                for element in document.select(&link_selector) {
                    if let Some(href) = element.value().attr("href") {
                        let text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                        links.push(serde_json::json!({
                            "href": href,
                            "text": text
                        }));
                    }
                }
            }

            // Extract images
            let mut images = Vec::new();
            if let Ok(img_selector) = Selector::parse("img[src]") {
                for element in document.select(&img_selector) {
                    if let Some(src) = element.value().attr("src") {
                        let alt = element.value().attr("alt").unwrap_or("");
                        images.push(serde_json::json!({
                            "src": src,
                            "alt": alt
                        }));
                    }
                }
            }

            // Extract basic metadata
            let metadata = extraction::extract_metadata(&html_content);

            result["basic"] = serde_json::json!({
                "links": links,
                "images": images,
                "metadata": metadata
            });
        }

        // 2. Extract content by custom selectors
        if let Some(selectors) = extract_by_selectors {
            let document = Html::parse_document(&html_content);
            let mut selector_results = serde_json::Map::new();

            for (name, selector_str) in selectors {
                if let Some(selector_str) = selector_str.as_str() {
                    // SECURITY: Validate selector length
                    if let Err(e) = limit_input_length(selector_str, MAX_SELECTOR_LENGTH, "CSS selector") {
                        selector_results.insert(
                            name,
                            Value::String(format!("Selector rejected: {}", e))
                        );
                        continue;
                    }
                    if let Ok(selector) = Selector::parse(selector_str) {
                        let mut matches = Vec::new();
                        for element in document.select(&selector) {
                            let text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                            matches.push(Value::String(text));
                        }
                        selector_results.insert(name, Value::Array(matches));
                    } else {
                        selector_results.insert(
                            name,
                            Value::String(format!("Invalid selector: {}", selector_str))
                        );
                    }
                }
            }

            result["by_selector"] = Value::Object(selector_results);
        }

        // 3. Extract readable content using readability algorithms
        if extract_readable {
            let output_format = match readability_format {
                "markdown" => crate::features::readability::OutputFormat::Markdown,
                "text" => crate::features::readability::OutputFormat::Text,
                "structured" => crate::features::readability::OutputFormat::Structured,
                _ => crate::features::readability::OutputFormat::Markdown,
            };

            let mut extractor = crate::features::readability::ReadabilityExtractor::new();
            let document = scraper::Html::parse_document(&html_content);

            let options = crate::features::readability::ExtractionOptions {
                base_url: url.unwrap_or("").to_string(),
                include_images,
                include_metadata,
                output_format,
                min_content_score,
                max_link_density: 0.25,
                min_paragraph_count: 2,
            };

            // Check if content is likely plain text/code (minimal HTML structure)
            let is_plain_text_content = Self::is_plain_text_content(&html_content);

            match extractor.extract(&document, &options) {
                Ok(extraction_result) => {
                    if extraction_result.success {
                        result["readable"] = serde_json::json!({
                            "content": extraction_result.content.content,
                            "format": readability_format,
                            "metadata": extraction_result.content.metadata,
                            "quality": extraction_result.quality,
                            "processing_time_ms": extraction_result.processing_time_ms
                        });
                    } else if is_plain_text_content {
                        // Fallback: for plain text/code content, return raw text
                        result["readable"] = serde_json::json!({
                            "content": Self::extract_plain_text(&html_content),
                            "format": "text",
                            "is_plain_text_fallback": true,
                            "note": "Content detected as plain text/code - HTML readability extraction not applicable"
                        });
                    } else {
                        result["readable"] = serde_json::json!({
                            "error": extraction_result.error.unwrap_or("Extraction failed".to_string())
                        });
                    }
                },
                Err(e) => {
                    if is_plain_text_content {
                        // Fallback: for plain text/code content, return raw text
                        result["readable"] = serde_json::json!({
                            "content": Self::extract_plain_text(&html_content),
                            "format": "text",
                            "is_plain_text_fallback": true,
                            "note": "Content detected as plain text/code - HTML readability extraction not applicable"
                        });
                    } else {
                        result["readable"] = serde_json::json!({
                            "error": format!("Readability extraction failed: {}", e)
                        });
                    }
                }
            }
        }

        // 4. Extract structured content (tables, lists, code blocks)
        if extract_structured {
            let mut structured = serde_json::json!({});

            for content_type in &content_types {
                match content_type.as_ref() {
                    "tables" => {
                        let tables = extraction::extract_tables(&html_content);
                        structured["tables"] = Value::Array(tables);
                    },
                    "lists" => {
                        let lists = extraction::extract_lists(&html_content);
                        structured["lists"] = Value::Array(lists);
                    },
                    "code_blocks" => {
                        let code_blocks = extraction::extract_code_blocks(&html_content);
                        structured["code_blocks"] = Value::Array(code_blocks);
                    },
                    "metadata" => {
                        let metadata = extraction::extract_metadata(&html_content);
                        structured["metadata"] = metadata;
                    },
                    _ => {}
                }
            }

            // Add summary
            let mut summary = serde_json::json!({
                "total_tables": 0,
                "total_lists": 0,
                "total_code_blocks": 0,
                "has_metadata": false
            });

            if let Some(tables) = structured["tables"].as_array() {
                summary["total_tables"] = Value::Number(serde_json::Number::from(tables.len()));
            }
            if let Some(lists) = structured["lists"].as_array() {
                summary["total_lists"] = Value::Number(serde_json::Number::from(lists.len()));
            }
            if let Some(code_blocks) = structured["code_blocks"].as_array() {
                summary["total_code_blocks"] = Value::Number(serde_json::Number::from(code_blocks.len()));
            }
            if let Some(metadata) = structured["metadata"].as_object() {
                summary["has_metadata"] = Value::Bool(!metadata.is_empty());
            }

            structured["summary"] = summary;
            result["structured"] = structured;
        }

        // Wrap result in MCP text content format
        let mcp_content = serde_json::json!({
            "type": "text",
            "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
        });
        McpResponse::success(mcp_content)
    }

    /// Detect if content is likely plain text/code (minimal HTML structure)
    ///
    /// Returns true if:
    /// - Content has very few HTML tags
    /// - Content lacks typical HTML structure (html, body, div, p tags)
    /// - Content appears to be source code or plain text
    fn is_plain_text_content(content: &str) -> bool {
        let content_trimmed = content.trim();

        // If content starts with common code patterns, it's likely plain text
        let code_indicators = [
            "import ", "export ", "const ", "let ", "var ", "function ", "class ",
            "use ", "mod ", "pub ", "fn ", "struct ", "impl ", "enum ",  // Rust
            "#include", "#define", "int ", "void ", "char ",  // C/C++
            "package ", "interface ",  // Go/Java
            "def ", "from ", "class ",  // Python
            "<?php", "<?=",  // PHP
            "#!/", "#!", "---",  // Shell/YAML
        ];

        for indicator in &code_indicators {
            if content_trimmed.starts_with(indicator) {
                return true;
            }
        }

        // Count HTML-specific tags
        let html_tags = ["<html", "<body", "<div", "<p>", "<article", "<section",
                         "<main", "<header", "<footer", "<nav", "<aside"];

        let mut html_tag_count = 0;
        for tag in &html_tags {
            if content.to_lowercase().contains(tag) {
                html_tag_count += 1;
            }
        }

        // If we have very few HTML structural tags (0-1), likely plain text
        // Also check the ratio of < characters to total content
        let angle_bracket_count = content.matches('<').count();
        let content_len = content.len();

        // Plain text/code typically has < for comparisons, but not many
        // HTML has lots of < for tags
        let angle_bracket_ratio = if content_len > 0 {
            angle_bracket_count as f32 / content_len as f32
        } else {
            0.0
        };

        // If less than 1 HTML tag per 500 chars on average, likely plain text
        // Or if we found no structural HTML tags
        html_tag_count == 0 && angle_bracket_ratio < 0.02
    }

    /// Extract plain text from content, handling both HTML and raw text
    fn extract_plain_text(content: &str) -> String {
        let content_trimmed = content.trim();

        // If it looks like it might have some HTML, try to parse it
        if content_trimmed.contains("</") || content_trimmed.contains("/>") {
            let document = Html::parse_document(content);
            let text = document.root_element().text().collect::<Vec<_>>().join("");
            let cleaned = text.trim().to_string();

            // If parsing resulted in meaningful content, use it
            if !cleaned.is_empty() {
                return cleaned;
            }
        }

        // Otherwise return the raw content (it's already plain text)
        content_trimmed.to_string()
    }
}
