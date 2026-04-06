//! Range API implementation
//!
//! The Range represents a fragment of a document that can contain nodes and parts of text nodes.
//! This module provides the native Range implementation for the Boa JavaScript engine.

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsValue,
    builtins::BuiltInBuilder,
    builtins::{BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
};
use boa_gc::{Finalize, Trace};

/// How to position the boundary point in relation to the node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeHowType {
    /// START_TO_START comparison
    StartToStart = 0,
    /// START_TO_END comparison
    StartToEnd = 1,
    /// END_TO_END comparison
    EndToEnd = 2,
    /// END_TO_START comparison
    EndToStart = 3,
}

/// The Range object represents a document fragment
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct RangeData {
    /// The start boundary-point node
    start_container: Option<JsValue>,
    /// The start boundary-point offset
    start_offset: u32,
    /// The end boundary-point node
    end_container: Option<JsValue>,
    /// The end boundary-point offset
    end_offset: u32,
    /// Whether the range is collapsed
    collapsed: bool,
    /// Common ancestor container (cached)
    common_ancestor_container: Option<JsValue>,
}

impl Default for RangeData {
    fn default() -> Self {
        Self::new()
    }
}

impl RangeData {
    pub fn new() -> Self {
        Self {
            start_container: None,
            start_offset: 0,
            end_container: None,
            end_offset: 0,
            collapsed: true,
            common_ancestor_container: None,
        }
    }

    /// Get the start container
    pub fn start_container(&self) -> Option<&JsValue> {
        self.start_container.as_ref()
    }

    /// Get the start offset
    pub fn start_offset(&self) -> u32 {
        self.start_offset
    }

    /// Get the end container
    pub fn end_container(&self) -> Option<&JsValue> {
        self.end_container.as_ref()
    }

    /// Get the end offset
    pub fn end_offset(&self) -> u32 {
        self.end_offset
    }

    /// Set the start of the range
    pub fn set_start(&mut self, node: JsValue, offset: u32) -> JsResult<()> {
        self.start_container = Some(node.clone());
        self.start_offset = offset;

        // If end is not set or start is after end, update end
        if self.end_container.is_none() {
            self.end_container = Some(node);
            self.end_offset = offset;
        }

        self.update_state();
        eprintln!("Range: Set start to node with offset {}", offset);
        Ok(())
    }

    /// Set the end of the range
    pub fn set_end(&mut self, node: JsValue, offset: u32) -> JsResult<()> {
        self.end_container = Some(node.clone());
        self.end_offset = offset;

        // If start is not set, update start
        if self.start_container.is_none() {
            self.start_container = Some(node);
            self.start_offset = offset;
        }

        self.update_state();
        eprintln!("Range: Set end to node with offset {}", offset);
        Ok(())
    }

    /// Set both start and end to the same position (collapse)
    fn set_start_and_end(
        &mut self,
        node: JsValue,
        start_offset: u32,
        end_offset: u32,
    ) -> JsResult<()> {
        self.start_container = Some(node.clone());
        self.start_offset = start_offset;
        self.end_container = Some(node);
        self.end_offset = end_offset;

        self.update_state();
        eprintln!(
            "Range: Set range from offset {} to {}",
            start_offset, end_offset
        );
        Ok(())
    }

    /// Collapse the range to a single point
    fn collapse(&mut self, to_start: bool) {
        if to_start {
            if let Some(start_node) = &self.start_container {
                self.end_container = Some(start_node.clone());
                self.end_offset = self.start_offset;
            }
        } else if let Some(end_node) = &self.end_container {
            self.start_container = Some(end_node.clone());
            self.start_offset = self.end_offset;
        }

        self.update_state();
        eprintln!(
            "Range: Collapsed to {}",
            if to_start { "start" } else { "end" }
        );
    }

    /// Select the contents of a node
    fn select_node_contents(&mut self, node: JsValue) -> JsResult<()> {
        // For a complete implementation, we'd need to determine the node's children count
        // For now, we'll set the range to span the entire node
        self.start_container = Some(node.clone());
        self.start_offset = 0;
        self.end_container = Some(node);
        // In a real implementation, we'd calculate the actual end offset based on node type
        self.end_offset = 0; // Would be node.childNodes.length for element nodes

        self.update_state();
        eprintln!("Range: Selected node contents");
        Ok(())
    }

    /// Select an entire node
    fn select_node(&mut self, node: JsValue) -> JsResult<()> {
        // For a complete implementation, we'd need to access the parent node
        // For now, we'll approximate by selecting the node's contents
        self.select_node_contents(node)?;
        eprintln!("Range: Selected entire node");
        Ok(())
    }

