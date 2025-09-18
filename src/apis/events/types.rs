use anyhow::Result;
use boa_engine::JsValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct EventListener {
    pub id: String,
    pub event_type: String,
    pub callback: JsValue,
    pub element_id: String,
    pub capture: bool,
    pub passive: bool,
    pub once: bool,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomEvent {
    pub name: String,
    pub detail: serde_json::Value,
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub created_at: u64,
}

#[derive(Debug, Clone)]
pub struct EventTarget {
    pub id: String,
    pub tag_name: String,
    pub listeners: Vec<String>, // listener IDs
    pub parent_id: Option<String>,
    pub children_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct QueuedEvent {
    pub event_type: String,
    pub target_id: String,
    pub event_data: DomEvent,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct DomEvent {
    pub event_type: String,
    pub target_id: String,
    pub current_target_id: String,
    pub bubbles: bool,
    pub cancelable: bool,
    pub composed: bool,
    pub default_prevented: bool,
    pub propagation_stopped: bool,
    pub immediate_propagation_stopped: bool,
    pub event_phase: EventPhase,
    pub timestamp: u64,
    pub data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPhase {
    None = 0,
    CapturingPhase = 1,
    AtTarget = 2,
    BubblingPhase = 3,
}

impl EventListener {
    pub fn new(event_type: String, callback: JsValue, element_id: String) -> Self {
        Self {
            id: format!("listener_{}", rand::random::<u32>()),
            event_type,
            callback,
            element_id,
            capture: false,
            passive: false,
            once: false,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
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
            id: format!("listener_{}", rand::random::<u32>()),
            event_type,
            callback,
            element_id,
            capture,
            passive,
            once,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

impl DomEvent {
    pub fn new(event_type: String, target_id: String) -> Self {
        Self {
            event_type,
            target_id: target_id.clone(),
            current_target_id: target_id,
            bubbles: true,
            cancelable: true,
            composed: false,
            default_prevented: false,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
            event_phase: EventPhase::None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            data: HashMap::new(),
        }
    }

    pub fn with_data(mut self, key: String, value: serde_json::Value) -> Self {
        self.data.insert(key, value);
        self
    }

    pub fn prevent_default(&mut self) {
        if self.cancelable {
            self.default_prevented = true;
        }
    }

    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    pub fn stop_immediate_propagation(&mut self) {
        self.immediate_propagation_stopped = true;
        self.propagation_stopped = true;
    }
}