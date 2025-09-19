// MCP Performance Tests - Benchmarking and stress testing the MCP server
use std::time::{Duration, Instant};
use serde_json::{json, Value};
use anyhow::Result;

use super::mcp_harness::*;

#[derive(Debug, Clone)]
struct PerformanceMetrics {
    pub operation: String,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub avg_duration: Duration,
    pub total_operations: usize,
    pub success_rate: f64,
    pub errors: Vec<String>,
}

impl PerformanceMetrics {
    pub fn new(operation: String) -> Self {
        Self {
            operation,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
            avg_duration: Duration::ZERO,
            total_operations: 0,
            success_rate: 0.0,
            errors: Vec::new(),
        }
    }

    pub fn add_measurement(&mut self, duration: Duration, success: bool, error: Option<String>) {
        self.total_operations += 1;

        if success {
            self.min_duration = self.min_duration.min(duration);
            self.max_duration = self.max_duration.max(duration);
        }

        if let Some(err) = error {
            self.errors.push(err);
        }
    }

    pub fn finalize(&mut self, total_duration: Duration) {
        let successful = self.total_operations - self.errors.len();
        self.success_rate = successful as f64 / self.total_operations as f64;

        if successful > 0 {
            self.avg_duration = total_duration / successful as u32;
        }

        if self.min_duration == Duration::MAX {
            self.min_duration = Duration::ZERO;
        }
    }

    pub fn print_summary(&self) {
        println!("\n=== Performance Metrics: {} ===", self.operation);
        println!("Total Operations: {}", self.total_operations);
        println!("Success Rate: {:.1}%", self.success_rate * 100.0);
        println!("Min Duration: {:?}", self.min_duration);
        println!("Max Duration: {:?}", self.max_duration);
        println!("Avg Duration: {:?}", self.avg_duration);
        if !self.errors.is_empty() {
            println!("Error Count: {}", self.errors.len());
            println!("Sample Errors: {:?}", &self.errors[..self.errors.len().min(3)]);
        }
        println!("=====================================\n");
    }
}

fn run_performance_test<F>(
    test_name: &str,
    iterations: usize,
    mut operation: F,
) -> PerformanceMetrics
where
    F: FnMut() -> Result<Duration>,
{
    let mut metrics = PerformanceMetrics::new(test_name.to_string());
    let total_start = Instant::now();

    println!("Running performance test: {} ({} iterations)", test_name, iterations);

    for i in 0..iterations {
        match operation() {
            Ok(duration) => {
                metrics.add_measurement(duration, true, None);
            }
            Err(e) => {
                metrics.add_measurement(Duration::ZERO, false, Some(e.to_string()));
            }
        }

        // Progress indicator for long tests
        if i % (iterations / 10).max(1) == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }

    let total_duration = total_start.elapsed();
    metrics.finalize(total_duration);
    println!(); // New line after progress dots

    metrics
}

#[test]
fn test_initialization_performance() {
    let iterations = 10;

    let metrics = run_performance_test("MCP Initialization", iterations, || {
        let start = Instant::now();
        let mut harness = McpTestHarness::new()?;
        harness.initialize()?;
        Ok(start.elapsed())
    });

    metrics.print_summary();

    // Performance assertions
    assert!(metrics.success_rate >= 0.8, "Should have at least 80% success rate");
    assert!(metrics.avg_duration < Duration::from_secs(5), "Average initialization should be under 5 seconds");
}

#[test]
fn test_tools_list_performance() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");
    let iterations = 50;

    let metrics = run_performance_test("Tools List", iterations, || {
        let start = Instant::now();
        harness.list_tools()?;
        Ok(start.elapsed())
    });

    metrics.print_summary();

    // Performance assertions
    assert!(metrics.success_rate >= 0.95, "Tools list should have high success rate");
    assert!(metrics.avg_duration < Duration::from_secs(1), "Tools list should be very fast");
    assert!(metrics.max_duration < Duration::from_secs(5), "No tools list call should take more than 5 seconds");
}

