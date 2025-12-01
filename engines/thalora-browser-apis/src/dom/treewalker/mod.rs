//! TreeWalker interface implementation for DOM Level 4
//!
//! The TreeWalker interface represents the nodes of a document subtree and a position within them.
//! https://dom.spec.whatwg.org/#interface-treewalker

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    string::JsString,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsValue,
    property::PropertyDescriptorBuilder,
};
use boa_gc::{Finalize, Trace, GcRefCell};

/// NodeFilter constants - what types of nodes to show
pub mod node_filter {
    /// Show all nodes
    pub const SHOW_ALL: u32 = 0xFFFFFFFF;
    /// Show Element nodes
    pub const SHOW_ELEMENT: u32 = 0x1;
    /// Show Attribute nodes (deprecated)
    pub const SHOW_ATTRIBUTE: u32 = 0x2;
    /// Show Text nodes
    pub const SHOW_TEXT: u32 = 0x4;
    /// Show CDATASection nodes (deprecated)
    pub const SHOW_CDATA_SECTION: u32 = 0x8;
    /// Show EntityReference nodes (deprecated)
    pub const SHOW_ENTITY_REFERENCE: u32 = 0x10;
    /// Show Entity nodes (deprecated)
    pub const SHOW_ENTITY: u32 = 0x20;
    /// Show ProcessingInstruction nodes
    pub const SHOW_PROCESSING_INSTRUCTION: u32 = 0x40;
    /// Show Comment nodes
    pub const SHOW_COMMENT: u32 = 0x80;
    /// Show Document nodes
    pub const SHOW_DOCUMENT: u32 = 0x100;
    /// Show DocumentType nodes
    pub const SHOW_DOCUMENT_TYPE: u32 = 0x200;
    /// Show DocumentFragment nodes
    pub const SHOW_DOCUMENT_FRAGMENT: u32 = 0x400;
    /// Show Notation nodes (deprecated)
    pub const SHOW_NOTATION: u32 = 0x800;

    /// NodeFilter result: accept the node
    pub const FILTER_ACCEPT: u16 = 1;
    /// NodeFilter result: reject the node (skip this node and its descendants)
    pub const FILTER_REJECT: u16 = 2;
    /// NodeFilter result: skip this node but not its descendants
    pub const FILTER_SKIP: u16 = 3;
}

/// The TreeWalker data implementation
#[derive(Debug, Trace, Finalize, JsData)]
pub struct TreeWalkerData {
    /// The root node of the TreeWalker
    root: GcRefCell<Option<JsObject>>,
    /// The current node
    current_node: GcRefCell<Option<JsObject>>,
    /// What types of nodes to show
    #[unsafe_ignore_trace]
    what_to_show: u32,
    /// The optional filter function
    filter: GcRefCell<Option<JsObject>>,
}

