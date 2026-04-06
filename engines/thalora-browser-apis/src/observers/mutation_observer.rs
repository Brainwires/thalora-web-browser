//! MutationObserver Web API implementation for Boa
//!
//! Native implementation of the MutationObserver standard
//! https://dom.spec.whatwg.org/#interface-mutationobserver
//!
//! This implements the complete MutationObserver interface for watching DOM changes

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    value::JsValue,
};
use boa_gc::{Finalize, Trace};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Global registry of active mutation observers for DOM integration
static MUTATION_OBSERVERS: Lazy<Arc<Mutex<Vec<WeakObserverRef>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

/// Weak reference to an observer (using object pointer as ID)
#[derive(Debug, Clone)]
struct WeakObserverRef {
    observer_id: usize,
}

/// JavaScript `MutationObserver` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub struct MutationObserver;

impl IntrinsicObject for MutationObserver {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::observe, js_string!("observe"), 2)
            .method(Self::disconnect, js_string!("disconnect"), 0)
            .method(Self::take_records, js_string!("takeRecords"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for MutationObserver {
    const NAME: JsString = js_string!("MutationObserver");
}

impl BuiltInConstructor for MutationObserver {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::mutation_observer;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::mutation_observer,
            context,
        )?;

        // Get the callback function (required parameter)
        let callback = args.get_or_undefined(0);
        if !callback.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("MutationObserver constructor requires a callback function")
                .into());
        }

        // Create MutationObserver data
        let observer_data = MutationObserverData {
            callback: callback.clone(),
            observations: HashMap::new(),
            records: Vec::new(),
            is_observing: false,
        };

        let observer_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            observer_data,
        );

        Ok(observer_obj.into())
    }
}

