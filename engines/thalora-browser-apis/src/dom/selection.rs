//! Selection API implementation
//!
//! The Selection API represents the range of text selected by the user or the current
//! position of the caret. A user may make a selection from left to right (in document order)
//! or right to left (reverse of document order).
//!
//! This module provides the native Selection implementation for the Boa JavaScript engine.

use boa_gc::{Finalize, Trace};
use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsValue,
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    Context,
};

use super::range::RangeData;

/// Selection type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionType {
    /// No selection
    None,
    /// Caret (collapsed selection)
    Caret,
    /// Range selection
    Range,
}

impl SelectionType {
    /// Convert to JavaScript string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            SelectionType::None => "None",
            SelectionType::Caret => "Caret",
            SelectionType::Range => "Range",
        }
    }
}

/// The Selection object data
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct SelectionData {
    /// The node in which the selection begins (anchor)
    anchor_node: Option<JsValue>,
    /// The offset within the anchor node where the selection begins
    anchor_offset: u32,
    /// The node in which the selection ends (focus)
    focus_node: Option<JsValue>,
    /// The offset within the focus node where the selection ends
    focus_offset: u32,
    /// The ranges in this selection
    ranges: Vec<JsObject>,
    /// Whether the selection is collapsed (anchor and focus are at the same position)
    #[unsafe_ignore_trace]
    is_collapsed: bool,
    /// The selection type
    #[unsafe_ignore_trace]
    selection_type: SelectionType,
}

impl Default for SelectionData {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectionData {
    pub fn new() -> Self {
        Self {
            anchor_node: None,
            anchor_offset: 0,
            focus_node: None,
            focus_offset: 0,
            ranges: Vec::new(),
            is_collapsed: true,
            selection_type: SelectionType::None,
        }
    }

    /// Update internal state based on current ranges
    fn update_state(&mut self) {
        if self.ranges.is_empty() {
            self.is_collapsed = true;
            self.selection_type = SelectionType::None;
            self.anchor_node = None;
            self.anchor_offset = 0;
            self.focus_node = None;
            self.focus_offset = 0;
        } else {
            // For simplicity, use the first range's boundaries
            // In a full implementation, anchor/focus might differ from range boundaries
            // depending on selection direction
            self.is_collapsed = self.anchor_node == self.focus_node &&
                               self.anchor_offset == self.focus_offset;
            self.selection_type = if self.is_collapsed {
                SelectionType::Caret
            } else {
                SelectionType::Range
            };
        }
    }

    /// Add a range to the selection
    pub fn add_range(&mut self, range: JsObject) -> JsResult<()> {
        // According to spec, most browsers only support one range
        // But we'll support multiple for completeness
        self.ranges.push(range.clone());

        // Update anchor/focus from the range
        if let Some(range_data) = range.downcast_ref::<RangeData>() {
            if self.ranges.len() == 1 {
                // First range - set anchor from start, focus from end
                self.anchor_node = range_data.start_container().cloned();
                self.anchor_offset = range_data.start_offset();
                self.focus_node = range_data.end_container().cloned();
                self.focus_offset = range_data.end_offset();
            }
        }

        self.update_state();
        eprintln!("Selection: Added range, now have {} range(s)", self.ranges.len());
        Ok(())
    }

    /// Remove all ranges from the selection
    pub fn remove_all_ranges(&mut self) {
        self.ranges.clear();
        self.anchor_node = None;
        self.anchor_offset = 0;
        self.focus_node = None;
        self.focus_offset = 0;
        self.update_state();
        eprintln!("Selection: Removed all ranges");
    }

    /// Remove a specific range from the selection
    pub fn remove_range(&mut self, range: &JsObject) -> JsResult<()> {
        // Find and remove the range by reference equality
        let initial_len = self.ranges.len();
        self.ranges.retain(|r| !JsObject::equals(r, range));

        if self.ranges.len() < initial_len {
            self.update_state();
            eprintln!("Selection: Removed range");
        }
        Ok(())
    }

