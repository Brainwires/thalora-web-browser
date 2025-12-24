// Live Integration Tests
// Tests that make real HTTP requests to external services

mod search_engines {
    include!("integration/search_engines_live_test.rs");
}