#[test]
fn test_ai_memory_performance() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");
    let iterations = 30;

    // Test store performance
    let store_metrics = {
        let mut metrics = PerformanceMetrics::new("AI Memory Store".to_string());
        let total_start = Instant::now();

        for i in 0..iterations {
            let key = format!("perf_test_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default());
            let data = json!({
                "test_data": "performance test data",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "iteration": rand::random::<u32>()
            });

            let start = Instant::now();
            let result = harness.call_tool("ai_memory_store_research", json!({
                "key": key,
                "topic": "performance testing",
                "summary": "Performance test data for AI memory benchmarking",
                "tags": ["performance", "test"]
            }));

            match result {
                Ok(response) => {
                    if response.is_error {
                        metrics.add_measurement(Duration::ZERO, false, Some("Store operation returned error".to_string()));
                    } else {
                        metrics.add_measurement(start.elapsed(), true, None);
                    }
                }
                Err(e) => {
                    metrics.add_measurement(Duration::ZERO, false, Some(e.to_string()));
                }
            }

            if i % (iterations / 10).max(1) == 0 {
                print!(".");
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }

        let total_duration = total_start.elapsed();
        metrics.finalize(total_duration);
        println!();
        metrics
    };

    store_metrics.print_summary();

    // Test retrieval performance
    let retrieve_metrics = {
        let mut metrics = PerformanceMetrics::new("AI Memory Retrieve".to_string());
        let total_start = Instant::now();

        for i in 0..iterations {
            let start = Instant::now();
            let result = harness.call_tool("ai_memory_search_research", json!({
                "tags": ["performance"],
                "limit": 1
            }));

            match result {
                Ok(response) => {
                    if response.is_error {
                        metrics.add_measurement(Duration::ZERO, false, Some("Search operation returned error".to_string()));
                    } else {
                        metrics.add_measurement(start.elapsed(), true, None);
                    }
                }
                Err(e) => {
                    metrics.add_measurement(Duration::ZERO, false, Some(e.to_string()));
                }
            }

            if i % (iterations / 10).max(1) == 0 {
                print!(".");
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }

        let total_duration = total_start.elapsed();
        metrics.finalize(total_duration);
        println!();
        metrics
    };

    retrieve_metrics.print_summary();

    // Performance assertions
    assert!(store_metrics.success_rate >= 0.9, "Store operations should have high success rate");
    assert!(retrieve_metrics.success_rate >= 0.9, "Retrieve operations should have high success rate");
    assert!(store_metrics.avg_duration < Duration::from_secs(2), "Store should be fast");
    assert!(retrieve_metrics.avg_duration < Duration::from_secs(2), "Retrieve should be fast");
}

#[test]
fn test_javascript_evaluation_performance() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");
    let iterations = 20;

    // Test simple expressions
    let simple_metrics = run_performance_test("JavaScript Simple", iterations, || {
        let expressions = vec![
            "1 + 1",
            "Math.random()",
            "new Date().toISOString()",
            "'hello'.toUpperCase()",
            "JSON.stringify({test: true})",
        ];

        let expr = expressions[rand::random::<usize>() % expressions.len()];
        let start = Instant::now();

        let response = harness.call_tool("cdp_runtime_evaluate", json!({
            "expression": expr,
            "await_promise": false
        }))?;

        if response.is_error {
            anyhow::bail!("JavaScript evaluation returned error");
        }

        Ok(start.elapsed())
    });

    simple_metrics.print_summary();

    // Test complex expressions
    let complex_metrics = run_performance_test("JavaScript Complex", iterations / 2, || {
        let complex_expr = r#"
            const data = Array.from({length: 1000}, (_, i) => ({
                id: i,
                value: Math.random(),
                timestamp: new Date().toISOString()
            }));

            const processed = data
                .filter(item => item.value > 0.5)
                .map(item => ({...item, doubled: item.value * 2}))
                .slice(0, 10);

            JSON.stringify({
                totalItems: data.length,
                processedItems: processed.length,
                sample: processed[0]
            });
        "#;

        let start = Instant::now();

        let response = harness.call_tool("cdp_runtime_evaluate", json!({
            "expression": complex_expr,
            "await_promise": false
        }))?;

        if response.is_error {
            anyhow::bail!("Complex JavaScript evaluation returned error");
        }

        Ok(start.elapsed())
    });

    complex_metrics.print_summary();

    // Performance assertions
    assert!(simple_metrics.success_rate >= 0.9, "Simple JS should have high success rate");
    assert!(simple_metrics.avg_duration < Duration::from_secs(1), "Simple JS should be very fast");
    assert!(complex_metrics.avg_duration < Duration::from_secs(5), "Complex JS should complete within 5 seconds");
}

#[test]
fn test_web_scraping_performance() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");
    let iterations = 5; // Fewer iterations for network operations

    let mut successful = 0;
    let mut total_duration = Duration::ZERO;

    for i in 0..iterations {
        let start = Instant::now();
        let result = harness.call_tool("scrape_url", json!({
            "url": "https://httpbin.org/html",
            "wait_for_js": false
        }));

        let duration = start.elapsed();
        total_duration += duration;

        if let Ok(response) = result {
            if !response.is_error {
                successful += 1;
            }
        }

        if i % 2 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }

    println!();
    let success_rate = successful as f64 / iterations as f64;
    let avg_duration = total_duration / iterations as u32;

    println!("Web Scraping Performance: {:.1}% success, {:?} avg", success_rate * 100.0, avg_duration);

    // Performance assertions (more lenient for network operations)
    assert!(success_rate >= 0.6, "Scraping should have reasonable success rate");
    assert!(avg_duration < Duration::from_secs(30), "Scraping should complete within 30 seconds");
}