impl TreeWalkerData {
    /// Create a new TreeWalker
    pub fn new(root: JsObject, what_to_show: u32, filter: Option<JsObject>) -> Self {
        Self {
            root: GcRefCell::new(Some(root.clone())),
            current_node: GcRefCell::new(Some(root)),
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

    /// Get the current node
    pub fn current_node(&self) -> Option<JsObject> {
        self.current_node.borrow().clone()
    }

    /// Set the current node
    pub fn set_current_node(&self, node: JsObject) {
        *self.current_node.borrow_mut() = Some(node);
    }

    /// Check if a node should be accepted based on whatToShow
    fn node_matches_what_to_show(&self, node: &JsObject, context: &mut Context) -> bool {
        // Get the nodeType
        let node_type = match node.get(js_string!("nodeType"), context) {
            Ok(val) => match val.to_u32(context) {
                Ok(n) => n,
                Err(_) => return false,
            },
            Err(_) => return false,
        };

        // Map nodeType to whatToShow bit
        let bit = match node_type {
            1 => node_filter::SHOW_ELEMENT,        // Element
            2 => node_filter::SHOW_ATTRIBUTE,      // Attr
            3 => node_filter::SHOW_TEXT,           // Text
            4 => node_filter::SHOW_CDATA_SECTION,  // CDATASection
            7 => node_filter::SHOW_PROCESSING_INSTRUCTION, // ProcessingInstruction
            8 => node_filter::SHOW_COMMENT,        // Comment
            9 => node_filter::SHOW_DOCUMENT,       // Document
            10 => node_filter::SHOW_DOCUMENT_TYPE, // DocumentType
            11 => node_filter::SHOW_DOCUMENT_FRAGMENT, // DocumentFragment
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
            // The filter can be a function or an object with acceptNode method
            let result = if filter.is_callable() {
                filter.call(&JsValue::undefined(), &[node.clone().into()], context)?
            } else {
                // Try to get acceptNode method
                let accept_node = filter.get(js_string!("acceptNode"), context)?;
                if let Some(func) = accept_node.as_callable() {
                    func.call(&filter.clone().into(), &[node.clone().into()], context)?
                } else {
                    return Ok(node_filter::FILTER_ACCEPT);
                }
            };

            // Convert result to u16
            Ok(result.to_u32(context)? as u16)
        } else {
            Ok(node_filter::FILTER_ACCEPT)
        }
    }
}

/// The `TreeWalker` object
#[derive(Debug, Trace, Finalize)]
pub struct TreeWalker;

impl TreeWalker {
    /// `TreeWalker.prototype.root` getter
    fn get_root(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker.root called on non-object")
        })?;

        let data = this_obj.downcast_ref::<TreeWalkerData>().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker.root called on non-TreeWalker object")
        })?;

        match data.root() {
            Some(root) => Ok(root.into()),
            None => Ok(JsValue::null()),
        }
    }

    /// `TreeWalker.prototype.whatToShow` getter
    fn get_what_to_show(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker.whatToShow called on non-object")
        })?;

        let data = this_obj.downcast_ref::<TreeWalkerData>().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker.whatToShow called on non-TreeWalker object")
        })?;

        Ok(JsValue::new(data.what_to_show()))
    }

    /// `TreeWalker.prototype.filter` getter
    fn get_filter(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker.filter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<TreeWalkerData>().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker.filter called on non-TreeWalker object")
        })?;

        match data.filter() {
            Some(filter) => Ok(filter.into()),
            None => Ok(JsValue::null()),
        }
    }

    /// `TreeWalker.prototype.currentNode` getter
    fn get_current_node(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker.currentNode called on non-object")
        })?;

        let data = this_obj.downcast_ref::<TreeWalkerData>().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker.currentNode called on non-TreeWalker object")
        })?;

        match data.current_node() {
            Some(node) => Ok(node.into()),
            None => Ok(JsValue::null()),
        }
    }

    /// `TreeWalker.prototype.currentNode` setter
    fn set_current_node(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker.currentNode called on non-object")
        })?;

        let data = this_obj.downcast_ref::<TreeWalkerData>().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker.currentNode called on non-TreeWalker object")
        })?;

        let node = args.get_or_undefined(0);
        let node_obj = node.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("currentNode must be a Node")
        })?;

        data.set_current_node(node_obj.clone());
        Ok(JsValue::undefined())
    }

    /// `TreeWalker.prototype.parentNode()`
    fn parent_node(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker.parentNode called on non-object")
        })?;

        let data = this_obj.downcast_ref::<TreeWalkerData>().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker.parentNode called on non-TreeWalker object")
        })?;

        let root = match data.root() {
            Some(r) => r,
            None => return Ok(JsValue::null()),
        };

        let mut node = match data.current_node() {
            Some(n) => n,
            None => return Ok(JsValue::null()),
        };

        // Walk up to find an accepted parent
        while !std::ptr::eq(node.as_ref(), root.as_ref()) {
            let parent = node.get(js_string!("parentNode"), context)?;
            if parent.is_null() || parent.is_undefined() {
                return Ok(JsValue::null());
            }

            let parent_obj = parent.as_object().ok_or_else(|| {
                JsNativeError::typ().with_message("parentNode is not an object")
            })?;

            node = parent_obj.clone();

            // Filter the node
            let filter_result = data.filter_node(&node, context)?;
            if filter_result == node_filter::FILTER_ACCEPT {
                data.set_current_node(node.clone());
                return Ok(node.into());
            }
        }

        Ok(JsValue::null())
    }

    /// `TreeWalker.prototype.firstChild()`
    fn first_child(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::traverse_children(this, context, true)
    }

    /// `TreeWalker.prototype.lastChild()`
    fn last_child(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::traverse_children(this, context, false)
    }

    /// Helper to traverse to first or last child
    fn traverse_children(this: &JsValue, context: &mut Context, first: bool) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker method called on non-object")
        })?;

        let data = this_obj.downcast_ref::<TreeWalkerData>().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker method called on non-TreeWalker object")
        })?;

        let node = match data.current_node() {
            Some(n) => n,
            None => return Ok(JsValue::null()),
        };

        // Get the appropriate child
        let child_prop = if first { "firstChild" } else { "lastChild" };
        let sibling_prop = if first { "nextSibling" } else { "previousSibling" };

        let mut child = node.get(js_string!(child_prop), context)?;

        while !child.is_null() && !child.is_undefined() {
            let child_obj = match child.as_object() {
                Some(obj) => obj.clone(),
                None => return Ok(JsValue::null()),
            };

            let filter_result = data.filter_node(&child_obj, context)?;

            if filter_result == node_filter::FILTER_ACCEPT {
                data.set_current_node(child_obj.clone());
                return Ok(child_obj.into());
            }

            if filter_result == node_filter::FILTER_SKIP {
                // Try to descend into this node's children
                let grandchild = child_obj.get(js_string!(child_prop), context)?;
                if !grandchild.is_null() && !grandchild.is_undefined() {
                    child = grandchild;
                    continue;
                }
            }

            // Try sibling
            loop {
                let sibling = child_obj.get(js_string!(sibling_prop), context)?;
                if !sibling.is_null() && !sibling.is_undefined() {
                    child = sibling;
                    break;
                }

                // Go up to parent
                let parent = child_obj.get(js_string!("parentNode"), context)?;
                if parent.is_null() || parent.is_undefined() {
                    return Ok(JsValue::null());
                }

                let parent_obj = parent.as_object().ok_or_else(|| {
                    JsNativeError::typ().with_message("parentNode is not an object")
                })?;

                // Check if we've reached the current node or root
                if std::ptr::eq(parent_obj.as_ref(), node.as_ref()) {
                    return Ok(JsValue::null());
                }

                child = parent;
                break;
            }
        }

        Ok(JsValue::null())
    }

    /// `TreeWalker.prototype.previousSibling()`
    fn previous_sibling(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::traverse_siblings(this, context, false)
    }

    /// `TreeWalker.prototype.nextSibling()`
    fn next_sibling(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::traverse_siblings(this, context, true)
    }

    /// Helper to traverse to previous or next sibling
    fn traverse_siblings(this: &JsValue, context: &mut Context, next: bool) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker method called on non-object")
        })?;

        let data = this_obj.downcast_ref::<TreeWalkerData>().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker method called on non-TreeWalker object")
        })?;

        let root = match data.root() {
            Some(r) => r,
            None => return Ok(JsValue::null()),
        };

        let mut node = match data.current_node() {
            Some(n) => n,
            None => return Ok(JsValue::null()),
        };

        if std::ptr::eq(node.as_ref(), root.as_ref()) {
            return Ok(JsValue::null());
        }

        let sibling_prop = if next { "nextSibling" } else { "previousSibling" };
        let child_prop = if next { "firstChild" } else { "lastChild" };

        loop {
            let sibling = node.get(js_string!(sibling_prop), context)?;

            while !sibling.is_null() && !sibling.is_undefined() {
                let sibling_obj = match sibling.as_object() {
                    Some(obj) => obj.clone(),
                    None => break,
                };

                let filter_result = data.filter_node(&sibling_obj, context)?;

                if filter_result == node_filter::FILTER_ACCEPT {
                    data.set_current_node(sibling_obj.clone());
                    return Ok(sibling_obj.into());
                }

                // If SKIP, try children
                if filter_result == node_filter::FILTER_SKIP {
                    let child = sibling_obj.get(js_string!(child_prop), context)?;
                    if !child.is_null() && !child.is_undefined() {
                        node = sibling_obj;
                        continue;
                    }
                }

                break;
            }

            // Go up to parent
            let parent = node.get(js_string!("parentNode"), context)?;
            if parent.is_null() || parent.is_undefined() {
                return Ok(JsValue::null());
            }

            let parent_obj = parent.as_object().ok_or_else(|| {
                JsNativeError::typ().with_message("parentNode is not an object")
            })?;

            if std::ptr::eq(parent_obj.as_ref(), root.as_ref()) {
                return Ok(JsValue::null());
            }

            node = parent_obj.clone();
        }
    }

    /// `TreeWalker.prototype.previousNode()`
    fn previous_node(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker.previousNode called on non-object")
        })?;

        let data = this_obj.downcast_ref::<TreeWalkerData>().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker.previousNode called on non-TreeWalker object")
        })?;

        let root = match data.root() {
            Some(r) => r,
            None => return Ok(JsValue::null()),
        };

        let mut node = match data.current_node() {
            Some(n) => n,
            None => return Ok(JsValue::null()),
        };

        while !std::ptr::eq(node.as_ref(), root.as_ref()) {
            // Try previous sibling and its last descendants
            let sibling = node.get(js_string!("previousSibling"), context)?;

            if !sibling.is_null() && !sibling.is_undefined() {
                let mut sibling_obj = sibling.as_object().ok_or_else(|| {
                    JsNativeError::typ().with_message("sibling is not an object")
                })?.clone();

                // Descend to last child
                loop {
                    let filter_result = data.filter_node(&sibling_obj, context)?;

                    if filter_result == node_filter::FILTER_REJECT {
                        // Skip this subtree
                        break;
                    }

                    let last_child = sibling_obj.get(js_string!("lastChild"), context)?;
                    if last_child.is_null() || last_child.is_undefined() {
                        if filter_result == node_filter::FILTER_ACCEPT {
                            data.set_current_node(sibling_obj.clone());
                            return Ok(sibling_obj.into());
                        }
                        break;
                    }

                    sibling_obj = last_child.as_object().ok_or_else(|| {
                        JsNativeError::typ().with_message("lastChild is not an object")
                    })?.clone();
                }

                node = sibling_obj;
                continue;
            }

            // Go to parent
            let parent = node.get(js_string!("parentNode"), context)?;
            if parent.is_null() || parent.is_undefined() {
                return Ok(JsValue::null());
            }

            let parent_obj = parent.as_object().ok_or_else(|| {
                JsNativeError::typ().with_message("parentNode is not an object")
            })?;

            if std::ptr::eq(parent_obj.as_ref(), root.as_ref()) {
                return Ok(JsValue::null());
            }

            let filter_result = data.filter_node(&parent_obj, context)?;
            if filter_result == node_filter::FILTER_ACCEPT {
                data.set_current_node(parent_obj.clone());
                return Ok(parent_obj.into());
            }

            node = parent_obj.clone();
        }

        Ok(JsValue::null())
    }

    /// `TreeWalker.prototype.nextNode()`
    fn next_node(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker.nextNode called on non-object")
        })?;

        let data = this_obj.downcast_ref::<TreeWalkerData>().ok_or_else(|| {
            JsNativeError::typ().with_message("TreeWalker.nextNode called on non-TreeWalker object")
        })?;

        let root = match data.root() {
            Some(r) => r,
            None => return Ok(JsValue::null()),
        };

        let mut node = match data.current_node() {
            Some(n) => n,
            None => return Ok(JsValue::null()),
        };

        loop {
            // Try first child
            let filter_result = data.filter_node(&node, context)?;

            if filter_result != node_filter::FILTER_REJECT {
                let first_child = node.get(js_string!("firstChild"), context)?;
                if !first_child.is_null() && !first_child.is_undefined() {
                    let child_obj = first_child.as_object().ok_or_else(|| {
                        JsNativeError::typ().with_message("firstChild is not an object")
                    })?;

                    let child_filter = data.filter_node(&child_obj, context)?;
                    if child_filter == node_filter::FILTER_ACCEPT {
                        data.set_current_node(child_obj.clone());
                        return Ok(child_obj.into());
                    }

                    node = child_obj.clone();
                    continue;
                }
            }

            // Try siblings and parents' siblings
            loop {
                if std::ptr::eq(node.as_ref(), root.as_ref()) {
                    return Ok(JsValue::null());
                }

                let sibling = node.get(js_string!("nextSibling"), context)?;
                if !sibling.is_null() && !sibling.is_undefined() {
                    let sibling_obj = sibling.as_object().ok_or_else(|| {
                        JsNativeError::typ().with_message("sibling is not an object")
                    })?;

                    let sibling_filter = data.filter_node(&sibling_obj, context)?;
                    if sibling_filter == node_filter::FILTER_ACCEPT {
                        data.set_current_node(sibling_obj.clone());
                        return Ok(sibling_obj.into());
                    }

                    node = sibling_obj.clone();
                    break;
                }

                // Go to parent
                let parent = node.get(js_string!("parentNode"), context)?;
                if parent.is_null() || parent.is_undefined() {
                    return Ok(JsValue::null());
                }

                node = parent.as_object().ok_or_else(|| {
                    JsNativeError::typ().with_message("parentNode is not an object")
                })?.clone();
            }
        }
    }

    /// Create a TreeWalker for document.createTreeWalker
    pub fn create(
        root: JsObject,
        what_to_show: u32,
        filter: Option<JsObject>,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        let data = TreeWalkerData::new(root, what_to_show, filter);
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().treewalker().prototype(),
            data,
        );
        Ok(obj)
    }
}

