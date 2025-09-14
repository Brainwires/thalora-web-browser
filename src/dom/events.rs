use boa_engine::JsValue;

#[derive(Debug, Clone)]
pub struct EventListener {
    pub event_type: String,
    pub callback: JsValue,
    pub element_id: String,
    pub capture: bool,
    pub passive: bool,
    pub once: bool,
}

impl EventListener {
    pub fn new(event_type: String, callback: JsValue, element_id: String) -> Self {
        Self {
            event_type,
            callback,
            element_id,
            capture: false,
            passive: false,
            once: false,
        }
    }

    pub fn with_options(
        event_type: String,
        callback: JsValue,
        element_id: String,
        capture: bool,
        passive: bool,
        once: bool,
    ) -> Self {
        Self {
            event_type,
            callback,
            element_id,
            capture,
            passive,
            once,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DomEvent {
    pub event_type: String,
    pub target_id: String,
    pub bubbles: bool,
    pub cancelable: bool,
    pub data: std::collections::HashMap<String, String>,
}

impl DomEvent {
    pub fn new(event_type: String, target_id: String) -> Self {
        Self {
            event_type,
            target_id,
            bubbles: true,
            cancelable: true,
            data: std::collections::HashMap::new(),
        }
    }

    pub fn with_data(mut self, key: String, value: String) -> Self {
        self.data.insert(key, value);
        self
    }
}