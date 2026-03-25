use scraper::{Html, Selector};
use serde_json::Value;

use crate::protocols::mcp::McpResponse;
use crate::protocols::mcp_server::core::McpServer;
use crate::protocols::security::{
    MAX_SELECTOR_LENGTH, MAX_URL_LENGTH, limit_input_length, sanitize_session_id,
    validate_url_for_navigation,
};

use super::extraction;

impl McpServer {
    /// Capture a point-in-time snapshot of a web page with all extraction capabilities
    pub(in crate::protocols::mcp_server) async fn handle_snapshot_url(
        &mut self,
        arguments: Value,
    ) -> McpResponse {
        let url = arguments["url"].as_str();
        let session_id = arguments.get("session_id").and_then(|v| v.as_str());

        // Validate that we have either URL or session_id
        if url.is_none() && session_id.is_none() {
            return McpResponse::error(
                -1,
                "Either 'url' or 'session_id' parameter is required".to_string(),
            );
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
        let wait_for_js = arguments
            .get("wait_for_js")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let _wait_timeout = arguments
            .get("wait_timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(5000);

        // What to extract (all default to true for comprehensive extraction)
        let extract_basic = arguments
            .get("extract_basic")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let extract_readable = arguments
            .get("extract_readable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let extract_structured = arguments
            .get("extract_structured")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let extract_by_selectors = arguments
            .get("selectors")
            .and_then(|v| v.as_object())
            .cloned();

        // Readability options
        let readability_format = arguments
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("markdown");
        let include_images = arguments
            .get("include_images")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let include_metadata = arguments
            .get("include_metadata")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let min_content_score = arguments
            .get("min_content_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.3) as f32;

        // Structured content options
        let content_types = arguments
            .get("content_types")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["tables", "lists", "code_blocks", "metadata"]);

        // Output size limit
        let max_output_size = arguments
            .get("max_output_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(50_000) as usize;

        // Pagination options (for future implementation)
        let _follow_pagination = arguments
            .get("follow_pagination")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let _max_pages = arguments
            .get("max_pages")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        // Navigate to URL if provided (or use existing session)
        let html_content = if let Some(url_str) = url {
            // SECURITY: Validate URL to prevent SSRF attacks (CWE-918)
            if let Err(e) = validate_url_for_navigation(url_str) {
                return McpResponse::error(-32602, format!("URL blocked for security: {}", e));
            }

            eprintln!("🔍 SNAPSHOT: Starting navigation to URL: {}", url_str);
            // Create temporary browser or use session
            let temp_browser = if let Some(sid) = session_id {
                // Try to get existing session browser
                if let Some(session_browser) = self.browser_tools.get_session_browser(sid) {
                    eprintln!(
                        "🔍 SNAPSHOT: Using existing session browser for session: {}",
                        sid
                    );
                    session_browser
                } else {
                    eprintln!(
                        "🔍 SNAPSHOT: Session {} not found, creating new browser",
                        sid
                    );
                    crate::engine::browser::HeadlessWebBrowser::new()
                }
            } else {
                eprintln!("🔍 SNAPSHOT: Creating temporary browser");
                crate::engine::browser::HeadlessWebBrowser::new()
            };

            eprintln!("🔍 SNAPSHOT: Browser created");

            // Navigate to URL
            {
                eprintln!("🔍 SNAPSHOT: Acquiring browser lock for navigation");
                let mut browser = match temp_browser.lock() {
                    Ok(b) => {
                        eprintln!("🔍 SNAPSHOT: Browser lock acquired");
                        b
                    }
                    Err(_) => {
                        eprintln!("🔍 SNAPSHOT: Failed to acquire browser lock");
                        return McpResponse::error(
                            -1,
                            "Failed to acquire browser lock".to_string(),
                        );
                    }
                };

                eprintln!("🔍 SNAPSHOT: Calling navigate_to_with_options");
                match browser.navigate_to_with_options(url_str, wait_for_js).await {
                    Ok(_) => {
                        eprintln!("🔍 SNAPSHOT: Navigation successful");
                    }
                    Err(e) => {
                        eprintln!("🔍 SNAPSHOT: Navigation failed: {}", e);
                        return McpResponse::error(-1, format!("Failed to navigate to URL: {}", e));
                    }
                }
            }

            eprintln!("🔍 SNAPSHOT: Getting HTML content");
            // Get HTML content
            let html = {
                let browser = match temp_browser.lock() {
                    Ok(b) => b,
                    Err(_) => {
                        eprintln!("🔍 SNAPSHOT: Failed to acquire browser lock for content");
                        return McpResponse::error(
                            -1,
                            "Failed to acquire browser lock".to_string(),
                        );
                    }
                };
                browser.get_current_content()
            };

            eprintln!("🔍 SNAPSHOT: HTML content retrieved, dropping browser");
            // Explicitly drop browser after getting content (Drop impl will handle cleanup)
            drop(temp_browser);
            eprintln!("🔍 SNAPSHOT: Browser dropped");

            html
        } else {
            // Get content from existing session
            let session_id_str = session_id.unwrap(); // We know it exists from earlier check
            eprintln!(
                "🔍 SNAPSHOT: Getting content from session: {}",
                session_id_str
            );

            match self.browser_tools.get_session_browser(session_id_str) {
                Some(browser) => match browser.lock() {
                    Ok(browser_guard) => {
                        let content = browser_guard.get_current_content();
                        if content.is_empty() {
                            return McpResponse::error(
                                -1,
                                format!(
                                    "Session '{}' has no content. Navigate to a URL first.",
                                    session_id_str
                                ),
                            );
                        }
                        eprintln!("🔍 SNAPSHOT: Got {} chars from session", content.len());
                        content
                    }
                    Err(_) => {
                        return McpResponse::error(
                            -1,
                            "Failed to acquire session browser lock".to_string(),
                        );
                    }
                },
                None => {
                    return McpResponse::error(
                        -1,
                        format!(
                            "Session '{}' not found. Create a session first using browser_session_management.",
                            session_id_str
                        ),
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
                        let text = element
                            .text()
                            .collect::<Vec<_>>()
                            .join(" ")
                            .trim()
                            .to_string();
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
                    if let Err(e) =
                        limit_input_length(selector_str, MAX_SELECTOR_LENGTH, "CSS selector")
                    {
                        selector_results
                            .insert(name, Value::String(format!("Selector rejected: {}", e)));
                        continue;
                    }
                    if let Ok(selector) = Selector::parse(selector_str) {
                        let mut matches = Vec::new();
                        for element in document.select(&selector) {
                            let text = element
                                .text()
                                .collect::<Vec<_>>()
                                .join(" ")
                                .trim()
                                .to_string();
                            matches.push(Value::String(text));
                        }
                        selector_results.insert(name, Value::Array(matches));
                    } else {
                        selector_results.insert(
                            name,
                            Value::String(format!("Invalid selector: {}", selector_str)),
                        );
                    }
                }
            }

            result["by_selector"] = Value::Object(selector_results);
        }

        // 3. Extract structured content (tables, lists, code blocks)
        //    Done BEFORE readable extraction so structured data is available as fallback
        let mut structured_data: Option<Value> = None;

        if extract_structured || extract_readable {
            // When extract_readable is requested, we auto-extract tables + lists
            // for fallback purposes even if extract_structured wasn't explicitly requested
            let mut structured = serde_json::json!({});

            let types_to_extract = if extract_structured {
                content_types.clone()
            } else {
                // For fallback only: extract tables and lists
                vec!["tables", "lists"]
            };

            for content_type in &types_to_extract {
                match content_type.as_ref() {
                    "tables" => {
                        let tables = extraction::extract_tables(&html_content);
                        structured["tables"] = Value::Array(tables);
                    }
                    "lists" => {
                        let lists = extraction::extract_lists(&html_content);
                        structured["lists"] = Value::Array(lists);
                    }
                    "code_blocks" => {
                        let code_blocks = extraction::extract_code_blocks(&html_content);
                        structured["code_blocks"] = Value::Array(code_blocks);
                    }
                    "metadata" => {
                        let metadata = extraction::extract_metadata(&html_content);
                        structured["metadata"] = metadata;
                    }
                    _ => {}
                }
            }

            // Store for potential fallback use
            structured_data = Some(structured.clone());

            // Only include structured in output if explicitly requested
            if extract_structured {
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
                    summary["total_code_blocks"] =
                        Value::Number(serde_json::Number::from(code_blocks.len()));
                }
                if let Some(metadata) = structured["metadata"].as_object() {
                    summary["has_metadata"] = Value::Bool(!metadata.is_empty());
                }

                structured["summary"] = summary;
                result["structured"] = structured;
            }
        }

        // 4. Extract readable content using readability algorithms
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

            let mut readability_succeeded = false;

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
                        readability_succeeded = true;
                    } else if is_plain_text_content {
                        // Fallback: for plain text/code content, return raw text
                        result["readable"] = serde_json::json!({
                            "content": Self::extract_plain_text(&html_content),
                            "format": "text",
                            "is_plain_text_fallback": true,
                            "note": "Content detected as plain text/code - HTML readability extraction not applicable"
                        });
                        readability_succeeded = true;
                    }
                }
                Err(_) => {
                    if is_plain_text_content {
                        // Fallback: for plain text/code content, return raw text
                        result["readable"] = serde_json::json!({
                            "content": Self::extract_plain_text(&html_content),
                            "format": "text",
                            "is_plain_text_fallback": true,
                            "note": "Content detected as plain text/code - HTML readability extraction not applicable"
                        });
                        readability_succeeded = true;
                    }
                }
            }

            // Fallback: synthesize readable content from structured data if readability failed
            if !readability_succeeded {
                if let Some(ref structured) = structured_data {
                    let metadata_val = result.get("basic").and_then(|b| b.get("metadata"));
                    let synthesized = Self::synthesize_readable_from_structured(
                        structured,
                        metadata_val,
                        readability_format,
                    );
                    if !synthesized.is_empty() {
                        result["readable"] = serde_json::json!({
                            "content": synthesized,
                            "format": readability_format,
                            "is_structured_fallback": true,
                            "note": "Readable content synthesized from structured data (tables/lists)"
                        });
                    } else {
                        result["readable"] = serde_json::json!({
                            "error": "Readability extraction failed and no structured data available for fallback"
                        });
                    }
                } else {
                    result["readable"] = serde_json::json!({
                        "error": "Readability extraction failed"
                    });
                }
            }
        }

        // Enforce output size limit with tiered truncation
        if max_output_size > 0 {
            Self::enforce_output_limit(&mut result, max_output_size);
        }

        // Use compact JSON when output is large, pretty JSON when small
        let result_text = if max_output_size > 0 {
            let compact = serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string());
            if compact.len() > 20_000 {
                compact
            } else {
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
            }
        } else {
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
        };

