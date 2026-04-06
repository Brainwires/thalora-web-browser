//! NodeList interface implementation for DOM Level 4
//!
//! The NodeList interface represents a collection of nodes.
//! https://dom.spec.whatwg.org/#interface-nodelist
//!
//! This implementation supports both:
//! - **Static NodeLists**: Snapshot of nodes at creation time (e.g., querySelectorAll)
//! - **Live NodeLists**: Always reflects current DOM state (e.g., Node.childNodes)

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    string::{JsString, StaticJsStrings},
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex, Weak};

/// Source for a live NodeList - provides nodes dynamically
#[derive(Debug, Clone)]
pub enum NodeListSource {
    /// Static list of nodes (snapshot)
    Static(Arc<Mutex<Vec<JsObject>>>),
    /// Live reference to a parent node's child list
    /// Uses Weak to avoid preventing parent GC
    LiveChildNodes(Weak<Mutex<Vec<JsObject>>>),
}

// Finalize implementation for NodeListSource
impl Finalize for NodeListSource {
    fn finalize(&self) {
        // No special cleanup needed - Arc/Weak handle their own cleanup
    }
}

// Manual Trace implementation since we can't auto-derive for Arc/Weak
// This is safe because:
// 1. Static nodes are owned by this NodeList and don't contain cycles
// 2. Live nodes are referenced weakly, so the parent owns them and handles tracing
unsafe impl Trace for NodeListSource {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // Arc and Weak are not GC-managed, but the JsObjects inside are
        // The parent NodeData holds the strong references to the actual nodes
        // For Static: the nodes are also in the DOM tree which traces them
        // For Live: we only hold a weak reference, no tracing needed
    }

    unsafe fn trace_non_roots(&self) {
        // No GC handles in heap to trace
    }

    fn run_finalizer(&self) {
        self.finalize();
    }
}

/// The NodeList data implementation
#[derive(Debug, Trace, Finalize, JsData)]
pub struct NodeListData {
    /// The source of nodes (static or live reference)
    #[unsafe_ignore_trace]
    source: NodeListSource,
    /// Whether this is a live NodeList (updates automatically)
    live: bool,
}

impl NodeListData {
    /// Create a new static NodeList with the given nodes
    pub fn new(nodes: Vec<JsObject>, live: bool) -> Self {
        Self {
            source: NodeListSource::Static(Arc::new(Mutex::new(nodes))),
            live,
        }
    }

    /// Create a live NodeList that references a parent's child list
    pub fn new_live(child_nodes: Arc<Mutex<Vec<JsObject>>>) -> Self {
        Self {
            source: NodeListSource::LiveChildNodes(Arc::downgrade(&child_nodes)),
            live: true,
        }
    }

    /// Create an empty NodeList
    pub fn empty() -> Self {
        Self::new(Vec::new(), false)
    }

    /// Get the nodes from the source (handles both static and live)
    fn get_nodes(&self) -> Vec<JsObject> {
        match &self.source {
            NodeListSource::Static(arc) => arc.lock().unwrap().clone(),
            NodeListSource::LiveChildNodes(weak) => {
                // Upgrade weak reference to get current nodes
                weak.upgrade()
                    .map(|arc| arc.lock().unwrap().clone())
                    .unwrap_or_default()
            }
        }
    }

    /// Get the length of the NodeList
    pub fn length(&self) -> usize {
        match &self.source {
            NodeListSource::Static(arc) => arc.lock().unwrap().len(),
            NodeListSource::LiveChildNodes(weak) => weak
                .upgrade()
                .map(|arc| arc.lock().unwrap().len())
                .unwrap_or(0),
        }
    }

    /// Get the node at the specified index
    pub fn get_item(&self, index: usize) -> Option<JsObject> {
        match &self.source {
            NodeListSource::Static(arc) => arc.lock().unwrap().get(index).cloned(),
            NodeListSource::LiveChildNodes(weak) => weak
                .upgrade()
                .and_then(|arc| arc.lock().unwrap().get(index).cloned()),
        }
    }

    /// Get all nodes as a vector
    pub fn nodes(&self) -> Vec<JsObject> {
        self.get_nodes()
    }

    /// Add a node to the list (only for static NodeLists)
    /// Live NodeLists are modified through their parent
    pub fn add_node(&self, node: JsObject) {
        if let NodeListSource::Static(arc) = &self.source {
            arc.lock().unwrap().push(node);
        }
        // For live NodeLists, modifications go through the parent NodeData
    }

    /// Remove a node from the list (only for static NodeLists)
    /// Live NodeLists are modified through their parent
    pub fn remove_node(&self, node: &JsObject) {
        if let NodeListSource::Static(arc) = &self.source {
            arc.lock()
                .unwrap()
                .retain(|n| !std::ptr::eq(n.as_ref(), node.as_ref()));
        }
        // For live NodeLists, modifications go through the parent NodeData
    }

