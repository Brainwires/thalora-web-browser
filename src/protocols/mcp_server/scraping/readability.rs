use serde_json::Value;

use crate::protocols::mcp::McpResponse;
use crate::protocols::mcp_server::core::McpServer;

impl McpServer {
    /// Extract readable content using readability algorithm
    /// This is a dedicated method for the browse_readable_content MCP tool
    pub(in crate::protocols::mcp_server) async fn browse_readable_content(
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

        // Navigation options
        let wait_for_js = arguments
            .get("wait_for_js")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Readability options
        let format = arguments
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

        // Navigate to URL if provided (or use existing session)
        let html_content = if let Some(url_str) = url {
            // Create temporary browser
            let temp_browser = crate::engine::browser::HeadlessWebBrowser::new();

            // Navigate to URL
            {
                let mut browser = match temp_browser.lock() {
                    Ok(b) => b,
                    Err(_) => {
                        return McpResponse::error(
                            -1,
                            "Failed to acquire browser lock".to_string(),
                        );
                    }
                };

                match browser.navigate_to_with_options(url_str, wait_for_js).await {
                    Ok(_) => {}
                    Err(e) => {
                        return McpResponse::error(-1, format!("Failed to navigate to URL: {}", e));
                    }
                }
            }

            // Get HTML content
            let html = {
                let browser = match temp_browser.lock() {
                    Ok(b) => b,
                    Err(_) => {
                        return McpResponse::error(
                            -1,
                            "Failed to acquire browser lock".to_string(),
                        );
                    }
                };
                browser.get_current_content()
            };

            // Explicitly drop browser after getting content (Drop impl will handle cleanup)
            drop(temp_browser);

            html
        } else {
            // Get content from existing session
            let session_id_str = session_id.unwrap(); // We know it exists from earlier check

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
                            "Session '{}' not found. Create a session first.",
                            session_id_str
                        ),
                    );
                }
            }
        };

        // Extract readable content using readability algorithm
        let output_format = match format {
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

        match extractor.extract(&document, &options) {
            Ok(extraction_result) => {
                if extraction_result.success {
                    let result = serde_json::json!({
                        "url": url.unwrap_or(""),
                        "content": extraction_result.content.content,
                        "format": format,
                        "metadata": extraction_result.content.metadata,
                        "quality": extraction_result.quality,
                        "processing_time_ms": extraction_result.processing_time_ms,
                        "success": true
                    });
                    McpResponse::success(result)
                } else {
                    McpResponse::error(
                        -1,
                        extraction_result
                            .error
                            .unwrap_or("Extraction failed".to_string()),
                    )
                }
            }
            Err(e) => McpResponse::error(-1, format!("Readability extraction failed: {}", e)),
        }
    }
}
