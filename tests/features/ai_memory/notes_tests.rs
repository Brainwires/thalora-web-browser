// Tests for AiMemoryHeap note management

use thalora::features::{AiMemoryHeap, NotePriority, MemorySearchCriteria, MemorySortBy};
use tempfile::TempDir;

fn create_test_memory() -> (AiMemoryHeap, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_file = temp_dir.path().join("test_memory.json");
    let memory = AiMemoryHeap::new(&cache_file).expect("Failed to create memory heap");
    (memory, temp_dir)
}

#[test]
fn test_store_note() {
    let (mut memory, _temp) = create_test_memory();

    memory.store_note(
        "note_001",
        "Project Ideas",
        "List of potential project ideas for 2025",
        "Ideas",
        vec!["projects".to_string(), "planning".to_string()],
        NotePriority::High,
    ).expect("Failed to store note");

    let stats = memory.get_statistics();
    assert_eq!(stats.note_count, 1);
}

#[test]
fn test_store_note_all_priorities() {
    let (mut memory, _temp) = create_test_memory();

    let priorities = vec![
        NotePriority::Low,
        NotePriority::Medium,
        NotePriority::High,
        NotePriority::Critical,
    ];

    for (i, priority) in priorities.into_iter().enumerate() {
        memory.store_note(
            &format!("priority_note_{}", i),
            &format!("Priority Test {}", i),
            "Content",
            "Test",
            vec![],
            priority,
        ).expect("Failed to store");
    }

    let stats = memory.get_statistics();
    assert_eq!(stats.note_count, 4);
}

#[test]
fn test_update_note() {
    let (mut memory, _temp) = create_test_memory();

    memory.store_note(
        "update_note",
        "Original Title",
        "Original content",
        "Original",
        vec!["original".to_string()],
        NotePriority::Medium,
    ).expect("Failed to store");

    let updated = memory.update_note("update_note", |note| {
        note.title = "Updated Title".to_string();
        note.content = "Updated content with more details".to_string();
        note.priority = NotePriority::High;
    }).expect("Failed to update");

    assert!(updated, "Update should return true");

    // Search to verify the update
    let criteria = MemorySearchCriteria {
        query: Some("Updated".to_string()),
        tags: None,
        date_range: None,
        category: None,
        limit: Some(10),
        sort_by: MemorySortBy::Relevance,
    };

    let results = memory.search_notes(&criteria);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.title, "Updated Title");
}

#[test]
fn test_update_nonexistent_note() {
    let (mut memory, _temp) = create_test_memory();

    let updated = memory.update_note("nonexistent", |note| {
        note.title = "Should not work".to_string();
    }).expect("Should not error");

    assert!(!updated, "Update of nonexistent note should return false");
}

#[test]
fn test_search_notes_by_query() {
    let (mut memory, _temp) = create_test_memory();

    memory.store_note("n1", "Rust Tips", "Tips for Rust development", "Tech", vec!["rust".to_string()], NotePriority::Medium)
        .expect("Failed");
    memory.store_note("n2", "Python Tips", "Tips for Python", "Tech", vec!["python".to_string()], NotePriority::Medium)
        .expect("Failed");
    memory.store_note("n3", "Rust Best Practices", "Best practices for Rust", "Tech", vec!["rust".to_string()], NotePriority::High)
        .expect("Failed");

    let criteria = MemorySearchCriteria {
        query: Some("Rust".to_string()),
        tags: None,
        date_range: None,
        category: None,
        limit: Some(10),
        sort_by: MemorySortBy::Relevance,
    };

    let results = memory.search_notes(&criteria);
    assert_eq!(results.len(), 2, "Should find 2 Rust-related notes");
}

#[test]
fn test_search_notes_by_tags() {
    let (mut memory, _temp) = create_test_memory();

    memory.store_note("tagged1", "Note A", "Content A", "Cat", vec!["important".to_string(), "work".to_string()], NotePriority::High)
        .expect("Failed");
    memory.store_note("tagged2", "Note B", "Content B", "Cat", vec!["personal".to_string()], NotePriority::Low)
        .expect("Failed");
    memory.store_note("tagged3", "Note C", "Content C", "Cat", vec!["important".to_string(), "urgent".to_string()], NotePriority::Critical)
        .expect("Failed");

    let criteria = MemorySearchCriteria {
        query: None,
        tags: Some(vec!["important".to_string()]),
        date_range: None,
        category: None,
        limit: Some(10),
        sort_by: MemorySortBy::Relevance,
    };

    let results = memory.search_notes(&criteria);
    assert_eq!(results.len(), 2, "Should find 2 important notes");
}

#[test]
fn test_note_with_long_content() {
    let (mut memory, _temp) = create_test_memory();

    let long_content = "A".repeat(100000); // 100KB of content

    memory.store_note(
        "long_note",
        "Long Note",
        &long_content,
        "Test",
        vec![],
        NotePriority::Medium,
    ).expect("Failed to store long note");

    let criteria = MemorySearchCriteria {
        query: Some("Long Note".to_string()),
        tags: None,
        date_range: None,
        category: None,
        limit: Some(10),
        sort_by: MemorySortBy::Relevance,
    };

    let results = memory.search_notes(&criteria);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.content.len(), 100000);
}

