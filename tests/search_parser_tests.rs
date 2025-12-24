// Web Search Parser Tests
// Tests for parsing search results from DuckDuckGo, Bing, Google, and Startpage

mod duckduckgo {
    include!("protocols/search/duckduckgo_tests.rs");
}

mod bing {
    include!("protocols/search/bing_tests.rs");
}

mod google {
    include!("protocols/search/google_tests.rs");
}

mod startpage {
    include!("protocols/search/startpage_tests.rs");
}