    /// Get a range at the specified index
    pub fn get_range_at(&self, index: usize) -> Option<&JsObject> {
        self.ranges.get(index)
    }

    /// Collapse the selection to a single point
    pub fn collapse(&mut self, node: JsValue, offset: u32) {
        self.ranges.clear();
        self.anchor_node = Some(node.clone());
        self.anchor_offset = offset;
        self.focus_node = Some(node);
        self.focus_offset = offset;
        self.is_collapsed = true;
        self.selection_type = SelectionType::Caret;
        eprintln!("Selection: Collapsed to offset {}", offset);
    }

    /// Collapse to the start of the selection
    pub fn collapse_to_start(&mut self) {
        if let Some(anchor) = self.anchor_node.clone() {
            let offset = self.anchor_offset;
            self.collapse(anchor, offset);
            eprintln!("Selection: Collapsed to start");
        }
    }

    /// Collapse to the end of the selection
    pub fn collapse_to_end(&mut self) {
        if let Some(focus) = self.focus_node.clone() {
            let offset = self.focus_offset;
            self.collapse(focus, offset);
            eprintln!("Selection: Collapsed to end");
        }
    }

    /// Extend the selection to a new point
    pub fn extend(&mut self, node: JsValue, offset: u32) {
        self.focus_node = Some(node);
        self.focus_offset = offset;
        self.update_state();
        eprintln!("Selection: Extended to offset {}", offset);
    }

    /// Set the selection using base and extent points
    pub fn set_base_and_extent(
        &mut self,
        anchor_node: JsValue,
        anchor_offset: u32,
        focus_node: JsValue,
        focus_offset: u32,
    ) {
        self.anchor_node = Some(anchor_node);
        self.anchor_offset = anchor_offset;
        self.focus_node = Some(focus_node);
        self.focus_offset = focus_offset;
        self.ranges.clear(); // Clear existing ranges
        self.update_state();
        eprintln!(
            "Selection: Set base and extent (anchor offset: {}, focus offset: {})",
            anchor_offset, focus_offset
        );
    }

    /// Check if the selection contains a node
    pub fn contains_node(&self, node: &JsValue, allow_partial: bool) -> bool {
        // Simplified implementation - check if node is anchor or focus
        let is_anchor = self.anchor_node.as_ref().map_or(false, |n| n == node);
        let is_focus = self.focus_node.as_ref().map_or(false, |n| n == node);

        if allow_partial {
            is_anchor || is_focus
        } else {
            is_anchor && is_focus
        }
    }

    /// Select all children of a node
    pub fn select_all_children(&mut self, node: JsValue) {
        self.anchor_node = Some(node.clone());
        self.anchor_offset = 0;
        self.focus_node = Some(node);
        // In a real implementation, focus_offset would be the number of child nodes
        self.focus_offset = 0;
        self.update_state();
        eprintln!("Selection: Selected all children of node");
    }

    /// Get the number of ranges
    pub fn range_count(&self) -> usize {
        self.ranges.len()
    }

    /// Get the selection type as a string
    pub fn type_string(&self) -> &'static str {
        self.selection_type.as_str()
    }
}

/// The `Selection` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct Selection;

