//! NodeIterator interface implementation for DOM Level 4
//!
//! The NodeIterator interface represents an iterator over the members of a list of the nodes in a subtree of the DOM.
//! https://dom.spec.whatwg.org/#interface-nodeiterator

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    string::JsString,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace, GcRefCell};

// Re-use node_filter constants from TreeWalker
use super::treewalker::node_filter;

/// The NodeIterator data implementation
#[derive(Debug, Trace, Finalize, JsData)]
pub struct NodeIteratorData {
    /// The root node of the NodeIterator
    root: GcRefCell<Option<JsObject>>,
    /// The reference node (position marker)
    reference_node: GcRefCell<Option<JsObject>>,
    /// Whether the pointer is before the reference node
    #[unsafe_ignore_trace]
    pointer_before_reference: std::sync::atomic::AtomicBool,
    /// What types of nodes to show
    #[unsafe_ignore_trace]
    what_to_show: u32,
    /// The optional filter function
    filter: GcRefCell<Option<JsObject>>,
}

impl NodeIteratorData {
    /// Create a new NodeIterator
    pub fn new(root: JsObject, what_to_show: u32, filter: Option<JsObject>) -> Self {
        Self {
            root: GcRefCell::new(Some(root.clone())),
            reference_node: GcRefCell::new(Some(root)),
            pointer_before_reference: std::sync::atomic::AtomicBool::new(true),
            what_to_show,
            filter: GcRefCell::new(filter),
        }
    }

    /// Get the root node
    pub fn root(&self) -> Option<JsObject> {
        self.root.borrow().clone()
    }

    /// Get what to show filter
    pub fn what_to_show(&self) -> u32 {
        self.what_to_show
    }

    /// Get the filter function
    pub fn filter(&self) -> Option<JsObject> {
        self.filter.borrow().clone()
    }

    /// Get the reference node
    pub fn reference_node(&self) -> Option<JsObject> {
        self.reference_node.borrow().clone()
    }

    /// Set the reference node
    pub fn set_reference_node(&self, node: JsObject) {
        *self.reference_node.borrow_mut() = Some(node);
    }