        // Wrap result in MCP text content format
        let mcp_content = serde_json::json!({
            "type": "text",
            "text": result_text
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
            "import ",
            "export ",
            "const ",
            "let ",
            "var ",
            "function ",
            "class ",
            "use ",
            "mod ",
            "pub ",
            "fn ",
            "struct ",
            "impl ",
            "enum ", // Rust
            "#include",
            "#define",
            "int ",
            "void ",
            "char ", // C/C++
            "package ",
            "interface ", // Go/Java
            "def ",
            "from ",
            "class ", // Python
            "<?php",
            "<?=", // PHP
            "#!/",
            "#!",
            "---", // Shell/YAML
        ];

        for indicator in &code_indicators {
            if content_trimmed.starts_with(indicator) {
                return true;
            }
        }

        // Count HTML-specific tags
        let html_tags = [
            "<html", "<body", "<div", "<p>", "<article", "<section", "<main", "<header", "<footer",
            "<nav", "<aside",
        ];

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

    /// Synthesize readable content from structured data (tables/lists)
    /// when readability extraction fails.
    fn synthesize_readable_from_structured(
        structured: &Value,
        metadata: Option<&Value>,
        format: &str,
    ) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Include page title/description from metadata if available
        if let Some(meta) = metadata {
            if let Some(title) = meta.get("title").and_then(|v| v.as_str()) {
                if !title.is_empty() {
                    match format {
                        "markdown" => parts.push(format!("# {}\n", title)),
                        _ => parts.push(format!("{}\n", title)),
                    }
                }
            }
            if let Some(desc) = meta.get("description").and_then(|v| v.as_str()) {
                if !desc.is_empty() {
                    parts.push(format!("{}\n", desc));
                }
            }
        }

        // Convert tables to markdown/text
        if let Some(tables) = structured.get("tables").and_then(|v| v.as_array()) {
            for table in tables {
                let mut table_lines: Vec<String> = Vec::new();

                // Headers
                if let Some(headers) = table.get("headers").and_then(|v| v.as_array()) {
                    let header_strs: Vec<&str> =
                        headers.iter().filter_map(|h| h.as_str()).collect();
                    if !header_strs.is_empty() {
                        match format {
                            "markdown" => {
                                table_lines.push(format!("| {} |", header_strs.join(" | ")));
                                let separator: Vec<&str> =
                                    header_strs.iter().map(|_| "---").collect();
                                table_lines.push(format!("| {} |", separator.join(" | ")));
                            }
                            _ => {
                                table_lines.push(header_strs.join("\t"));
                            }
                        }
                    }
                }

                // Rows
                if let Some(rows) = table.get("rows").and_then(|v| v.as_array()) {
                    for row in rows {
                        if let Some(cells) = row.as_array() {
                            let cell_strs: Vec<&str> =
                                cells.iter().filter_map(|c| c.as_str()).collect();
                            match format {
                                "markdown" => {
                                    table_lines.push(format!("| {} |", cell_strs.join(" | ")));
                                }
                                _ => {
                                    table_lines.push(cell_strs.join("\t"));
                                }
                            }
                        }
                    }
                }

                if !table_lines.is_empty() {
                    // Add caption if present
                    if let Some(caption) = table.get("caption").and_then(|v| v.as_str()) {
                        if !caption.is_empty() {
                            match format {
                                "markdown" => parts.push(format!("**{}**\n", caption)),
                                _ => parts.push(format!("{}\n", caption)),
                            }
                        }
                    }
                    parts.push(table_lines.join("\n"));
                    parts.push(String::new()); // blank line separator
                }
            }
        }

        // Convert lists to bullet points
        if let Some(lists) = structured.get("lists").and_then(|v| v.as_array()) {
            for list in lists {
                if let Some(items) = list.get("items").and_then(|v| v.as_array()) {
                    let mut list_lines: Vec<String> = Vec::new();
                    for item in items {
                        if let Some(text) = item.as_str() {
                            if !text.is_empty() {
                                match format {
                                    "markdown" => list_lines.push(format!("- {}", text)),
                                    _ => list_lines.push(format!("  * {}", text)),
                                }
                            }
                        } else if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                            if !text.is_empty() {
                                match format {
                                    "markdown" => list_lines.push(format!("- {}", text)),
                                    _ => list_lines.push(format!("  * {}", text)),
                                }
                            }
                        }
                    }
                    if !list_lines.is_empty() {
                        parts.push(list_lines.join("\n"));
                        parts.push(String::new()); // blank line separator
                    }
                }
            }
        }