    /// Clear all nodes (only for static NodeLists)
    /// Live NodeLists are modified through their parent
    pub fn clear(&self) {
        if let NodeListSource::Static(arc) = &self.source {
            arc.lock().unwrap().clear();
        }
        // For live NodeLists, modifications go through the parent NodeData
    }

    /// Replace all nodes (for static NodeLists)
    pub fn replace_nodes(&self, nodes: Vec<JsObject>) {
        if let NodeListSource::Static(arc) = &self.source {
            *arc.lock().unwrap() = nodes;
        }
    }

    /// Check if this is a live NodeList
    pub fn is_live(&self) -> bool {
        self.live
    }

    /// `NodeList.prototype.length` getter
    fn get_length_accessor(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.length called on non-object")
        })?;

        let nodelist_data = this_obj.downcast_ref::<NodeListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.length called on non-NodeList object")
        })?;

        Ok(JsValue::new(nodelist_data.length() as i32))
    }

    /// `NodeList.prototype.item(index)`
    fn item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.item called on non-object")
        })?;

        let nodelist_data = this_obj.downcast_ref::<NodeListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.item called on non-NodeList object")
        })?;

        let index = args.get_or_undefined(0).to_length(context)? as usize;

        match nodelist_data.get_item(index) {
            Some(node) => Ok(node.into()),
            None => Ok(JsValue::null()),
        }
    }

    /// `NodeList.prototype.forEach(callback, thisArg)`
    fn for_each(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.forEach called on non-object")
        })?;

        let nodelist_data = this_obj.downcast_ref::<NodeListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.forEach called on non-NodeList object")
        })?;

        let callback = args.get_or_undefined(0);
        let this_arg = args.get_or_undefined(1);

        if !callback.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("NodeList.forEach callback is not callable")
                .into());
        }

        let nodes = nodelist_data.nodes();
        for (index, node) in nodes.iter().enumerate() {
            let args = [node.clone().into(), JsValue::new(index), this.clone()];
            // Use public API - get callable and call it
            if let Some(func) = callback.as_callable() {
                func.call(this_arg, &args, context)?;
            }
        }

        Ok(JsValue::undefined())
    }

    /// `NodeList.prototype.keys()`
    fn keys(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.keys called on non-object")
        })?;

        let nodelist_data = this_obj.downcast_ref::<NodeListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.keys called on non-NodeList object")
        })?;

        // Create an array iterator over the indices
        let length = nodelist_data.length();
        let indices: Vec<JsValue> = (0..length).map(JsValue::new).collect();
        let array = boa_engine::builtins::array::Array::array_create(length as u64, None, context)?;

        for (i, index) in indices.iter().enumerate() {
            array.create_data_property_or_throw(i, index.clone(), context)?;
        }

        // Return array iterator (simplified implementation)
        // In a full implementation, this would return a proper Iterator
        Ok(array.into())
    }

    /// `NodeList.prototype.values()`
    fn values(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.values called on non-object")
        })?;

        let nodelist_data = this_obj.downcast_ref::<NodeListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.values called on non-NodeList object")
        })?;

        // Create an array with the node values
        let nodes = nodelist_data.nodes();
        let array =
            boa_engine::builtins::array::Array::array_create(nodes.len() as u64, None, context)?;

        for (i, node) in nodes.iter().enumerate() {
            array.create_data_property_or_throw(i, node.clone(), context)?;
        }

        // Return array (simplified implementation)
        // In a full implementation, this would return a proper Iterator
        Ok(array.into())
    }

    /// `NodeList.prototype.entries()`
    fn entries(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.entries called on non-object")
        })?;

        let nodelist_data = this_obj.downcast_ref::<NodeListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.entries called on non-NodeList object")
        })?;

        // Create an array of [index, node] pairs
        let nodes = nodelist_data.nodes();
        let array =
            boa_engine::builtins::array::Array::array_create(nodes.len() as u64, None, context)?;

        for (i, node) in nodes.iter().enumerate() {
            let entry = boa_engine::builtins::array::Array::array_create(2, None, context)?;
            entry.create_data_property_or_throw(0, JsValue::new(i), context)?;
            entry.create_data_property_or_throw(1, node.clone(), context)?;
            array.create_data_property_or_throw(i, entry, context)?;
        }

        // Return array (simplified implementation)
        // In a full implementation, this would return a proper Iterator
        Ok(array.into())
    }
}

/// The `NodeList` object
#[derive(Debug, Trace, Finalize)]
pub struct NodeList;

impl NodeList {
    /// Create a new static NodeList from a vector of nodes
    pub fn create_from_nodes(
        nodes: Vec<JsObject>,
        live: bool,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        let nodelist_data = NodeListData::new(nodes, live);

        let nodelist_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().nodelist().prototype(),
            nodelist_data,
        );

