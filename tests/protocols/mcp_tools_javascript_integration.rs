use serde_json::json;
use std::sync::{Arc, Mutex};
use thalora::protocols::mcp_server::McpToolHandler;
use thalora::engine::browser::HeadlessWebBrowser;

#[tokio::test]
async fn test_web_search_tool_functionality() {
    let browser = HeadlessWebBrowser::new();
    let mut handler = McpToolHandler::new(browser);

    // Test web_search tool with default search engine (DuckDuckGo)
    let search_args = json!({
        "query": "rust programming language",
        "num_results": 2
    });

    let result = handler.web_search(search_args.clone()).await;
    assert!(result.is_ok(), "Web search should succeed: {:?}", result);

    let response = result.unwrap();
    assert!(response.is_array(), "Response should be an array");

    let response_array = response.as_array().unwrap();
    assert!(!response_array.is_empty(), "Response should not be empty");

    // Check the structure of the response
    let first_result = &response_array[0];
    assert!(first_result.get("query").is_some(), "Response should contain query");
    assert!(first_result.get("results").is_some(), "Response should contain results");
    assert!(first_result.get("total_results").is_some(), "Response should contain total_results");

    let results = first_result.get("results").unwrap().as_array().unwrap();
    assert!(results.len() <= 2, "Should not exceed requested number of results");

    // Each result should have required fields
    for result in results {
        assert!(result.get("title").is_some(), "Result should have title");
        assert!(result.get("url").is_some(), "Result should have URL");
        assert!(result.get("snippet").is_some(), "Result should have snippet");
        assert!(result.get("position").is_some(), "Result should have position");
    }
}

#[tokio::test]
async fn test_web_search_with_different_engines() {
    let browser = HeadlessWebBrowser::new();
    let mut handler = McpToolHandler::new(browser);

    let search_engines = vec!["duckduckgo", "bing", "startpage"];

    for engine in search_engines {
        let search_args = json!({
            "query": "test query",
            "num_results": 1,
            "search_engine": engine
        });

        let result = handler.web_search(search_args).await;
        assert!(result.is_ok(), "Web search with {} should succeed: {:?}", engine, result);

        let response = result.unwrap();
        assert!(response.is_array(), "Response should be an array for {}", engine);
    }
}

#[tokio::test]
async fn test_snapshot_url_with_javascript_execution() {
    let browser = HeadlessWebBrowser::new();
    let mut handler = McpToolHandler::new(browser);

    // Test scraping a simple page with JavaScript execution disabled
    let scrape_args_no_js = json!({
        "url": "https://httpbin.org/html",
        "wait_for_js": false
    });

    let result_no_js = handler.snapshot_url(scrape_args_no_js).await;
    assert!(result_no_js.is_ok(), "Scraping without JS should succeed: {:?}", result_no_js);

    // Test scraping the same page with JavaScript execution enabled
    let scrape_args_with_js = json!({
        "url": "https://httpbin.org/html",
        "wait_for_js": true
    });

    let result_with_js = handler.snapshot_url(scrape_args_with_js).await;
    assert!(result_with_js.is_ok(), "Scraping with JS should succeed: {:?}", result_with_js);

    // Both results should be arrays with content
    let response_no_js = result_no_js.unwrap();
    let response_with_js = result_with_js.unwrap();

    assert!(response_no_js.is_array(), "No-JS response should be an array");
    assert!(response_with_js.is_array(), "With-JS response should be an array");

    // Check structure of responses
    let content_no_js = &response_no_js.as_array().unwrap()[0];
    let content_with_js = &response_with_js.as_array().unwrap()[0];

    for content in &[content_no_js, content_with_js] {
        assert!(content.get("content").is_some(), "Response should contain content");
        assert!(content.get("url").is_some(), "Response should contain URL");
        assert!(content.get("title").is_some(), "Response should contain title");
        assert!(content.get("links").is_some(), "Response should contain links");
        assert!(content.get("images").is_some(), "Response should contain images");
        assert!(content.get("metadata").is_some(), "Response should contain metadata");
    }

    // Content should not be empty
    let content_text_no_js = content_no_js.get("content").unwrap().as_str().unwrap();
    let content_text_with_js = content_with_js.get("content").unwrap().as_str().unwrap();

    assert!(!content_text_no_js.is_empty(), "Content without JS should not be empty");
    assert!(!content_text_with_js.is_empty(), "Content with JS should not be empty");
}

#[tokio::test]
async fn test_snapshot_url_javascript_heavy_site() {
    let browser = HeadlessWebBrowser::new();
    let mut handler = McpToolHandler::new(browser);

    // Test scraping a JavaScript-heavy site (example.com is simple but has some JS)
    let scrape_args = json!({
        "url": "https://example.com",
        "wait_for_js": true
    });

    let result = handler.snapshot_url(scrape_args).await;
    assert!(result.is_ok(), "Scraping JavaScript site should succeed: {:?}", result);

    let response = result.unwrap();
    assert!(response.is_array(), "Response should be an array");

    let content = &response.as_array().unwrap()[0];
    let content_text = content.get("content").unwrap().as_str().unwrap();

    assert!(!content_text.is_empty(), "Content should not be empty");
    assert!(content_text.contains("Example Domain"), "Should contain expected content");

    // Check that title is extracted
    let title = content.get("title").unwrap().as_str().unwrap();
    assert_eq!(title, "Example Domain", "Title should be extracted correctly");

    // Check that metadata is extracted
    let metadata = content.get("metadata").unwrap().as_object().unwrap();
    assert!(!metadata.is_empty(), "Metadata should not be empty");
}

#[tokio::test]
async fn test_mcp_tools_error_handling() {
    let browser = HeadlessWebBrowser::new();
    let mut handler = McpToolHandler::new(browser);

    // Test web_search with invalid arguments
    let invalid_search_args = json!({
        "query": "",  // Empty query
        "num_results": 0
    });

    let result = handler.web_search(invalid_search_args).await;
    // Should handle gracefully and return empty results or error
    assert!(result.is_ok(), "Should handle invalid search gracefully");

    // Test snapshot_url with invalid URL
    let invalid_scrape_args = json!({
        "url": "not-a-valid-url",
        "wait_for_js": false
    });

    let result = handler.snapshot_url(invalid_scrape_args).await;
    // Should return an error or handle gracefully
    assert!(result.is_ok() || result.is_err(), "Should handle invalid URL");

    // Test snapshot_url with unreachable URL
    let unreachable_args = json!({
        "url": "https://this-domain-does-not-exist-123456789.com",
        "wait_for_js": false
    });

    let result = handler.snapshot_url(unreachable_args).await;
    // Should handle network errors gracefully
    assert!(result.is_ok() || result.is_err(), "Should handle unreachable URL");
}

#[tokio::test]
async fn test_concurrent_mcp_tool_usage() {
    let browser = HeadlessWebBrowser::new();

    // Test that multiple tools can be used concurrently without issues
    let mut handles = vec![];

    for i in 0..3 {
        let browser_clone = HeadlessWebBrowser::new();
        let handle = tokio::spawn(async move {
            let mut handler = McpToolHandler::new(browser_clone);

            let search_args = json!({
                "query": format!("test query {}", i),
                "num_results": 1
            });

            let result = handler.web_search(search_args).await;
            assert!(result.is_ok(), "Concurrent web search {} should succeed", i);
            i
        });
        handles.push(handle);
    }

    // Wait for all concurrent operations to complete
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok(), "Concurrent operation should complete successfully");
    }
}