#[test]
fn test_google_search_performance() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");
    let iterations = 3; // Very few iterations for search operations

    let queries = vec![
        "rust programming",
        "web browser automation",
        "javascript engine",
    ];

    let mut successful = 0;
    let mut total_duration = Duration::ZERO;

    for i in 0..iterations {
        let query = &queries[i % queries.len()];
        let start = Instant::now();

        let result = harness.call_tool("google_search", json!({
            "query": query,
            "num_results": 1
        }));

        let duration = start.elapsed();
        total_duration += duration;

        if let Ok(response) = result {
            if !response.is_error {
                successful += 1;
            }
        }

        print!(".");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }

    println!();
    let avg_duration = total_duration / iterations as u32;

    println!("Google Search Performance: {}/{} successful, {:?} avg", successful, iterations, avg_duration);

    // Performance assertions (very lenient for search operations)
    assert!(avg_duration < Duration::from_secs(45), "Search should complete within 45 seconds");
}

#[test]
fn test_mixed_workload_performance() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");
    let iterations = 30;

    let mut successful = 0;
    let mut total_duration = Duration::ZERO;

    for i in 0..iterations {
        let op_type = match i % 4 {
            0 => "list_tools",
            1 => "js_eval",
            2 => "memory_search",
            _ => "memory_store",
        };

        let start = Instant::now();
        let result = match op_type {
            "list_tools" => harness.list_tools().map(|_| ()),
            "js_eval" => {
                harness.call_tool("cdp_runtime_evaluate", json!({"expression": "Math.PI * 2"}))
                    .map(|_| ())
            }
            "memory_search" => {
                harness.call_tool("ai_memory_search_research", json!({"query": "test", "limit": 1}))
                    .map(|_| ())
            }
            "memory_store" => {
                harness.call_tool("ai_memory_store_research", json!({
                    "key": format!("mixed_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()),
                    "data": {"mixed": "workload test"},
                    "tags": ["mixed", "performance"]
                })).map(|_| ())
            }
            _ => unreachable!(),
        };

        let duration = start.elapsed();
        total_duration += duration;

        if result.is_ok() {
            successful += 1;
        }

        if i % 5 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }

    println!();
    let success_rate = successful as f64 / iterations as f64;
    let avg_duration = total_duration / iterations as u32;

    println!("Mixed Workload: {:.1}% success, {:?} avg", success_rate * 100.0, avg_duration);

    // Performance assertions
    assert!(success_rate >= 0.8, "Mixed workload should have good success rate");
    assert!(avg_duration < Duration::from_secs(3), "Mixed operations should be reasonably fast");
}

#[test]
fn test_stress_test_rapid_requests() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");
    let iterations = 100;
    let max_duration = Duration::from_secs(60);

    println!("Running stress test with {} rapid requests...", iterations);
    let start_time = Instant::now();

    let mut successes = 0;
    let mut failures = 0;
    let mut durations = Vec::new();

    for i in 0..iterations {
        let op_start = Instant::now();

        // Alternate between different operations
        let result = match i % 4 {
            0 => harness.list_tools().map(|_| ()),
            1 => harness.call_tool("ai_memory_search_research", json!({
                "query": "stress",
                "limit": 1
            })).map(|_| ()),
            2 => harness.call_tool("cdp_runtime_evaluate", json!({
                "expression": "Date.now()"
            })).map(|_| ()),
            _ => harness.call_tool("ai_memory_store_research", json!({
                "key": format!("stress_{}", i),
                "data": {"stress_test": i},
                "tags": ["stress"]
            })).map(|_| ()),
        };

        let op_duration = op_start.elapsed();
        durations.push(op_duration);

        match result {
            Ok(_) => successes += 1,
            Err(_) => failures += 1,
        }

        // Very small delay to avoid overwhelming
        std::thread::sleep(Duration::from_millis(5));

        // Progress indicator
        if i % 10 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }

        // Early exit if taking too long
        if start_time.elapsed() > max_duration {
            println!("\nStopping early due to timeout");
            break;
        }
    }

    let total_duration = start_time.elapsed();
    println!(); // New line after progress dots

    // Calculate statistics
    let success_rate = successes as f64 / (successes + failures) as f64;
    let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
    let min_duration = durations.iter().min().unwrap_or(&Duration::ZERO);
    let max_duration = durations.iter().max().unwrap_or(&Duration::ZERO);

    println!("\n=== Stress Test Results ===");
    println!("Total Operations: {}", successes + failures);
    println!("Successes: {}", successes);
    println!("Failures: {}", failures);
    println!("Success Rate: {:.1}%", success_rate * 100.0);
    println!("Total Duration: {:?}", total_duration);
    println!("Throughput: {:.1} ops/sec", (successes + failures) as f64 / total_duration.as_secs_f64());
    println!("Min Op Duration: {:?}", min_duration);
    println!("Max Op Duration: {:?}", max_duration);
    println!("Avg Op Duration: {:?}", avg_duration);
    println!("============================\n");

    // Assertions for stress test
    assert!(success_rate >= 0.7, "Should maintain at least 70% success rate under stress");
    assert!(harness.is_running(), "Server should still be running after stress test");

    // Final health check
    let health_check = harness.list_tools();
    assert!(health_check.is_ok(), "Server should still be responsive after stress test");
}

