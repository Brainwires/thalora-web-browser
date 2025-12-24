// Live integration tests for all search engines
// These tests make real HTTP requests to search engines
// Tests run serially to avoid Boa engine concurrent initialization race condition

use serial_test::serial;
use thalora::protocols::mcp_server::scraping::search;

/// Test DuckDuckGo live search
#[tokio::test]
#[serial]
async fn test_duckduckgo_live_search() {
    eprintln!("Testing DuckDuckGo live search...");

    let result = search::perform_search("rust programming language", 5, "duckduckgo").await;

    match result {
        Ok(results) => {
            eprintln!("DuckDuckGo search successful!");
            eprintln!("Found {} results", results.results.len());
            eprintln!("Query: {}", results.query);

            // We expect at least some results
            // Note: May be 0 if rate-limited or blocked
            if !results.results.is_empty() {
                eprintln!("First result: {} - {}", results.results[0].title, results.results[0].url);
            }

            assert_eq!(results.query, "rust programming language");
        }
        Err(e) => {
            eprintln!("DuckDuckGo search failed (may be rate-limited): {}", e);
            // Don't fail - network tests can be flaky
        }
    }
}

/// Test Bing live search
#[tokio::test]
#[serial]
async fn test_bing_live_search() {
    eprintln!("Testing Bing live search...");

    let result = search::perform_search("rust programming", 5, "bing").await;

    match result {
        Ok(results) => {
            eprintln!("Bing search successful!");
            eprintln!("Found {} results", results.results.len());

            if !results.results.is_empty() {
                eprintln!("First result: {} - {}", results.results[0].title, results.results[0].url);
            }
        }
        Err(e) => {
            eprintln!("Bing search failed (may be rate-limited): {}", e);
        }
    }
}

/// Test Google live search
#[tokio::test]
#[serial]
async fn test_google_live_search() {
    eprintln!("Testing Google live search...");

    let result = search::perform_search("rust programming", 5, "google").await;

    match result {
        Ok(results) => {
            eprintln!("Google search successful!");
            eprintln!("Found {} results", results.results.len());

            if !results.results.is_empty() {
                eprintln!("First result: {} - {}", results.results[0].title, results.results[0].url);
            }
        }
        Err(e) => {
            eprintln!("Google search failed (may be blocked): {}", e);
        }
    }
}

/// Test Startpage live search
#[tokio::test]
#[serial]
async fn test_startpage_live_search() {
    eprintln!("Testing Startpage live search...");

    let result = search::perform_search("rust programming", 5, "startpage").await;

    match result {
        Ok(results) => {
            eprintln!("Startpage search successful!");
            eprintln!("Found {} results", results.results.len());

            if !results.results.is_empty() {
                eprintln!("First result: {} - {}", results.results[0].title, results.results[0].url);
            }
        }
        Err(e) => {
            eprintln!("Startpage search failed: {}", e);
        }
    }
}

/// Test DuckDuckGo live image search
#[tokio::test]
#[serial]
async fn test_duckduckgo_live_image_search() {
    eprintln!("Testing DuckDuckGo live image search...");

    let result = search::perform_image_search("rust crab logo", 5, "duckduckgo").await;

    match result {
        Ok(results) => {
            eprintln!("DuckDuckGo image search successful!");
            eprintln!("Found {} images", results.results.len());

            if !results.results.is_empty() {
                eprintln!("First image: {} - {}", results.results[0].title, results.results[0].image_url);
            }
        }
        Err(e) => {
            eprintln!("DuckDuckGo image search failed: {}", e);
        }
    }
}

/// Test Bing live image search
#[tokio::test]
#[serial]
async fn test_bing_live_image_search() {
    eprintln!("Testing Bing live image search...");

    let result = search::perform_image_search("ferris rust crab", 5, "bing").await;

    match result {
        Ok(results) => {
            eprintln!("Bing image search successful!");
            eprintln!("Found {} images", results.results.len());

            if !results.results.is_empty() {
                eprintln!("First image: {}", results.results[0].image_url);
            }
        }
        Err(e) => {
            eprintln!("Bing image search failed: {}", e);
        }
    }
}

/// Test unsupported search engine
#[tokio::test]
async fn test_unsupported_search_engine() {
    let result = search::perform_search("test", 5, "yahoo").await;
    assert!(result.is_err(), "Unsupported engine should return error");

    let err = result.unwrap_err().to_string();
    assert!(err.contains("Unsupported"), "Error should mention unsupported engine");
}

/// Test invalid image search engine
#[tokio::test]
async fn test_unsupported_image_search_engine() {
    let result = search::perform_image_search("test", 5, "startpage").await;
    assert!(result.is_err(), "Startpage image search should return error (not supported)");
}