    /// Update internal state after boundary changes
    fn update_state(&mut self) {
        // Check if range is collapsed
        self.collapsed =
            self.start_container == self.end_container && self.start_offset == self.end_offset;

        // Update common ancestor container (simplified implementation)
        if let (Some(start), Some(end)) = (&self.start_container, &self.end_container) {
            if start == end {
                self.common_ancestor_container = Some(start.clone());
            } else {
                // In a real implementation, we'd traverse the DOM tree to find the common ancestor
                // For now, we'll use the start container as a fallback
                self.common_ancestor_container = Some(start.clone());
            }
        }
    }

    /// Compare boundary points with another range
    fn compare_boundary_points(&self, how: RangeHowType, other: &RangeData) -> i16 {
        // Simplified comparison implementation
        // In a real implementation, this would need proper DOM tree position comparison

        let (this_node, this_offset) = match how {
            RangeHowType::StartToStart | RangeHowType::StartToEnd => {
                (&self.start_container, self.start_offset)
            }
            RangeHowType::EndToEnd | RangeHowType::EndToStart => {
                (&self.end_container, self.end_offset)
            }
        };

        let (other_node, other_offset) = match how {
            RangeHowType::StartToStart | RangeHowType::EndToStart => {
                (&other.start_container, other.start_offset)
            }
            RangeHowType::StartToEnd | RangeHowType::EndToEnd => {
                (&other.end_container, other.end_offset)
            }
        };

        // Simple offset comparison (would need proper DOM position comparison)
        if this_node == other_node {
            if this_offset < other_offset {
                -1
            } else if this_offset > other_offset {
                1
            } else {
                0
            }
        } else {
            // Different nodes - would need tree position comparison
            0
        }
    }

    /// Extract the contents of the range (simplified)
    fn extract_contents(&mut self) -> JsResult<JsValue> {
        // In a real implementation, this would create a DocumentFragment
        // and move the range contents into it
        eprintln!("Range: Extracted contents (simplified implementation)");

        // Collapse the range after extraction
        self.collapse(true);

        // Return a mock DocumentFragment-like object
        Ok(JsValue::null())
    }

    /// Clone the contents of the range
    fn clone_contents(&self) -> JsResult<JsValue> {
        // In a real implementation, this would create a DocumentFragment
        // with cloned copies of the range contents
        eprintln!("Range: Cloned contents (simplified implementation)");
        Ok(JsValue::null())
    }

    /// Insert a node at the start of the range
    fn insert_node(&mut self, _node: JsValue) -> JsResult<()> {
        // In a real implementation, this would insert the node into the DOM
        // and update the range boundaries accordingly
        eprintln!("Range: Inserted node (simplified implementation)");
        Ok(())
    }

    /// Surround the range contents with a node
    fn surround_contents(&mut self, _new_parent: JsValue) -> JsResult<()> {
        // In a real implementation, this would:
        // 1. Extract the range contents
        // 2. Insert the new parent at the range position
        // 3. Append the extracted contents to the new parent
        // 4. Update the range to select the new parent's contents
        eprintln!("Range: Surrounded contents with new parent (simplified implementation)");
        Ok(())
    }

    /// Clone the range
    fn clone_range(&self) -> Self {
        Self {
            start_container: self.start_container.clone(),
            start_offset: self.start_offset,
            end_container: self.end_container.clone(),
            end_offset: self.end_offset,
            collapsed: self.collapsed,
            common_ancestor_container: self.common_ancestor_container.clone(),
        }
    }

    /// Detach the range (make it inert)
    fn detach(&mut self) {
        self.start_container = None;
        self.start_offset = 0;
        self.end_container = None;
        self.end_offset = 0;
        self.collapsed = true;
        self.common_ancestor_container = None;
        eprintln!("Range: Detached");
    }
}

/// The `Range` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct Range;

