// Tests for AiMemoryHeap bookmark management

use thalora::features::{AiMemoryHeap, MemorySearchCriteria, MemorySortBy};
use tempfile::TempDir;

fn create_test_memory() -> (AiMemoryHeap, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_file = temp_dir.path().join("test_memory.json");
    let memory = AiMemoryHeap::new(&cache_file).expect("Failed to create memory heap");
    (memory, temp_dir)
}

#[test]
fn test_store_bookmark() {
    let (mut memory, _temp) = create_test_memory();

    memory.store_bookmark(
        "rust_docs",
        "https://doc.rust-lang.org",
        "Rust Documentation",
        "Official Rust language documentation",
        "The Rust Programming Language - A systems programming language...",
        vec!["rust".to_string(), "documentation".to_string(), "programming".to_string()],
    ).expect("Failed to store bookmark");

    let stats = memory.get_statistics();
    assert_eq!(stats.bookmark_count, 1);
}

#[test]
fn test_access_bookmark() {
    let (mut memory, _temp) = create_test_memory();

    memory.store_bookmark(
        "access_test",
        "https://example.com",
        "Example Site",
        "Test description",
        "Preview content",
        vec!["test".to_string()],
    ).expect("Failed to store");

    // Access the bookmark
    let bookmark = memory.access_bookmark("access_test");
    assert!(bookmark.is_some(), "Bookmark should be accessible");

    let bookmark = bookmark.unwrap();
    assert_eq!(bookmark.url, "https://example.com");
    assert_eq!(bookmark.title, "Example Site");
    assert!(bookmark.access_count >= 1, "Access count should be incremented");
}

#[test]
fn test_access_nonexistent_bookmark() {
    let (mut memory, _temp) = create_test_memory();

    let result = memory.access_bookmark("nonexistent");
    assert!(result.is_none(), "Should return None for nonexistent bookmark");
}

#[test]
fn test_access_count_increments() {
    let (mut memory, _temp) = create_test_memory();

    memory.store_bookmark(
        "count_test",
        "https://count.test",
        "Count Test",
        "Testing access count",
        "Preview",
        vec![],
    ).expect("Failed to store");

    // Access multiple times
    for i in 1..=5 {
        let bookmark = memory.access_bookmark("count_test").expect("Should exist");
        assert!(bookmark.access_count >= i, "Access count should be at least {}", i);
    }
}

#[test]
fn test_search_bookmarks_by_query() {
    let (mut memory, _temp) = create_test_memory();

    memory.store_bookmark("bm1", "https://rust.com", "Rust Programming", "Rust lang", "Rust content", vec!["rust".to_string()])
        .expect("Failed");
    memory.store_bookmark("bm2", "https://python.com", "Python Programming", "Python lang", "Python content", vec!["python".to_string()])
        .expect("Failed");
    memory.store_bookmark("bm3", "https://rust-cli.com", "Rust CLI Tools", "CLI in Rust", "CLI content", vec!["rust".to_string(), "cli".to_string()])
        .expect("Failed");

    let criteria = MemorySearchCriteria {
        query: Some("Rust".to_string()),
        tags: None,
        date_range: None,
        category: None,
        limit: Some(10),
        sort_by: MemorySortBy::Relevance,
    };

    let results = memory.search_bookmarks(&criteria);
    assert_eq!(results.len(), 2, "Should find 2 Rust-related bookmarks");
}

#[test]
fn test_search_bookmarks_by_tags() {
    let (mut memory, _temp) = create_test_memory();

    memory.store_bookmark("tagged1", "https://a.com", "A", "D", "P", vec!["frontend".to_string(), "react".to_string()])
        .expect("Failed");
    memory.store_bookmark("tagged2", "https://b.com", "B", "D", "P", vec!["backend".to_string(), "rust".to_string()])
        .expect("Failed");
    memory.store_bookmark("tagged3", "https://c.com", "C", "D", "P", vec!["frontend".to_string(), "vue".to_string()])
        .expect("Failed");

    let criteria = MemorySearchCriteria {
        query: None,
        tags: Some(vec!["frontend".to_string()]),
        date_range: None,
        category: None,
        limit: Some(10),
        sort_by: MemorySortBy::Relevance,
    };

    let results = memory.search_bookmarks(&criteria);
    assert_eq!(results.len(), 2, "Should find 2 frontend bookmarks");
}

