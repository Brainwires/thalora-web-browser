// Tests for AiMemoryHeap research management

use thalora::features::{AiMemoryHeap, ResearchEntry, MemorySearchCriteria, MemorySortBy};
use std::collections::HashMap;
use tempfile::TempDir;

fn create_test_memory() -> (AiMemoryHeap, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_file = temp_dir.path().join("test_memory.json");
    let memory = AiMemoryHeap::new(&cache_file).expect("Failed to create memory heap");
    (memory, temp_dir)
}

fn create_research_entry(topic: &str, summary: &str, tags: Vec<&str>, confidence: f64) -> ResearchEntry {
    ResearchEntry {
        topic: topic.to_string(),
        summary: summary.to_string(),
        findings: vec![],
        sources: vec![],
        tags: tags.into_iter().map(|s| s.to_string()).collect(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        confidence_score: confidence,
        related_topics: vec![],
    }
}

#[test]
fn test_store_and_get_research() {
    let (mut memory, _temp) = create_test_memory();

    let entry = ResearchEntry {
        topic: "Rust async programming".to_string(),
        summary: "Overview of async/await in Rust".to_string(),
        findings: vec![
            "Uses futures for async operations".to_string(),
            "tokio is the most popular runtime".to_string(),
        ],
        sources: vec!["https://rust-lang.org".to_string()],
        tags: vec!["rust".to_string(), "async".to_string()],
        confidence_score: 0.95,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        related_topics: vec![],
    };

    memory.store_research("rust_async", entry.clone()).expect("Failed to store research");

    let retrieved = memory.get_research("rust_async");
    assert!(retrieved.is_some(), "Research should be retrievable");

    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.topic, "Rust async programming");
    assert_eq!(retrieved.confidence_score, 0.95);
    assert_eq!(retrieved.findings.len(), 2);
}

#[test]
fn test_get_nonexistent_research() {
    let (memory, _temp) = create_test_memory();

    let result = memory.get_research("nonexistent_key");
    assert!(result.is_none(), "Should return None for nonexistent key");
}

#[test]
fn test_update_research() {
    let (mut memory, _temp) = create_test_memory();

    let entry = create_research_entry("Initial topic", "Initial summary", vec![], 0.5);
    memory.store_research("update_test", entry).expect("Failed to store");

    let updated = memory.update_research("update_test", |e| {
        e.summary = "Updated summary".to_string();
        e.confidence_score = 0.9;
    }).expect("Failed to update");

    assert!(updated, "Update should return true");

    let retrieved = memory.get_research("update_test").unwrap();
    assert_eq!(retrieved.summary, "Updated summary");
    assert_eq!(retrieved.confidence_score, 0.9);
}

#[test]
fn test_update_nonexistent_research() {
    let (mut memory, _temp) = create_test_memory();

    let updated = memory.update_research("nonexistent", |e| {
        e.summary = "Should not happen".to_string();
    }).expect("Update should not error");

    assert!(!updated, "Update of nonexistent key should return false");
}

#[test]
fn test_search_research_by_query() {
    let (mut memory, _temp) = create_test_memory();

    // Store multiple research entries
    let entries = vec![
        ("rust_web", "Rust web frameworks", vec!["rust", "web"]),
        ("python_ml", "Python machine learning", vec!["python", "ml"]),
        ("rust_cli", "Rust CLI tools", vec!["rust", "cli"]),
    ];

    for (key, topic, tags) in entries {
        let entry = create_research_entry(topic, &format!("Summary about {}", topic), tags, 0.8);
        memory.store_research(key, entry).expect("Failed to store");
    }

    // Search for "Rust"
    let criteria = MemorySearchCriteria {
        query: Some("Rust".to_string()),
        tags: None,
        date_range: None,
        category: None,
        limit: Some(10),
        sort_by: MemorySortBy::Relevance,
    };

    let results = memory.search_research(&criteria);
    assert_eq!(results.len(), 2, "Should find 2 Rust-related entries");
}

#[test]
fn test_search_research_by_tags() {
    let (mut memory, _temp) = create_test_memory();

    memory.store_research("entry1", create_research_entry("Entry A", "Has tag A", vec!["tagA", "common"], 0.8)).unwrap();
    memory.store_research("entry2", create_research_entry("Entry B", "Has tag B", vec!["tagB", "common"], 0.8)).unwrap();

    let criteria = MemorySearchCriteria {
        query: None,
        tags: Some(vec!["tagA".to_string()]),
        date_range: None,
        category: None,
        limit: Some(10),
        sort_by: MemorySortBy::Relevance,
    };

    let results = memory.search_research(&criteria);
    assert_eq!(results.len(), 1, "Should find 1 entry with tagA");
}

#[test]
fn test_research_persistence() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_file = temp_dir.path().join("persist_test.json");

    // Create and populate memory
    {
        let mut memory = AiMemoryHeap::new(&cache_file).expect("Failed to create memory");
        let entry = create_research_entry("Persistent topic", "This should persist", vec!["persistent"], 0.9);
        memory.store_research("persist_key", entry).expect("Failed to store");
    }

    // Create new memory instance from same file
    {
        let memory = AiMemoryHeap::new(&cache_file).expect("Failed to reload memory");
        let retrieved = memory.get_research("persist_key");
        assert!(retrieved.is_some(), "Data should persist across instances");
        assert_eq!(retrieved.unwrap().topic, "Persistent topic");
    }
}

#[test]
fn test_multiple_research_entries() {
    let (mut memory, _temp) = create_test_memory();

    // Store 100 entries
    for i in 0..100 {
        let entry = create_research_entry(
            &format!("Topic {}", i),
            &format!("Summary for topic {}", i),
            vec![&format!("tag{}", i % 10)],
            (i as f64) / 100.0,
        );
        memory.store_research(&format!("key_{}", i), entry).expect("Failed to store");
    }

    // Verify all entries exist
    for i in 0..100 {
        let key = format!("key_{}", i);
        assert!(memory.get_research(&key).is_some(), "Entry {} should exist", i);
    }

    let stats = memory.get_statistics();
    assert_eq!(stats.research_count, 100);
}

#[test]
fn test_default_search_criteria() {
    let criteria = MemorySearchCriteria::default();
    assert!(criteria.query.is_none());
    assert!(criteria.tags.is_none());
    assert!(criteria.date_range.is_none());
    assert!(criteria.category.is_none());
    assert!(criteria.limit.is_none());
}