impl IntrinsicObject for Selection {
    fn init(realm: &Realm) {
        // Create accessor functions for properties
        let anchor_node_getter = BuiltInBuilder::callable(realm, get_anchor_node)
            .name(js_string!("get anchorNode"))
            .build();

        let anchor_offset_getter = BuiltInBuilder::callable(realm, get_anchor_offset)
            .name(js_string!("get anchorOffset"))
            .build();

        let focus_node_getter = BuiltInBuilder::callable(realm, get_focus_node)
            .name(js_string!("get focusNode"))
            .build();

        let focus_offset_getter = BuiltInBuilder::callable(realm, get_focus_offset)
            .name(js_string!("get focusOffset"))
            .build();

        let is_collapsed_getter = BuiltInBuilder::callable(realm, get_is_collapsed)
            .name(js_string!("get isCollapsed"))
            .build();

        let range_count_getter = BuiltInBuilder::callable(realm, get_range_count)
            .name(js_string!("get rangeCount"))
            .build();

        let type_getter = BuiltInBuilder::callable(realm, get_type)
            .name(js_string!("get type"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Property accessors
            .accessor(
                js_string!("anchorNode"),
                Some(anchor_node_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("anchorOffset"),
                Some(anchor_offset_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("focusNode"),
                Some(focus_node_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("focusOffset"),
                Some(focus_offset_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("isCollapsed"),
                Some(is_collapsed_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("rangeCount"),
                Some(range_count_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("type"),
                Some(type_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            // Methods
            .method(add_range, js_string!("addRange"), 1)
            .method(collapse, js_string!("collapse"), 2)
            .method(collapse_to_end, js_string!("collapseToEnd"), 0)
            .method(collapse_to_start, js_string!("collapseToStart"), 0)
            .method(contains_node, js_string!("containsNode"), 2)
            .method(delete_from_document, js_string!("deleteFromDocument"), 0)
            .method(empty, js_string!("empty"), 0)
            .method(extend, js_string!("extend"), 2)
            .method(get_range_at, js_string!("getRangeAt"), 1)
            .method(modify, js_string!("modify"), 3)
            .method(remove_all_ranges, js_string!("removeAllRanges"), 0)
            .method(remove_range, js_string!("removeRange"), 1)
            .method(select_all_children, js_string!("selectAllChildren"), 1)
            .method(set_base_and_extent, js_string!("setBaseAndExtent"), 4)
            .method(to_string, js_string!("toString"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Selection {
    const NAME: JsString = StaticJsStrings::SELECTION;
}

impl BuiltInConstructor for Selection {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::selection;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Selection objects are typically not created directly with `new`
        // They are obtained from window.getSelection() or document.getSelection()
        // But we'll support construction for completeness
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("calling Selection constructor without `new` is forbidden")
                .into());
        }

        let data = SelectionData::new();
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::selection, context)?;
        let selection =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, data);
        Ok(selection.into())
    }
}

// Property getters

fn get_anchor_node(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let selection_data = this_obj.downcast_ref::<SelectionData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-Selection object")
    })?;

    Ok(selection_data.anchor_node.clone().unwrap_or(JsValue::null()))
}

fn get_anchor_offset(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let selection_data = this_obj.downcast_ref::<SelectionData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-Selection object")
    })?;

    Ok(JsValue::from(selection_data.anchor_offset))
}

fn get_focus_node(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let selection_data = this_obj.downcast_ref::<SelectionData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-Selection object")
    })?;

    Ok(selection_data.focus_node.clone().unwrap_or(JsValue::null()))
}

fn get_focus_offset(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let selection_data = this_obj.downcast_ref::<SelectionData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-Selection object")
    })?;

    Ok(JsValue::from(selection_data.focus_offset))
}

fn get_is_collapsed(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let selection_data = this_obj.downcast_ref::<SelectionData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-Selection object")
    })?;

    Ok(JsValue::from(selection_data.is_collapsed))
}

fn get_range_count(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let selection_data = this_obj.downcast_ref::<SelectionData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-Selection object")
    })?;

    Ok(JsValue::from(selection_data.range_count() as u32))
}

fn get_type(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let selection_data = this_obj.downcast_ref::<SelectionData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-Selection object")
    })?;

    Ok(JsValue::from(js_string!(selection_data.type_string())))
}

// Methods

fn add_range(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let range = args.get_or_undefined(0).as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("First argument must be a Range object")
    })?;

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        selection_data.add_range(range.clone())?;
    }

    Ok(JsValue::undefined())
}

fn collapse(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let node = args.get_or_undefined(0).clone();
    let offset = match args.get_or_undefined(1).to_integer_or_infinity(context)? {
        boa_engine::value::IntegerOrInfinity::Integer(i) => i.max(0) as u32,
        _ => 0,
    };

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        selection_data.collapse(node, offset);
    }

    Ok(JsValue::undefined())
}

fn collapse_to_end(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        selection_data.collapse_to_end();
    }

    Ok(JsValue::undefined())
}

