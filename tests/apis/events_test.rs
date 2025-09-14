// Tests for src/apis/events.rs
#[cfg(test)]
mod events_tests {
    use synaptic::apis::events::*;

    #[test]
    fn test_event_listener_creation() {
        let listener = EventListener::new(
            "click".to_string(),
            boa_engine::JsValue::undefined(),
            "element1".to_string()
        );
        assert_eq!(listener.event_type, "click");
        assert_eq!(listener.element_id, "element1");
    }

    #[test]
    fn test_dom_event_creation() {
        let event = DomEvent::new("click".to_string(), "button1".to_string());
        assert_eq!(event.event_type, "click");
        assert_eq!(event.target_id, "button1");
    }
}