impl IntrinsicObject for Range {
    fn init(realm: &Realm) {
        let start_container_func = BuiltInBuilder::callable(realm, get_start_container)
            .name(js_string!("get startContainer"))
            .build();

        let start_offset_func = BuiltInBuilder::callable(realm, get_start_offset)
            .name(js_string!("get startOffset"))
            .build();

        let end_container_func = BuiltInBuilder::callable(realm, get_end_container)
            .name(js_string!("get endContainer"))
            .build();

        let end_offset_func = BuiltInBuilder::callable(realm, get_end_offset)
            .name(js_string!("get endOffset"))
            .build();

        let collapsed_func = BuiltInBuilder::callable(realm, get_collapsed)
            .name(js_string!("get collapsed"))
            .build();

        let common_ancestor_func = BuiltInBuilder::callable(realm, get_common_ancestor_container)
            .name(js_string!("get commonAncestorContainer"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("startContainer"),
                Some(start_container_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("startOffset"),
                Some(start_offset_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("endContainer"),
                Some(end_container_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("endOffset"),
                Some(end_offset_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("collapsed"),
                Some(collapsed_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("commonAncestorContainer"),
                Some(common_ancestor_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(set_start, js_string!("setStart"), 2)
            .method(set_end, js_string!("setEnd"), 2)
            .method(set_start_before, js_string!("setStartBefore"), 1)
            .method(set_start_after, js_string!("setStartAfter"), 1)
            .method(set_end_before, js_string!("setEndBefore"), 1)
            .method(set_end_after, js_string!("setEndAfter"), 1)
            .method(collapse, js_string!("collapse"), 1)
            .method(select_node, js_string!("selectNode"), 1)
            .method(select_node_contents, js_string!("selectNodeContents"), 1)
            .method(
                compare_boundary_points,
                js_string!("compareBoundaryPoints"),
                2,
            )
            .method(delete_contents, js_string!("deleteContents"), 0)
            .method(extract_contents, js_string!("extractContents"), 0)
            .method(clone_contents, js_string!("cloneContents"), 0)
            .method(insert_node, js_string!("insertNode"), 1)
            .method(surround_contents, js_string!("surroundContents"), 1)
            .method(clone_range, js_string!("cloneRange"), 0)
            .method(to_string, js_string!("toString"), 0)
            .method(detach, js_string!("detach"), 0)
            .method(
                create_contextual_fragment,
                js_string!("createContextualFragment"),
                1,
            )
            .method(
                get_bounding_client_rect,
                js_string!("getBoundingClientRect"),
                0,
            )
            .method(get_client_rects, js_string!("getClientRects"), 0)
            .method(is_point_in_range, js_string!("isPointInRange"), 2)
            .method(compare_point, js_string!("comparePoint"), 2)
            .method(intersects_node, js_string!("intersectsNode"), 1)
            .static_property(
                js_string!("START_TO_START"),
                0,
                Attribute::READONLY.union(Attribute::NON_ENUMERABLE),
            )
            .static_property(
                js_string!("START_TO_END"),
                1,
                Attribute::READONLY.union(Attribute::NON_ENUMERABLE),
            )
            .static_property(
                js_string!("END_TO_END"),
                2,
                Attribute::READONLY.union(Attribute::NON_ENUMERABLE),
            )
            .static_property(
                js_string!("END_TO_START"),
                3,
                Attribute::READONLY.union(Attribute::NON_ENUMERABLE),
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Range {
    const NAME: JsString = StaticJsStrings::RANGE;
}

impl BuiltInConstructor for Range {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::range;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If new_target is undefined then this function was called without new
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling Range constructor without `new` is forbidden")
                .into());
        }

        let data = RangeData::new();
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::range, context)?;
        let range =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, data);
        Ok(range.into())
    }
}

/// Get the start container of the range.
fn get_start_container(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let range_data = this_obj.downcast_ref::<RangeData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Range method called on non-Range object")
    })?;
    Ok(range_data
        .start_container
        .clone()
        .unwrap_or(JsValue::null()))
}

/// Get the start offset of the range.
fn get_start_offset(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let range_data = this_obj.downcast_ref::<RangeData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Range method called on non-Range object")
    })?;
    Ok(JsValue::from(range_data.start_offset))
}

/// Get the end container of the range.
fn get_end_container(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let range_data = this_obj.downcast_ref::<RangeData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Range method called on non-Range object")
    })?;
    Ok(range_data.end_container.clone().unwrap_or(JsValue::null()))
}

/// Get the end offset of the range.
fn get_end_offset(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let range_data = this_obj.downcast_ref::<RangeData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Range method called on non-Range object")
    })?;
    Ok(JsValue::from(range_data.end_offset))
}

/// Get whether the range is collapsed.
fn get_collapsed(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let range_data = this_obj.downcast_ref::<RangeData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Range method called on non-Range object")
    })?;
    Ok(JsValue::from(range_data.collapsed))
}