fn collapse_to_start(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        selection_data.collapse_to_start();
    }

    Ok(JsValue::undefined())
}

fn contains_node(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let node = args.get_or_undefined(0);
    let allow_partial = args.get_or_undefined(1).to_boolean();

    let selection_data = this_obj.downcast_ref::<SelectionData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-Selection object")
    })?;

    let contains = selection_data.contains_node(node, allow_partial);
    Ok(JsValue::from(contains))
}

fn delete_from_document(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    // In a real implementation, this would delete the selected content from the document
    // and collapse the selection to the start
    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        selection_data.collapse_to_start();
        eprintln!("Selection: deleteFromDocument called (content deletion not implemented in headless browser)");
    }

    Ok(JsValue::undefined())
}

fn empty(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // empty() is an alias for removeAllRanges()
    remove_all_ranges(this, args, context)
}

fn extend(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let node = args.get_or_undefined(0).clone();
    let offset = match args.get_or_undefined(1).to_integer_or_infinity(context)? {
        boa_engine::value::IntegerOrInfinity::Integer(i) => i.max(0) as u32,
        _ => 0,
    };

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        selection_data.extend(node, offset);
    }

    Ok(JsValue::undefined())
}

fn get_range_at(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let index = match args.get_or_undefined(0).to_integer_or_infinity(context)? {
        boa_engine::value::IntegerOrInfinity::Integer(i) => i.max(0) as usize,
        _ => 0,
    };

    let selection_data = this_obj.downcast_ref::<SelectionData>().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-Selection object")
    })?;

    match selection_data.get_range_at(index) {
        Some(range) => Ok(range.clone().into()),
        None => Err(JsNativeError::range()
            .with_message(format!("Index {} is out of range", index))
            .into()),
    }
}

fn modify(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let alter = args.get_or_undefined(0).to_string(context)?;
    let direction = args.get_or_undefined(1).to_string(context)?;
    let granularity = args.get_or_undefined(2).to_string(context)?;

    // In a real implementation, this would modify the selection based on:
    // alter: "move" or "extend"
    // direction: "forward", "backward", "left", "right"
    // granularity: "character", "word", "sentence", "line", "paragraph", "lineboundary", "sentenceboundary", "paragraphboundary", "documentboundary"

    eprintln!(
        "Selection: modify called with alter={}, direction={}, granularity={}",
        alter.to_std_string_escaped(),
        direction.to_std_string_escaped(),
        granularity.to_std_string_escaped()
    );

    Ok(JsValue::undefined())
}

fn remove_all_ranges(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        selection_data.remove_all_ranges();
    }

    Ok(JsValue::undefined())
}

fn remove_range(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let range = args.get_or_undefined(0).as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("First argument must be a Range object")
    })?;

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        selection_data.remove_range(&range)?;
    }

    Ok(JsValue::undefined())
}

fn select_all_children(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let node = args.get_or_undefined(0).clone();

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        selection_data.select_all_children(node);
    }

    Ok(JsValue::undefined())
}

fn set_base_and_extent(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    let anchor_node = args.get_or_undefined(0).clone();
    let anchor_offset = match args.get_or_undefined(1).to_integer_or_infinity(context)? {
        boa_engine::value::IntegerOrInfinity::Integer(i) => i.max(0) as u32,
        _ => 0,
    };
    let focus_node = args.get_or_undefined(2).clone();
    let focus_offset = match args.get_or_undefined(3).to_integer_or_infinity(context)? {
        boa_engine::value::IntegerOrInfinity::Integer(i) => i.max(0) as u32,
        _ => 0,
    };

    if let Some(mut selection_data) = this_obj.downcast_mut::<SelectionData>() {
        selection_data.set_base_and_extent(anchor_node, anchor_offset, focus_node, focus_offset);
    }

    Ok(JsValue::undefined())
}

fn to_string(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let _this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Selection method called on non-object")
    })?;

    // In a real implementation, this would return the text content of the selection
    // For a headless browser, we return an empty string
    eprintln!("Selection: toString called");
    Ok(JsValue::from(js_string!("")))
}
