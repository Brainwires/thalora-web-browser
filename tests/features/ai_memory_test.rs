// Tests for src/features/ai_memory.rs
#[cfg(test)]
mod ai_memory_tests {
    use synaptic::features::ai_memory::*;

    #[test]
    fn test_ai_memory_heap_creation() {
        let memory = AiMemoryHeap::new().expect("Failed to create AiMemoryHeap");
        let stats = memory.get_statistics();
        assert_eq!(stats.total_research_entries, 0);
        assert_eq!(stats.total_credentials, 0);
    }

    #[test]
    fn test_research_entry_operations() {
        let mut memory = AiMemoryHeap::new().expect("Failed to create AiMemoryHeap");

        let entry = ResearchEntry {
            topic: "test topic".to_string(),
            query: "test query".to_string(),
            results: vec!["result1".to_string(), "result2".to_string()],
            sources: vec!["source1".to_string()],
            timestamp: chrono::Utc::now(),
            relevance_score: 0.8,
            summary: Some("test summary".to_string()),
            tags: vec!["tag1".to_string()],
            session_id: Some("session1".to_string()),
        };

        let result = memory.store_research("test_key", entry);
        assert!(result.is_ok());

        let retrieved = memory.get_research("test_key");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().topic, "test topic");
    }
}