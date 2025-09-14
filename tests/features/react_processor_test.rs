// Tests for src/features/react_processor.rs
#[cfg(test)]
mod react_processor_tests {
    use synaptic::features::react_processor::*;

    #[test]
    fn test_react_processor_creation() {
        let processor = ReactProcessor::new();
        // Basic creation test - more complex tests would need mock HTML
        assert_eq!(processor.streaming_data.len(), 0);
    }

    #[test]
    fn test_process_next_streaming_with_empty_html() {
        let mut processor = ReactProcessor::new();
        let result = processor.process_next_streaming("");
        // Should handle empty HTML gracefully
        assert!(result.is_ok());
    }
}