        Ok(nodelist_obj.upcast())
    }

    /// Create a live NodeList that references a parent's child_nodes Arc.
    /// This is the proper implementation for Node.childNodes per DOM spec.
    /// The returned NodeList will always reflect the current state of the parent's children.
    pub fn create_live_child_nodes(
        child_nodes_arc: Arc<Mutex<Vec<JsObject>>>,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        let nodelist_data = NodeListData::new_live(child_nodes_arc);

        let nodelist_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().nodelist().prototype(),
            nodelist_data,
        );

        Ok(nodelist_obj.upcast())
    }

    /// Static method implementations for BuiltInBuilder
    fn get_length_accessor(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        NodeListData::get_length_accessor(this, args, context)
    }

    fn item(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.item called on non-object")
        })?;

        let nodelist_data = this_obj.downcast_ref::<NodeListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.item called on non-NodeList object")
        })?;

        let index = args.get_or_undefined(0).to_length(context)? as usize;
        match nodelist_data.get_item(index) {
            Some(node) => Ok(node.into()),
            None => Ok(JsValue::null()),
        }
    }

    fn for_each(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.forEach called on non-object")
        })?;

        let nodelist_data = this_obj.downcast_ref::<NodeListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.forEach called on non-NodeList object")
        })?;

        let callback = args.get_or_undefined(0);
        let this_arg = args.get_or_undefined(1);

        if !callback.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("NodeList.forEach callback is not callable")
                .into());
        }

        let nodes = nodelist_data.nodes();
        for (index, node) in nodes.iter().enumerate() {
            let args = [node.clone().into(), JsValue::new(index), this.clone()];
            // Use public API - get callable and call it
            if let Some(func) = callback.as_callable() {
                func.call(this_arg, &args, context)?;
            }
        }

        Ok(JsValue::undefined())
    }

    fn keys(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.keys called on non-object")
        })?;

        let nodelist_data = this_obj.downcast_ref::<NodeListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.keys called on non-NodeList object")
        })?;

        let length = nodelist_data.length();
        let indices: Vec<JsValue> = (0..length).map(JsValue::new).collect();
        let array = boa_engine::builtins::array::Array::array_create(length as u64, None, context)?;

        for (i, index) in indices.iter().enumerate() {
            array.create_data_property_or_throw(i, index.clone(), context)?;
        }

        Ok(array.into())
    }

    fn values(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.values called on non-object")
        })?;

        let nodelist_data = this_obj.downcast_ref::<NodeListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.values called on non-NodeList object")
        })?;

        let nodes = nodelist_data.nodes();
        let array =
            boa_engine::builtins::array::Array::array_create(nodes.len() as u64, None, context)?;

        for (i, node) in nodes.iter().enumerate() {
            array.create_data_property_or_throw(i, node.clone(), context)?;
        }

        Ok(array.into())
    }

    fn entries(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.entries called on non-object")
        })?;

        let nodelist_data = this_obj.downcast_ref::<NodeListData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeList.entries called on non-NodeList object")
        })?;

        let nodes = nodelist_data.nodes();
        let array =
            boa_engine::builtins::array::Array::array_create(nodes.len() as u64, None, context)?;

        for (i, node) in nodes.iter().enumerate() {
            let entry = boa_engine::builtins::array::Array::array_create(2, None, context)?;
            entry.create_data_property_or_throw(0, JsValue::new(i), context)?;
            entry.create_data_property_or_throw(1, node.clone(), context)?;
            array.create_data_property_or_throw(i, entry, context)?;
        }

        Ok(array.into())
    }
}

impl IntrinsicObject for NodeList {
    fn init(realm: &Realm) {
        let length_get_func = BuiltInBuilder::callable(realm, Self::get_length_accessor)
            .name(js_string!("get length"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Methods
            .method(Self::item, js_string!("item"), 1)
            .method(Self::for_each, js_string!("forEach"), 1)
            .method(Self::keys, js_string!("keys"), 0)
            .method(Self::values, js_string!("values"), 0)
            .method(Self::entries, js_string!("entries"), 0)
            // Properties
            .accessor(
                js_string!("length"),
                Some(length_get_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for NodeList {
    const NAME: JsString = StaticJsStrings::NODELIST;
}

impl BuiltInConstructor for NodeList {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::nodelist;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // NodeList constructor should be called with 'new'
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Constructor NodeList requires 'new'")
                .into());
        }

        // Create a new empty NodeList object
        let nodelist_data = NodeListData::empty();

        let nodelist_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().nodelist().prototype(),
            nodelist_data,
        );

        Ok(nodelist_obj.upcast().into())
    }
}

#[cfg(test)]
mod tests;