/// Get the common ancestor container.
fn get_common_ancestor_container(
    this: &JsValue,
    _: &[JsValue],
    _: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let range_data = this_obj.downcast_ref::<RangeData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Range method called on non-Range object")
    })?;
    Ok(range_data
        .common_ancestor_container
        .clone()
        .unwrap_or(JsValue::null()))
}

/// Set the start of the range.
fn set_start(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let node = args.get_or_undefined(0);
    let offset = match args.get_or_undefined(1).to_integer_or_infinity(context)? {
        boa_engine::value::IntegerOrInfinity::Integer(i) => i.max(0) as u32,
        _ => 0,
    };

    if let Some(mut range_data) = this_obj.downcast_mut::<RangeData>() {
        range_data.set_start(node.clone(), offset)?;
    }

    Ok(JsValue::undefined())
}

/// Set the end of the range.
fn set_end(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let node = args.get_or_undefined(0);
    let offset = match args.get_or_undefined(1).to_integer_or_infinity(context)? {
        boa_engine::value::IntegerOrInfinity::Integer(i) => i.max(0) as u32,
        _ => 0,
    };

    if let Some(mut range_data) = this_obj.downcast_mut::<RangeData>() {
        range_data.set_end(node.clone(), offset)?;
    }

    Ok(JsValue::undefined())
}

/// Set the start before a node.
fn set_start_before(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let node = args.get_or_undefined(0);

    if let Some(mut range_data) = this_obj.downcast_mut::<RangeData>() {
        // In a real implementation, we'd set the start to the position before the node
        // For now, we'll set it to the node with offset 0
        range_data.set_start(node.clone(), 0)?;
    }

    Ok(JsValue::undefined())
}

/// Set the start after a node.
fn set_start_after(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let node = args.get_or_undefined(0);

    if let Some(mut range_data) = this_obj.downcast_mut::<RangeData>() {
        // In a real implementation, we'd set the start to the position after the node
        // For now, we'll set it to the node with offset 1
        range_data.set_start(node.clone(), 1)?;
    }

    Ok(JsValue::undefined())
}

/// Set the end before a node.
fn set_end_before(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let node = args.get_or_undefined(0);

    if let Some(mut range_data) = this_obj.downcast_mut::<RangeData>() {
        // In a real implementation, we'd set the end to the position before the node
        // For now, we'll set it to the node with offset 0
        range_data.set_end(node.clone(), 0)?;
    }

    Ok(JsValue::undefined())
}

/// Set the end after a node.
fn set_end_after(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let node = args.get_or_undefined(0);

    if let Some(mut range_data) = this_obj.downcast_mut::<RangeData>() {
        // In a real implementation, we'd set the end to the position after the node
        // For now, we'll set it to the node with offset 1
        range_data.set_end(node.clone(), 1)?;
    }

    Ok(JsValue::undefined())
}

/// Collapse the range to a single point.
fn collapse(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let to_start = args.get_or_undefined(0).to_boolean();

    if let Some(mut range_data) = this_obj.downcast_mut::<RangeData>() {
        range_data.collapse(to_start);
    }

    Ok(JsValue::undefined())
}

/// Select an entire node.
fn select_node(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let node = args.get_or_undefined(0);

    if let Some(mut range_data) = this_obj.downcast_mut::<RangeData>() {
        range_data.select_node(node.clone())?;
    }

    Ok(JsValue::undefined())
}

/// Select the contents of a node.
fn select_node_contents(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let node = args.get_or_undefined(0);

    if let Some(mut range_data) = this_obj.downcast_mut::<RangeData>() {
        range_data.select_node_contents(node.clone())?;
    }

    Ok(JsValue::undefined())
}

/// Compare boundary points with another range.
fn compare_boundary_points(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let how = match args.get_or_undefined(0).to_integer_or_infinity(context)? {
        boa_engine::value::IntegerOrInfinity::Integer(i) => match i {
            0 => RangeHowType::StartToStart,
            1 => RangeHowType::StartToEnd,
            2 => RangeHowType::EndToEnd,
            3 => RangeHowType::EndToStart,
            _ => {
                return Err(JsNativeError::range()
                    .with_message("Invalid how parameter")
                    .into());
            }
        },
        _ => {
            return Err(JsNativeError::range()
                .with_message("Invalid how parameter")
                .into());
        }
    };

    let other_range_obj = args.get_or_undefined(1).as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Second argument must be a Range object")
    })?;

    let range_data = this_obj.downcast_ref::<RangeData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Range method called on non-Range object")
    })?;

    let other_range_data = other_range_obj.downcast_ref::<RangeData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Second argument must be a Range object")
    })?;

    let result = range_data.compare_boundary_points(how, &other_range_data);
    Ok(JsValue::from(result))
}