#[test]
fn test_bookmark_with_long_content() {
    let (mut memory, _temp) = create_test_memory();

    let long_preview = "A".repeat(10000);
    let long_description = "B".repeat(5000);

    memory.store_bookmark(
        "long_content",
        "https://long.content.test",
        "Long Content Test",
        &long_description,
        &long_preview,
        vec!["test".to_string()],
    ).expect("Failed to store long content");

    let bookmark = memory.access_bookmark("long_content").expect("Should exist");
    assert_eq!(bookmark.content_preview.len(), 10000);
    assert_eq!(bookmark.description.len(), 5000);
}

#[test]
fn test_bookmark_with_unicode() {
    let (mut memory, _temp) = create_test_memory();

    memory.store_bookmark(
        "unicode_bm",
        "https://example.com/中文",
        "中文标题 - Japanese 日本語",
        "描述 с кириллицей",
        "内容预览 🌟🎉",
        vec!["中文".to_string(), "日本語".to_string()],
    ).expect("Failed to store unicode bookmark");

    let bookmark = memory.access_bookmark("unicode_bm").expect("Should exist");
    assert_eq!(bookmark.title, "中文标题 - Japanese 日本語");
    assert!(bookmark.tags.contains(&"中文".to_string()));
}

#[test]
fn test_bookmark_url_validation() {
    let (mut memory, _temp) = create_test_memory();

    // Various URL formats
    let urls = vec![
        "https://example.com",
        "http://localhost:8080/path?query=value",
        "https://sub.domain.example.com/path/to/resource",
        "file:///local/path",
    ];

    for (i, url) in urls.iter().enumerate() {
        memory.store_bookmark(
            &format!("url_test_{}", i),
            url,
            &format!("URL Test {}", i),
            "Description",
            "Preview",
            vec![],
        ).expect(&format!("Failed to store URL: {}", url));
    }

    let stats = memory.get_statistics();
    assert_eq!(stats.bookmark_count, urls.len());
}

#[test]
fn test_bookmark_overwrite() {
    let (mut memory, _temp) = create_test_memory();

    // Store initial
    memory.store_bookmark("overwrite_bm", "https://old.url", "Old Title", "Old desc", "Old preview", vec!["old".to_string()])
        .expect("Failed");

    // Overwrite
    memory.store_bookmark("overwrite_bm", "https://new.url", "New Title", "New desc", "New preview", vec!["new".to_string()])
        .expect("Failed");

    let bookmark = memory.access_bookmark("overwrite_bm").expect("Should exist");
    assert_eq!(bookmark.url, "https://new.url");
    assert_eq!(bookmark.title, "New Title");
    assert!(bookmark.tags.contains(&"new".to_string()));
}

#[test]
fn test_bookmark_persistence() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_file = temp_dir.path().join("bm_persist.json");

    // Store bookmark
    {
        let mut memory = AiMemoryHeap::new(&cache_file).expect("Failed to create");
        memory.store_bookmark(
            "persist_bm",
            "https://persist.test",
            "Persistent Bookmark",
            "Should persist",
            "Persistent preview",
            vec!["persistent".to_string()],
        ).expect("Failed to store");
    }

    // Reload and verify
    {
        let mut memory = AiMemoryHeap::new(&cache_file).expect("Failed to reload");
        let bookmark = memory.access_bookmark("persist_bm").expect("Should persist");
        assert_eq!(bookmark.url, "https://persist.test");
        assert_eq!(bookmark.title, "Persistent Bookmark");
    }
}

#[test]
fn test_bookmark_many_tags() {
    let (mut memory, _temp) = create_test_memory();

    let many_tags: Vec<String> = (0..50).map(|i| format!("tag_{}", i)).collect();

    memory.store_bookmark(
        "many_tags_bm",
        "https://tags.test",
        "Many Tags",
        "Description",
        "Preview",
        many_tags.clone(),
    ).expect("Failed to store");

    let bookmark = memory.access_bookmark("many_tags_bm").expect("Should exist");
    assert_eq!(bookmark.tags.len(), 50);
}

#[test]
fn test_bookmark_search_limit() {
    let (mut memory, _temp) = create_test_memory();

    // Store 20 bookmarks
    for i in 0..20 {
        memory.store_bookmark(
            &format!("bm_{}", i),
            &format!("https://test{}.com", i),
            "Test Bookmark",
            "Test description",
            "Test preview",
            vec!["test".to_string()],
        ).expect("Failed");
    }

    let criteria = MemorySearchCriteria {
        query: Some("Test".to_string()),
        tags: None,
        date_range: None,
        category: None,
        limit: Some(5),
        sort_by: MemorySortBy::Relevance,
    };

    let results = memory.search_bookmarks(&criteria);
    assert!(results.len() <= 5, "Should respect limit");
}
