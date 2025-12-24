// Tests for AiMemoryHeap session management

use thalora::features::{AiMemoryHeap, SessionStatus};
use serde_json::json;
use tempfile::TempDir;

fn create_test_memory() -> (AiMemoryHeap, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_file = temp_dir.path().join("test_memory.json");
    let memory = AiMemoryHeap::new(&cache_file).expect("Failed to create memory heap");
    (memory, temp_dir)
}

#[test]
fn test_start_session() {
    let (mut memory, _temp) = create_test_memory();

    let objectives = vec![
        "Research topic A".to_string(),
        "Implement feature B".to_string(),
        "Test component C".to_string(),
    ];

    memory.start_session(
        "session_001",
        "Working on project X",
        objectives.clone(),
    ).expect("Failed to start session");

    let active = memory.get_active_sessions();
    assert_eq!(active.len(), 1, "Should have 1 active session");

    let (id, session) = &active[0];
    assert_eq!(*id, "session_001");
    assert_eq!(session.context, "Working on project X");
    assert_eq!(session.objectives.len(), 3);
    assert!(matches!(session.status, SessionStatus::Active));
}

#[test]
fn test_update_session_progress() {
    let (mut memory, _temp) = create_test_memory();

    memory.start_session("progress_session", "Test context", vec!["Objective 1".to_string()])
        .expect("Failed to start");

    // Update with various progress data
    memory.update_session_progress("progress_session", "step1_complete", json!(true))
        .expect("Failed to update");
    memory.update_session_progress("progress_session", "files_processed", json!(42))
        .expect("Failed to update");
    memory.update_session_progress("progress_session", "current_state", json!({"phase": "analysis"}))
        .expect("Failed to update");

    let active = memory.get_active_sessions();
    assert_eq!(active.len(), 1);

    let (_, session) = &active[0];
    assert_eq!(session.progress.get("step1_complete"), Some(&json!(true)));
    assert_eq!(session.progress.get("files_processed"), Some(&json!(42)));
}

#[test]
fn test_update_nonexistent_session() {
    let (mut memory, _temp) = create_test_memory();

    let result = memory.update_session_progress("nonexistent", "key", json!("value"))
        .expect("Should not error");

    assert!(!result, "Update of nonexistent session should return false");
}

#[test]
fn test_complete_session_task() {
    let (mut memory, _temp) = create_test_memory();

    let objectives = vec![
        "Task A".to_string(),
        "Task B".to_string(),
        "Task C".to_string(),
    ];

    memory.start_session("task_session", "Testing tasks", objectives)
        .expect("Failed to start");

    // Complete tasks
    let completed1 = memory.complete_session_task("task_session", "Task A")
        .expect("Failed to complete");
    assert!(completed1, "Should complete Task A");

    let completed2 = memory.complete_session_task("task_session", "Task B")
        .expect("Failed to complete");
    assert!(completed2, "Should complete Task B");

    let active = memory.get_active_sessions();
    let (_, session) = &active[0];
    assert!(session.completed_tasks.contains(&"Task A".to_string()));
    assert!(session.completed_tasks.contains(&"Task B".to_string()));
}

#[test]
fn test_complete_task_nonexistent_session() {
    let (mut memory, _temp) = create_test_memory();

    let result = memory.complete_session_task("nonexistent", "Some task")
        .expect("Should not error");

    assert!(!result, "Complete task on nonexistent session should return false");
}

#[test]
fn test_multiple_active_sessions() {
    let (mut memory, _temp) = create_test_memory();

    memory.start_session("session_a", "Context A", vec!["Obj A".to_string()])
        .expect("Failed to start A");
    memory.start_session("session_b", "Context B", vec!["Obj B".to_string()])
        .expect("Failed to start B");
    memory.start_session("session_c", "Context C", vec!["Obj C".to_string()])
        .expect("Failed to start C");

    let active = memory.get_active_sessions();
    assert_eq!(active.len(), 3, "Should have 3 active sessions");
}

#[test]
fn test_session_with_empty_objectives() {
    let (mut memory, _temp) = create_test_memory();

    memory.start_session("empty_obj_session", "No objectives", vec![])
        .expect("Failed to start session with empty objectives");

    let active = memory.get_active_sessions();
    assert_eq!(active.len(), 1);

    let (_, session) = &active[0];
    assert!(session.objectives.is_empty());
}

#[test]
fn test_session_progress_overwrite() {
    let (mut memory, _temp) = create_test_memory();

    memory.start_session("overwrite_session", "Test", vec![])
        .expect("Failed to start");

    // Set initial value
    memory.update_session_progress("overwrite_session", "counter", json!(1))
        .expect("Failed to update");

    // Overwrite value
    memory.update_session_progress("overwrite_session", "counter", json!(100))
        .expect("Failed to update");

    let active = memory.get_active_sessions();
    let (_, session) = &active[0];
    assert_eq!(session.progress.get("counter"), Some(&json!(100)));
}

#[test]
fn test_session_complex_progress_data() {
    let (mut memory, _temp) = create_test_memory();

    memory.start_session("complex_session", "Complex data test", vec![])
        .expect("Failed to start");

    let complex_data = json!({
        "nested": {
            "level1": {
                "level2": {
                    "value": 42
                }
            }
        },
        "array": [1, 2, 3, 4, 5],
        "mixed": [
            {"name": "item1"},
            {"name": "item2"}
        ]
    });

    memory.update_session_progress("complex_session", "complex", complex_data.clone())
        .expect("Failed to update");

    let active = memory.get_active_sessions();
    let (_, session) = &active[0];
    assert_eq!(session.progress.get("complex"), Some(&complex_data));
}

#[test]
fn test_session_persistence() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_file = temp_dir.path().join("session_persist.json");

    // Create session
    {
        let mut memory = AiMemoryHeap::new(&cache_file).expect("Failed to create");
        memory.start_session("persist_session", "Persistent context", vec!["Goal 1".to_string()])
            .expect("Failed to start");
        memory.update_session_progress("persist_session", "saved_data", json!("important"))
            .expect("Failed to update");
    }

    // Reload and verify
    {
        let memory = AiMemoryHeap::new(&cache_file).expect("Failed to reload");
        let active = memory.get_active_sessions();
        assert_eq!(active.len(), 1);

        let (id, session) = &active[0];
        assert_eq!(*id, "persist_session");
        assert_eq!(session.context, "Persistent context");
        assert_eq!(session.progress.get("saved_data"), Some(&json!("important")));
    }
}

#[test]
fn test_session_statistics() {
    let (mut memory, _temp) = create_test_memory();

    memory.start_session("s1", "c1", vec![]).unwrap();
    memory.start_session("s2", "c2", vec![]).unwrap();

    let stats = memory.get_statistics();
    assert_eq!(stats.session_count, 2);
}