impl MutationObserver {
    /// `MutationObserver.prototype.observe()` method
    fn observe(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("MutationObserver.observe called on non-object")
        })?;

        let target = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // Validate target (should be a Node, but for now accept any object)
        if !target.is_object() {
            return Err(JsNativeError::typ()
                .with_message("MutationObserver.observe: target must be a Node")
                .into());
        }

        // Parse options (MutationObserverInit)
        let mut config = MutationObserverConfig::default();

        if let Some(options_obj) = options.as_object() {
            // Parse childList option
            if let Ok(child_list) = options_obj.get(js_string!("childList"), context) {
                config.child_list = child_list.to_boolean();
            }

            // Parse attributes option
            if let Ok(attributes) = options_obj.get(js_string!("attributes"), context) {
                config.attributes = Some(attributes.to_boolean());
            }

            // Parse characterData option
            if let Ok(character_data) = options_obj.get(js_string!("characterData"), context) {
                config.character_data = Some(character_data.to_boolean());
            }

            // Parse subtree option
            if let Ok(subtree) = options_obj.get(js_string!("subtree"), context) {
                config.subtree = subtree.to_boolean();
            }

            // Parse attributeOldValue option
            if let Ok(attr_old_value) = options_obj.get(js_string!("attributeOldValue"), context) {
                config.attribute_old_value = Some(attr_old_value.to_boolean());
            }

            // Parse characterDataOldValue option
            if let Ok(char_old_value) =
                options_obj.get(js_string!("characterDataOldValue"), context)
            {
                config.character_data_old_value = Some(char_old_value.to_boolean());
            }

            // Parse attributeFilter option
            if let Ok(attr_filter) = options_obj.get(js_string!("attributeFilter"), context)
                && let Some(filter_obj) = attr_filter.as_object()
            {
                let mut filter_vec = Vec::new();
                // Get length of array
                if let Ok(length_val) = filter_obj.get(js_string!("length"), context) {
                    let length = length_val.to_u32(context).unwrap_or(0);
                    for i in 0..length {
                        if let Ok(item) = filter_obj.get(i, context)
                            && let Ok(s) = item.to_string(context)
                        {
                            filter_vec.push(s.to_std_string_escaped());
                        }
                    }
                }
                if !filter_vec.is_empty() {
                    config.attribute_filter = Some(filter_vec);
                }
            }
        }

        // Validate configuration per spec
        // If attributeOldValue or attributeFilter is set, attributes is implied true
        if (config.attribute_old_value.unwrap_or(false) || config.attribute_filter.is_some())
            && config.attributes.is_none()
        {
            config.attributes = Some(true);
        }

        // If characterDataOldValue is set, characterData is implied true
        if config.character_data_old_value.unwrap_or(false) && config.character_data.is_none() {
            config.character_data = Some(true);
        }

        // Validate: at least one of childList, attributes, or characterData must be true
        let observes_children = config.child_list;
        let observes_attributes = config.attributes.unwrap_or(false);
        let observes_character_data = config.character_data.unwrap_or(false);

        if !observes_children && !observes_attributes && !observes_character_data {
            return Err(JsNativeError::typ()
                .with_message("MutationObserver.observe: At least one of childList, attributes, or characterData must be true")
                .into());
        }

        // Store the target object reference for later use
        let target_obj = target.as_object().unwrap().clone();

        // Update observer data
        if let Some(mut observer_data) = observer_obj.downcast_mut::<MutationObserverData>() {
            let target_id = format!("{:p}", target_obj.as_ref());
            observer_data.observations.insert(
                target_id,
                ObservationEntry {
                    target: target_obj,
                    config,
                },
            );
            observer_data.is_observing = true;
        }

        // Register in global observer registry for mutation notifications
        register_observer(observer_obj.clone());

        Ok(JsValue::undefined())
    }

    /// `MutationObserver.prototype.disconnect()` method
    fn disconnect(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("MutationObserver.disconnect called on non-object")
        })?;

        if let Some(mut observer_data) = observer_obj.downcast_mut::<MutationObserverData>() {
            observer_data.observations.clear();
            observer_data.records.clear();
            observer_data.is_observing = false;
        }

        // Unregister from global observer registry
        unregister_observer(&observer_obj);

        Ok(JsValue::undefined())
    }

    /// `MutationObserver.prototype.takeRecords()` method
    fn take_records(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("MutationObserver.takeRecords called on non-object")
        })?;

        let mut observer_data = observer_obj
            .downcast_mut::<MutationObserverData>()
            .ok_or_else(|| {
                JsNativeError::typ().with_message(
                    "MutationObserver.takeRecords called on non-MutationObserver object",
                )
            })?;

        // Take records and clear the queue
        let records = std::mem::take(&mut observer_data.records);

        // Create JavaScript array of MutationRecord objects
        let records_array =
            boa_engine::builtins::array::Array::array_create(records.len() as u64, None, context)?;

        for (index, record) in records.into_iter().enumerate() {
            let record_obj = record.to_js_object(context)?;
            records_array.set(index, record_obj, false, context)?;
        }

        Ok(records_array.into())
    }
}

/// Internal data for MutationObserver instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct MutationObserverData {
    /// The callback function to call when mutations occur
    callback: JsValue,
    /// Map of target nodes being observed and their configurations
    #[unsafe_ignore_trace]
    observations: HashMap<String, ObservationEntry>,
    /// Queue of mutation records waiting to be delivered
    #[unsafe_ignore_trace]
    records: Vec<MutationRecordData>,
    /// Whether this observer is currently observing any targets
    #[unsafe_ignore_trace]
    is_observing: bool,
}

impl MutationObserverData {
    /// Queue a mutation record
    pub fn queue_record(&mut self, record: MutationRecordData) {
        self.records.push(record);
    }

    /// Get the callback function
    pub fn callback(&self) -> &JsValue {
        &self.callback
    }

    /// Check if observing a target
    pub fn is_observing_target(&self, target_id: &str) -> bool {
        self.observations.contains_key(target_id)
    }

    /// Get configuration for a target
    pub fn get_config(&self, target_id: &str) -> Option<&MutationObserverConfig> {
        self.observations.get(target_id).map(|e| &e.config)
    }

