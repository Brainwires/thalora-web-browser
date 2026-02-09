//! Element data types, structs, and impl blocks

use boa_engine::{
    object::JsObject,
    value::JsValue,
    Context, JsData, JsResult,
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicU32, Ordering}};
use std::sync::OnceLock;

/// Global node ID counter for unique DOM node identification
static NODE_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// Global DOM synchronization for updating document HTML content
pub static GLOBAL_DOM_SYNC: OnceLock<DomSync> = OnceLock::new();

/// Bridge between Element DOM changes and Document HTML content
pub struct DomSync {
    document_html_updater: Mutex<Option<Box<dyn Fn(&str) + Send + Sync>>>,
}

impl DomSync {
    pub fn new() -> Self {
        Self {
            document_html_updater: Mutex::new(None),
        }
    }

    pub fn set_updater(&self, updater: Box<dyn Fn(&str) + Send + Sync>) {
        *self.document_html_updater.lock().unwrap() = Some(updater);
    }

    fn update_document_html(&self, html: &str) {
        if let Some(updater) = self.document_html_updater.lock().unwrap().as_ref() {
            updater(html);
        } else {
        }
    }
}

/// Internal data for Element objects - represents a real DOM node
#[derive(Debug, Trace, Finalize, JsData)]
pub struct ElementData {
    /// Unique node ID for DOM tree operations
    #[unsafe_ignore_trace]
    node_id: u32,
    /// Element tag name (e.g., "div", "span", "body")
    #[unsafe_ignore_trace]
    tag_name: Arc<Mutex<String>>,
    /// Element namespace URI
    #[unsafe_ignore_trace]
    namespace_uri: Arc<Mutex<Option<String>>>,
    /// Element ID attribute
    #[unsafe_ignore_trace]
    id: Arc<Mutex<String>>,
    /// Element class attribute
    #[unsafe_ignore_trace]
    class_name: Arc<Mutex<String>>,
    /// Inner HTML content - parsed and maintained as real DOM
    #[unsafe_ignore_trace]
    pub(crate) inner_html: Arc<Mutex<String>>,
    /// Text content - computed from child text nodes
    #[unsafe_ignore_trace]
    text_content: Arc<Mutex<String>>,
    /// All element attributes (id, class, data-*, etc.)
    #[unsafe_ignore_trace]
    pub(crate) attributes: Arc<Mutex<HashMap<String, String>>>,
    /// Child elements in DOM tree order
    #[unsafe_ignore_trace]
    pub(crate) children: Arc<Mutex<Vec<JsObject>>>,
    /// Parent element in DOM tree
    #[unsafe_ignore_trace]
    parent_node: Arc<Mutex<Option<JsObject>>>,
    /// Computed CSS style object
    #[unsafe_ignore_trace]
    pub(crate) style: Arc<Mutex<CSSStyleDeclaration>>,
    /// Element's bounding box for layout
    #[unsafe_ignore_trace]
    bounding_rect: Arc<Mutex<DOMRect>>,
    /// Event listeners attached to this element
    #[unsafe_ignore_trace]
    event_listeners: Arc<Mutex<HashMap<String, Vec<JsValue>>>>,
    /// Shadow root for Shadow DOM API
    #[unsafe_ignore_trace]
    shadow_root: Arc<Mutex<Option<JsObject>>>,
    /// Next sibling in DOM tree
    #[unsafe_ignore_trace]
    next_sibling: Arc<Mutex<Option<JsObject>>>,
    /// Previous sibling in DOM tree
    #[unsafe_ignore_trace]
    previous_sibling: Arc<Mutex<Option<JsObject>>>,
    /// Assigned slot name for Shadow DOM slotting (internal [[AssignedSlot]])
    #[unsafe_ignore_trace]
    assigned_slot_name: Arc<Mutex<Option<String>>>,
    /// Layout dimensions (offset properties)
    #[unsafe_ignore_trace]
    offset_dimensions: Arc<Mutex<LayoutDimensions>>,
    /// Client dimensions (inner dimensions excluding borders/scrollbars)
    #[unsafe_ignore_trace]
    client_dimensions: Arc<Mutex<ClientDimensions>>,
    /// Scroll dimensions and position
    #[unsafe_ignore_trace]
    scroll_state: Arc<Mutex<ScrollState>>,
    /// Whether this element's children have been modified by JavaScript (appendChild, etc.)
    /// Used by serialize_to_html to decide whether to walk live children or use cached innerHTML
    #[unsafe_ignore_trace]
    pub(crate) modified_by_js: Arc<Mutex<bool>>,
}

/// CSS Style Declaration for real style computation
#[derive(Debug, Clone)]
pub struct CSSStyleDeclaration {
    /// CSS properties and values
    properties: HashMap<String, String>,
    /// Computed styles from cascading
    computed: HashMap<String, String>,
}