/// Delete the contents of the range.
fn delete_contents(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    if let Some(mut range_data) = this_obj.downcast_mut::<RangeData>() {
        // In a real implementation, this would remove the range contents from the DOM
        range_data.collapse(true);
        eprintln!("Range: Deleted contents");
    }

    Ok(JsValue::undefined())
}

/// Extract the contents of the range.
fn extract_contents(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let mut range_data = this_obj.downcast_mut::<RangeData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Range method called on non-Range object")
    })?;

    range_data.extract_contents()
}

/// Clone the contents of the range.
fn clone_contents(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let range_data = this_obj.downcast_ref::<RangeData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Range method called on non-Range object")
    })?;
    range_data.clone_contents()
}

/// Insert a node at the start of the range.
fn insert_node(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let node = args.get_or_undefined(0);

    if let Some(mut range_data) = this_obj.downcast_mut::<RangeData>() {
        range_data.insert_node(node.clone())?;
    }

    Ok(JsValue::undefined())
}

/// Surround the range contents with a node.
fn surround_contents(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let new_parent = args.get_or_undefined(0);

    if let Some(mut range_data) = this_obj.downcast_mut::<RangeData>() {
        range_data.surround_contents(new_parent.clone())?;
    }

    Ok(JsValue::undefined())
}

/// Clone the range.
fn clone_range(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let range_data = this_obj.downcast_ref::<RangeData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Range method called on non-Range object")
    })?;

    let cloned_data = range_data.clone_range();
    let range_constructor =
        StandardConstructors::range(context.intrinsics().constructors()).constructor();
    let prototype = get_prototype_from_constructor(
        &range_constructor.into(),
        StandardConstructors::range,
        context,
    )?;
    let cloned_range = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        prototype,
        cloned_data,
    );
    Ok(cloned_range.into())
}

/// Convert the range to a string.
fn to_string(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let _range_data = this_obj.downcast_ref::<RangeData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Range method called on non-Range object")
    })?;

    // In a real implementation, this would return the text content of the range
    Ok(JsValue::from(js_string!("")))
}

/// Detach the range.
fn detach(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    if let Some(mut range_data) = this_obj.downcast_mut::<RangeData>() {
        range_data.detach();
    }

    Ok(JsValue::undefined())
}

/// Create a DocumentFragment from an HTML string, parsed in the context of the range.
///
/// This method parses the provided HTML string and returns a DocumentFragment
/// containing the resulting nodes. The parsing context is determined by the
/// range's start container.
fn create_contextual_fragment(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let _this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let fragment_string = args.get_or_undefined(0).to_string(context)?;

    // In a full implementation, this would:
    // 1. Determine the parsing context from the start container
    // 2. Parse the HTML string according to HTML fragment parsing algorithm
    // 3. Return a DocumentFragment with the parsed nodes

    eprintln!(
        "Range: createContextualFragment called with: {}",
        fragment_string.to_std_string_escaped()
    );

    // Return a basic DocumentFragment
    // Note: Full HTML fragment parsing requires integrating html5ever or similar
    // For now, we create an empty DocumentFragment as a placeholder
    // A complete implementation would:
    // 1. Use the HTML fragment parsing algorithm from WHATWG spec
    // 2. Parse the fragment_string in the context of the range's start container
    // 3. Handle script elements, template elements, and other special cases
    use crate::dom::document_fragment::DocumentFragmentData;
    let fragment_data = DocumentFragmentData::new();
    let fragment_proto = context
        .intrinsics()
        .constructors()
        .document_fragment()
        .prototype();
    let fragment_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        fragment_proto,
        fragment_data,
    );
    Ok(fragment_obj.into())
}

/// Get a DOMRect object representing the bounding rectangle of the range's contents.
///
/// Returns a DOMRect with x, y, width, height, top, right, bottom, left properties.
fn get_bounding_client_rect(
    this: &JsValue,
    _: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let _this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    // Create a DOMRect-like object
    // In a real implementation, this would calculate the actual bounding box
    // based on the rendered positions of the range's contents
    let rect = boa_engine::object::ObjectInitializer::new(context)
        .property(js_string!("x"), 0.0, Attribute::all())
        .property(js_string!("y"), 0.0, Attribute::all())
        .property(js_string!("width"), 0.0, Attribute::all())
        .property(js_string!("height"), 0.0, Attribute::all())
        .property(js_string!("top"), 0.0, Attribute::all())
        .property(js_string!("right"), 0.0, Attribute::all())
        .property(js_string!("bottom"), 0.0, Attribute::all())
        .property(js_string!("left"), 0.0, Attribute::all())
        .build();

    eprintln!("Range: getBoundingClientRect called (returning zero-rect for headless browser)");
    Ok(rect.into())
}