    /// Check if there are pending records
    pub fn has_pending_records(&self) -> bool {
        !self.records.is_empty()
    }
}

/// Entry for an observed target
#[derive(Debug)]
struct ObservationEntry {
    target: JsObject,
    config: MutationObserverConfig,
}

/// Configuration for what mutations to observe
#[derive(Debug, Clone, Default)]
pub struct MutationObserverConfig {
    /// Observe changes to the list of child nodes
    pub child_list: bool,
    /// Observe changes to attributes
    pub attributes: Option<bool>,
    /// Observe changes to character data
    pub character_data: Option<bool>,
    /// Observe changes to descendants
    pub subtree: bool,
    /// Include old attribute values in records
    pub attribute_old_value: Option<bool>,
    /// Include old character data values in records
    pub character_data_old_value: Option<bool>,
    /// Filter for specific attribute names
    pub attribute_filter: Option<Vec<String>>,
}

impl MutationObserverConfig {
    /// Check if this config observes attribute changes
    pub fn observes_attributes(&self) -> bool {
        self.attributes.unwrap_or(false)
    }

    /// Check if this config observes character data changes
    pub fn observes_character_data(&self) -> bool {
        self.character_data.unwrap_or(false)
    }

    /// Check if a specific attribute should be observed
    pub fn should_observe_attribute(&self, attr_name: &str) -> bool {
        if !self.observes_attributes() {
            return false;
        }
        match &self.attribute_filter {
            Some(filter) => filter.iter().any(|name| name == attr_name),
            None => true, // No filter means observe all attributes
        }
    }
}

/// Represents a single mutation record (internal data)
#[derive(Debug, Clone)]
pub struct MutationRecordData {
    /// Type of mutation: "childList", "attributes", or "characterData"
    pub mutation_type: MutationType,
    /// The node that was mutated
    pub target: Option<JsObject>,
    /// Nodes that were added (for childList)
    pub added_nodes: Vec<JsObject>,
    /// Nodes that were removed (for childList)
    pub removed_nodes: Vec<JsObject>,
    /// Previous sibling of added/removed nodes
    pub previous_sibling: Option<JsObject>,
    /// Next sibling of added/removed nodes
    pub next_sibling: Option<JsObject>,
    /// Name of changed attribute (for attributes type)
    pub attribute_name: Option<String>,
    /// Namespace of changed attribute (for attributes type)
    pub attribute_namespace: Option<String>,
    /// Old value of attribute or character data
    pub old_value: Option<String>,
}

/// Type of mutation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutationType {
    ChildList,
    Attributes,
    CharacterData,
}

impl MutationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MutationType::ChildList => "childList",
            MutationType::Attributes => "attributes",
            MutationType::CharacterData => "characterData",
        }
    }
}

impl MutationRecordData {
    /// Create a new childList mutation record
    pub fn child_list(
        target: JsObject,
        added_nodes: Vec<JsObject>,
        removed_nodes: Vec<JsObject>,
        previous_sibling: Option<JsObject>,
        next_sibling: Option<JsObject>,
    ) -> Self {
        Self {
            mutation_type: MutationType::ChildList,
            target: Some(target),
            added_nodes,
            removed_nodes,
            previous_sibling,
            next_sibling,
            attribute_name: None,
            attribute_namespace: None,
            old_value: None,
        }
    }

    /// Create a new attributes mutation record
    pub fn attributes(
        target: JsObject,
        attribute_name: String,
        attribute_namespace: Option<String>,
        old_value: Option<String>,
    ) -> Self {
        Self {
            mutation_type: MutationType::Attributes,
            target: Some(target),
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
            previous_sibling: None,
            next_sibling: None,
            attribute_name: Some(attribute_name),
            attribute_namespace,
            old_value,
        }
    }

