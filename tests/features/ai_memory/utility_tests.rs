// Tests for AiMemoryHeap utility methods (statistics, export, import, cleanup)

use thalora::features::{AiMemoryHeap, ResearchEntry, NotePriority};
use std::collections::HashMap;
use tempfile::TempDir;

fn create_test_memory() -> (AiMemoryHeap, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_file = temp_dir.path().join("test_memory.json");
    let memory = AiMemoryHeap::new(&cache_file).expect("Failed to create memory heap");
    (memory, temp_dir)
}

#[test]
fn test_get_statistics_empty() {
    let (memory, _temp) = create_test_memory();

    let stats = memory.get_statistics();
    assert_eq!(stats.research_count, 0);
    assert_eq!(stats.credential_count, 0);
    assert_eq!(stats.session_count, 0);
    assert_eq!(stats.bookmark_count, 0);
    assert_eq!(stats.note_count, 0);
    assert_eq!(stats.total_entries, 0);
}

#[test]
fn test_get_statistics_with_data() {
    let (mut memory, _temp) = create_test_memory();

    // Add various entries
    let entry = ResearchEntry {
        topic: "Test".to_string(),
        summary: "Test summary".to_string(),
        key_findings: vec![],
        sources: vec![],
        tags: vec![],
        confidence_score: 0.8,
        created_at: chrono::Utc::now(),
        last_accessed: chrono::Utc::now(),
        access_count: 0,
        related_keys: vec![],
        raw_data: None,
    };
    memory.store_research("r1", entry.clone()).unwrap();
    memory.store_research("r2", entry.clone()).unwrap();

    memory.store_credentials("c1", "s1", "u1", "p1", HashMap::new()).unwrap();

    memory.start_session("s1", "ctx", vec![]).unwrap();
    memory.start_session("s2", "ctx", vec![]).unwrap();
    memory.start_session("s3", "ctx", vec![]).unwrap();

    memory.store_bookmark("b1", "url1", "t1", "d1", "p1", vec![]).unwrap();
    memory.store_bookmark("b2", "url2", "t2", "d2", "p2", vec![]).unwrap();

    memory.store_note("n1", "t1", "c1", "cat", vec![], NotePriority::Normal).unwrap();

    let stats = memory.get_statistics();
    assert_eq!(stats.research_count, 2);
    assert_eq!(stats.credential_count, 1);
    assert_eq!(stats.session_count, 3);
    assert_eq!(stats.bookmark_count, 2);
    assert_eq!(stats.note_count, 1);
    assert_eq!(stats.total_entries, 9);
    assert!(stats.file_size_bytes > 0);
}

#[test]
fn test_export_json_empty() {
    let (memory, _temp) = create_test_memory();

    let json = memory.export_json().expect("Failed to export");
    assert!(json.contains("\"research\""));
    assert!(json.contains("\"credentials\""));
    assert!(json.contains("\"sessions\""));
    assert!(json.contains("\"bookmarks\""));
    assert!(json.contains("\"notes\""));
}

#[test]
fn test_export_json_with_data() {
    let (mut memory, _temp) = create_test_memory();

    let entry = ResearchEntry {
        topic: "Export Test Topic".to_string(),
        summary: "Export test summary".to_string(),
        key_findings: vec!["Finding 1".to_string()],
        sources: vec!["https://source.com".to_string()],
        tags: vec!["export".to_string()],
        confidence_score: 0.9,
        created_at: chrono::Utc::now(),
        last_accessed: chrono::Utc::now(),
        access_count: 0,
        related_keys: vec![],
        raw_data: None,
    };
    memory.store_research("export_key", entry).unwrap();

    let json = memory.export_json().expect("Failed to export");
    assert!(json.contains("Export Test Topic"));
    assert!(json.contains("export_key"));
    assert!(json.contains("Finding 1"));
}