        parts.join("\n").trim().to_string()
    }

    /// Enforce output size limit with tiered truncation.
    ///
    /// Progressively removes content in this order:
    /// - Tier 1: Truncate links to 20 and images to 10
    /// - Tier 2: Truncate tables to 10, rows per table to 50
    /// - Tier 3: Truncate readable content at paragraph/sentence boundary
    /// - Tier 4: Add truncation warning
    fn enforce_output_limit(result: &mut Value, max_size: usize) {
        let original_size = serde_json::to_string(result).map(|s| s.len()).unwrap_or(0);

        if original_size <= max_size {
            return;
        }

        // Tier 1: Truncate links and images
        if let Some(basic) = result.get_mut("basic") {
            if let Some(links) = basic.get_mut("links").and_then(|v| v.as_array_mut()) {
                if links.len() > 20 {
                    links.truncate(20);
                }
            }
            if let Some(images) = basic.get_mut("images").and_then(|v| v.as_array_mut()) {
                if images.len() > 10 {
                    images.truncate(10);
                }
            }
        }

        let current_size = serde_json::to_string(result).map(|s| s.len()).unwrap_or(0);
        if current_size <= max_size {
            result["_truncation_warning"] = serde_json::json!({
                "original_size": original_size,
                "final_size": current_size,
                "tiers_applied": ["links_images"]
            });
            return;
        }

        // Tier 2: Truncate tables
        if let Some(structured) = result.get_mut("structured") {
            if let Some(tables) = structured.get_mut("tables").and_then(|v| v.as_array_mut()) {
                if tables.len() > 10 {
                    tables.truncate(10);
                }
                for table in tables.iter_mut() {
                    if let Some(rows) = table.get_mut("rows").and_then(|v| v.as_array_mut()) {
                        if rows.len() > 50 {
                            rows.truncate(50);
                        }
                    }
                }
            }
        }

        let current_size = serde_json::to_string(result).map(|s| s.len()).unwrap_or(0);
        if current_size <= max_size {
            result["_truncation_warning"] = serde_json::json!({
                "original_size": original_size,
                "final_size": current_size,
                "tiers_applied": ["links_images", "tables"]
            });
            return;
        }

        // Tier 3: Truncate readable content at paragraph/sentence boundary
        if let Some(readable) = result.get_mut("readable") {
            if let Some(content) = readable
                .get_mut("content")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
            {
                let target_len = max_size / 2; // Use roughly half the budget for readable content
                if content.len() > target_len {
                    let truncated = Self::truncate_at_boundary(&content, target_len);
                    readable["content"] = Value::String(truncated);
                    readable["content_truncated"] = Value::Bool(true);
                }
            }
        }

        let final_size = serde_json::to_string(result).map(|s| s.len()).unwrap_or(0);
        result["_truncation_warning"] = serde_json::json!({
            "original_size": original_size,
            "final_size": final_size,
            "tiers_applied": ["links_images", "tables", "readable_content"]
        });
    }

    /// Truncate text at the nearest paragraph or sentence boundary
    fn truncate_at_boundary(text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            return text.to_string();
        }

        let search_region = &text[..max_len];

        // Try to find a paragraph break (double newline)
        if let Some(pos) = search_region.rfind("\n\n") {
            if pos > max_len / 2 {
                return format!("{}...", &text[..pos]);
            }
        }

        // Try to find a sentence break
        for delimiter in &[". ", ".\n", "! ", "? "] {
            if let Some(pos) = search_region.rfind(delimiter) {
                if pos > max_len / 2 {
                    return format!("{}...", &text[..pos + delimiter.len() - 1]);
                }
            }
        }

        // Fall back to a newline
        if let Some(pos) = search_region.rfind('\n') {
            if pos > max_len / 3 {
                return format!("{}...", &text[..pos]);
            }
        }

        // Last resort: truncate at max_len
        format!("{}...", &text[..max_len])
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