#[test]
fn test_note_with_unicode() {
    let (mut memory, _temp) = create_test_memory();

    memory.store_note(
        "unicode_note",
        "多语言笔记 - Multilingual Note",
        "内容：这是一个测试\nСодержание: это тест\n🎉🌟💡",
        "国际化",
        vec!["中文".to_string(), "русский".to_string(), "emoji".to_string()],
        NotePriority::Medium,
    ).expect("Failed to store unicode note");

    let criteria = MemorySearchCriteria {
        query: Some("多语言".to_string()),
        tags: None,
        date_range: None,
        category: None,
        limit: Some(10),
        sort_by: MemorySortBy::Relevance,
    };

    let results = memory.search_notes(&criteria);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_note_categories() {
    let (mut memory, _temp) = create_test_memory();

    let categories = vec!["Work", "Personal", "Ideas", "Learning", "Projects"];

    for (i, category) in categories.iter().enumerate() {
        memory.store_note(
            &format!("cat_note_{}", i),
            &format!("Note in {}", category),
            "Content",
            category,
            vec![],
            NotePriority::Medium,
        ).expect("Failed to store");
    }

    let stats = memory.get_statistics();
    assert_eq!(stats.note_count, 5);
}

#[test]
fn test_note_overwrite() {
    let (mut memory, _temp) = create_test_memory();

    // Store initial
    memory.store_note("overwrite_note", "Old Title", "Old content", "Old", vec!["old".to_string()], NotePriority::Low)
        .expect("Failed");

    // Overwrite
    memory.store_note("overwrite_note", "New Title", "New content", "New", vec!["new".to_string()], NotePriority::Critical)
        .expect("Failed");

    let criteria = MemorySearchCriteria {
        query: Some("New Title".to_string()),
        tags: None,
        date_range: None,
        category: None,
        limit: Some(10),
        sort_by: MemorySortBy::Relevance,
    };

    let results = memory.search_notes(&criteria);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.title, "New Title");
    assert!(matches!(results[0].1.priority, NotePriority::Critical));
}

#[test]
fn test_note_persistence() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_file = temp_dir.path().join("note_persist.json");

    // Store note
    {
        let mut memory = AiMemoryHeap::new(&cache_file).expect("Failed to create");
        memory.store_note(
            "persist_note",
            "Persistent Note",
            "This note should persist",
            "Persistence",
            vec!["persistent".to_string()],
            NotePriority::High,
        ).expect("Failed to store");
    }

    // Reload and verify
    {
        let memory = AiMemoryHeap::new(&cache_file).expect("Failed to reload");

        let criteria = MemorySearchCriteria {
            query: Some("Persistent".to_string()),
            tags: None,
            date_range: None,
            category: None,
            limit: Some(10),
            sort_by: MemorySortBy::Relevance,
        };

        let results = memory.search_notes(&criteria);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.title, "Persistent Note");
    }
}

#[test]
fn test_note_markdown_content() {
    let (mut memory, _temp) = create_test_memory();

    let markdown_content = r#"
# Heading 1

## Heading 2

- List item 1
- List item 2
  - Nested item

**Bold text** and *italic text*

```rust
fn main() {
    println!("Hello, world!");
}
```

| Column 1 | Column 2 |
|----------|----------|
| A        | B        |
"#;

    memory.store_note(
        "markdown_note",
        "Markdown Note",
        markdown_content,
        "Documentation",
        vec!["markdown".to_string()],
        NotePriority::Medium,
    ).expect("Failed to store markdown note");

    let criteria = MemorySearchCriteria {
        query: Some("Markdown".to_string()),
        tags: None,
        date_range: None,
        category: None,
        limit: Some(10),
        sort_by: MemorySortBy::Relevance,
    };

    let results = memory.search_notes(&criteria);
    assert_eq!(results.len(), 1);
    assert!(results[0].1.content.contains("```rust"));
}

#[test]
fn test_note_search_limit() {
    let (mut memory, _temp) = create_test_memory();

    // Store many notes
    for i in 0..30 {
        memory.store_note(
            &format!("note_{}", i),
            &format!("Test Note {}", i),
            "Test content",
            "Test",
            vec!["test".to_string()],
            NotePriority::Medium,
        ).expect("Failed");
    }

    let criteria = MemorySearchCriteria {
        query: Some("Test".to_string()),
        tags: None,
        date_range: None,
        category: None,
        limit: Some(10),
        sort_by: MemorySortBy::Relevance,
    };

    let results = memory.search_notes(&criteria);
    assert!(results.len() <= 10, "Should respect limit");
}

#[test]
fn test_note_archive_priority() {
    let (mut memory, _temp) = create_test_memory();

    memory.store_note(
        "archive_note",
        "Archived Note",
        "This is an archived note",
        "Archive",
        vec!["archived".to_string()],
        NotePriority::Archive,
    ).expect("Failed to store archived note");

    let stats = memory.get_statistics();
    assert_eq!(stats.note_count, 1);
}