impl IntrinsicObject for TreeWalker {
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

        let current_node_getter = BuiltInBuilder::callable(realm, Self::get_current_node)
            .name(js_string!("get currentNode"))
            .build();

        let current_node_setter = BuiltInBuilder::callable(realm, Self::set_current_node)
            .name(js_string!("set currentNode"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::parent_node, js_string!("parentNode"), 0)
            .method(Self::first_child, js_string!("firstChild"), 0)
            .method(Self::last_child, js_string!("lastChild"), 0)
            .method(Self::previous_sibling, js_string!("previousSibling"), 0)
            .method(Self::next_sibling, js_string!("nextSibling"), 0)
            .method(Self::previous_node, js_string!("previousNode"), 0)
            .method(Self::next_node, js_string!("nextNode"), 0)
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
                js_string!("currentNode"),
                Some(current_node_getter),
                Some(current_node_setter),
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for TreeWalker {
    const NAME: JsString = js_string!("TreeWalker");
}

impl BuiltInConstructor for TreeWalker {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::treewalker;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // TreeWalker should not be constructed directly, use document.createTreeWalker
        Err(JsNativeError::typ()
            .with_message("TreeWalker cannot be constructed directly")
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_filter_constants() {
        assert_eq!(node_filter::SHOW_ALL, 0xFFFFFFFF);
        assert_eq!(node_filter::SHOW_ELEMENT, 0x1);
        assert_eq!(node_filter::FILTER_ACCEPT, 1);
        assert_eq!(node_filter::FILTER_REJECT, 2);
        assert_eq!(node_filter::FILTER_SKIP, 3);
    }
}