    /// Create a new characterData mutation record
    pub fn character_data(target: JsObject, old_value: Option<String>) -> Self {
        Self {
            mutation_type: MutationType::CharacterData,
            target: Some(target),
            added_nodes: Vec::new(),
            removed_nodes: Vec::new(),
            previous_sibling: None,
            next_sibling: None,
            attribute_name: None,
            attribute_namespace: None,
            old_value,
        }
    }

    /// Convert to a JavaScript MutationRecord object
    pub fn to_js_object(&self, context: &mut Context) -> JsResult<JsObject> {
        let obj = JsObject::with_object_proto(context.intrinsics());

        // Set type
        obj.set(
            js_string!("type"),
            js_string!(self.mutation_type.as_str()),
            false,
            context,
        )?;

        // Set target
        obj.set(
            js_string!("target"),
            self.target.clone().map_or(JsValue::null(), |t| t.into()),
            false,
            context,
        )?;

        // Set addedNodes as NodeList-like array
        let added_nodes_array = boa_engine::builtins::array::Array::array_create(
            self.added_nodes.len() as u64,
            None,
            context,
        )?;
        for (i, node) in self.added_nodes.iter().enumerate() {
            added_nodes_array.set(i, node.clone(), false, context)?;
        }
        obj.set(js_string!("addedNodes"), added_nodes_array, false, context)?;

        // Set removedNodes as NodeList-like array
        let removed_nodes_array = boa_engine::builtins::array::Array::array_create(
            self.removed_nodes.len() as u64,
            None,
            context,
        )?;
        for (i, node) in self.removed_nodes.iter().enumerate() {
            removed_nodes_array.set(i, node.clone(), false, context)?;
        }
        obj.set(
            js_string!("removedNodes"),
            removed_nodes_array,
            false,
            context,
        )?;

        // Set previousSibling
        obj.set(
            js_string!("previousSibling"),
            self.previous_sibling
                .clone()
                .map_or(JsValue::null(), |s| s.into()),
            false,
            context,
        )?;

        // Set nextSibling
        obj.set(
            js_string!("nextSibling"),
            self.next_sibling
                .clone()
                .map_or(JsValue::null(), |s| s.into()),
            false,
            context,
        )?;

        // Set attributeName
        obj.set(
            js_string!("attributeName"),
            self.attribute_name
                .as_ref()
                .map_or(JsValue::null(), |n| js_string!(n.as_str()).into()),
            false,
            context,
        )?;

        // Set attributeNamespace
        obj.set(
            js_string!("attributeNamespace"),
            self.attribute_namespace
                .as_ref()
                .map_or(JsValue::null(), |n| js_string!(n.as_str()).into()),
            false,
            context,
        )?;

        // Set oldValue
        obj.set(
            js_string!("oldValue"),
            self.old_value
                .as_ref()
                .map_or(JsValue::null(), |v| js_string!(v.as_str()).into()),
            false,
            context,
        )?;

        Ok(obj)
    }
}

// ============================================================================
// MutationRecord JavaScript class (frozen interface per spec)
// ============================================================================

/// JavaScript `MutationRecord` interface - read-only record of a mutation
#[derive(Debug, Copy, Clone)]
pub struct MutationRecord;