/// Get a list of DOMRect objects representing the client rectangles of the range.
///
/// For inline elements that span multiple lines, this returns multiple rectangles.
fn get_client_rects(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    // Create a DOMRectList-like object with array-like properties
    // In a real implementation, this would return an array of DOMRects
    // representing each line/box of the range's contents
    let rect_list = boa_engine::object::ObjectInitializer::new(context)
        .property(js_string!("length"), 0, Attribute::READONLY)
        .build();

    eprintln!("Range: getClientRects called (returning empty list for headless browser)");
    Ok(rect_list.into())
}

/// Check if a point (node + offset) is within the range.
///
/// Returns true if the point is between the range's start and end boundaries.
fn is_point_in_range(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let node = args.get_or_undefined(0);
    let offset = match args.get_or_undefined(1).to_integer_or_infinity(context)? {
        boa_engine::value::IntegerOrInfinity::Integer(i) => i.max(0) as u32,
        _ => 0,
    };

    let range_data = this_obj.downcast_ref::<RangeData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Range method called on non-Range object")
    })?;

    // Simplified implementation: check if the node matches and offset is within range
    // A full implementation would need proper DOM tree position comparison
    let in_range = if let (Some(start), Some(end)) =
        (&range_data.start_container, &range_data.end_container)
    {
        if start == node && end == node {
            // Same node - check offset bounds
            offset >= range_data.start_offset && offset <= range_data.end_offset
        } else if start == node {
            // Point is at start node - check if offset is >= start_offset
            offset >= range_data.start_offset
        } else if end == node {
            // Point is at end node - check if offset is <= end_offset
            offset <= range_data.end_offset
        } else {
            // Different nodes - would need tree position comparison
            // For now, return false
            false
        }
    } else {
        false
    };

    eprintln!(
        "Range: isPointInRange called with offset {} -> {}",
        offset, in_range
    );
    Ok(JsValue::from(in_range))
}

/// Compare a point (node + offset) to the range.
///
/// Returns:
/// - -1 if the point is before the range
/// - 0 if the point is within the range
/// - 1 if the point is after the range
fn compare_point(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let node = args.get_or_undefined(0);
    let offset = match args.get_or_undefined(1).to_integer_or_infinity(context)? {
        boa_engine::value::IntegerOrInfinity::Integer(i) => i.max(0) as u32,
        _ => 0,
    };

    let range_data = this_obj.downcast_ref::<RangeData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Range method called on non-Range object")
    })?;

    // Simplified implementation
    // A full implementation would need proper DOM tree position comparison
    let result = if let (Some(start), Some(end)) =
        (&range_data.start_container, &range_data.end_container)
    {
        if start == node && end == node {
            // Same node
            if offset < range_data.start_offset {
                -1i16
            } else if offset > range_data.end_offset {
                1i16
            } else {
                0i16
            }
        } else if start == node {
            if offset < range_data.start_offset {
                -1i16
            } else {
                0i16 // Within range
            }
        } else if end == node {
            if offset > range_data.end_offset {
                1i16
            } else {
                0i16 // Within range
            }
        } else {
            // Different nodes - would need tree position comparison
            0i16
        }
    } else {
        0i16
    };

    eprintln!(
        "Range: comparePoint called with offset {} -> {}",
        offset, result
    );
    Ok(JsValue::from(result))
}

/// Check if the range intersects a given node.
///
/// Returns true if any part of the node is within the range.
fn intersects_node(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Range method called on non-object"))?;

    let node = args.get_or_undefined(0);

    let range_data = this_obj.downcast_ref::<RangeData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Range method called on non-Range object")
    })?;

    // Simplified implementation: check if node is one of the boundary containers
    // or the common ancestor. A full implementation would need DOM tree traversal.
    let intersects = if let (Some(start), Some(end)) =
        (&range_data.start_container, &range_data.end_container)
    {
        start == node
            || end == node
            || (range_data.common_ancestor_container.as_ref() == Some(node))
    } else {
        false
    };

    eprintln!("Range: intersectsNode called -> {}", intersects);
    Ok(JsValue::from(intersects))
}