/// DOM Rectangle for element positioning
#[derive(Debug, Clone)]
pub struct DOMRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl CSSStyleDeclaration {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            computed: HashMap::new(),
        }
    }

    pub fn set_property(&mut self, property: &str, value: &str) {
        self.properties.insert(property.to_string(), value.to_string());
        self.compute_property(property, value);
    }

    pub fn get_property(&self, property: &str) -> Option<&String> {
        self.computed.get(property).or_else(|| self.properties.get(property))
    }

    /// Iterate over all computed properties
    pub fn iter_properties(&self) -> impl Iterator<Item = (&String, &String)> {
        // Combine properties and computed, with computed taking precedence
        self.properties.iter().chain(
            self.computed.iter().filter(|(k, _)| !self.properties.contains_key(*k))
        )
    }

    fn compute_property(&mut self, property: &str, value: &str) {
        // Real CSS property computation with inheritance and cascading
        match property {
            "width" | "height" => {
                // Handle different units (px, %, em, rem, vh, vw)
                let computed_value = self.compute_length_value(value);
                self.computed.insert(property.to_string(), computed_value);
            },
            "color" | "background-color" => {
                // Handle color values (hex, rgb, rgba, hsl, named colors)
                let computed_color = self.compute_color_value(value);
                self.computed.insert(property.to_string(), computed_color);
            },
            "display" => {
                // Validate display values
                let valid_display = match value {
                    "block" | "inline" | "inline-block" | "flex" | "grid" | "none" => value,
                    _ => "block" // Default fallback
                };
                self.computed.insert(property.to_string(), valid_display.to_string());
            },
            _ => {
                // Store as-is for other properties
                self.computed.insert(property.to_string(), value.to_string());
            }
        }
    }

    fn compute_length_value(&self, value: &str) -> String {
        // Parse and compute length values
        if value.ends_with("px") {
            value.to_string() // Already in pixels
        } else if value.ends_with("%") {
            // Would need parent context for percentage calculation
            value.to_string()
        } else if value.ends_with("em") {
            // Would need font-size context
            value.to_string()
        } else if let Ok(num) = value.parse::<f64>() {
            format!("{}px", num) // Treat unitless as pixels
        } else {
            value.to_string()
        }
    }

    fn compute_color_value(&self, value: &str) -> String {
        // Parse and normalize color values
        if value.starts_with("#") {
            value.to_string() // Hex color
        } else if value.starts_with("rgb") {
            value.to_string() // RGB/RGBA color
        } else {
            // Named colors or invalid - normalize to hex
            match value {
                "red" => "#ff0000".to_string(),
                "green" => "#008000".to_string(),
                "blue" => "#0000ff".to_string(),
                "black" => "#000000".to_string(),
                "white" => "#ffffff".to_string(),
                _ => value.to_string()
            }
        }
    }
}

impl DOMRect {
    pub(crate) fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }

    pub(crate) fn update_bounds(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
        self.top = y;
        self.left = x;
        self.right = x + width;
        self.bottom = y + height;
    }
}

/// Layout dimensions for offset* properties
#[derive(Debug, Clone)]
pub struct LayoutDimensions {
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub left: f64,
}

impl LayoutDimensions {
    fn new() -> Self {
        // Default to reasonable values for a typical element
        Self {
            width: 0.0,
            height: 0.0,
            top: 0.0,
            left: 0.0,
        }
    }
}

/// Client dimensions (inner area excluding borders and scrollbars)
#[derive(Debug, Clone)]
pub struct ClientDimensions {
    pub width: f64,
    pub height: f64,
    pub top: f64,  // Border width top
    pub left: f64, // Border width left
}

impl ClientDimensions {
    fn new() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            top: 0.0,
            left: 0.0,
        }
    }
}

/// Scroll state for scroll* properties
#[derive(Debug, Clone)]
pub struct ScrollState {
    pub width: f64,   // Total scrollable width
    pub height: f64,  // Total scrollable height
    pub top: f64,     // Current scroll position Y
    pub left: f64,    // Current scroll position X
}

impl ScrollState {
    fn new() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            top: 0.0,
            left: 0.0,
        }
    }
}

/// Execute a closure with `&ElementData` from a JsObject, dispatching across all element types.
/// Returns `None` if the object doesn't contain any known element data.
/// This is used internally by ElementData methods that iterate children.
fn try_with_element_data<T>(obj: &JsObject, f: impl FnOnce(&ElementData) -> T) -> Option<T> {
    use crate::dom::html_iframe_element::HTMLIFrameElementData;
    use crate::dom::html_script_element::HTMLScriptElementData;

    if let Some(el) = obj.downcast_ref::<ElementData>() {
        Some(f(&*el))
    } else if let Some(iframe) = obj.downcast_ref::<HTMLIFrameElementData>() {
        Some(f(iframe.element_data()))
    } else if let Some(script) = obj.downcast_ref::<HTMLScriptElementData>() {
        Some(f(script.element_data()))
    } else {
        None
    }
}