impl IntrinsicObject for MutationRecord {
    fn init(realm: &Realm) {
        // MutationRecord has no constructor - it's created internally
        // But we need to define the prototype for proper instanceof checks

        let type_getter = BuiltInBuilder::callable(realm, get_type)
            .name(js_string!("get type"))
            .build();

        let target_getter = BuiltInBuilder::callable(realm, get_target)
            .name(js_string!("get target"))
            .build();

        let added_nodes_getter = BuiltInBuilder::callable(realm, get_added_nodes)
            .name(js_string!("get addedNodes"))
            .build();

        let removed_nodes_getter = BuiltInBuilder::callable(realm, get_removed_nodes)
            .name(js_string!("get removedNodes"))
            .build();

        let previous_sibling_getter = BuiltInBuilder::callable(realm, get_previous_sibling)
            .name(js_string!("get previousSibling"))
            .build();

        let next_sibling_getter = BuiltInBuilder::callable(realm, get_next_sibling)
            .name(js_string!("get nextSibling"))
            .build();

        let attribute_name_getter = BuiltInBuilder::callable(realm, get_attribute_name)
            .name(js_string!("get attributeName"))
            .build();

        let attribute_namespace_getter = BuiltInBuilder::callable(realm, get_attribute_namespace)
            .name(js_string!("get attributeNamespace"))
            .build();

        let old_value_getter = BuiltInBuilder::callable(realm, get_old_value)
            .name(js_string!("get oldValue"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("type"),
                Some(type_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("target"),
                Some(target_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("addedNodes"),
                Some(added_nodes_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("removedNodes"),
                Some(removed_nodes_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("previousSibling"),
                Some(previous_sibling_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("nextSibling"),
                Some(next_sibling_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("attributeName"),
                Some(attribute_name_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("attributeNamespace"),
                Some(attribute_namespace_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("oldValue"),
                Some(old_value_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for MutationRecord {
    const NAME: JsString = js_string!("MutationRecord");
}

impl BuiltInConstructor for MutationRecord {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 18; // Accessors on prototype
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::mutation_record;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // MutationRecord cannot be constructed directly
        Err(JsNativeError::typ()
            .with_message("MutationRecord is not a constructor")
            .into())
    }
}

// MutationRecord getters - these work on plain objects with the right properties
fn get_type(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MutationRecord getter called on non-object")
    })?;
    obj.get(js_string!("type"), context)
}

fn get_target(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MutationRecord getter called on non-object")
    })?;
    obj.get(js_string!("target"), context)
}

fn get_added_nodes(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MutationRecord getter called on non-object")
    })?;
    obj.get(js_string!("addedNodes"), context)
}

fn get_removed_nodes(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MutationRecord getter called on non-object")
    })?;
    obj.get(js_string!("removedNodes"), context)
}

fn get_previous_sibling(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MutationRecord getter called on non-object")
    })?;
    obj.get(js_string!("previousSibling"), context)
}

fn get_next_sibling(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MutationRecord getter called on non-object")
    })?;
    obj.get(js_string!("nextSibling"), context)
}

fn get_attribute_name(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MutationRecord getter called on non-object")
    })?;
    obj.get(js_string!("attributeName"), context)
}

fn get_attribute_namespace(
    this: &JsValue,
    _: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MutationRecord getter called on non-object")
    })?;
    obj.get(js_string!("attributeNamespace"), context)
}

fn get_old_value(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("MutationRecord getter called on non-object")
    })?;
    obj.get(js_string!("oldValue"), context)
}

// --- Global MutationObserver notification system ---

use std::cell::RefCell;

thread_local! {
    /// Thread-local registry of active observer objects for notification dispatch.
    /// Boa's JsObject is !Send, so we use thread_local instead of a global static.
    static OBSERVER_REGISTRY: RefCell<Vec<JsObject>> = const { RefCell::new(Vec::new()) };
}

/// Register an observer in the thread-local registry for notification dispatch.
pub fn register_observer(observer: JsObject) {
    OBSERVER_REGISTRY.with(|registry| {
        let mut reg = registry.borrow_mut();
        // Avoid duplicate registration (compare by pointer)
        let ptr = observer.as_ref() as *const _ as usize;
        if !reg.iter().any(|o| o.as_ref() as *const _ as usize == ptr) {
            reg.push(observer);
        }
    });
}

/// Unregister an observer from the thread-local registry.
pub fn unregister_observer(observer: &JsObject) {
    OBSERVER_REGISTRY.with(|registry| {
        let mut reg = registry.borrow_mut();
        let ptr = observer.as_ref() as *const _ as usize;
        reg.retain(|o| o.as_ref() as *const _ as usize != ptr);
    });
}