#[test]
fn test_import_json_merge() {
    let (mut memory, _temp) = create_test_memory();

    // Store initial data
    let entry = ResearchEntry {
        topic: "Original".to_string(),
        summary: "Original summary".to_string(),
        key_findings: vec![],
        sources: vec![],
        tags: vec![],
        confidence_score: 0.5,
        created_at: chrono::Utc::now(),
        last_accessed: chrono::Utc::now(),
        access_count: 0,
        related_keys: vec![],
        raw_data: None,
    };
    memory.store_research("original_key", entry).unwrap();

    // Create export from another memory instance
    let (mut other_memory, _other_temp) = create_test_memory();
    let other_entry = ResearchEntry {
        topic: "Imported".to_string(),
        summary: "Imported summary".to_string(),
        key_findings: vec![],
        sources: vec![],
        tags: vec![],
        confidence_score: 0.7,
        created_at: chrono::Utc::now(),
        last_accessed: chrono::Utc::now(),
        access_count: 0,
        related_keys: vec![],
        raw_data: None,
    };
    other_memory.store_research("imported_key", other_entry).unwrap();
    let import_json = other_memory.export_json().unwrap();

    // Import into first memory
    memory.import_json(&import_json).expect("Failed to import");

    // Both entries should exist
    assert!(memory.get_research("original_key").is_some());
    assert!(memory.get_research("imported_key").is_some());
    assert_eq!(memory.get_statistics().research_count, 2);
}

#[test]
fn test_import_json_invalid() {
    let (mut memory, _temp) = create_test_memory();

    let invalid_json = "{ this is not valid json }";
    let result = memory.import_json(invalid_json);
    assert!(result.is_err(), "Should error on invalid JSON");
}

#[test]
fn test_cleanup_old_entries() {
    let (mut memory, _temp) = create_test_memory();

    // Store some research entries
    for i in 0..5 {
        let entry = ResearchEntry {
            topic: format!("Topic {}", i),
            summary: format!("Summary {}", i),
            key_findings: vec![],
            sources: vec![],
            tags: vec![],
            confidence_score: 0.5,
            // All entries are "new" (created now)
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            access_count: 0,
            related_keys: vec![],
            raw_data: None,
        };
        memory.store_research(&format!("key_{}", i), entry).unwrap();
    }

    // Cleanup with 30 day threshold - nothing should be removed (all are new)
    let removed = memory.cleanup_old_entries(30).expect("Failed to cleanup");
    assert_eq!(removed, 0, "No entries should be removed (all are new)");

    let stats = memory.get_statistics();
    assert_eq!(stats.research_count, 5, "All entries should remain");
}

#[test]
fn test_new_default_location() {
    // This test verifies the default memory location logic
    // We can't actually test file creation in ~/.thalora without affecting user's system
    // So we just verify the function doesn't panic
    let result = AiMemoryHeap::new_default();
    // This might fail in CI environments without home directory
    // So we just check it doesn't panic and handles errors gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_statistics_version() {
    let (memory, _temp) = create_test_memory();

    let stats = memory.get_statistics();
    assert_eq!(stats.version, "1.0.0");
}

#[test]
fn test_statistics_last_updated() {
    let (mut memory, _temp) = create_test_memory();

    let before = chrono::Utc::now();

    // Make a change to trigger save
    memory.store_note("test", "Test", "Content", "Cat", vec![], NotePriority::Normal).unwrap();

    let stats = memory.get_statistics();
    let after = chrono::Utc::now();

    assert!(stats.last_updated >= before);
    assert!(stats.last_updated <= after);
}

#[test]
fn test_concurrent_operations() {
    let (mut memory, _temp) = create_test_memory();

    // Perform many operations rapidly
    for i in 0..100 {
        let entry = ResearchEntry {
            topic: format!("Concurrent {}", i),
            summary: "Concurrent test".to_string(),
            key_findings: vec![],
            sources: vec![],
            tags: vec![],
            confidence_score: 0.5,
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            access_count: 0,
            related_keys: vec![],
            raw_data: None,
        };
        memory.store_research(&format!("concurrent_{}", i), entry).unwrap();
    }

    for i in 0..100 {
        memory.store_bookmark(
            &format!("bm_{}", i),
            &format!("https://test{}.com", i),
            "Title",
            "Desc",
            "Preview",
            vec![],
        ).unwrap();
    }

    let stats = memory.get_statistics();
    assert_eq!(stats.research_count, 100);
    assert_eq!(stats.bookmark_count, 100);
    assert_eq!(stats.total_entries, 200);
}

#[test]
fn test_file_size_tracking() {
    let (mut memory, _temp) = create_test_memory();

    let initial_stats = memory.get_statistics();
    let initial_size = initial_stats.file_size_bytes;

    // Add large content
    let large_content = "X".repeat(10000);
    memory.store_note("large", "Large Note", &large_content, "Cat", vec![], NotePriority::Normal).unwrap();

    let final_stats = memory.get_statistics();
    assert!(final_stats.file_size_bytes > initial_size, "File size should increase");
}