    /// Get whether pointer is before reference
    pub fn pointer_before_reference(&self) -> bool {
        self.pointer_before_reference.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Set whether pointer is before reference
    pub fn set_pointer_before_reference(&self, before: bool) {
        self.pointer_before_reference.store(before, std::sync::atomic::Ordering::SeqCst);
    }

    /// Check if a node should be accepted based on whatToShow
    fn node_matches_what_to_show(&self, node: &JsObject, context: &mut Context) -> bool {
        // Get the nodeType
        let node_type = match node.upcast().get(js_string!("nodeType"), context) {
            Ok(val) => match val.to_u32(context) {
                Ok(n) => n,
                Err(_) => return false,
            },
            Err(_) => return false,
        };

        // Map nodeType to whatToShow bit
        let bit = match node_type {
            1 => node_filter::SHOW_ELEMENT,
            2 => node_filter::SHOW_ATTRIBUTE,
            3 => node_filter::SHOW_TEXT,
            4 => node_filter::SHOW_CDATA_SECTION,
            7 => node_filter::SHOW_PROCESSING_INSTRUCTION,
            8 => node_filter::SHOW_COMMENT,
            9 => node_filter::SHOW_DOCUMENT,
            10 => node_filter::SHOW_DOCUMENT_TYPE,
            11 => node_filter::SHOW_DOCUMENT_FRAGMENT,
            _ => return false,
        };

        (self.what_to_show & bit) != 0
    }

    /// Filter a node using the NodeFilter callback if present
    fn filter_node(&self, node: &JsObject, context: &mut Context) -> JsResult<u16> {
        // First check whatToShow
        if !self.node_matches_what_to_show(node, context) {
            return Ok(node_filter::FILTER_SKIP);
        }

        // Then check the filter function if present
        if let Some(filter) = self.filter() {
            let result = if filter.is_callable() {
                filter.call(&JsValue::undefined(), &[node.clone().into()], context)?
            } else {
                let accept_node = filter.get(js_string!("acceptNode"), context)?;
                if let Some(func) = accept_node.as_callable() {
                    func.call(&filter.clone().into(), &[node.clone().into()], context)?
                } else {
                    return Ok(node_filter::FILTER_ACCEPT);
                }
            };

            Ok(result.to_u32(context)? as u16)
        } else {
            Ok(node_filter::FILTER_ACCEPT)
        }
    }

    /// Get the next node in document order from a given node
    fn next_in_document_order(&self, node: &JsObject, context: &mut Context) -> JsResult<Option<JsObject>> {
        let root = match self.root() {
            Some(r) => r,
            None => return Ok(None),
        };

        // Try first child
        let first_child = node.upcast().get(js_string!("firstChild"), context)?;
        if !first_child.is_null() && !first_child.is_undefined() {
            if let Some(child) = first_child.as_object() {
                return Ok(Some(child.clone()));
            }
        }

        // Try next sibling
        let mut current = node.clone();
        loop {
            if std::ptr::eq(current.as_ref(), root.as_ref()) {
                return Ok(None);
            }

            let sibling = current.upcast().get(js_string!("nextSibling"), context)?;
            if !sibling.is_null() && !sibling.is_undefined() {
                if let Some(sib) = sibling.as_object() {
                    return Ok(Some(sib.clone()));
                }
            }

            // Go to parent
            let parent = current.upcast().get(js_string!("parentNode"), context)?;
            if parent.is_null() || parent.is_undefined() {
                return Ok(None);
            }

            current = match parent.as_object() {
                Some(p) => p.clone(),
                None => return Ok(None),
            };
        }
    }

    /// Get the previous node in document order from a given node
    fn previous_in_document_order(&self, node: &JsObject, context: &mut Context) -> JsResult<Option<JsObject>> {
        let root = match self.root() {
            Some(r) => r,
            None => return Ok(None),
        };

        // Try previous sibling's last descendant
        let sibling = node.upcast().get(js_string!("previousSibling"), context)?;
        if !sibling.is_null() && !sibling.is_undefined() {
            if let Some(mut sib) = sibling.as_object().map(|o| o.clone()) {
                // Go to last descendant
                loop {
                    let last_child = sib.upcast().get(js_string!("lastChild"), context)?;
                    if last_child.is_null() || last_child.is_undefined() {
                        return Ok(Some(sib));
                    }
                    sib = match last_child.as_object() {
                        Some(c) => c.clone(),
                        None => return Ok(Some(sib)),
                    };
                }
            }
        }

        // Go to parent
        if std::ptr::eq(node.as_ref(), root.as_ref()) {
            return Ok(None);
        }

        let parent = node.upcast().get(js_string!("parentNode"), context)?;
        if parent.is_null() || parent.is_undefined() {
            return Ok(None);
        }

        let parent_obj = match parent.as_object() {
            Some(p) => p.clone(),
            None => return Ok(None),
        };

        if std::ptr::eq(parent_obj.as_ref(), root.as_ref()) {
            return Ok(None);
        }

        Ok(Some(parent_obj))
    }
}

/// The `NodeIterator` object
#[derive(Debug, Trace, Finalize)]
pub struct NodeIterator;

impl NodeIterator {
    /// `NodeIterator.prototype.root` getter
    fn get_root(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeIterator.root called on non-object")
        })?;