/// Notify all registered MutationObservers about a mutation.
///
/// This should be called by DOM mutation methods (setAttribute, appendChild, etc.)
/// when they modify the DOM. The function checks each observer's target and config
/// to determine if the mutation should be recorded.
///
/// For `subtree: true`, this checks if `mutated_node` is a descendant of the
/// observer's target by walking up the parent chain.
///
/// `mutated_node_ptr` is the pointer-as-string ID of the node that was mutated.
/// `parent_ptrs` is a list of ancestor pointer IDs (from mutated node up to root),
/// used for subtree matching.
pub fn notify_attribute_mutation(
    mutated_node: &JsObject,
    attribute_name: &str,
    old_value: Option<String>,
) {
    let target_id = format!("{:p}", mutated_node.as_ref());

    OBSERVER_REGISTRY.with(|registry| {
        let observers = registry.borrow().clone();
        for observer_obj in &observers {
            if let Some(mut observer_data) = observer_obj.downcast_mut::<MutationObserverData>() {
                let mut should_record = false;
                let mut record_old_value = false;

                // Check direct target match
                if let Some(config) = observer_data.get_config(&target_id)
                    && config.should_observe_attribute(attribute_name)
                {
                    should_record = true;
                    record_old_value = config.attribute_old_value.unwrap_or(false);
                }

                // Check subtree observations
                if !should_record {
                    for entry in observer_data.observations.values() {
                        if entry.config.subtree
                            && entry.config.should_observe_attribute(attribute_name)
                            && is_descendant_of(mutated_node, &entry.target)
                        {
                            should_record = true;
                            record_old_value = entry.config.attribute_old_value.unwrap_or(false);
                            break;
                        }
                    }
                }

                if should_record {
                    let record = MutationRecordData::attributes(
                        mutated_node.clone(),
                        attribute_name.to_string(),
                        None,
                        if record_old_value {
                            old_value.clone()
                        } else {
                            None
                        },
                    );
                    observer_data.queue_record(record);
                }
            }
        }
    });
}

/// Notify about childList mutations (appendChild, removeChild, etc.)
pub fn notify_child_list_mutation(
    parent_node: &JsObject,
    added_nodes: Vec<JsObject>,
    removed_nodes: Vec<JsObject>,
    previous_sibling: Option<JsObject>,
    next_sibling: Option<JsObject>,
) {
    let target_id = format!("{:p}", parent_node.as_ref());

    OBSERVER_REGISTRY.with(|registry| {
        let observers = registry.borrow().clone();
        for observer_obj in &observers {
            if let Some(mut observer_data) = observer_obj.downcast_mut::<MutationObserverData>() {
                let mut should_record = false;

                if let Some(config) = observer_data.get_config(&target_id)
                    && config.child_list
                {
                    should_record = true;
                }

                // Check subtree observations
                if !should_record {
                    for entry in observer_data.observations.values() {
                        if entry.config.subtree
                            && entry.config.child_list
                            && (is_descendant_of(parent_node, &entry.target)
                                || std::ptr::eq(parent_node.as_ref(), entry.target.as_ref()))
                        {
                            should_record = true;
                            break;
                        }
                    }
                }

                if should_record {
                    let record = MutationRecordData::child_list(
                        parent_node.clone(),
                        added_nodes.clone(),
                        removed_nodes.clone(),
                        previous_sibling.clone(),
                        next_sibling.clone(),
                    );
                    observer_data.queue_record(record);
                }
            }
        }
    });
}

/// Check if `node` is a descendant of `ancestor` by walking up the parent chain.
fn is_descendant_of(node: &JsObject, ancestor: &JsObject) -> bool {
    use crate::dom::element::ElementData;

    // Walk up the parent chain from node
    let mut current = node.clone();
    for _ in 0..100 {
        // Safety limit to prevent infinite loops
        let parent = if let Some(element_data) = current.downcast_ref::<ElementData>() {
            element_data.get_parent_node()
        } else {
            None
        };

        match parent {
            Some(p) => {
                if std::ptr::eq(p.as_ref(), ancestor.as_ref()) {
                    return true;
                }
                current = p;
            }
            None => break,
        }
    }
    false
}