#[test]
fn test_memory_usage_pattern() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    println!("Testing memory usage patterns...");

    // Store increasingly large data to test memory handling
    let data_sizes = vec![
        ("small", json!({"data": "x".repeat(100)})),
        ("medium", json!({"data": "x".repeat(1000)})),
        ("large", json!({"data": "x".repeat(10000)})),
        ("xlarge", json!({"data": "x".repeat(50000)})),
    ];

    for (size_name, data) in data_sizes {
        let start = Instant::now();

        let result = harness.call_tool("ai_memory_store_research", json!({
            "key": format!("memory_test_{}", size_name),
            "data": data,
            "tags": ["memory_test", size_name]
        }));

        let duration = start.elapsed();

        match result {
            Ok(response) => {
                if response.is_error {
                    println!("Error storing {} data: {:?}", size_name, response.content);
                } else {
                    println!("Successfully stored {} data in {:?}", size_name, duration);
                }
            }
            Err(e) => {
                println!("Failed to store {} data: {}", size_name, e);
            }
        }

        // Verify server is still responsive
        assert!(harness.is_running(), "Server should still be running after storing {} data", size_name);

        std::thread::sleep(Duration::from_millis(100));
    }

    // Test retrieval of large data
    let retrieve_result = harness.call_tool("ai_memory_get_research", json!({
        "key": "memory_test_large"
    }));

    if let Ok(response) = retrieve_result {
        if !response.is_error {
            println!("Successfully retrieved large data");
        }
    }

    println!("Memory usage pattern test completed");
}

// Helper to add random dependency to prevent optimization
use std::sync::atomic::{AtomicU64, Ordering};
static RANDOM_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn test_performance_baseline() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    // Establish baseline performance for each operation type
    let baseline_tests = vec![
        ("list_tools", 10),
        ("simple_js", 10),
        ("memory_search", 10),
    ];

    println!("\n=== Performance Baseline ===");

    for (test_name, iterations) in baseline_tests {
        let start = Instant::now();
        let mut successes = 0;

        for _ in 0..iterations {
            let _ = RANDOM_COUNTER.fetch_add(1, Ordering::Relaxed); // Prevent optimization

            let result = match test_name {
                "list_tools" => harness.list_tools().map(|_| ()),
                "simple_js" => harness.call_tool("cdp_runtime_evaluate", json!({
                    "expression": "42"
                })).map(|_| ()),
                "memory_search" => harness.call_tool("ai_memory_search_research", json!({
                    "query": "baseline",
                    "limit": 1
                })).map(|_| ()),
                _ => Ok(()),
            };

            if result.is_ok() {
                successes += 1;
            }
        }

        let avg_duration = start.elapsed() / iterations as u32;
        let success_rate = successes as f64 / iterations as f64;

        println!("{}: {:.1}% success, {:?} avg", test_name, success_rate * 100.0, avg_duration);

        // Store baseline in AI memory for reference
        let _ = harness.call_tool("ai_memory_store_research", json!({
            "key": format!("baseline_{}", test_name),
            "data": {
                "test_name": test_name,
                "iterations": iterations,
                "success_rate": success_rate,
                "avg_duration_ms": avg_duration.as_millis(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            },
            "tags": ["baseline", "performance"]
        }));
    }

    println!("==============================\n");
}