        let data = this_obj.downcast_ref::<NodeIteratorData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeIterator.root called on non-NodeIterator object")
        })?;

        match data.root() {
            Some(root) => Ok(root.into()),
            None => Ok(JsValue::null()),
        }
    }

    /// `NodeIterator.prototype.whatToShow` getter
    fn get_what_to_show(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeIterator.whatToShow called on non-object")
        })?;

        let data = this_obj.downcast_ref::<NodeIteratorData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeIterator.whatToShow called on non-NodeIterator object")
        })?;

        Ok(JsValue::new(data.what_to_show()))
    }

    /// `NodeIterator.prototype.filter` getter
    fn get_filter(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeIterator.filter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<NodeIteratorData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeIterator.filter called on non-NodeIterator object")
        })?;

        match data.filter() {
            Some(filter) => Ok(filter.into()),
            None => Ok(JsValue::null()),
        }
    }

    /// `NodeIterator.prototype.referenceNode` getter
    fn get_reference_node(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeIterator.referenceNode called on non-object")
        })?;

        let data = this_obj.downcast_ref::<NodeIteratorData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeIterator.referenceNode called on non-NodeIterator object")
        })?;

        match data.reference_node() {
            Some(node) => Ok(node.into()),
            None => Ok(JsValue::null()),
        }
    }

    /// `NodeIterator.prototype.pointerBeforeReferenceNode` getter
    fn get_pointer_before_reference_node(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeIterator.pointerBeforeReferenceNode called on non-object")
        })?;

        let data = this_obj.downcast_ref::<NodeIteratorData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeIterator.pointerBeforeReferenceNode called on non-NodeIterator object")
        })?;

        Ok(JsValue::new(data.pointer_before_reference()))
    }

    /// `NodeIterator.prototype.nextNode()`
    fn next_node(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeIterator.nextNode called on non-object")
        })?;

        let data = this_obj.downcast_ref::<NodeIteratorData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeIterator.nextNode called on non-NodeIterator object")
        })?;

        let mut node = match data.reference_node() {
            Some(n) => n,
            None => return Ok(JsValue::null()),
        };

        let mut before_node = data.pointer_before_reference();

        loop {
            if !before_node {
                // Move to next node in document order
                node = match data.next_in_document_order(&node, context)? {
                    Some(n) => n,
                    None => return Ok(JsValue::null()),
                };
            }
            before_node = false;

            // Filter the node
            let filter_result = data.filter_node(&node, context)?;
            if filter_result == node_filter::FILTER_ACCEPT {
                data.set_reference_node(node.clone());
                data.set_pointer_before_reference(false);
                return Ok(node.into());
            }
        }
    }

    /// `NodeIterator.prototype.previousNode()`
    fn previous_node(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeIterator.previousNode called on non-object")
        })?;

        let data = this_obj.downcast_ref::<NodeIteratorData>().ok_or_else(|| {
            JsNativeError::typ().with_message("NodeIterator.previousNode called on non-NodeIterator object")
        })?;

        let mut node = match data.reference_node() {
            Some(n) => n,
            None => return Ok(JsValue::null()),
        };

        let mut before_node = data.pointer_before_reference();

        loop {
            if before_node {
                // Move to previous node in document order
                node = match data.previous_in_document_order(&node, context)? {
                    Some(n) => n,
                    None => return Ok(JsValue::null()),
                };
            }
            before_node = true;

            // Filter the node
            let filter_result = data.filter_node(&node, context)?;
            if filter_result == node_filter::FILTER_ACCEPT {
                data.set_reference_node(node.clone());
                data.set_pointer_before_reference(true);
                return Ok(node.into());
            }
        }
    }

    /// `NodeIterator.prototype.detach()`
    /// This method is now a no-op per the DOM spec
    fn detach(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // detach() is now a no-op per the DOM spec
        Ok(JsValue::undefined())
    }

    /// Create a NodeIterator for document.createNodeIterator
    pub fn create(
        root: JsObject,
        what_to_show: u32,
        filter: Option<JsObject>,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        let data = NodeIteratorData::new(root, what_to_show, filter);
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().nodeiterator().prototype(),
            data,
        );
        Ok(obj.upcast())
    }
}

impl IntrinsicObject for NodeIterator {
    fn init(realm: &Realm) {
        let root_getter = BuiltInBuilder::callable(realm, Self::get_root)
            .name(js_string!("get root"))
            .build();

        let what_to_show_getter = BuiltInBuilder::callable(realm, Self::get_what_to_show)
            .name(js_string!("get whatToShow"))
            .build();

        let filter_getter = BuiltInBuilder::callable(realm, Self::get_filter)
            .name(js_string!("get filter"))
            .build();

        let reference_node_getter = BuiltInBuilder::callable(realm, Self::get_reference_node)
            .name(js_string!("get referenceNode"))
            .build();

        let pointer_before_getter = BuiltInBuilder::callable(realm, Self::get_pointer_before_reference_node)
            .name(js_string!("get pointerBeforeReferenceNode"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::next_node, js_string!("nextNode"), 0)
            .method(Self::previous_node, js_string!("previousNode"), 0)
            .method(Self::detach, js_string!("detach"), 0)
            .accessor(
                js_string!("root"),
                Some(root_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("whatToShow"),
                Some(what_to_show_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("filter"),
                Some(filter_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("referenceNode"),
                Some(reference_node_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("pointerBeforeReferenceNode"),
                Some(pointer_before_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for NodeIterator {
    const NAME: JsString = js_string!("NodeIterator");
}

impl BuiltInConstructor for NodeIterator {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::nodeiterator;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // NodeIterator should not be constructed directly, use document.createNodeIterator
        Err(JsNativeError::typ()
            .with_message("NodeIterator cannot be constructed directly")
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_filter_reexport() {
        // Verify we can access node_filter constants through the re-export
        assert_eq!(node_filter::SHOW_ALL, 0xFFFFFFFF);
        assert_eq!(node_filter::FILTER_ACCEPT, 1);
    }
}