impl ElementData {
    pub(crate) fn new() -> Self {
        let node_id = NODE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            node_id,
            tag_name: Arc::new(Mutex::new("".to_string())),
            namespace_uri: Arc::new(Mutex::new(None)),
            id: Arc::new(Mutex::new("".to_string())),
            class_name: Arc::new(Mutex::new("".to_string())),
            inner_html: Arc::new(Mutex::new("".to_string())),
            text_content: Arc::new(Mutex::new("".to_string())),
            attributes: Arc::new(Mutex::new(HashMap::new())),
            children: Arc::new(Mutex::new(Vec::new())),
            parent_node: Arc::new(Mutex::new(None)),
            style: Arc::new(Mutex::new(CSSStyleDeclaration::new())),
            bounding_rect: Arc::new(Mutex::new(DOMRect::new())),
            event_listeners: Arc::new(Mutex::new(HashMap::new())),
            shadow_root: Arc::new(Mutex::new(None)),
            next_sibling: Arc::new(Mutex::new(None)),
            previous_sibling: Arc::new(Mutex::new(None)),
            assigned_slot_name: Arc::new(Mutex::new(None)),
            offset_dimensions: Arc::new(Mutex::new(LayoutDimensions::new())),
            client_dimensions: Arc::new(Mutex::new(ClientDimensions::new())),
            scroll_state: Arc::new(Mutex::new(ScrollState::new())),
            modified_by_js: Arc::new(Mutex::new(false)),
        }
    }

    pub fn with_tag_name(tag_name: String) -> Self {
        let mut data = Self::new();
        *data.tag_name.lock().unwrap() = tag_name;
        data
    }

    /// Check if this element or any descendant has been modified by JavaScript
    pub fn has_modified_descendants(&self) -> bool {
        if *self.modified_by_js.lock().unwrap() {
            return true;
        }
        let children = self.children.lock().unwrap();
        children.iter().any(|child| {
            try_with_element_data(child, |cd| cd.has_modified_descendants()).unwrap_or(false)
        })
    }

    /// Mark this element as modified by JavaScript
    pub fn mark_modified_by_js(&self) {
        *self.modified_by_js.lock().unwrap() = true;
    }

    pub fn get_tag_name(&self) -> String {
        self.tag_name.lock().unwrap().clone()
    }

    pub fn set_tag_name(&self, tag_name: String) {
        *self.tag_name.lock().unwrap() = tag_name;
    }

    pub fn get_namespace_uri(&self) -> Option<String> {
        self.namespace_uri.lock().unwrap().clone()
    }

    pub fn set_namespace_uri(&self, namespace_uri: Option<String>) {
        *self.namespace_uri.lock().unwrap() = namespace_uri;
    }

    pub fn get_id(&self) -> String {
        self.id.lock().unwrap().clone()
    }

    pub fn set_id(&self, id: String) {
        *self.id.lock().unwrap() = id;
    }

    pub fn get_class_name(&self) -> String {
        self.class_name.lock().unwrap().clone()
    }

    pub fn set_class_name(&self, class_name: String) {
        *self.class_name.lock().unwrap() = class_name;
    }

    pub fn get_inner_html(&self) -> String {
        self.inner_html.lock().unwrap().clone()
    }

    /// Serialize the element's inner content to HTML.
    /// When the element or any descendant has been modified by JS, walks live
    /// children to capture dynamically created nodes. Otherwise returns the
    /// cached `inner_html` string.
    pub fn serialize_inner_html(&self) -> String {
        let is_modified = *self.modified_by_js.lock().unwrap();
        let needs_walk = is_modified || self.has_modified_descendants();

        let children = self.children.lock().unwrap();
        let tag = self.get_tag_name();

        // Debug: log key elements during serialization
        let id_str = self.get_id();
        let is_key_element = matches!(tag.to_lowercase().as_str(), "html" | "body" | "main" | "div" | "section")
            && (!id_str.is_empty() || tag.eq_ignore_ascii_case("html") || tag.eq_ignore_ascii_case("body"));
        let id_suffix = if id_str.is_empty() { String::new() } else { format!(" id=\"{}\"", id_str) };
        if is_key_element {
            eprintln!("DEBUG SERIALIZE: <{}{}> children={}, modified={}, needs_walk={}, inner_html_len={}",
                tag, id_suffix, children.len(), is_modified, needs_walk, self.get_inner_html().len(),
            );
        }

        if children.is_empty() {
            // Leaf node: use cached innerHTML (text content, or set via innerHTML)
            let inner = self.get_inner_html();
            if inner.is_empty() {
                self.get_text_content()
            } else {
                inner
            }
        } else if needs_walk {
            let mut html = String::new();
            for child in children.iter() {
                if let Some(child_html) = try_with_element_data(child, |child_data| {
                    child_data.serialize_to_html()
                }) {
                    html.push_str(&child_html);
                } else if let Some(text_data) = child.downcast_ref::<crate::dom::text::TextData>() {
                    html.push_str(&text_data.character_data().get_data());
                } else if let Some(comment_data) = child.downcast_ref::<crate::dom::comment::CommentData>() {
                    html.push_str(&format!("<!--{}-->", comment_data.character_data().get_data()));
                }
            }
            if is_key_element {
                eprintln!("DEBUG SERIALIZE: <{}{}> walked {} children, produced {} bytes",
                    tag, id_suffix, children.len(), html.len());
            }
            html
        } else {
            // Unmodified subtree: use cached innerHTML
            if is_key_element {
                eprintln!("DEBUG SERIALIZE: <{}{}> using CACHED innerHTML ({} bytes)",
                    tag, id_suffix, self.get_inner_html().len());
            }
            self.get_inner_html()
        }
    }

    pub fn set_inner_html(&self, html: String) {
        *self.inner_html.lock().unwrap() = html.clone();

        // Parse HTML and update DOM tree
        self.parse_and_update_children(&html);

        // Recompute text content from parsed children
        self.recompute_text_content();

        // CRITICAL: Update the document's HTML content so querySelector can find changes
        self.update_document_html_content();
    }

    /// Parse HTML string and create child elements
    fn parse_and_update_children(&self, html: &str) {
        let mut children = self.children.lock().unwrap();
        children.clear();

        // Simple HTML parser - in real implementation would use proper HTML5 parser
        let parsed_elements = self.simple_html_parse(html);
        children.extend(parsed_elements);
    }

    /// Simple HTML parser for basic tag parsing
    fn simple_html_parse(&self, html: &str) -> Vec<JsObject> {
        let mut elements = Vec::new();
        let mut current_pos = 0;
        let html_bytes = html.as_bytes();

        while current_pos < html_bytes.len() {
            if html_bytes[current_pos] == b'<' {
                // Find end of tag
                if let Some(tag_end) = html[current_pos..].find('>') {
                    let tag_content = &html[current_pos + 1..current_pos + tag_end];

                    if !tag_content.starts_with('/') { // Not a closing tag
                        // Parse opening tag
                        let parts: Vec<&str> = tag_content.split_whitespace().collect();
                        if let Some(tag_name) = parts.first() {
                            // Create new element
                            let element_data = ElementData::with_tag_name(tag_name.to_uppercase());

                            // Parse attributes
                            for attr_part in parts.iter().skip(1) {
                                if let Some(eq_pos) = attr_part.find('=') {
                                    let attr_name = &attr_part[..eq_pos];
                                    let attr_value = &attr_part[eq_pos + 1..].trim_matches('"');
                                    element_data.set_attribute(attr_name.to_string(), attr_value.to_string());
                                }
                            }

                            // Create JsObject for the element
                            let element = JsObject::from_proto_and_data(None, element_data);
                            elements.push(element);
                        }
                    }

                    current_pos += tag_end + 1;
                } else {
                    current_pos += 1;
                }
            } else {
                // Text content - find next tag or end
                let text_start = current_pos;
                let text_end = html[current_pos..].find('<').map(|pos| current_pos + pos).unwrap_or(html.len());

                let text_content = html[text_start..text_end].trim();
                if !text_content.is_empty() {
                    // Create text node as element with special tag
                    let text_element = ElementData::with_tag_name("#text".to_string());
                    text_element.set_text_content(text_content.to_string());

                    let text_obj = JsObject::from_proto_and_data(None, text_element);
                    elements.push(text_obj);
                }

                current_pos = text_end;
            }
        }

        elements
    }

    /// Set innerHTML with context - needed for proper iframe creation
    /// When iframes are parsed via innerHTML, they need a Context to initialize
    /// their contentWindow and contentDocument properly
    pub fn set_inner_html_with_context(&self, html: String, context: &mut Context) -> JsResult<()> {
        *self.inner_html.lock().unwrap() = html.clone();

        // Parse HTML and update DOM tree with context awareness for iframes
        self.parse_and_update_children_with_context(&html, context)?;

        // Recompute text content from parsed children
        self.recompute_text_content();

        // CRITICAL: Update the document's HTML content so querySelector can find changes
        self.update_document_html_content();

        Ok(())
    }

    /// Parse HTML string and create child elements with context (for iframe support)
    fn parse_and_update_children_with_context(&self, html: &str, context: &mut Context) -> JsResult<()> {
        let parsed_elements = super::dom_manipulation::parse_html_elements_with_context(html, context)?;

        let mut children = self.children.lock().unwrap();
        children.clear();
        children.extend(parsed_elements);

        Ok(())
    }

    /// Recompute text content from all child text nodes
    pub(super) fn recompute_text_content(&self) {
        let children = self.children.lock().unwrap();
        let mut text_parts = Vec::new();

        for child in children.iter() {
            if let Some(text) = try_with_element_data(child, |child_data| {
                child_data.get_text_content()
            }) {
                text_parts.push(text);
            }
        }

        *self.text_content.lock().unwrap() = text_parts.join("");
    }

    /// Update the document's HTML content to reflect DOM changes.
    /// This is CRITICAL for querySelector to find dynamically added content.
    ///
    /// Note: We intentionally do NOT rebuild the full document HTML here.
    /// querySelector works on the original HTML + in-memory element state.
    /// The live DOM is captured via `document.documentElement.outerHTML` after
    /// JS execution completes (see navigate_to_with_js_option).
    pub(super) fn update_document_html_content(&self) {
        let _dom_sync = GLOBAL_DOM_SYNC.get_or_init(|| DomSync::new());
        // Signal that DOM has been modified without corrupting the full HTML.
        // Individual element changes are tracked in-memory and serialized on demand
        // via serialize_to_html() which walks the live children vector.
    }

    /// Serialize this element and all children to HTML string.
    /// Walks the live `children` vector to capture dynamically appended nodes
    /// (e.g. via `appendChild`). Falls back to cached `inner_html` only when
    /// the children vector is empty (text-only content or innerHTML-set leaves).
    pub(super) fn serialize_to_html(&self) -> String {
        let tag_name = self.get_tag_name();

        // Text nodes serialize as just their text content
        if tag_name == "#text" {
            return self.get_text_content();
        }

        let mut html = format!("<{}", tag_name);

        // Add attributes
        let attributes = self.attributes.lock().unwrap();
        for (name, value) in attributes.iter() {
            html.push_str(&format!(" {}=\"{}\"", name, value));
        }
        html.push('>');

        html.push_str(&self.serialize_inner_html());

        // Void elements don't get closing tags
        let tag_lower = tag_name.to_lowercase();
        if !matches!(tag_lower.as_str(), "area" | "base" | "br" | "col" | "embed" |
            "hr" | "img" | "input" | "link" | "meta" | "param" | "source" |
            "track" | "wbr") {
            html.push_str(&format!("</{}>", tag_name));
        }

        html
    }

    pub fn get_text_content(&self) -> String {
        self.text_content.lock().unwrap().clone()
    }

    pub fn set_text_content(&self, content: String) {
        // Per WHATWG spec: setting textContent must remove all child nodes first
        self.children.lock().unwrap().clear();
        *self.inner_html.lock().unwrap() = String::new();
        *self.text_content.lock().unwrap() = content;
        self.mark_modified_by_js();
    }

    pub fn get_attribute(&self, name: &str) -> Option<String> {
        self.attributes.lock().unwrap().get(name).cloned()
    }

    pub fn set_attribute(&self, name: String, value: String) {
        // Sync certain attributes with their corresponding DOM properties
        match name.as_str() {
            "class" => {
                // "class" attribute syncs with className property
                *self.class_name.lock().unwrap() = value.clone();
            }
            "id" => {
                // "id" attribute syncs with id property
                *self.id.lock().unwrap() = value.clone();
            }
            _ => {}
        }
        self.attributes.lock().unwrap().insert(name, value);
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.lock().unwrap().contains_key(name)
    }

    pub fn remove_attribute(&self, name: &str) {
        self.attributes.lock().unwrap().remove(name);
    }

    pub fn append_child(&self, child: JsObject) {
        self.children.lock().unwrap().push(child);
    }

    /// Insert a child at the beginning (for prepend)
    pub fn prepend_child(&self, child: JsObject) {
        self.children.lock().unwrap().insert(0, child);
    }

    /// Insert a node after a reference node
    pub fn insert_after(&self, new_child: JsObject, reference: &JsObject) {
        let mut children = self.children.lock().unwrap();
        if let Some(idx) = children.iter().position(|c| JsObject::equals(c, reference)) {
            children.insert(idx + 1, new_child);
        } else {
            // If reference not found, append at the end
            children.push(new_child);
        }
    }

    /// Insert a node before a reference node (for before())
    pub fn insert_before_elem(&self, new_child: JsObject, reference: &JsObject) {
        let mut children = self.children.lock().unwrap();
        if let Some(idx) = children.iter().position(|c| JsObject::equals(c, reference)) {
            children.insert(idx, new_child);
        } else {
            // If reference not found, append at the end
            children.push(new_child);
        }
    }

    /// Clear all children (for replaceChildren)
    pub fn clear_children(&self) {
        self.children.lock().unwrap().clear();
    }

    pub fn remove_child(&self, child: &JsObject) {
        self.children.lock().unwrap().retain(|c| !JsObject::equals(c, child));
    }

    pub fn get_children(&self) -> Vec<JsObject> {
        self.children.lock().unwrap().clone()
    }

    pub fn get_parent_node(&self) -> Option<JsObject> {
        self.parent_node.lock().unwrap().clone()
    }

    pub fn set_parent_node(&self, parent: Option<JsObject>) {
        *self.parent_node.lock().unwrap() = parent;
    }

    pub fn get_next_sibling(&self) -> Option<JsObject> {
        self.next_sibling.lock().unwrap().clone()
    }

    pub fn set_next_sibling(&self, sibling: Option<JsObject>) {
        *self.next_sibling.lock().unwrap() = sibling;
    }

    pub fn get_previous_sibling(&self) -> Option<JsObject> {
        self.previous_sibling.lock().unwrap().clone()
    }

    pub fn set_previous_sibling(&self, sibling: Option<JsObject>) {
        *self.previous_sibling.lock().unwrap() = sibling;
    }

    pub fn get_first_child(&self) -> Option<JsObject> {
        self.children.lock().unwrap().first().cloned()
    }

    pub fn get_last_child(&self) -> Option<JsObject> {
        self.children.lock().unwrap().last().cloned()
    }

    /// Insert a child before a reference node
    pub fn insert_before(&self, new_child: JsObject, reference_child: Option<&JsObject>) -> Option<JsObject> {
        let mut children = self.children.lock().unwrap();

        if let Some(ref_child) = reference_child {
            // Find the reference child index
            if let Some(index) = children.iter().position(|c| JsObject::equals(c, ref_child)) {
                // Set up sibling relationships
                try_with_element_data(&new_child, |new_child_data| {
                    // Set previous sibling
                    if index > 0 {
                        new_child_data.set_previous_sibling(Some(children[index - 1].clone()));
                        try_with_element_data(&children[index - 1], |prev_data| {
                            prev_data.set_next_sibling(Some(new_child.clone()));
                        });
                    }
                    // Set next sibling (the reference child)
                    new_child_data.set_next_sibling(Some(ref_child.clone()));
                });
                try_with_element_data(ref_child, |ref_data| {
                    ref_data.set_previous_sibling(Some(new_child.clone()));
                });

                children.insert(index, new_child.clone());
                return Some(new_child);
            }
        }

        // If no reference child or not found, append to end
        children.push(new_child.clone());
        Some(new_child)
    }

    /// Replace an old child with a new child
    pub fn replace_child(&self, new_child: JsObject, old_child: &JsObject) -> Option<JsObject> {
        let mut children = self.children.lock().unwrap();

        if let Some(index) = children.iter().position(|c| JsObject::equals(c, old_child)) {
            // Copy sibling relationships from old to new
            let old_siblings = try_with_element_data(old_child, |old_data| {
                let prev = old_data.get_previous_sibling();
                let next = old_data.get_next_sibling();
                // Clear old child's relationships
                old_data.set_parent_node(None);
                old_data.set_previous_sibling(None);
                old_data.set_next_sibling(None);
                (prev, next)
            });
            if let Some((prev, next)) = old_siblings {
                try_with_element_data(&new_child, |new_data| {
                    new_data.set_previous_sibling(prev);
                    new_data.set_next_sibling(next);
                });
            }

            // Update siblings to point to new child
            if index > 0 {
                try_with_element_data(&children[index - 1], |prev_data| {
                    prev_data.set_next_sibling(Some(new_child.clone()));
                });
            }
            if index < children.len() - 1 {
                try_with_element_data(&children[index + 1], |next_data| {
                    next_data.set_previous_sibling(Some(new_child.clone()));
                });
            }

            children[index] = new_child;
            return Some(old_child.clone());
        }

        None
    }

    /// Check if this element contains another node
    pub fn contains_node(&self, other: &JsObject) -> bool {
        // Check children recursively
        let children = self.children.lock().unwrap();
        for child in children.iter() {
            if JsObject::equals(child, other) {
                return true;
            }
            if let Some(true) = try_with_element_data(child, |child_data| {
                child_data.contains_node(other)
            }) {
                return true;
            }
        }
        false
    }

    /// Find closest ancestor matching selector (including self)
    pub fn find_closest(&self, selector: &str, self_obj: &JsObject) -> Option<JsObject> {
        // Check if this element matches
        if self.matches_selector(selector) {
            return Some(self_obj.clone());
        }

        // Check parent
        if let Some(parent) = self.get_parent_node() {
            if let Some(result) = try_with_element_data(&parent, |parent_data| {
                parent_data.find_closest(selector, &parent)
            }) {
                return result;
            }
        }

        None
    }

    /// Clone this element (optionally deep)
    pub fn clone_element(&self, deep: bool, context: &mut Context) -> JsResult<JsObject> {
        let new_data = ElementData::new();

        // Copy basic properties
        new_data.set_tag_name(self.get_tag_name());
        new_data.set_id(self.get_id());
        new_data.set_class_name(self.get_class_name());

        // Copy attributes
        {
            let src_attrs = self.attributes.lock().unwrap();
            let mut dst_attrs = new_data.attributes.lock().unwrap();
            for (k, v) in src_attrs.iter() {
                dst_attrs.insert(k.clone(), v.clone());
            }
        }

        // Copy innerHTML only for shallow clone
        if !deep {
            new_data.set_inner_html(self.get_inner_html());
        }

        // Create JsObject with HTMLElement prototype (ensures instanceof HTMLElement works)
        let prototype = context.intrinsics().constructors().html_element().prototype();
        let cloned = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            new_data,
        );

        // Deep clone: recursively clone children
        // Note: We collect children first to avoid holding locks across clone_element calls
        // which need &mut Context.
        if deep {
            let child_list: Vec<JsObject> = self.children.lock().unwrap().clone();
            for child in child_list.iter() {
                // We need to dispatch across element types but can't use try_with_element_data
                // here because clone_element requires &mut Context which can't be in the closure.
                // Instead, extract ElementData reference, call clone_element, then release.
                use crate::dom::html_iframe_element::HTMLIFrameElementData;
                use crate::dom::html_script_element::HTMLScriptElementData;

                let cloned_child = if let Some(child_data) = child.downcast_ref::<ElementData>() {
                    Some(child_data.clone_element(true, context)?)
                } else if let Some(iframe) = child.downcast_ref::<HTMLIFrameElementData>() {
                    Some(iframe.element_data().clone_element(true, context)?)
                } else if let Some(script) = child.downcast_ref::<HTMLScriptElementData>() {
                    Some(script.element_data().clone_element(true, context)?)
                } else if let Some(text_data) = child.downcast_ref::<crate::dom::text::TextData>() {
                    // Clone text node
                    let cloned_text = crate::dom::text::TextData::new(text_data.character_data().get_data());
                    let text_obj = JsObject::from_proto_and_data_with_shared_shape(
                        context.root_shape(),
                        context.intrinsics().constructors().text().prototype(),
                        cloned_text,
                    );
                    Some(text_obj.upcast())
                } else if let Some(comment_data) = child.downcast_ref::<crate::dom::comment::CommentData>() {
                    // Clone comment node
                    let cloned_comment = crate::dom::comment::CommentData::new(comment_data.character_data().get_data());
                    let comment_obj = JsObject::from_proto_and_data_with_shared_shape(
                        context.root_shape(),
                        context.intrinsics().constructors().comment().prototype(),
                        cloned_comment,
                    );
                    Some(comment_obj.upcast())
                } else {
                    None
                };

                if let Some(cloned_child) = cloned_child {
                    if let Ok(cloned_data) = cloned.try_borrow() {
                        cloned_data.data().append_child(cloned_child);
                    }
                }
            }
        }

        Ok(cloned.upcast())
    }

    pub fn get_style(&self) -> CSSStyleDeclaration {
        self.style.lock().unwrap().clone()
    }

    /// Add event listener to this element
    pub fn add_event_listener(&self, event_type: String, listener: JsValue) {
        self.event_listeners.lock().unwrap()
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(listener);
    }

    /// Remove event listener from this element
    pub fn remove_event_listener(&self, event_type: &str, listener: &JsValue) {
        if let Some(listeners) = self.event_listeners.lock().unwrap().get_mut(event_type) {
            listeners.retain(|l| !JsValue::same_value(l, listener));
        }
    }

    /// Get event listeners for a specific event type
    pub fn get_event_listeners(&self, event_type: &str) -> Option<Vec<JsValue>> {
        self.event_listeners.lock().unwrap()
            .get(event_type)
            .cloned()
    }

    /// Attach shadow root for Shadow DOM API
    pub fn attach_shadow_root(&self, shadow_root: JsObject) {
        if let Ok(mut guard) = self.shadow_root.try_lock() {
            *guard = Some(shadow_root);
        } else {
            eprintln!("WARNING: Shadow DOM mutex was locked, skipping shadow root attachment");
        }
    }

    /// Get shadow root (returns None if no shadow root or mode is 'closed')
    pub fn get_shadow_root(&self) -> Option<JsObject> {
        if let Ok(guard) = self.shadow_root.try_lock() {
            guard.clone()
        } else {
            eprintln!("WARNING: Shadow DOM mutex was locked, returning None");
            None
        }
    }

    /// Get assigned slot name for Shadow DOM slotting
    pub fn get_assigned_slot_name(&self) -> Option<String> {
        self.assigned_slot_name.lock().unwrap().clone()
    }

    /// Set assigned slot name for Shadow DOM slotting (internal [[AssignedSlot]])
    pub fn set_assigned_slot_name(&self, slot_name: String) {
        *self.assigned_slot_name.lock().unwrap() = Some(slot_name);
    }

    /// Clear assigned slot (when removed from slotting)
    pub fn clear_assigned_slot(&self) {
        *self.assigned_slot_name.lock().unwrap() = None;
    }

    // =========================================================================
    // Layout dimension getters (offset* properties)
    // =========================================================================

    /// Get offsetWidth - element width including borders
    pub fn get_offset_width(&self) -> f64 {
        self.offset_dimensions.lock().unwrap().width
    }

    /// Get offsetHeight - element height including borders
    pub fn get_offset_height(&self) -> f64 {
        self.offset_dimensions.lock().unwrap().height
    }

    /// Get offsetTop - top position relative to offsetParent
    pub fn get_offset_top(&self) -> f64 {
        self.offset_dimensions.lock().unwrap().top
    }

    /// Get offsetLeft - left position relative to offsetParent
    pub fn get_offset_left(&self) -> f64 {
        self.offset_dimensions.lock().unwrap().left
    }

    /// Set offset dimensions (used internally when layout is computed)
    pub fn set_offset_dimensions(&self, width: f64, height: f64, top: f64, left: f64) {
        let mut dims = self.offset_dimensions.lock().unwrap();
        dims.width = width;
        dims.height = height;
        dims.top = top;
        dims.left = left;
    }

    // =========================================================================
    // Client dimension getters (client* properties)
    // =========================================================================

    /// Get clientWidth - inner width excluding borders and scrollbars
    pub fn get_client_width(&self) -> f64 {
        self.client_dimensions.lock().unwrap().width
    }

    /// Get clientHeight - inner height excluding borders and scrollbars
    pub fn get_client_height(&self) -> f64 {
        self.client_dimensions.lock().unwrap().height
    }

    /// Get clientTop - top border width
    pub fn get_client_top(&self) -> f64 {
        self.client_dimensions.lock().unwrap().top
    }

    /// Get clientLeft - left border width
    pub fn get_client_left(&self) -> f64 {
        self.client_dimensions.lock().unwrap().left
    }

    /// Set client dimensions (used internally when layout is computed)
    pub fn set_client_dimensions(&self, width: f64, height: f64, top: f64, left: f64) {
        let mut dims = self.client_dimensions.lock().unwrap();
        dims.width = width;
        dims.height = height;
        dims.top = top;
        dims.left = left;
    }

    // =========================================================================
    // Scroll dimension and position getters/setters
    // =========================================================================

    /// Get scrollWidth - total width of content including overflow
    pub fn get_scroll_width(&self) -> f64 {
        self.scroll_state.lock().unwrap().width
    }

    /// Get scrollHeight - total height of content including overflow
    pub fn get_scroll_height(&self) -> f64 {
        self.scroll_state.lock().unwrap().height
    }

    /// Get scrollTop - current vertical scroll position
    pub fn get_scroll_top(&self) -> f64 {
        self.scroll_state.lock().unwrap().top
    }

    /// Set scrollTop - set vertical scroll position
    pub fn set_scroll_top(&self, value: f64) {
        let mut scroll = self.scroll_state.lock().unwrap();
        // Clamp to valid range
        let max_scroll = (scroll.height - self.get_client_height()).max(0.0);
        scroll.top = value.max(0.0).min(max_scroll);
    }

    /// Get scrollLeft - current horizontal scroll position
    pub fn get_scroll_left(&self) -> f64 {
        self.scroll_state.lock().unwrap().left
    }

    /// Set scrollLeft - set horizontal scroll position
    pub fn set_scroll_left(&self, value: f64) {
        let mut scroll = self.scroll_state.lock().unwrap();
        // Clamp to valid range
        let max_scroll = (scroll.width - self.get_client_width()).max(0.0);
        scroll.left = value.max(0.0).min(max_scroll);
    }

    /// Set scroll dimensions (used internally when content size is computed)
    pub fn set_scroll_dimensions(&self, width: f64, height: f64) {
        let mut scroll = self.scroll_state.lock().unwrap();
        scroll.width = width;
        scroll.height = height;
    }

    /// Dispatch event on this element
    pub fn dispatch_event(&self, event_type: &str, event_data: &JsValue, context: &mut Context) -> JsResult<()> {
        let listeners = self.event_listeners.lock().unwrap();
        if let Some(event_listeners) = listeners.get(event_type) {
            for listener in event_listeners {
                if listener.is_callable() {
                    let _ = listener.as_callable().unwrap().call(
                        &JsValue::undefined(),
                        &[event_data.clone()],
                        context,
                    );
                }
            }
        }
        Ok(())
    }

    /// Update CSS style property with real computation
    pub fn set_style_property(&self, property: &str, value: &str) {
        let mut style = self.style.lock().unwrap();
        style.set_property(property, value);

        // Update layout if this affects positioning
        if matches!(property, "width" | "height" | "position" | "left" | "top") {
            self.recompute_layout();
        }
    }

    /// Get CSS style property value
    pub fn get_style_property(&self, property: &str) -> Option<String> {
        let style = self.style.lock().unwrap();
        style.get_property(property).cloned()
    }

    /// Recompute element layout and bounding box
    fn recompute_layout(&self) {
        let style = self.style.lock().unwrap();
        let mut rect = self.bounding_rect.lock().unwrap();

        // Get computed dimensions
        let width = style.get_property("width")
            .and_then(|w| self.parse_length_value(w))
            .unwrap_or(0.0);

        let height = style.get_property("height")
            .and_then(|h| self.parse_length_value(h))
            .unwrap_or(0.0);

        let left = style.get_property("left")
            .and_then(|l| self.parse_length_value(l))
            .unwrap_or(0.0);

        let top = style.get_property("top")
            .and_then(|t| self.parse_length_value(t))
            .unwrap_or(0.0);

        // Update bounding rectangle
        rect.update_bounds(left, top, width, height);
    }

    /// Parse length value to pixels
    fn parse_length_value(&self, value: &str) -> Option<f64> {
        if let Some(px_value) = value.strip_suffix("px") {
            px_value.parse().ok()
        } else if let Ok(num) = value.parse::<f64>() {
            Some(num) // Treat unitless as pixels
        } else {
            None
        }
    }

    /// Get bounding client rectangle
    /// First checks the global layout registry for computed positions,
    /// then falls back to the stored bounding_rect
    pub fn get_bounding_client_rect(&self) -> DOMRect {
        // Try to get computed layout from the registry
        let element_id = self.get_element_identifier();
        let tag = self.tag_name.lock().unwrap().to_lowercase();

        // Query the layout registry
        let layout = crate::layout_registry::get_element_layout(&element_id, &tag);

        // If we got a non-zero layout from the registry, use it
        if layout.width > 0.0 || layout.height > 0.0 {
            return DOMRect {
                x: layout.x,
                y: layout.y,
                width: layout.width,
                height: layout.height,
                top: layout.top,
                right: layout.right,
                bottom: layout.bottom,
                left: layout.left,
            };
        }

        // Fall back to stored bounding_rect
        self.bounding_rect.lock().unwrap().clone()
    }

    /// Get a unique identifier for this element for layout registry lookups
    pub(super) fn get_element_identifier(&self) -> String {
        // Prefer ID attribute
        let id = self.get_id();
        if !id.is_empty() {
            return format!("#{}", id);
        }

        // Use tag + first class
        let tag = self.tag_name.lock().unwrap().to_lowercase();
        let class_list = self.class_name.lock().unwrap();
        if !class_list.is_empty() {
            let first_class = class_list.split_whitespace().next().unwrap_or("");
            if !first_class.is_empty() {
                return format!("{}.{}", tag, first_class);
            }
        }

        tag
    }

    /// Real DOM tree traversal - get element by ID
    pub fn find_element_by_id(&self, self_obj: &JsObject, id: &str) -> Option<JsObject> {
        // Check this element
        if self.get_id() == id {
            return Some(self_obj.clone());
        }

        // Recursively search children
        let children = self.children.lock().unwrap();
        for child in children.iter() {
            // Recurse into child (child checks itself first)
            if let Some(found) = try_with_element_data(child, |child_data| {
                child_data.find_element_by_id(child, id)
            }).flatten() {
                return Some(found);
            }
        }

        None
    }

    /// CSS selector matching
    pub fn matches_selector(&self, selector: &str) -> bool {
        // Simple selector matching - real implementation would use CSS parser
        if selector.starts_with('#') {
            // ID selector
            let id = &selector[1..];
            return self.get_id() == id;
        } else if selector.starts_with('.') {
            // Class selector
            let class = &selector[1..];
            return self.get_class_name().split_whitespace().any(|c| c == class);
        } else {
            // Tag selector
            return self.get_tag_name().to_lowercase() == selector.to_lowercase();
        }
    }

    /// Query selector implementation
    pub fn query_selector(&self, self_obj: &JsObject, selector: &str) -> Option<JsObject> {
        // Check this element
        if self.matches_selector(selector) {
            return Some(self_obj.clone());
        }

        // Recursively search children
        let children = self.children.lock().unwrap();
        for child in children.iter() {
            // Recurse into child (child checks itself first)
            if let Some(found) = try_with_element_data(child, |child_data| {
                child_data.query_selector(child, selector)
            }).flatten() {
                return Some(found);
            }
        }

        None
    }

    /// Query all elements matching selector
    pub fn query_selector_all(&self, self_obj: &JsObject, selector: &str) -> Vec<JsObject> {
        let mut results = Vec::new();

        // Check this element
        if self.matches_selector(selector) {
            results.push(self_obj.clone());
        }

        // Recursively search children
        let children = self.children.lock().unwrap();
        for child in children.iter() {
            // Recurse into child (child checks itself first)
            if let Some(deeper) = try_with_element_data(child, |child_data| {
                child_data.query_selector_all(child, selector)
            }) {
                results.extend(deeper);
            }
        }

        results
    }
}
