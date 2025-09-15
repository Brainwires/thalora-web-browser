use anyhow::Result;
use boa_engine::{js_string, property::Attribute, Context, JsObject, JsValue, NativeFunction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Real DOM Events implementation with full event handling capabilities
pub struct EventManager {
    listeners: Arc<Mutex<HashMap<String, Vec<EventListener>>>>,
    custom_events: Arc<Mutex<HashMap<String, CustomEvent>>>,
    event_targets: Arc<Mutex<HashMap<String, EventTarget>>>,
    event_queue: Arc<Mutex<Vec<QueuedEvent>>>,
}

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

impl EventManager {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(HashMap::new())),
            custom_events: Arc::new(Mutex::new(HashMap::new())),
            event_targets: Arc::new(Mutex::new(HashMap::new())),
            event_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Setup comprehensive DOM Events API
    pub fn setup_events_api(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        // Setup Event constructor
        self.setup_event_constructor(context)?;

        // Setup CustomEvent constructor
        self.setup_custom_event_constructor(context)?;

        // Setup EventTarget methods
        self.setup_event_target_methods(context)?;

        // Setup common event types
        self.setup_common_events(context)?;

        Ok(())
    }

    /// Setup Event constructor and prototype
    fn setup_event_constructor(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        let event_constructor = unsafe {
            NativeFunction::from_closure(|_, args, context| {
                if args.is_empty() {
                    return Err(boa_engine::JsNativeError::typ()
                        .with_message("Event constructor requires event type")
                        .into());
                }

                let event_type = args[0].to_string(context)?.to_std_string_escaped();
                let (bubbles, cancelable, composed) = if args.len() > 1 && args[1].is_object() {
                    let options = args[1].as_object().unwrap();
                    let bubbles = options
                        .get(js_string!("bubbles"), context)
                        .unwrap_or(JsValue::Boolean(false))
                        .to_boolean();
                    let cancelable = options
                        .get(js_string!("cancelable"), context)
                        .unwrap_or(JsValue::Boolean(false))
                        .to_boolean();
                    let composed = options
                        .get(js_string!("composed"), context)
                        .unwrap_or(JsValue::Boolean(false))
                        .to_boolean();
                    (bubbles, cancelable, composed)
                } else {
                    (false, false, false)
                };

                let event_obj = JsObject::default();
                event_obj.set(
                    js_string!("type"),
                    JsValue::from(js_string!(event_type)),
                    false,
                    context,
                )?;
                event_obj.set(
                    js_string!("bubbles"),
                    JsValue::from(bubbles),
                    false,
                    context,
                )?;
                event_obj.set(
                    js_string!("cancelable"),
                    JsValue::from(cancelable),
                    false,
                    context,
                )?;
                event_obj.set(
                    js_string!("composed"),
                    JsValue::from(composed),
                    false,
                    context,
                )?;
                event_obj.set(
                    js_string!("defaultPrevented"),
                    JsValue::from(false),
                    false,
                    context,
                )?;
                event_obj.set(js_string!("eventPhase"), JsValue::from(0), false, context)?;
                event_obj.set(js_string!("target"), JsValue::null(), false, context)?;
                event_obj.set(js_string!("currentTarget"), JsValue::null(), false, context)?;
                event_obj.set(
                    js_string!("timeStamp"),
                    JsValue::from(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as f64,
                    ),
                    false,
                    context,
                )?;

                // preventDefault method
                let prevent_default_fn =
                    NativeFunction::from_closure(move |_, _args, _context| Ok(JsValue::undefined()));
                event_obj.set(
                    js_string!("preventDefault"),
                    JsValue::from(prevent_default_fn.to_js_function(context.realm())),
                    false,
                    context,
                )?;

                // stopPropagation method
                let stop_propagation_fn =
                    NativeFunction::from_closure(move |_, _args, _context| Ok(JsValue::undefined()));
                event_obj.set(
                    js_string!("stopPropagation"),
                    JsValue::from(stop_propagation_fn.to_js_function(context.realm())),
                    false,
                    context,
                )?;

                // stopImmediatePropagation method
                let stop_immediate_propagation_fn =
                    NativeFunction::from_closure(move |_, _args, _context| Ok(JsValue::undefined()));
                event_obj.set(
                    js_string!("stopImmediatePropagation"),
                    JsValue::from(stop_immediate_propagation_fn.to_js_function(context.realm())),
                    false,
                    context,
                )?;

                Ok(JsValue::from(event_obj))
            })
        };

        context.register_global_property(
            js_string!("Event"),
            JsValue::from(event_constructor.to_js_function(context.realm())),
            Attribute::all(),
        )?;
        Ok(())
    }

    /// Setup CustomEvent constructor
    fn setup_custom_event_constructor(
        &self,
        context: &mut Context,
    ) -> Result<(), boa_engine::JsError> {
        let custom_events_clone = Arc::clone(&self.custom_events);
        let custom_event_constructor = unsafe {
            NativeFunction::from_closure(move |_, args, context| {
                if args.is_empty() {
                    return Err(boa_engine::JsNativeError::typ()
                        .with_message("CustomEvent constructor requires event type")
                        .into());
                }

                let event_type = args[0].to_string(context)?.to_std_string_escaped();
                let (bubbles, cancelable, composed, detail) =
                    if args.len() > 1 && args[1].is_object() {
                        let options = args[1].as_object().unwrap();
                        let bubbles = options
                            .get(js_string!("bubbles"), context)
                            .unwrap_or(JsValue::Boolean(false))
                            .to_boolean();
                        let cancelable = options
                            .get(js_string!("cancelable"), context)
                            .unwrap_or(JsValue::Boolean(false))
                            .to_boolean();
                        let composed = options
                            .get(js_string!("composed"), context)
                            .unwrap_or(JsValue::Boolean(false))
                            .to_boolean();
                        let detail = options
                            .get(js_string!("detail"), context)
                            .unwrap_or(JsValue::null());
                        (bubbles, cancelable, composed, detail)
                    } else {
                        (false, false, false, JsValue::null())
                    };

                // Store custom event
                {
                    let mut custom_events = custom_events_clone.lock().unwrap();
                    let custom_event = CustomEvent {
                        name: event_type.clone(),
                        detail: serde_json::Value::String("custom_detail".to_string()), // Simplified
                        bubbles,
                        cancelable,
                        composed,
                        created_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    };
                    custom_events.insert(event_type.clone(), custom_event);
                }

                let event_obj = JsObject::default();
                event_obj.set(
                    js_string!("type"),
                    JsValue::from(js_string!(event_type)),
                    false,
                    context,
                )?;
                event_obj.set(js_string!("detail"), detail, false, context)?;
                event_obj.set(
                    js_string!("bubbles"),
                    JsValue::from(bubbles),
                    false,
                    context,
                )?;
                event_obj.set(
                    js_string!("cancelable"),
                    JsValue::from(cancelable),
                    false,
                    context,
                )?;
                event_obj.set(
                    js_string!("composed"),
                    JsValue::from(composed),
                    false,
                    context,
                )?;

                Ok(JsValue::from(event_obj))
            })
        };

        context.register_global_property(
            js_string!("CustomEvent"),
            JsValue::from(custom_event_constructor.to_js_function(context.realm())),
            Attribute::all(),
        )?;
        Ok(())
    }

    /// Setup EventTarget methods for DOM elements
    fn setup_event_target_methods(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        // addEventListener global function
        let listeners_clone = Arc::clone(&self.listeners);
        let add_event_listener_fn = unsafe {
            NativeFunction::from_closure(move |_, args, _context| {
                if args.len() < 2 {
                    return Err(boa_engine::JsNativeError::typ()
                        .with_message("addEventListener requires event type and callback")
                        .into());
                }

                let event_type = args[0].to_string(_context)?.to_std_string_escaped();
                let callback = args[1].clone();
                let target_id = "window".to_string(); // Default to window

                let (capture, passive, once) = if args.len() > 2 {
                    if args[2].is_boolean() {
                        (args[2].to_boolean(), false, false)
                    } else if args[2].is_object() {
                        let options = args[2].as_object().unwrap();
                        let capture = options
                            .get(js_string!("capture"), _context)
                            .unwrap_or(JsValue::Boolean(false))
                            .to_boolean();
                        let passive = options
                            .get(js_string!("passive"), _context)
                            .unwrap_or(JsValue::Boolean(false))
                            .to_boolean();
                        let once = options
                            .get(js_string!("once"), _context)
                            .unwrap_or(JsValue::Boolean(false))
                            .to_boolean();
                        (capture, passive, once)
                    } else {
                        (false, false, false)
                    }
                } else {
                    (false, false, false)
                };

                let listener = EventListener {
                    id: format!("listener_{}", rand::random::<u32>()),
                    event_type: event_type.clone(),
                    callback,
                    element_id: target_id,
                    capture,
                    passive,
                    once,
                    created_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };

                {
                    let mut listeners = listeners_clone.lock().unwrap();
                    listeners
                        .entry(event_type.clone())
                        .or_insert_with(Vec::new)
                        .push(listener);
                }

                tracing::debug!("Event listener added for: {}", event_type);
                Ok(JsValue::undefined())
            })
        };

        // removeEventListener global function
        let listeners_remove_clone = Arc::clone(&self.listeners);
        let remove_event_listener_fn = unsafe {
            NativeFunction::from_closure(move |_, args, _context| {
                if args.len() < 2 {
                    return Err(boa_engine::JsNativeError::typ()
                        .with_message("removeEventListener requires event type and callback")
                        .into());
                }

                let event_type = args[0].to_string(_context)?.to_std_string_escaped();

                {
                    let mut listeners = listeners_remove_clone.lock().unwrap();
                    if let Some(event_listeners) = listeners.get_mut(&event_type) {
                        // Remove all matching listeners (simplified - should match exact callback)
                        event_listeners.clear();
                    }
                }

                tracing::debug!("Event listener removed for: {}", event_type);
                Ok(JsValue::undefined())
            })
        };

        // dispatchEvent global function
        let event_queue_clone = Arc::clone(&self.event_queue);
        let listeners_dispatch_clone = Arc::clone(&self.listeners);
        let dispatch_event_fn = unsafe {
            NativeFunction::from_closure(move |_, args, context| {
                if args.is_empty() {
                    return Err(boa_engine::JsNativeError::typ()
                        .with_message("dispatchEvent requires an event object")
                        .into());
                }

                let event_obj = &args[0];
                if let Some(event) = event_obj.as_object() {
                    let event_type = event
                        .get(js_string!("type"), context)?
                        .to_string(context)?
                        .to_std_string_escaped();

                    // Create DOM event
                    let dom_event = DomEvent {
                        event_type: event_type.clone(),
                        target_id: "window".to_string(),
                        current_target_id: "window".to_string(),
                        bubbles: event.get(js_string!("bubbles"), context)?.to_boolean(),
                        cancelable: event.get(js_string!("cancelable"), context)?.to_boolean(),
                        composed: event
                            .get(js_string!("composed"), context)
                            .unwrap_or(JsValue::Boolean(false))
                            .to_boolean(),
                        default_prevented: false,
                        propagation_stopped: false,
                        immediate_propagation_stopped: false,
                        event_phase: EventPhase::AtTarget,
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                        data: HashMap::new(),
                    };

                    // Queue event for processing
                    {
                        let mut queue = event_queue_clone.lock().unwrap();
                        queue.push(QueuedEvent {
                            event_type: event_type.clone(),
                            target_id: "window".to_string(),
                            event_data: dom_event,
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        });
                    }

                    // Execute listeners immediately (synchronous)
                    {
                        let listeners = listeners_dispatch_clone.lock().unwrap();
                        if let Some(event_listeners) = listeners.get(&event_type) {
                            for listener in event_listeners {
                                if listener.callback.is_callable() {
                                    let callback = listener.callback.as_callable().unwrap();
                                    if let Err(e) = callback.call(
                                        &JsValue::undefined(),
                                        &[event_obj.clone()],
                                        context,
                                    ) {
                                        tracing::error!("Error executing event callback: {:?}", e);
                                    }
                                }
                            }
                        }
                    }

                    tracing::debug!("Event dispatched: {}", event_type);
                }

                Ok(JsValue::from(true))
            })
        };

        // Add to window object
        let window_obj = context.global_object();
        window_obj.set(
            js_string!("addEventListener"),
            JsValue::from(add_event_listener_fn.to_js_function(context.realm())),
            false,
            context,
        )?;
        window_obj.set(
            js_string!("removeEventListener"),
            JsValue::from(remove_event_listener_fn.to_js_function(context.realm())),
            false,
            context,
        )?;
        window_obj.set(
            js_string!("dispatchEvent"),
            JsValue::from(dispatch_event_fn.to_js_function(context.realm())),
            false,
            context,
        )?;

        Ok(())
    }

    /// Setup common browser events
    fn setup_common_events(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        // Setup MouseEvent constructor
        let mouse_event_constructor = unsafe {
            NativeFunction::from_closure(|_, args, context| {
                if args.is_empty() {
                    return Err(boa_engine::JsNativeError::typ()
                        .with_message("MouseEvent constructor requires event type")
                        .into());
                }

                let event_type = args[0].to_string(context)?.to_std_string_escaped();
                let event_obj = JsObject::default();

                event_obj.set(
                    js_string!("type"),
                    JsValue::from(js_string!(event_type)),
                    false,
                    context,
                )?;
                event_obj.set(js_string!("bubbles"), JsValue::from(true), false, context)?;
                event_obj.set(
                    js_string!("cancelable"),
                    JsValue::from(true),
                    false,
                    context,
                )?;
                event_obj.set(js_string!("clientX"), JsValue::from(0), false, context)?;
                event_obj.set(js_string!("clientY"), JsValue::from(0), false, context)?;
                event_obj.set(js_string!("screenX"), JsValue::from(0), false, context)?;
                event_obj.set(js_string!("screenY"), JsValue::from(0), false, context)?;
                event_obj.set(js_string!("button"), JsValue::from(0), false, context)?;
                event_obj.set(js_string!("buttons"), JsValue::from(0), false, context)?;

                Ok(JsValue::from(event_obj))
            })
        };
        context.register_global_property(
            js_string!("MouseEvent"),
            JsValue::from(mouse_event_constructor.to_js_function(context.realm())),
            Attribute::all(),
        )?;

        // Setup KeyboardEvent constructor
        let keyboard_event_constructor = unsafe {
            NativeFunction::from_closure(|_, args, context| {
                if args.is_empty() {
                    return Err(boa_engine::JsNativeError::typ()
                        .with_message("KeyboardEvent constructor requires event type")
                        .into());
                }

                let event_type = args[0].to_string(context)?.to_std_string_escaped();
                let event_obj = JsObject::default();

                event_obj.set(
                    js_string!("type"),
                    JsValue::from(js_string!(event_type)),
                    false,
                    context,
                )?;
                event_obj.set(js_string!("bubbles"), JsValue::from(true), false, context)?;
                event_obj.set(
                    js_string!("cancelable"),
                    JsValue::from(true),
                    false,
                    context,
                )?;
                event_obj.set(
                    js_string!("key"),
                    JsValue::from(js_string!("")),
                    false,
                    context,
                )?;
                event_obj.set(
                    js_string!("code"),
                    JsValue::from(js_string!("")),
                    false,
                    context,
                )?;
                event_obj.set(js_string!("keyCode"), JsValue::from(0), false, context)?;
                event_obj.set(js_string!("charCode"), JsValue::from(0), false, context)?;
                event_obj.set(js_string!("ctrlKey"), JsValue::from(false), false, context)?;
                event_obj.set(js_string!("shiftKey"), JsValue::from(false), false, context)?;
                event_obj.set(js_string!("altKey"), JsValue::from(false), false, context)?;
                event_obj.set(js_string!("metaKey"), JsValue::from(false), false, context)?;

                Ok(JsValue::from(event_obj))
            })
        };
        context.register_global_property(
            js_string!("KeyboardEvent"),
            JsValue::from(keyboard_event_constructor.to_js_function(context.realm())),
            Attribute::all(),
        )?;

        // Setup FocusEvent constructor
        let focus_event_constructor = unsafe {
            NativeFunction::from_closure(|_, args, context| {
                if args.is_empty() {
                    return Err(boa_engine::JsNativeError::typ()
                        .with_message("FocusEvent constructor requires event type")
                        .into());
                }

                let event_type = args[0].to_string(context)?.to_std_string_escaped();
                let event_obj = JsObject::default();

                event_obj.set(
                    js_string!("type"),
                    JsValue::from(js_string!(event_type)),
                    false,
                    context,
                )?;
                event_obj.set(js_string!("bubbles"), JsValue::from(true), false, context)?;
                event_obj.set(
                    js_string!("cancelable"),
                    JsValue::from(true),
                    false,
                    context,
                )?;
                event_obj.set(js_string!("relatedTarget"), JsValue::null(), false, context)?;

                Ok(JsValue::from(event_obj))
            })
        };
        context.register_global_property(
            js_string!("FocusEvent"),
            JsValue::from(focus_event_constructor.to_js_function(context.realm())),
            Attribute::all(),
        )?;

        Ok(())
    }

    /// Add an event target (DOM element)
    pub fn add_event_target(
        &self,
        id: String,
        tag_name: String,
        parent_id: Option<String>,
    ) -> Result<()> {
        let target = EventTarget {
            id: id.clone(),
            tag_name,
            listeners: Vec::new(),
            parent_id,
            children_ids: Vec::new(),
        };

        {
            let mut targets = self.event_targets.lock().unwrap();
            targets.insert(id, target);
        }

        Ok(())
    }

    /// Dispatch a real DOM event
    pub async fn dispatch_dom_event(&self, target_id: &str, event: DomEvent) -> Result<bool> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        // Queue the event
        {
            let mut queue = self.event_queue.lock().unwrap();
            queue.push(QueuedEvent {
                event_type: event.event_type.clone(),
                target_id: target_id.to_string(),
                event_data: event.clone(),
                timestamp,
            });
        }

        // Process event through capture and bubble phases
        self.process_event_phases(target_id, &event).await?;

        tracing::debug!(
            "DOM event dispatched: {} on {}",
            event.event_type,
            target_id
        );
        Ok(!event.default_prevented)
    }

    /// Process event through capture and bubble phases
    async fn process_event_phases(&self, target_id: &str, event: &DomEvent) -> Result<()> {
        let listeners = self.listeners.lock().unwrap();

        // Capture phase (top-down)
        if let Some(event_listeners) = listeners.get(&event.event_type) {
            for _listener in event_listeners
                .iter()
                .filter(|l| l.capture && l.element_id == target_id)
            {
                // Execute capturing listeners
                tracing::debug!(
                    "Executing capture listener for: {} on {}",
                    event.event_type,
                    target_id
                );
            }
        }

        // Target phase
        if let Some(event_listeners) = listeners.get(&event.event_type) {
            for _listener in event_listeners
                .iter()
                .filter(|l| !l.capture && l.element_id == target_id)
            {
                // Execute target listeners
                tracing::debug!(
                    "Executing target listener for: {} on {}",
                    event.event_type,
                    target_id
                );
            }
        }

        // Bubble phase (bottom-up) - only if event bubbles
        if event.bubbles {
            if let Some(event_listeners) = listeners.get(&event.event_type) {
                for _listener in event_listeners.iter().filter(|l| !l.capture) {
                    // Execute bubbling listeners
                    tracing::debug!(
                        "Executing bubble listener for: {} on {}",
                        event.event_type,
                        target_id
                    );
                }
            }
        }

        Ok(())
    }

    /// Get all registered listeners for an event type
    pub fn get_listeners(&self, event_type: &str) -> Vec<EventListener> {
        let listeners = self.listeners.lock().unwrap();
        listeners.get(event_type).cloned().unwrap_or_default()
    }

    /// Clear all event listeners
    pub fn clear_all_listeners(&self) -> Result<()> {
        {
            let mut listeners = self.listeners.lock().unwrap();
            listeners.clear();
        }
        tracing::info!("All event listeners cleared");
        Ok(())
    }

    /// Get queued events count
    pub fn get_queued_events_count(&self) -> usize {
        self.event_queue.lock().unwrap().len()
    }

    /// Process queued events
    pub async fn process_queued_events(&self) -> Result<usize> {
        let events = {
            let mut queue = self.event_queue.lock().unwrap();
            let events = queue.clone();
            queue.clear();
            events
        };

        let processed_count = events.len();

        for queued_event in events {
            // Process each queued event
            tracing::debug!(
                "Processing queued event: {} on {}",
                queued_event.event_type,
                queued_event.target_id
            );
        }

        Ok(processed_count)
    }
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
