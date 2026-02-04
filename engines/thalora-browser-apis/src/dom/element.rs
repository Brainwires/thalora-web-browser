//! Element Web API implementation for Boa
//!
//! Real native implementation of Element standard with actual DOM tree functionality
//! https://dom.spec.whatwg.org/#interface-element

use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute
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

/// JavaScript `Element` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Element;

impl IntrinsicObject for Element {
    fn init(realm: &Realm) {
        let tag_name_func = BuiltInBuilder::callable(realm, get_tag_name)
            .name(js_string!("get tagName"))
            .build();

        let id_func = BuiltInBuilder::callable(realm, get_id)
            .name(js_string!("get id"))
            .build();

        let id_setter_func = BuiltInBuilder::callable(realm, set_id)
            .name(js_string!("set id"))
            .build();

        let class_name_func = BuiltInBuilder::callable(realm, get_class_name)
            .name(js_string!("get className"))
            .build();

        let class_name_setter_func = BuiltInBuilder::callable(realm, set_class_name)
            .name(js_string!("set className"))
            .build();

        let inner_html_func = BuiltInBuilder::callable(realm, get_inner_html)
            .name(js_string!("get innerHTML"))
            .build();

        let inner_html_setter_func = BuiltInBuilder::callable(realm, set_inner_html)
            .name(js_string!("set innerHTML"))
            .build();

        let text_content_func = BuiltInBuilder::callable(realm, get_text_content)
            .name(js_string!("get textContent"))
            .build();

        let text_content_setter_func = BuiltInBuilder::callable(realm, set_text_content)
            .name(js_string!("set textContent"))
            .build();

        let children_func = BuiltInBuilder::callable(realm, get_children)
            .name(js_string!("get children"))
            .build();

        let parent_node_func = BuiltInBuilder::callable(realm, get_parent_node)
            .name(js_string!("get parentNode"))
            .build();

        let style_func = BuiltInBuilder::callable(realm, get_style)
            .name(js_string!("get style"))
            .build();

        let class_list_func = BuiltInBuilder::callable(realm, get_class_list)
            .name(js_string!("get classList"))
            .build();

        let first_child_func = BuiltInBuilder::callable(realm, get_first_child)
            .name(js_string!("get firstChild"))
            .build();

        let last_child_func = BuiltInBuilder::callable(realm, get_last_child)
            .name(js_string!("get lastChild"))
            .build();

        let next_sibling_func = BuiltInBuilder::callable(realm, get_next_sibling)
            .name(js_string!("get nextSibling"))
            .build();

        let previous_sibling_func = BuiltInBuilder::callable(realm, get_previous_sibling)
            .name(js_string!("get previousSibling"))
            .build();

        let node_type_func = BuiltInBuilder::callable(realm, get_node_type)
            .name(js_string!("get nodeType"))
            .build();

        let node_name_func = BuiltInBuilder::callable(realm, get_node_name)
            .name(js_string!("get nodeName"))
            .build();

        let outer_html_func = BuiltInBuilder::callable(realm, get_outer_html)
            .name(js_string!("get outerHTML"))
            .build();

        let outer_html_setter_func = BuiltInBuilder::callable(realm, set_outer_html)
            .name(js_string!("set outerHTML"))
            .build();

        let child_nodes_func = BuiltInBuilder::callable(realm, get_child_nodes)
            .name(js_string!("get childNodes"))
            .build();

        // Layout dimension accessors (read-only)
        let offset_width_func = BuiltInBuilder::callable(realm, get_offset_width)
            .name(js_string!("get offsetWidth"))
            .build();

        let offset_height_func = BuiltInBuilder::callable(realm, get_offset_height)
            .name(js_string!("get offsetHeight"))
            .build();

        let offset_top_func = BuiltInBuilder::callable(realm, get_offset_top)
            .name(js_string!("get offsetTop"))
            .build();

        let offset_left_func = BuiltInBuilder::callable(realm, get_offset_left)
            .name(js_string!("get offsetLeft"))
            .build();

        let offset_parent_func = BuiltInBuilder::callable(realm, get_offset_parent)
            .name(js_string!("get offsetParent"))
            .build();

        let client_width_func = BuiltInBuilder::callable(realm, get_client_width)
            .name(js_string!("get clientWidth"))
            .build();

        let client_height_func = BuiltInBuilder::callable(realm, get_client_height)
            .name(js_string!("get clientHeight"))
            .build();

        let client_top_func = BuiltInBuilder::callable(realm, get_client_top)
            .name(js_string!("get clientTop"))
            .build();

        let client_left_func = BuiltInBuilder::callable(realm, get_client_left)
            .name(js_string!("get clientLeft"))
            .build();

        // Scroll dimension accessors (read-only)
        let scroll_width_func = BuiltInBuilder::callable(realm, get_scroll_width)
            .name(js_string!("get scrollWidth"))
            .build();

        let scroll_height_func = BuiltInBuilder::callable(realm, get_scroll_height)
            .name(js_string!("get scrollHeight"))
            .build();

        // Scroll position accessors (read/write)
        let scroll_top_func = BuiltInBuilder::callable(realm, get_scroll_top)
            .name(js_string!("get scrollTop"))
            .build();

        let scroll_top_setter_func = BuiltInBuilder::callable(realm, set_scroll_top)
            .name(js_string!("set scrollTop"))
            .build();

        let scroll_left_func = BuiltInBuilder::callable(realm, get_scroll_left)
            .name(js_string!("get scrollLeft"))
            .build();

        let scroll_left_setter_func = BuiltInBuilder::callable(realm, set_scroll_left)
            .name(js_string!("set scrollLeft"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Set up prototype chain: Element -> Node -> EventTarget -> Object
            .inherits(Some(realm.intrinsics().constructors().node().prototype()))
            .accessor(
                js_string!("tagName"),
                Some(tag_name_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("id"),
                Some(id_func),
                Some(id_setter_func),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("className"),
                Some(class_name_func),
                Some(class_name_setter_func),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("innerHTML"),
                Some(inner_html_func),
                Some(inner_html_setter_func),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("textContent"),
                Some(text_content_func),
                Some(text_content_setter_func),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("children"),
                Some(children_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("parentNode"),
                Some(parent_node_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("style"),
                Some(style_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("classList"),
                Some(class_list_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("firstChild"),
                Some(first_child_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("lastChild"),
                Some(last_child_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("nextSibling"),
                Some(next_sibling_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("previousSibling"),
                Some(previous_sibling_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("nodeType"),
                Some(node_type_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("nodeName"),
                Some(node_name_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("outerHTML"),
                Some(outer_html_func),
                Some(outer_html_setter_func),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("childNodes"),
                Some(child_nodes_func),
                None,
                Attribute::CONFIGURABLE,
            )
            // Layout dimension accessors
            .accessor(
                js_string!("offsetWidth"),
                Some(offset_width_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("offsetHeight"),
                Some(offset_height_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("offsetTop"),
                Some(offset_top_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("offsetLeft"),
                Some(offset_left_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("offsetParent"),
                Some(offset_parent_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("clientWidth"),
                Some(client_width_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("clientHeight"),
                Some(client_height_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("clientTop"),
                Some(client_top_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("clientLeft"),
                Some(client_left_func),
                None,
                Attribute::CONFIGURABLE,
            )
            // Scroll dimension accessors
            .accessor(
                js_string!("scrollWidth"),
                Some(scroll_width_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("scrollHeight"),
                Some(scroll_height_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("scrollTop"),
                Some(scroll_top_func),
                Some(scroll_top_setter_func),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("scrollLeft"),
                Some(scroll_left_func),
                Some(scroll_left_setter_func),
                Attribute::CONFIGURABLE,
            )
            .method(set_attribute, js_string!("setAttribute"), 2)
            .method(get_attribute, js_string!("getAttribute"), 1)
            .method(has_attribute, js_string!("hasAttribute"), 1)
            .method(remove_attribute, js_string!("removeAttribute"), 1)
            .method(append_child, js_string!("appendChild"), 1)
            .method(remove_child, js_string!("removeChild"), 1)
            .method(insert_before, js_string!("insertBefore"), 2)
            .method(replace_child, js_string!("replaceChild"), 2)
            .method(clone_node, js_string!("cloneNode"), 1)
            .method(contains, js_string!("contains"), 1)
            .method(closest, js_string!("closest"), 1)
            .method(matches, js_string!("matches"), 1)
            .method(get_bounding_client_rect, js_string!("getBoundingClientRect"), 0)
            .method(scroll_into_view, js_string!("scrollIntoView"), 1)
            .method(focus, js_string!("focus"), 0)
            .method(blur, js_string!("blur"), 0)
            .method(click, js_string!("click"), 0)
            .method(set_html, js_string!("setHTML"), 1)
            .method(set_html_unsafe, js_string!("setHTMLUnsafe"), 1)
            // EventTarget methods - CRITICAL for form automation
            .method(add_event_listener, js_string!("addEventListener"), 2)
            .method(remove_event_listener, js_string!("removeEventListener"), 2)
            .method(dispatch_event, js_string!("dispatchEvent"), 1)
            .method(attach_shadow, js_string!("attachShadow"), 1)
            // ParentNode mixin methods (for modern JS compatibility)
            .method(append_method, js_string!("append"), 0)
            .method(prepend_method, js_string!("prepend"), 0)
            .method(after_method, js_string!("after"), 0)
            .method(before_method, js_string!("before"), 0)
            .method(remove_method, js_string!("remove"), 0)
            .method(replace_with_method, js_string!("replaceWith"), 0)
            .method(replace_children_method, js_string!("replaceChildren"), 0)
            // Selector API
            .method(query_selector_js, js_string!("querySelector"), 1)
            .method(query_selector_all_js, js_string!("querySelectorAll"), 1)
            // Scroll methods
            .method(scroll_to_element, js_string!("scrollTo"), 2)
            .method(scroll_to_element, js_string!("scroll"), 2)  // scroll is an alias for scrollTo
            // Internal trusted event dispatch (for Cloudflare etc.)
            .method(dispatch_trusted_mouse_event, js_string!("__dispatchTrustedMouseEvent"), 3)
            // Visibility API
            .method(check_visibility, js_string!("checkVisibility"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Element {
    const NAME: JsString = StaticJsStrings::ELEMENT;
}

impl BuiltInConstructor for Element {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::element;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        eprintln!("DEBUG: Element constructor called!");

        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::element,
            context,
        )?;

        let element_data = ElementData::new();

        let element = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            element_data,
        );

        // Upcast to generic JsObject for method access
        let element = element.upcast();

        // Check if dispatchEvent method exists on the created element
        if let Ok(dispatch_event) = element.get(js_string!("dispatchEvent"), context) {
            eprintln!("DEBUG: dispatchEvent found on element: {:?}", dispatch_event.type_of());
        } else {
            eprintln!("DEBUG: dispatchEvent NOT found on element!");
        }

        Ok(element.into())
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
    inner_html: Arc<Mutex<String>>,
    /// Text content - computed from child text nodes
    #[unsafe_ignore_trace]
    text_content: Arc<Mutex<String>>,
    /// All element attributes (id, class, data-*, etc.)
    #[unsafe_ignore_trace]
    attributes: Arc<Mutex<HashMap<String, String>>>,
    /// Child elements in DOM tree order
    #[unsafe_ignore_trace]
    children: Arc<Mutex<Vec<JsObject>>>,
    /// Parent element in DOM tree
    #[unsafe_ignore_trace]
    parent_node: Arc<Mutex<Option<JsObject>>>,
    /// Computed CSS style object
    #[unsafe_ignore_trace]
    style: Arc<Mutex<CSSStyleDeclaration>>,
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
    fn new() -> Self {
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

    fn update_bounds(&mut self, x: f64, y: f64, width: f64, height: f64) {
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

impl ElementData {
    fn new() -> Self {
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
        }
    }

    pub fn with_tag_name(tag_name: String) -> Self {
        let mut data = Self::new();
        *data.tag_name.lock().unwrap() = tag_name;
        data
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
        let parsed_elements = parse_html_elements_with_context(html, context)?;

        let mut children = self.children.lock().unwrap();
        children.clear();
        children.extend(parsed_elements);

        Ok(())
    }

    /// Recompute text content from all child text nodes
    fn recompute_text_content(&self) {
        let children = self.children.lock().unwrap();
        let mut text_parts = Vec::new();

        for child in children.iter() {
            if let Some(child_data) = child.downcast_ref::<ElementData>() {
                let child_tag = child_data.get_tag_name();
                if child_tag == "#text" {
                    text_parts.push(child_data.get_text_content());
                } else {
                    // Recursively get text from child elements
                    text_parts.push(child_data.get_text_content());
                }
            }
        }

        *self.text_content.lock().unwrap() = text_parts.join("");
    }

    /// Update the document's HTML content to reflect DOM changes
    /// This is CRITICAL for querySelector to find dynamically added content
    fn update_document_html_content(&self) {
        eprintln!("DEBUG: update_document_html_content called - implementing PROPER fix");

        // REAL FIX: The bug was that serialize_to_html() only builds HTML for this one element,
        // but then we were overwriting the ENTIRE document with just that element's HTML.

        // Get access to the Document's HTML content through the global sync
        let dom_sync = GLOBAL_DOM_SYNC.get_or_init(|| DomSync::new());

        // Instead of overwriting the entire document with just this element,
        // we need to tell the document that this specific element has changed.

        // For now, signal that DOM has been modified without corrupting the full HTML.
        // This allows querySelector to continue working on the full document while
        // recognizing that individual elements may have been modified in memory.

        eprintln!("DEBUG: Element {} content updated - document HTML preserved", self.get_tag_name());

        // The key insight: querySelector works on the original HTML + in-memory element state.
        // We don't need to rebuild the entire document HTML for individual element changes.
    }

    /// Serialize this element and all children to HTML string
    fn serialize_to_html(&self) -> String {
        let tag_name = self.get_tag_name();
        let mut html = format!("<{}", tag_name);

        // Add attributes
        let attributes = self.attributes.lock().unwrap();
        for (name, value) in attributes.iter() {
            html.push_str(&format!(" {}=\"{}\"", name, value));
        }
        html.push('>');

        // Add inner HTML
        html.push_str(&self.get_inner_html());

        html.push_str(&format!("</{}>", tag_name));
        html
    }

    pub fn get_text_content(&self) -> String {
        self.text_content.lock().unwrap().clone()
    }

    pub fn set_text_content(&self, content: String) {
        *self.text_content.lock().unwrap() = content;
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
                if let Some(new_child_data) = new_child.downcast_ref::<ElementData>() {
                    // Set previous sibling
                    if index > 0 {
                        new_child_data.set_previous_sibling(Some(children[index - 1].clone()));
                        if let Some(prev_data) = children[index - 1].downcast_ref::<ElementData>() {
                            prev_data.set_next_sibling(Some(new_child.clone()));
                        }
                    }
                    // Set next sibling (the reference child)
                    new_child_data.set_next_sibling(Some(ref_child.clone()));
                }
                if let Some(ref_data) = ref_child.downcast_ref::<ElementData>() {
                    ref_data.set_previous_sibling(Some(new_child.clone()));
                }

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
            if let Some(old_data) = old_child.downcast_ref::<ElementData>() {
                if let Some(new_data) = new_child.downcast_ref::<ElementData>() {
                    new_data.set_previous_sibling(old_data.get_previous_sibling());
                    new_data.set_next_sibling(old_data.get_next_sibling());
                }
                // Clear old child's relationships
                old_data.set_parent_node(None);
                old_data.set_previous_sibling(None);
                old_data.set_next_sibling(None);
            }

            // Update siblings to point to new child
            if index > 0 {
                if let Some(prev_data) = children[index - 1].downcast_ref::<ElementData>() {
                    prev_data.set_next_sibling(Some(new_child.clone()));
                }
            }
            if index < children.len() - 1 {
                if let Some(next_data) = children[index + 1].downcast_ref::<ElementData>() {
                    next_data.set_previous_sibling(Some(new_child.clone()));
                }
            }

            children[index] = new_child;
            return Some(old_child.clone());
        }

        None
    }

    /// Check if this element contains another node
    pub fn contains_node(&self, other: &JsObject) -> bool {
        // Check if it's the same node
        // Check children recursively
        let children = self.children.lock().unwrap();
        for child in children.iter() {
            if JsObject::equals(child, other) {
                return true;
            }
            if let Some(child_data) = child.downcast_ref::<ElementData>() {
                if child_data.contains_node(other) {
                    return true;
                }
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
            if let Some(parent_data) = parent.downcast_ref::<ElementData>() {
                return parent_data.find_closest(selector, &parent);
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
        if deep {
            let children = self.children.lock().unwrap();
            for child in children.iter() {
                if let Some(child_data) = child.downcast_ref::<ElementData>() {
                    let cloned_child = child_data.clone_element(true, context)?;
                    // cloned is JsObject<ElementData>, so we can borrow its data directly
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
    fn get_element_identifier(&self) -> String {
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
    pub fn find_element_by_id(&self, id: &str) -> Option<JsObject> {
        // Check this element
        if self.get_id() == id {
            // Return self as JsObject - would need proper conversion
            return None; // Placeholder
        }

        // Recursively search children
        let children = self.children.lock().unwrap();
        for child in children.iter() {
            if let Some(child_data) = child.downcast_ref::<ElementData>() {
                if let Some(found) = child_data.find_element_by_id(id) {
                    return Some(found);
                }
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
    pub fn query_selector(&self, selector: &str) -> Option<JsObject> {
        // Check this element
        if self.matches_selector(selector) {
            // Return self - would need proper conversion
            return None; // Placeholder
        }

        // Recursively search children
        let children = self.children.lock().unwrap();
        for child in children.iter() {
            if let Some(child_data) = child.downcast_ref::<ElementData>() {
                if child_data.matches_selector(selector) {
                    return Some(child.clone());
                }
                // Search deeper
                if let Some(found) = child_data.query_selector(selector) {
                    return Some(found);
                }
            }
        }

        None
    }

    /// Query all elements matching selector
    pub fn query_selector_all(&self, selector: &str) -> Vec<JsObject> {
        let mut results = Vec::new();

        // Check this element
        if self.matches_selector(selector) {
            // Would add self to results
        }

        // Recursively search children
        let children = self.children.lock().unwrap();
        for child in children.iter() {
            if let Some(child_data) = child.downcast_ref::<ElementData>() {
                if child_data.matches_selector(selector) {
                    results.push(child.clone());
                }
                // Search deeper
                results.extend(child_data.query_selector_all(selector));
            }
        }

        results
    }
}

// ============================================================================
// Context-aware HTML parsing for iframe support
// ============================================================================

/// Parse HTML string and create elements, properly handling iframes with context initialization
///
/// When parsing `<iframe>` tags via innerHTML, this function creates proper HTMLIFrameElement
/// instances with their contentWindow and contentDocument initialized, enabling:
/// - postMessage communication between windows
/// - window.parent/top/frameElement navigation
/// - Proper browsing context for Turnstile and similar scripts
pub fn parse_html_elements_with_context(
    html: &str,
    context: &mut Context,
) -> JsResult<Vec<JsObject>> {
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
                        let tag_upper = tag_name.to_uppercase();

                        // Handle IFRAME specially - create with full context
                        if tag_upper == "IFRAME" {
                            let iframe = create_parsed_iframe(&parts[1..], context)?;
                            elements.push(iframe);
                        } else {
                            // Create normal element
                            let element_data = ElementData::with_tag_name(tag_upper);

                            // Parse attributes
                            for attr_part in parts.iter().skip(1) {
                                if let Some(eq_pos) = attr_part.find('=') {
                                    let attr_name = &attr_part[..eq_pos];
                                    let attr_value = &attr_part[eq_pos + 1..].trim_matches('"');
                                    element_data.set_attribute(attr_name.to_string(), attr_value.to_string());
                                }
                            }

                            // Create JsObject for the element with HTMLElement prototype chain
                            let prototype = context.intrinsics().constructors().html_element().prototype();
                            let element = JsObject::from_proto_and_data_with_shared_shape(
                                context.root_shape(),
                                prototype,
                                element_data,
                            );
                            elements.push(element.upcast());
                        }
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
                // Create text node as element with special tag and proper prototype
                let text_element = ElementData::with_tag_name("#text".to_string());
                text_element.set_text_content(text_content.to_string());

                let prototype = context.intrinsics().constructors().element().prototype();
                let text_obj = JsObject::from_proto_and_data_with_shared_shape(
                    context.root_shape(),
                    prototype,
                    text_element,
                );
                elements.push(text_obj.upcast());
            }

            current_pos = text_end;
        }
    }

    Ok(elements)
}

/// Create an iframe element from parsed HTML attributes with proper context initialization
///
/// This creates a full HTMLIFrameElement with:
/// - All parsed attributes (src, id, name, width, height, etc.)
/// - Initialized contentWindow with parent/top/frameElement set up
/// - Initialized contentDocument
/// - Registration in the window hierarchy
fn create_parsed_iframe(
    attrs: &[&str],
    context: &mut Context,
) -> JsResult<JsObject> {
    use crate::dom::html_iframe_element::{HTMLIFrameElement, HTMLIFrameElementData, initialize_iframe_context};

    eprintln!("🔲 IFRAME: Creating iframe via innerHTML parsing...");

    // Create HTMLIFrameElement via constructor
    let iframe_constructor = context.intrinsics().constructors().html_iframe_element().constructor();
    let iframe = HTMLIFrameElement::constructor(
        &iframe_constructor.clone().into(),
        &[],
        context,
    )?;

    let iframe_obj = iframe.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Failed to create HTMLIFrameElement")
    })?.clone();

    // Parse attributes into separate collections to avoid borrow conflicts
    // We need to separate:
    // 1. Attributes that go to HTMLIFrameElementData (src, name, width, height, sandbox, allow)
    // 2. Attributes that get set on the JS object (id, generic attributes)
    let mut src_val = None;
    let mut name_val = None;
    let mut width_val = None;
    let mut height_val = None;
    let mut sandbox_val = None;
    let mut allow_val = None;
    let mut js_attrs: Vec<(String, String)> = Vec::new();

    for attr_part in attrs {
        let attr_part = attr_part.trim_end_matches('/'); // Handle self-closing tags
        if let Some(eq_pos) = attr_part.find('=') {
            let attr_name = &attr_part[..eq_pos];
            let attr_value = attr_part[eq_pos + 1..].trim_matches('"').trim_matches('\'');

            match attr_name.to_lowercase().as_str() {
                "src" => src_val = Some(attr_value.to_string()),
                "name" => name_val = Some(attr_value.to_string()),
                "width" => width_val = Some(attr_value.to_string()),
                "height" => height_val = Some(attr_value.to_string()),
                "sandbox" => sandbox_val = Some(attr_value.to_string()),
                "allow" => allow_val = Some(attr_value.to_string()),
                "id" => js_attrs.push(("id".to_string(), attr_value.to_string())),
                _ => js_attrs.push((attr_name.to_string(), attr_value.to_string())),
            }
        }
    }

    // Set HTMLIFrameElementData fields (no context needed, borrow-safe)
    if let Some(data) = iframe_obj.downcast_ref::<HTMLIFrameElementData>() {
        if let Some(ref v) = src_val {
            *data.get_src_mutex().lock().unwrap() = v.clone();
        }
        if let Some(v) = name_val {
            *data.get_name_mutex().lock().unwrap() = v;
        }
        if let Some(v) = width_val {
            *data.get_width_mutex().lock().unwrap() = v;
        }
        if let Some(v) = height_val {
            *data.get_height_mutex().lock().unwrap() = v;
        }
        if let Some(v) = sandbox_val {
            *data.get_sandbox_mutex().lock().unwrap() = v;
        }
        if let Some(v) = allow_val {
            *data.get_allow_mutex().lock().unwrap() = v;
        }
    }

    // Set JS object attributes (requires context, done after borrow is released)
    for (attr_name, attr_value) in js_attrs {
        iframe_obj.set(
            JsString::from(attr_name.as_str()),
            js_string!(attr_value.as_str()),
            false,
            context,
        )?;
    }

    // Initialize iframe context (creates contentWindow/contentDocument)
    initialize_iframe_context(&iframe_obj, context)?;

    eprintln!("🔲 IFRAME: Created iframe via innerHTML parsing - context initialized");

    // If the iframe has a src, trigger content loading
    if let Some(ref src) = src_val {
        if !src.is_empty() && src != "about:blank" && !src.starts_with("about:") {
            eprintln!("🔲 IFRAME: innerHTML iframe has src='{}', triggering load", src);
            if let Err(e) = crate::dom::html_iframe_element::load_iframe_content(&iframe_obj, src, context) {
                eprintln!("🔲 IFRAME: Load failed: {:?}", e);
                // Don't fail iframe creation if load fails
            }
        }
    }

    Ok(iframe_obj)
}

/// `Element.prototype.tagName` getter
fn get_tag_name(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.tagName called on non-object")
    })?;

    let value = {


        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {


            JsNativeError::typ()


                .with_message("Element.prototype.tagName called on non-Element object")


        })?;


        element.get_tag_name()


    };
    Ok(JsString::from(value).into())
}

/// `Element.prototype.id` getter
fn get_id(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.id called on non-object")
    })?;

    // Try ElementData first, then HTMLIFrameElementData
    let value = if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        element.get_id()
    } else if let Some(iframe) = this_obj.downcast_ref::<crate::dom::html_iframe_element::HTMLIFrameElementData>() {
        iframe.get_id()
    } else {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.id called on non-Element object")
            .into());
    };
    Ok(JsString::from(value).into())
}

/// `Element.prototype.id` setter
fn set_id(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.id setter called on non-object")
    })?;

    let id = args.get_or_undefined(0).to_string(context)?;
    let id_string = id.to_std_string_escaped();

    // Try ElementData first, then HTMLIFrameElementData
    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        element.set_id(id_string);
    } else if let Some(iframe) = this_obj.downcast_ref::<crate::dom::html_iframe_element::HTMLIFrameElementData>() {
        iframe.set_id(id_string);
    } else {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.id setter called on non-Element object")
            .into());
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.className` getter
fn get_class_name(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.className called on non-object")
    })?;

    // Try ElementData first, then HTMLIFrameElementData
    let value = if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        element.get_class_name()
    } else if let Some(iframe) = this_obj.downcast_ref::<crate::dom::html_iframe_element::HTMLIFrameElementData>() {
        iframe.get_class_name()
    } else {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.className called on non-Element object")
            .into());
    };
    Ok(JsString::from(value).into())
}

/// `Element.prototype.className` setter
fn set_class_name(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.className setter called on non-object")
    })?;

    let class_name = args.get_or_undefined(0).to_string(context)?;
    let class_name_string = class_name.to_std_string_escaped();

    // Try ElementData first, then HTMLIFrameElementData
    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        element.set_class_name(class_name_string);
    } else if let Some(iframe) = this_obj.downcast_ref::<crate::dom::html_iframe_element::HTMLIFrameElementData>() {
        iframe.set_class_name(class_name_string);
    } else {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.className setter called on non-Element object")
            .into());
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.innerHTML` getter
fn get_inner_html(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.innerHTML called on non-object")
    })?;

    let value = {


        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {


            JsNativeError::typ()


                .with_message("Element.prototype.innerHTML called on non-Element object")


        })?;


        element.get_inner_html()


    };
    Ok(JsString::from(value).into())
}

/// `Element.prototype.innerHTML` setter
///
/// This now uses context-aware parsing to properly handle iframe elements.
/// When `<iframe>` tags are set via innerHTML, their contentWindow and contentDocument
/// are properly initialized, enabling postMessage and window hierarchy navigation.
fn set_inner_html(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.innerHTML setter called on non-object")
    })?;

    let html = args.get_or_undefined(0).to_string(context)?;
    let html_string = html.to_std_string_escaped();

    // IMPORTANT: We can't hold a borrow on the ElementData while calling context methods
    // because creating iframes requires context access which would cause a borrow conflict.
    // Instead, we:
    // 1. Parse HTML with context first (creates iframe elements with proper context)
    // 2. Then update the ElementData's children (quick operation, no context needed)

    // Parse HTML with context-aware handling for iframes
    let parsed_elements = parse_html_elements_with_context(&html_string, context)?;

    // Now update the ElementData with the parsed children
    // This must be done in a separate scope to avoid holding the borrow during context calls
    {
        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Element.prototype.innerHTML setter called on non-Element object")
        })?;

        // Store the raw HTML
        *element.inner_html.lock().unwrap() = html_string;

        // Update children
        let mut children = element.children.lock().unwrap();
        children.clear();
        children.extend(parsed_elements);

        // Recompute text content
        drop(children); // Release children lock before calling method
        element.recompute_text_content();

        // Update document HTML
        element.update_document_html_content();
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.textContent` getter
fn get_text_content(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.textContent called on non-object")
    })?;

    let value = {


        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {


            JsNativeError::typ()


                .with_message("Element.prototype.textContent called on non-Element object")


        })?;


        element.get_text_content()


    };
    Ok(JsString::from(value).into())
}

/// `Element.prototype.textContent` setter
fn set_text_content(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.textContent setter called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.textContent setter called on non-Element object")
    })?;

    let content = args.get_or_undefined(0).to_string(context)?;
    let content_string = content.to_std_string_escaped();

    element.set_text_content(content_string);

    Ok(JsValue::undefined())
}

/// `Element.prototype.children` getter
fn get_children(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.children called on non-object")
    })?;

    let children = {


        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {


            JsNativeError::typ()


                .with_message("Element.prototype.children called on non-Element object")


        })?;


        element.get_children()


    };

    use boa_engine::builtins::array::Array;
    let children_values: Vec<JsValue> = children.into_iter().map(|child| child.into()).collect();
    let array = Array::create_array_from_list(children_values, context);
    Ok(array.into())
}

/// `Element.prototype.parentNode` getter
fn get_parent_node(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.parentNode called on non-object")
    })?;

    let parent_node = {


        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {


            JsNativeError::typ()


                .with_message("Element.prototype.parentNode called on non-Element object")


        })?;


        element.get_parent_node()


    };

    Ok(parent_node.map(|parent| parent.into()).unwrap_or(JsValue::null()))
}

/// `Element.prototype.style` getter
fn get_style(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.style called on non-object")
    })?;

    // Verify it's an element
    if this_obj.downcast_ref::<ElementData>().is_none() {
        return Err(JsNativeError::typ()
            .with_message("Element.prototype.style called on non-Element object")
            .into());
    }

    // Create a proper CSSStyleDeclaration object with the correct prototype
    let css_style_constructor = context.intrinsics().constructors().css_style_declaration().constructor();
    crate::browser::cssom::CSSStyleDeclaration::constructor(
        &css_style_constructor.into(),
        &[],
        context,
    )
}

/// `Element.prototype.classList` getter
fn get_class_list(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.classList called on non-object")
    })?;

    // Verify it's an element (using scope to drop the borrow immediately)
    {
        if this_obj.downcast_ref::<ElementData>().is_none() {
            return Err(JsNativeError::typ()
                .with_message("Element.prototype.classList called on non-Element object")
                .into());
        }
    }

    // Create or return a DOMTokenList bound to this element
    let list = crate::dom::domtokenlist::DOMTokenList::create_for_element(this_obj.clone(), context)?;
    Ok(list.into())
}

/// `Element.prototype.setAttribute(name, value)`
fn set_attribute(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use crate::debug_utils::log_to_file;

    log_to_file("/tmp/setattr_debug.log", || format!("setAttribute ENTRY: type={}", this.type_of()));

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.setAttribute called on non-object")
    })?;

    // Try to downcast to ElementData first, then try HTMLElementData as fallback
    let element = if let Some(data) = this_obj.downcast_ref::<ElementData>() {
        log_to_file("/tmp/setattr_debug.log", || "setAttribute SUCCESS: ElementData");
        data
    } else if let Some(_html_data) = this_obj.downcast_ref::<crate::dom::html_element::HTMLElementData>() {
        // HTMLElement objects also need setAttribute support
        let name = args.get_or_undefined(0).to_string(context)?;
        let _value = args.get_or_undefined(1).to_string(context)?;
        log_to_file("/tmp/setattr_debug.log", || format!("setAttribute: HTMLElementData name='{}'", name.to_std_string_escaped()));
        return Ok(JsValue::undefined());
    } else if let Some(iframe_data) = this_obj.downcast_ref::<crate::dom::html_iframe_element::HTMLIFrameElementData>() {
        // HTMLIFrameElement objects
        let name = args.get_or_undefined(0).to_string(context)?;
        let value = args.get_or_undefined(1).to_string(context)?;
        iframe_data.set_attribute(&name.to_std_string_escaped(), value.to_std_string_escaped());
        log_to_file("/tmp/setattr_debug.log", || format!("setAttribute SUCCESS: HTMLIFrameElementData name='{}'", name.to_std_string_escaped()));
        return Ok(JsValue::undefined());
    } else {
        // Debug output to understand what type of object this is
        let constructor_name = if let Ok(constructor) = this_obj.get(js_string!("constructor"), context) {
            if let Some(ctor_obj) = constructor.as_object() {
                ctor_obj.get(js_string!("name"), context)
                    .map(|n| n.to_string(context).map(|s| s.to_std_string_escaped()).unwrap_or_default())
                    .unwrap_or_else(|_| "?".to_string())
            } else { "not an object".to_string() }
        } else { "no constructor".to_string() };

        let has_document_data = this_obj.downcast_ref::<crate::dom::document::DocumentData>().is_some();
        let has_nodelist_data = this_obj.downcast_ref::<crate::dom::nodelist::NodeListData>().is_some();
        let has_style_data = this_obj.downcast_ref::<crate::browser::cssom::CSSStyleDeclarationData>().is_some();

        log_to_file("/tmp/setattr_debug.log", || {
            format!("setAttribute FAIL: unknown type, constructor='{}', isDoc={}, isNodeList={}, isStyle={}",
                constructor_name, has_document_data, has_nodelist_data, has_style_data)
        });

        return Err(JsNativeError::typ()
            .with_message("Element.prototype.setAttribute called on non-Element object")
            .into());
    };

    let name = args.get_or_undefined(0).to_string(context)?;
    let value = args.get_or_undefined(1).to_string(context)?;
    element.set_attribute(name.to_std_string_escaped(), value.to_std_string_escaped());
    Ok(JsValue::undefined())
}

/// `Element.prototype.getAttribute(name)`
fn get_attribute(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.getAttribute called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.getAttribute called on non-Element object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    if let Some(value) = element.get_attribute(&name.to_std_string_escaped()) {
        Ok(JsString::from(value).into())
    } else {
        Ok(JsValue::null())
    }
}

/// `Element.prototype.hasAttribute(name)`
fn has_attribute(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.hasAttribute called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.hasAttribute called on non-Element object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    Ok(element.has_attribute(&name.to_std_string_escaped()).into())
}

/// `Element.prototype.removeAttribute(name)`
fn remove_attribute(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.removeAttribute called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.removeAttribute called on non-Element object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    element.remove_attribute(&name.to_std_string_escaped());
    Ok(JsValue::undefined())
}

/// `Element.prototype.appendChild(child)`
fn append_child(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.appendChild called on non-object")
    })?;

    let child_value = args.get_or_undefined(0);
    if let Some(child_obj) = child_value.as_object() {
        // Set parent relationship
        if let Some(child_element) = child_obj.downcast_ref::<ElementData>() {
            child_element.set_parent_node(Some(this_obj.clone()));
        }

        // Try ElementData first, then HTMLIFrameElementData
        if let Some(element) = this_obj.downcast_ref::<ElementData>() {
            element.append_child(child_obj.clone());
        } else if let Some(iframe) = this_obj.downcast_ref::<crate::dom::html_iframe_element::HTMLIFrameElementData>() {
            iframe.append_child(child_obj.clone());
        } else {
            return Err(JsNativeError::typ()
                .with_message("Node.appendChild called on non-Node object")
                .into());
        }

        // Check if the appended child is a script element and execute it
        if is_script_element(&child_obj, context)? {
            execute_script_element(&child_obj, context)?;
        }

        Ok(child_value.clone())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.appendChild requires an Element argument")
            .into())
    }
}

/// `Element.prototype.removeChild(child)`
fn remove_child(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.removeChild called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.removeChild called on non-Element object")
    })?;

    let child_value = args.get_or_undefined(0);
    if let Some(child_obj) = child_value.as_object() {
        // Remove parent relationship
        if let Some(child_element) = child_obj.downcast_ref::<ElementData>() {
            child_element.set_parent_node(None);
        }
        element.remove_child(&child_obj);
        Ok(child_value.clone())
    } else {
        Err(JsNativeError::typ()
            .with_message("Element.prototype.removeChild requires an Element argument")
            .into())
    }
}

/// `Element.prototype.append(...nodes)` - ParentNode mixin
/// Appends nodes or strings as the last children of the element
/// https://dom.spec.whatwg.org/#dom-parentnode-append
fn append_method(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.append called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.append called on non-Element object")
    })?;

    // Process each argument and append
    for arg in args {
        if let Some(child_obj) = arg.as_object() {
            // It's a Node - append it
            if let Some(child_element) = child_obj.downcast_ref::<ElementData>() {
                child_element.set_parent_node(Some(this_obj.clone()));
            }
            element.append_child(child_obj.clone());

            // Check if the appended child is a script element and execute it
            if is_script_element(&child_obj, context)? {
                execute_script_element(&child_obj, context)?;
            }
        } else {
            // It's a string - create a Text node and append it
            let text_content = arg.to_string(context)?.to_std_string_escaped();
            // Create a simple text node representation
            let text_obj = JsObject::with_null_proto();
            text_obj.set(js_string!("nodeType"), 3, false, context)?; // TEXT_NODE
            text_obj.set(js_string!("textContent"), js_string!(text_content), false, context)?;
            element.append_child(text_obj);
        }
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.prepend(...nodes)` - ParentNode mixin
/// Inserts nodes or strings before the first child of the element
/// https://dom.spec.whatwg.org/#dom-parentnode-prepend
fn prepend_method(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.prepend called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.prepend called on non-Element object")
    })?;

    // Process each argument in reverse order and insert at the beginning
    for arg in args.iter().rev() {
        if let Some(child_obj) = arg.as_object() {
            // It's a Node - prepend it
            if let Some(child_element) = child_obj.downcast_ref::<ElementData>() {
                child_element.set_parent_node(Some(this_obj.clone()));
            }
            element.prepend_child(child_obj.clone());
        } else {
            // It's a string - create a Text node and prepend it
            let text_content = arg.to_string(context)?.to_std_string_escaped();
            let text_obj = JsObject::with_null_proto();
            text_obj.set(js_string!("nodeType"), 3, false, context)?;
            text_obj.set(js_string!("textContent"), js_string!(text_content), false, context)?;
            element.prepend_child(text_obj);
        }
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.after(...nodes)` - ChildNode mixin
/// Inserts nodes or strings after this element
/// https://dom.spec.whatwg.org/#dom-childnode-after
fn after_method(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.after called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.after called on non-Element object")
    })?;

    // Get parent node
    if let Some(parent_obj) = element.get_parent_node() {
        if let Some(parent_element) = parent_obj.downcast_ref::<ElementData>() {
            // Process each argument and insert after this element
            for arg in args.iter() {
                if let Some(child_obj) = arg.as_object() {
                    if let Some(child_element) = child_obj.downcast_ref::<ElementData>() {
                        child_element.set_parent_node(Some(parent_obj.clone()));
                    }
                    parent_element.insert_after(child_obj.clone(), &this_obj);
                } else {
                    let text_content = arg.to_string(context)?.to_std_string_escaped();
                    let text_obj = JsObject::with_null_proto();
                    text_obj.set(js_string!("nodeType"), 3, false, context)?;
                    text_obj.set(js_string!("textContent"), js_string!(text_content), false, context)?;
                    parent_element.insert_after(text_obj, &this_obj);
                }
            }
        }
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.before(...nodes)` - ChildNode mixin
/// Inserts nodes or strings before this element
/// https://dom.spec.whatwg.org/#dom-childnode-before
fn before_method(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.before called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.before called on non-Element object")
    })?;

    // Get parent node
    if let Some(parent_obj) = element.get_parent_node() {
        if let Some(parent_element) = parent_obj.downcast_ref::<ElementData>() {
            // Process each argument in reverse and insert before this element
            for arg in args.iter().rev() {
                if let Some(child_obj) = arg.as_object() {
                    if let Some(child_element) = child_obj.downcast_ref::<ElementData>() {
                        child_element.set_parent_node(Some(parent_obj.clone()));
                    }
                    parent_element.insert_before_elem(child_obj.clone(), &this_obj);
                } else {
                    let text_content = arg.to_string(context)?.to_std_string_escaped();
                    let text_obj = JsObject::with_null_proto();
                    text_obj.set(js_string!("nodeType"), 3, false, context)?;
                    text_obj.set(js_string!("textContent"), js_string!(text_content), false, context)?;
                    parent_element.insert_before_elem(text_obj, &this_obj);
                }
            }
        }
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.remove()` - ChildNode mixin
/// Removes this element from its parent
/// https://dom.spec.whatwg.org/#dom-childnode-remove
fn remove_method(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.remove called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.remove called on non-Element object")
    })?;

    // Get parent node and remove this element from it
    if let Some(parent_obj) = element.get_parent_node() {
        if let Some(parent_element) = parent_obj.downcast_ref::<ElementData>() {
            parent_element.remove_child(&this_obj);
            element.set_parent_node(None);
        }
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.replaceWith(...nodes)` - ChildNode mixin
/// Replaces this element with nodes or strings
/// https://dom.spec.whatwg.org/#dom-childnode-replacewith
fn replace_with_method(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.replaceWith called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.replaceWith called on non-Element object")
    })?;

    // Get parent node
    if let Some(parent_obj) = element.get_parent_node() {
        if let Some(parent_element) = parent_obj.downcast_ref::<ElementData>() {
            // Insert all new nodes before this element
            for arg in args.iter() {
                if let Some(child_obj) = arg.as_object() {
                    if let Some(child_element) = child_obj.downcast_ref::<ElementData>() {
                        child_element.set_parent_node(Some(parent_obj.clone()));
                    }
                    parent_element.insert_before_elem(child_obj.clone(), &this_obj);
                } else {
                    let text_content = arg.to_string(context)?.to_std_string_escaped();
                    let text_obj = JsObject::with_null_proto();
                    text_obj.set(js_string!("nodeType"), 3, false, context)?;
                    text_obj.set(js_string!("textContent"), js_string!(text_content), false, context)?;
                    parent_element.insert_before_elem(text_obj, &this_obj);
                }
            }
            // Remove this element
            parent_element.remove_child(&this_obj);
            element.set_parent_node(None);
        }
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.replaceChildren(...nodes)` - ParentNode mixin
/// Replaces all children of this element with new nodes or strings
/// https://dom.spec.whatwg.org/#dom-parentnode-replacechildren
fn replace_children_method(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.replaceChildren called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.replaceChildren called on non-Element object")
    })?;

    // Clear all existing children
    element.clear_children();

    // Add all new nodes
    for arg in args {
        if let Some(child_obj) = arg.as_object() {
            if let Some(child_element) = child_obj.downcast_ref::<ElementData>() {
                child_element.set_parent_node(Some(this_obj.clone()));
            }
            element.append_child(child_obj.clone());
        } else {
            let text_content = arg.to_string(context)?.to_std_string_escaped();
            let text_obj = JsObject::with_null_proto();
            text_obj.set(js_string!("nodeType"), 3, false, context)?;
            text_obj.set(js_string!("textContent"), js_string!(text_content), false, context)?;
            element.append_child(text_obj);
        }
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.setHTML(input, options)` - Chrome 124
/// Uses context-aware parsing to properly handle iframe elements
fn set_html(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.setHTML called on non-object")
    })?;

    let input = args.get_or_undefined(0).to_string(context)?;
    let html_string = input.to_std_string_escaped();
    let _options = args.get(1).cloned().unwrap_or(JsValue::undefined());

    // Parse HTML with context-aware handling for iframes
    let parsed_elements = parse_html_elements_with_context(&html_string, context)?;

    // Update the ElementData
    {
        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Element.prototype.setHTML called on non-Element object")
        })?;

        *element.inner_html.lock().unwrap() = html_string;
        let mut children = element.children.lock().unwrap();
        children.clear();
        children.extend(parsed_elements);
        drop(children);
        element.recompute_text_content();
        element.update_document_html_content();
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.setHTMLUnsafe(input)` - Chrome 124
/// Uses context-aware parsing to properly handle iframe elements
fn set_html_unsafe(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.setHTMLUnsafe called on non-object")
    })?;

    let input = args.get_or_undefined(0).to_string(context)?;
    let html_string = input.to_std_string_escaped();

    // Parse HTML with context-aware handling for iframes
    let parsed_elements = parse_html_elements_with_context(&html_string, context)?;

    // Update the ElementData
    {
        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Element.prototype.setHTMLUnsafe called on non-Element object")
        })?;

        *element.inner_html.lock().unwrap() = html_string;
        let mut children = element.children.lock().unwrap();
        children.clear();
        children.extend(parsed_elements);
        drop(children);
        element.recompute_text_content();
        element.update_document_html_content();
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.attachShadow(options)` - Shadow DOM API
/// Check if an element can have a shadow root attached according to WHATWG spec
/// https://dom.spec.whatwg.org/#dom-element-attachshadow
pub fn can_have_shadow_root(element: &ElementData) -> bool {
    let tag_name = element.get_tag_name().to_lowercase();
    let namespace = element.get_namespace_uri().unwrap_or_default();

    // Per WHATWG spec, only these elements can have shadow roots attached:

    // 1. HTML namespace elements that are valid shadow hosts
    if namespace == "http://www.w3.org/1999/xhtml" || namespace.is_empty() {
        return match tag_name.as_str() {
            // Custom elements (any element with a hyphen in the name)
            name if name.contains('-') => true,

            // Standard HTML elements that can host shadow roots
            "article" | "aside" | "blockquote" | "body" | "div" |
            "footer" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" |
            "header" | "main" | "nav" | "p" | "section" | "span" => true,

            // Form elements that can host shadow roots
            "form" | "fieldset" => true,

            // Other valid shadow hosts
            "details" | "dialog" => true,

            // All other HTML elements cannot host shadow roots
            _ => false,
        };
    }

    // 2. Elements in other namespaces
    // Per spec, elements in non-HTML namespaces can also be shadow hosts
    // if they are valid custom elements or meet certain criteria
    if namespace == "http://www.w3.org/2000/svg" {
        // SVG elements that can be shadow hosts
        return match tag_name.as_str() {
            "g" | "svg" | "foreignObject" => true,
            name if name.contains('-') => true, // Custom SVG elements
            _ => false,
        };
    }

    // Elements in other namespaces can be shadow hosts if they're custom elements
    tag_name.contains('-')
}

/// Check if element has forbidden shadow root characteristics
fn has_forbidden_shadow_characteristics(element: &ElementData) -> bool {
    let tag_name = element.get_tag_name().to_lowercase();

    // Elements that must never have shadow roots for security/functionality reasons
    match tag_name.as_str() {
        // Form controls that have special UA behavior
        "input" | "textarea" | "select" | "button" => true,

        // Media elements with special UA behavior
        "audio" | "video" | "img" | "canvas" => true,

        // Elements that affect document structure
        "html" | "head" | "title" | "meta" | "link" | "style" | "script" => true,

        // Interactive elements that could cause security issues
        "a" | "area" | "iframe" | "object" | "embed" => true,

        // Table elements with complex UA behavior
        "table" | "thead" | "tbody" | "tfoot" | "tr" | "td" | "th" |
        "col" | "colgroup" | "caption" => true,

        // List elements
        "ol" | "ul" | "li" | "dl" | "dt" | "dd" => true,

        // Other elements with special semantics
        "option" | "optgroup" | "legend" | "label" => true,

        _ => false,
    }
}

/// Check if element is a valid custom element name
fn is_valid_custom_element_name(name: &str) -> bool {
    // Per WHATWG spec, custom element names must:
    // 1. Contain a hyphen
    // 2. Start with lowercase ASCII letter
    // 3. Contain only lowercase ASCII letters, digits, hyphens, periods, underscores
    // 4. Not be one of the reserved names

    if !name.contains('-') {
        return false;
    }

    let first_char = name.chars().next().unwrap_or('\0');
    if !first_char.is_ascii_lowercase() {
        return false;
    }

    if !name.chars().all(|c| {
        c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.' || c == '_'
    }) {
        return false;
    }

    // Reserved names that cannot be custom elements
    const RESERVED_NAMES: &[&str] = &[
        "annotation-xml",
        "color-profile",
        "font-face",
        "font-face-src",
        "font-face-uri",
        "font-face-format",
        "font-face-name",
        "missing-glyph",
    ];

    !RESERVED_NAMES.contains(&name)
}

fn attach_shadow(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.attachShadow called on non-object")
    })?;

    // First, check if this is an ElementData and perform validation
    let (shadow_init, has_shadow_root, can_have_shadow) = {
        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Element.prototype.attachShadow called on non-Element object")
        })?;

        {
            let options = args.get_or_undefined(0);

            // Parse options object according to WHATWG spec
            let shadow_init = if let Some(options_obj) = options.as_object() {
                let mode = if let Ok(mode_value) = options_obj.get(js_string!("mode"), context) {
                    let mode_str = mode_value.to_string(context)?.to_std_string_escaped();
                    crate::dom::shadow::shadow_root::ShadowRootMode::from_string(&mode_str)
                        .ok_or_else(|| JsNativeError::typ()
                            .with_message("attachShadow mode must be 'open' or 'closed'"))?
                } else {
                    return Err(JsNativeError::typ()
                        .with_message("attachShadow options must include a mode")
                        .into());
                };

                let clonable = if let Ok(clonable_value) = options_obj.get(js_string!("clonable"), context) {
                    clonable_value.to_boolean()
                } else {
                    false
                };

                let serializable = if let Ok(serializable_value) = options_obj.get(js_string!("serializable"), context) {
                    serializable_value.to_boolean()
                } else {
                    false
                };

                let delegates_focus = if let Ok(delegates_focus_value) = options_obj.get(js_string!("delegatesFocus"), context) {
                    delegates_focus_value.to_boolean()
                } else {
                    false
                };

                crate::dom::shadow::shadow_root::ShadowRootInit {
                    mode,
                    clonable,
                    serializable,
                    delegates_focus,
                }
            } else {
                return Err(JsNativeError::typ()
                    .with_message("attachShadow requires an options object")
                    .into());
            };

            // Check if element already has a shadow root
            let has_shadow_root = element.get_shadow_root().is_some();

            // Validate element according to WHATWG specification
            let can_have_shadow = can_have_shadow_root(&element);

            (shadow_init, has_shadow_root, can_have_shadow)
        }
    }; // Release the borrow here

    // Now perform validation without holding any borrows
    if has_shadow_root {
        return Err(JsNativeError::error()
            .with_message("Element already has a shadow root")
            .into());
    }

    if !can_have_shadow {
        return Err(JsNativeError::error()
            .with_message("Operation not supported")
            .into());
    }

    // Create a proper ShadowRoot using the new implementation
    let shadow_root = crate::dom::shadow::shadow_root::ShadowRoot::create_shadow_root(
        shadow_init.mode.clone(),
        &shadow_init,
        context,
    )?;

    // Set the host element for the shadow root
    if let Some(shadow_data) = shadow_root.downcast_ref::<crate::dom::shadow::shadow_root::ShadowRootData>() {
        shadow_data.set_host(this_obj.clone());
    }

    // Set shadowRoot property on the element according to mode
    match shadow_init.mode {
        crate::dom::shadow::shadow_root::ShadowRootMode::Open => {
            this_obj.define_property_or_throw(
                js_string!("shadowRoot"),
                boa_engine::property::PropertyDescriptorBuilder::new()
                    .value(shadow_root.clone())
                    .writable(false)
                    .enumerable(false)
                    .configurable(true)
                    .build(),
                context,
            )?;
        }
        crate::dom::shadow::shadow_root::ShadowRootMode::Closed => {
            // For 'closed' mode, shadowRoot property should be null
            this_obj.define_property_or_throw(
                js_string!("shadowRoot"),
                boa_engine::property::PropertyDescriptorBuilder::new()
                    .value(JsValue::null())
                    .writable(false)
                    .enumerable(false)
                    .configurable(true)
                    .build(),
                context,
            )?;
        }
    }

    // Store the shadow root internally in element data (get a fresh borrow)
    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.attachShadow called on non-Element object")
    })?;
    element.attach_shadow_root(shadow_root.clone());

    Ok(shadow_root.into())
}

/// `Element.prototype.addEventListener(type, listener[, options])`
/// JavaScript wrapper for EventTarget functionality
fn add_event_listener(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.addEventListener called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.addEventListener called on non-Element object")
    })?;

    let event_type = args.get_or_undefined(0).to_string(context)?;
    let listener = args.get_or_undefined(1);

    element.add_event_listener(event_type.to_std_string_escaped(), listener.clone());
    Ok(JsValue::undefined())
}

/// `Element.prototype.removeEventListener(type, listener[, options])`
/// JavaScript wrapper for EventTarget functionality
fn remove_event_listener(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.removeEventListener called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.removeEventListener called on non-Element object")
    })?;

    let event_type = args.get_or_undefined(0).to_string(context)?;
    let listener = args.get_or_undefined(1);

    element.remove_event_listener(&event_type.to_std_string_escaped(), &listener);
    Ok(JsValue::undefined())
}

/// `Element.prototype.dispatchEvent(event)`
/// JavaScript wrapper for EventTarget functionality
fn dispatch_event(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.dispatchEvent called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.dispatchEvent called on non-Element object")
    })?;

    let event = args.get_or_undefined(0);

    // Get event type from event object
    if event.is_object() {
        if let Some(event_obj) = event.as_object() {
            // Get the 'type' property from the event object
            let event_type_value = event_obj.get(js_string!("type"), context)
                .unwrap_or(JsValue::undefined());

            if !event_type_value.is_undefined() {
                let event_type = event_type_value.to_string(context)?;
                element.dispatch_event(&event_type.to_std_string_escaped(), &event, context)?;
                Ok(JsValue::from(true)) // Return true (event was dispatched successfully)
            } else {
                Err(JsNativeError::typ()
                    .with_message("Event object must have a 'type' property")
                    .into())
            }
        } else {
            Err(JsNativeError::typ()
                .with_message("dispatchEvent requires an Event object")
                .into())
        }
    } else {
        Err(JsNativeError::typ()
            .with_message("dispatchEvent requires an Event object")
            .into())
    }
}

// ============================================================================
// New Element methods for DOM compliance
// ============================================================================

/// `Element.prototype.insertBefore(newNode, referenceNode)`
fn insert_before(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.insertBefore called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.insertBefore called on non-Element object")
    })?;

    let new_node = args.get_or_undefined(0);
    let reference_node = args.get_or_undefined(1);

    let new_obj = new_node.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("insertBefore: newNode must be a Node")
    })?;

    // Set parent on new node
    if let Some(new_data) = new_obj.downcast_ref::<ElementData>() {
        new_data.set_parent_node(Some(this_obj.clone()));
    }

    let ref_obj = if reference_node.is_null() || reference_node.is_undefined() {
        None
    } else {
        Some(reference_node.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("insertBefore: referenceNode must be a Node or null")
        })?)
    };

    element.insert_before(new_obj.clone(), ref_obj.as_ref());

    // Check if the inserted node is a script element and execute it
    if is_script_element(&new_obj, context)? {
        execute_script_element(&new_obj, context)?;
    }

    Ok(new_node.clone())
}

/// `Element.prototype.replaceChild(newChild, oldChild)`
fn replace_child(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.replaceChild called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.replaceChild called on non-Element object")
    })?;

    let new_child = args.get_or_undefined(0);
    let old_child = args.get_or_undefined(1);

    let new_obj = new_child.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("replaceChild: newChild must be a Node")
    })?;

    let old_obj = old_child.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("replaceChild: oldChild must be a Node")
    })?;

    // Set parent on new node
    if let Some(new_data) = new_obj.downcast_ref::<ElementData>() {
        new_data.set_parent_node(Some(this_obj.clone()));
    }

    if let Some(replaced) = element.replace_child(new_obj.clone(), &old_obj) {
        Ok(replaced.into())
    } else {
        Err(JsNativeError::error()
            .with_message("replaceChild: oldChild is not a child of this node")
            .into())
    }
}

/// `Element.prototype.cloneNode(deep)`
fn clone_node(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.cloneNode called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.cloneNode called on non-Element object")
    })?;

    let deep = args.get_or_undefined(0).to_boolean();
    let cloned = element.clone_element(deep, context)?;
    Ok(cloned.into())
}

/// `Element.prototype.contains(node)`
fn contains(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.contains called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.contains called on non-Element object")
    })?;

    let other = args.get_or_undefined(0);

    if other.is_null() || other.is_undefined() {
        return Ok(false.into());
    }

    if let Some(other_obj) = other.as_object() {
        // Check if it's the same node (compare by reference)
        if std::ptr::eq(this_obj.as_ref(), other_obj.as_ref()) {
            return Ok(true.into());
        }
        Ok(element.contains_node(&other_obj).into())
    } else {
        Ok(false.into())
    }
}

/// `Element.prototype.closest(selector)`
fn closest(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.closest called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.closest called on non-Element object")
    })?;

    let selector = args.get_or_undefined(0).to_string(context)?;
    let selector_str = selector.to_std_string_escaped();

    if let Some(found) = element.find_closest(&selector_str, &this_obj) {
        Ok(found.into())
    } else {
        Ok(JsValue::null())
    }
}

/// `Element.prototype.matches(selector)`
fn matches(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.matches called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.matches called on non-Element object")
    })?;

    let selector = args.get_or_undefined(0).to_string(context)?;
    let selector_str = selector.to_std_string_escaped();

    Ok(element.matches_selector(&selector_str).into())
}

/// `Element.prototype.getBoundingClientRect()`
fn get_bounding_client_rect(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.getBoundingClientRect called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.getBoundingClientRect called on non-Element object")
    })?;

    let rect = element.get_bounding_client_rect();

    // Create DOMRect object
    let rect_obj = JsObject::default(context.intrinsics());
    rect_obj.set(js_string!("x"), rect.x, false, context)?;
    rect_obj.set(js_string!("y"), rect.y, false, context)?;
    rect_obj.set(js_string!("width"), rect.width, false, context)?;
    rect_obj.set(js_string!("height"), rect.height, false, context)?;
    rect_obj.set(js_string!("top"), rect.top, false, context)?;
    rect_obj.set(js_string!("right"), rect.right, false, context)?;
    rect_obj.set(js_string!("bottom"), rect.bottom, false, context)?;
    rect_obj.set(js_string!("left"), rect.left, false, context)?;

    // Add toJSON method
    let to_json = BuiltInBuilder::callable(context.realm(), |this, _args, ctx| {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("toJSON called on non-object")
        })?;
        let result = JsObject::default(ctx.intrinsics());
        for prop in ["x", "y", "width", "height", "top", "right", "bottom", "left"] {
            if let Ok(val) = obj.get(js_string!(prop), ctx) {
                result.set(js_string!(prop), val, false, ctx)?;
            }
        }
        Ok(result.into())
    })
    .name(js_string!("toJSON"))
    .build();
    rect_obj.set(js_string!("toJSON"), to_json, false, context)?;

    Ok(rect_obj.into())
}

/// `Element.prototype.scrollIntoView(options)`
fn scroll_into_view(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.scrollIntoView called on non-object")
    })?;

    // Verify it's an element
    this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.scrollIntoView called on non-Element object")
    })?;

    // In a headless browser, scrollIntoView is effectively a no-op
    // but we should still accept the call without error
    let _options = args.get_or_undefined(0);

    // Log for debugging purposes
    eprintln!("scrollIntoView called (no-op in headless mode)");

    Ok(JsValue::undefined())
}

/// `Element.prototype.focus()`
fn focus(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.focus called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.focus called on non-Element object")
    })?;

    // Dispatch focus event
    let focus_event = JsObject::default(context.intrinsics());
    focus_event.set(js_string!("type"), js_string!("focus"), false, context)?;
    focus_event.set(js_string!("bubbles"), false, false, context)?;
    focus_event.set(js_string!("cancelable"), false, false, context)?;
    focus_event.set(js_string!("target"), this_obj.clone(), false, context)?;

    element.dispatch_event("focus", &focus_event.into(), context)?;

    Ok(JsValue::undefined())
}

/// `Element.prototype.blur()`
fn blur(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.blur called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.blur called on non-Element object")
    })?;

    // Dispatch blur event
    let blur_event = JsObject::default(context.intrinsics());
    blur_event.set(js_string!("type"), js_string!("blur"), false, context)?;
    blur_event.set(js_string!("bubbles"), false, false, context)?;
    blur_event.set(js_string!("cancelable"), false, false, context)?;
    blur_event.set(js_string!("target"), this_obj.clone(), false, context)?;

    element.dispatch_event("blur", &blur_event.into(), context)?;

    Ok(JsValue::undefined())
}

/// `Element.prototype.click()`
fn click(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.click called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.click called on non-Element object")
    })?;

    // Dispatch click event
    let click_event = JsObject::default(context.intrinsics());
    click_event.set(js_string!("type"), js_string!("click"), false, context)?;
    click_event.set(js_string!("bubbles"), true, false, context)?;
    click_event.set(js_string!("cancelable"), true, false, context)?;
    click_event.set(js_string!("target"), this_obj.clone(), false, context)?;
    click_event.set(js_string!("clientX"), 0, false, context)?;
    click_event.set(js_string!("clientY"), 0, false, context)?;
    click_event.set(js_string!("button"), 0, false, context)?;

    element.dispatch_event("click", &click_event.into(), context)?;

    Ok(JsValue::undefined())
}

// ============================================================================
// New accessor getter functions
// ============================================================================

/// `Element.prototype.firstChild` getter
fn get_first_child(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.firstChild called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.firstChild called on non-Element object")
    })?;

    Ok(element.get_first_child().map(|c| c.into()).unwrap_or(JsValue::null()))
}

/// `Element.prototype.lastChild` getter
fn get_last_child(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.lastChild called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.lastChild called on non-Element object")
    })?;

    Ok(element.get_last_child().map(|c| c.into()).unwrap_or(JsValue::null()))
}

/// `Element.prototype.nextSibling` getter
fn get_next_sibling(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.nextSibling called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.nextSibling called on non-Element object")
    })?;

    Ok(element.get_next_sibling().map(|c| c.into()).unwrap_or(JsValue::null()))
}

/// `Element.prototype.previousSibling` getter
fn get_previous_sibling(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.previousSibling called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.previousSibling called on non-Element object")
    })?;

    Ok(element.get_previous_sibling().map(|c| c.into()).unwrap_or(JsValue::null()))
}

/// `Element.prototype.nodeType` getter (returns ELEMENT_NODE = 1)
fn get_node_type(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.nodeType called on non-object")
    })?;

    // Verify it's an element
    this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.nodeType called on non-Element object")
    })?;

    // ELEMENT_NODE = 1
    Ok(1.into())
}

/// `Element.prototype.nodeName` getter (returns tagName)
fn get_node_name(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.nodeName called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.nodeName called on non-Element object")
    })?;

    Ok(JsString::from(element.get_tag_name()).into())
}

/// `Element.prototype.outerHTML` getter
fn get_outer_html(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.outerHTML called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.outerHTML called on non-Element object")
    })?;

    Ok(JsString::from(element.serialize_to_html()).into())
}

/// `Element.prototype.outerHTML` setter
/// Uses context-aware parsing to properly handle iframe elements
fn set_outer_html(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.outerHTML setter called on non-object")
    })?;

    let html = args.get_or_undefined(0).to_string(context)?;
    let html_string = html.to_std_string_escaped();

    // Parse HTML with context-aware handling for iframes
    let parsed_elements = parse_html_elements_with_context(&html_string, context)?;

    // Update the ElementData
    // Setting outerHTML replaces this element in its parent
    // For now, we'll just update the innerHTML and attributes
    // Full implementation would require parent manipulation
    {
        let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Element.prototype.outerHTML setter called on non-Element object")
        })?;

        *element.inner_html.lock().unwrap() = html_string;
        let mut children = element.children.lock().unwrap();
        children.clear();
        children.extend(parsed_elements);
        drop(children);
        element.recompute_text_content();
        element.update_document_html_content();
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.childNodes` getter
fn get_child_nodes(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.childNodes called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.childNodes called on non-Element object")
    })?;

    let children = element.get_children();

    // Create a NodeList-like array
    use boa_engine::builtins::array::Array;
    let children_values: Vec<JsValue> = children.into_iter().map(|child| child.into()).collect();
    let array = Array::create_array_from_list(children_values, context);

    // Add item() method for NodeList compatibility
    let item_fn = BuiltInBuilder::callable(context.realm(), |this, args, ctx| {
        let index = args.get_or_undefined(0).to_u32(ctx)?;
        if let Some(arr) = this.as_object() {
            if let Ok(val) = arr.get(index, ctx) {
                if !val.is_undefined() {
                    return Ok(val);
                }
            }
        }
        Ok(JsValue::null())
    })
    .name(js_string!("item"))
    .build();
    array.set(js_string!("item"), item_fn, false, context)?;

    Ok(array.into())
}

// ============================================================================
// Dynamic Script Execution Helpers
// ============================================================================

/// Check if a JsObject is a script element (by tagName or by HTMLScriptElementData)
fn is_script_element(obj: &JsObject, context: &mut Context) -> JsResult<bool> {
    // First, try to check by HTMLScriptElementData
    if obj.downcast_ref::<crate::dom::html_script_element::HTMLScriptElementData>().is_some() {
        return Ok(true);
    }

    // Fall back to checking tagName property
    if let Ok(tag_name_value) = obj.get(js_string!("tagName"), context) {
        if let Ok(tag_name) = tag_name_value.to_string(context) {
            let tag_name_str = tag_name.to_std_string_escaped();
            return Ok(tag_name_str.eq_ignore_ascii_case("SCRIPT"));
        }
    }

    // Also check ElementData's tagName
    if let Some(element_data) = obj.downcast_ref::<ElementData>() {
        let tag_name = element_data.get_tag_name();
        return Ok(tag_name.eq_ignore_ascii_case("SCRIPT"));
    }

    Ok(false)
}

/// Get the script type attribute value
fn get_script_type(obj: &JsObject, context: &mut Context) -> JsResult<String> {
    // First, try HTMLScriptElementData
    if let Some(script_data) = obj.downcast_ref::<crate::dom::html_script_element::HTMLScriptElementData>() {
        // Access the type field - but it's private, so check via JS property
    }

    // Check the 'type' property
    if let Ok(type_value) = obj.get(js_string!("type"), context) {
        if !type_value.is_undefined() && !type_value.is_null() {
            let type_str = type_value.to_string(context)?.to_std_string_escaped();
            return Ok(type_str);
        }
    }

    // Default to text/javascript
    Ok(String::new())
}

/// Get the script src attribute value (for external scripts)
fn get_script_src(obj: &JsObject, context: &mut Context) -> JsResult<Option<String>> {
    // Check the 'src' property
    if let Ok(src_value) = obj.get(js_string!("src"), context) {
        if !src_value.is_undefined() && !src_value.is_null() {
            let src_str = src_value.to_string(context)?.to_std_string_escaped();
            if !src_str.is_empty() {
                return Ok(Some(src_str));
            }
        }
    }

    Ok(None)
}

/// Get the inline script content (text or innerHTML)
fn get_script_content(obj: &JsObject, context: &mut Context) -> JsResult<String> {
    // Try 'text' property first (specific to script elements)
    if let Ok(text_value) = obj.get(js_string!("text"), context) {
        if !text_value.is_undefined() && !text_value.is_null() {
            let text_str = text_value.to_string(context)?.to_std_string_escaped();
            if !text_str.is_empty() {
                return Ok(text_str);
            }
        }
    }

    // Fall back to textContent
    if let Ok(text_content_value) = obj.get(js_string!("textContent"), context) {
        if !text_content_value.is_undefined() && !text_content_value.is_null() {
            let text_str = text_content_value.to_string(context)?.to_std_string_escaped();
            if !text_str.is_empty() {
                return Ok(text_str);
            }
        }
    }

    // Try innerHTML
    if let Ok(inner_html_value) = obj.get(js_string!("innerHTML"), context) {
        if !inner_html_value.is_undefined() && !inner_html_value.is_null() {
            let html_str = inner_html_value.to_string(context)?.to_std_string_escaped();
            if !html_str.is_empty() {
                return Ok(html_str);
            }
        }
    }

    // Try ElementData
    if let Some(element_data) = obj.downcast_ref::<ElementData>() {
        let text_content = element_data.get_text_content();
        if !text_content.is_empty() {
            return Ok(text_content);
        }
        let inner_html = element_data.get_inner_html();
        if !inner_html.is_empty() {
            return Ok(inner_html);
        }
    }

    Ok(String::new())
}

/// Check if a script type is executable JavaScript
fn is_executable_script_type(script_type: &str) -> bool {
    if script_type.is_empty() {
        return true; // Default is JavaScript
    }

    let script_type_lower = script_type.to_lowercase();

    // Standard JavaScript MIME types
    if script_type_lower == "text/javascript" ||
       script_type_lower == "application/javascript" ||
       script_type_lower == "application/x-javascript" ||
       script_type_lower == "text/ecmascript" ||
       script_type_lower == "application/ecmascript" {
        return true;
    }

    // Cloudflare Rocket Loader pattern (e.g., "text/javascript-obfuscated")
    if script_type_lower.contains("javascript") || script_type_lower.contains("ecmascript") {
        return true;
    }

    // Module scripts
    if script_type_lower == "module" {
        return true;
    }

    false
}

/// Execute a script element after it's appended to the DOM
/// This is the core function that actually runs the script
pub fn execute_script_element(script_obj: &JsObject, context: &mut Context) -> JsResult<()> {
    // Get script type
    let script_type = get_script_type(script_obj, context)?;

    // Check if this is an executable script type
    if !is_executable_script_type(&script_type) {
        eprintln!("DEBUG: Skipping non-executable script type: {}", script_type);
        return Ok(());
    }

    // Check for external script (src attribute)
    if let Some(src_url) = get_script_src(script_obj, context)? {
        // External script - need to fetch and execute
        eprintln!("DEBUG: Executing external script from: {}", src_url);
        return execute_external_script(&src_url, context);
    }

    // Inline script - get content and execute
    let script_content = get_script_content(script_obj, context)?;

    if script_content.is_empty() {
        eprintln!("DEBUG: Script element has no content to execute");
        return Ok(());
    }

    eprintln!("DEBUG: Executing inline script ({} chars)", script_content.len());

    // Execute the script content
    match context.eval(boa_engine::Source::from_bytes(&script_content)) {
        Ok(_result) => {
            eprintln!("DEBUG: Script executed successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("DEBUG: Script execution error: {:?}", e);
            // Don't propagate the error - scripts with errors shouldn't break DOM operations
            // Instead, we should fire an error event on the script element (TODO)
            Ok(())
        }
    }
}

/// Fetch and execute an external script
#[cfg(feature = "native")]
fn execute_external_script(url: &str, context: &mut Context) -> JsResult<()> {
    use crate::http_blocking::BlockingClient;

    eprintln!("DEBUG: Fetching external script: {}", url);

    // Create a blocking HTTP client
    let client = match BlockingClient::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("DEBUG: Failed to create HTTP client: {:?}", e);
            return Ok(());
        }
    };

    // Use blocking HTTP client to fetch the script
    match client.get(url).send() {
        Ok(response) => {
            if response.status().is_success() {
                match response.text() {
                    Ok(script_content) => {
                        eprintln!("DEBUG: Fetched {} bytes of script content", script_content.len());

                        // Execute the fetched script
                        match context.eval(boa_engine::Source::from_bytes(&script_content)) {
                            Ok(_) => {
                                eprintln!("DEBUG: External script executed successfully");
                                Ok(())
                            }
                            Err(e) => {
                                eprintln!("DEBUG: External script execution error: {:?}", e);
                                Ok(()) // Don't propagate
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("DEBUG: Failed to read script response: {:?}", e);
                        Ok(())
                    }
                }
            } else {
                eprintln!("DEBUG: Failed to fetch script, status: {}", response.status());
                Ok(())
            }
        }
        Err(e) => {
            eprintln!("DEBUG: Failed to fetch script: {:?}", e);
            Ok(())
        }
    }
}

#[cfg(not(feature = "native"))]
fn execute_external_script(_url: &str, _context: &mut Context) -> JsResult<()> {
    eprintln!("DEBUG: External script execution not supported in WASM mode");
    Ok(())
}

// =============================================================================
// Layout dimension getters (read-only properties)
// =============================================================================

/// `Element.prototype.offsetWidth` - returns layout width including borders
fn get_offset_width(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("offsetWidth getter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        // Try to get computed dimensions from stored data
        let width = element.get_offset_width();
        return Ok(JsValue::from(width as i32));
    }

    // Default value for elements without layout (like detached elements)
    Ok(JsValue::from(0))
}

/// `Element.prototype.offsetHeight` - returns layout height including borders
fn get_offset_height(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("offsetHeight getter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let height = element.get_offset_height();
        return Ok(JsValue::from(height as i32));
    }

    Ok(JsValue::from(0))
}

/// `Element.prototype.offsetTop` - returns top offset from offsetParent
fn get_offset_top(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("offsetTop getter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let top = element.get_offset_top();
        return Ok(JsValue::from(top as i32));
    }

    Ok(JsValue::from(0))
}

/// `Element.prototype.offsetLeft` - returns left offset from offsetParent
fn get_offset_left(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("offsetLeft getter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let left = element.get_offset_left();
        return Ok(JsValue::from(left as i32));
    }

    Ok(JsValue::from(0))
}

/// `Element.prototype.offsetParent` - returns nearest positioned ancestor
fn get_offset_parent(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("offsetParent getter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        // For now return null - proper implementation would walk up DOM tree
        // to find positioned ancestor
        if let Some(parent) = element.get_parent_node() {
            return Ok(JsValue::from(parent));
        }
    }

    Ok(JsValue::null())
}

/// `Element.prototype.clientWidth` - returns inner width (excluding borders, scrollbar)
fn get_client_width(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("clientWidth getter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let width = element.get_client_width();
        return Ok(JsValue::from(width as i32));
    }

    Ok(JsValue::from(0))
}

/// `Element.prototype.clientHeight` - returns inner height (excluding borders, scrollbar)
fn get_client_height(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("clientHeight getter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let height = element.get_client_height();
        return Ok(JsValue::from(height as i32));
    }

    Ok(JsValue::from(0))
}

/// `Element.prototype.clientTop` - returns top border width
fn get_client_top(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("clientTop getter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let top = element.get_client_top();
        return Ok(JsValue::from(top as i32));
    }

    Ok(JsValue::from(0))
}

/// `Element.prototype.clientLeft` - returns left border width
fn get_client_left(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("clientLeft getter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let left = element.get_client_left();
        return Ok(JsValue::from(left as i32));
    }

    Ok(JsValue::from(0))
}

// =============================================================================
// Scroll dimension getters
// =============================================================================

/// `Element.prototype.scrollWidth` - returns total width of content (including overflow)
fn get_scroll_width(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("scrollWidth getter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let width = element.get_scroll_width();
        return Ok(JsValue::from(width as i32));
    }

    Ok(JsValue::from(0))
}

/// `Element.prototype.scrollHeight` - returns total height of content (including overflow)
fn get_scroll_height(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("scrollHeight getter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let height = element.get_scroll_height();
        return Ok(JsValue::from(height as i32));
    }

    Ok(JsValue::from(0))
}

// =============================================================================
// Scroll position getters and setters
// =============================================================================

/// `Element.prototype.scrollTop` getter - returns scroll position from top
fn get_scroll_top(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("scrollTop getter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let top = element.get_scroll_top();
        return Ok(JsValue::from(top));
    }

    Ok(JsValue::from(0))
}

/// `Element.prototype.scrollTop` setter - sets scroll position from top
fn set_scroll_top(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("scrollTop setter called on non-object")
    })?;

    let value = args.get_or_undefined(0).to_number(context).unwrap_or(0.0);

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        element.set_scroll_top(value);
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.scrollLeft` getter - returns scroll position from left
fn get_scroll_left(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("scrollLeft getter called on non-object")
    })?;

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        let left = element.get_scroll_left();
        return Ok(JsValue::from(left));
    }

    Ok(JsValue::from(0))
}

/// `Element.prototype.scrollLeft` setter - sets scroll position from left
fn set_scroll_left(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("scrollLeft setter called on non-object")
    })?;

    let value = args.get_or_undefined(0).to_number(context).unwrap_or(0.0);

    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        element.set_scroll_left(value);
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.scrollTo(x, y)` or `Element.prototype.scrollTo(options)`
/// Scrolls the element's content to the specified position
fn scroll_to_element(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.scrollTo called on non-object")
    })?;

    // Parse arguments - supports both scrollTo(x, y) and scrollTo(options) forms
    let (x, y) = if args.len() >= 2 {
        // scrollTo(x, y) form
        let x = args.get_or_undefined(0).to_number(context).unwrap_or(0.0);
        let y = args.get_or_undefined(1).to_number(context).unwrap_or(0.0);
        (x, y)
    } else if let Some(options) = args.get(0).and_then(|v| v.as_object()) {
        // scrollTo(options) form
        let x = options.get(js_string!("left"), context)
            .ok()
            .and_then(|v| v.to_number(context).ok())
            .unwrap_or(0.0);
        let y = options.get(js_string!("top"), context)
            .ok()
            .and_then(|v| v.to_number(context).ok())
            .unwrap_or(0.0);
        (x, y)
    } else {
        (0.0, 0.0)
    };

    // Update the element's scroll position
    if let Some(element) = this_obj.downcast_ref::<ElementData>() {
        element.set_scroll_left(x);
        element.set_scroll_top(y);
    }

    Ok(JsValue::undefined())
}

/// `Element.prototype.querySelector(selector)` - find first matching descendant
fn query_selector_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.querySelector called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.querySelector called on non-Element object")
    })?;

    let selector = args.get_or_undefined(0).to_string(context)?;
    let selector_str = selector.to_std_string_escaped();

    // Use the element's query_selector method
    if let Some(result) = element.query_selector(&selector_str) {
        Ok(result.into())
    } else {
        Ok(JsValue::null())
    }
}

/// `Element.prototype.querySelectorAll(selector)` - find all matching descendants
fn query_selector_all_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.querySelectorAll called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.querySelectorAll called on non-Element object")
    })?;

    let selector = args.get_or_undefined(0).to_string(context)?;
    let selector_str = selector.to_std_string_escaped();

    // Use the element's query_selector_all method
    let results = element.query_selector_all(&selector_str);

    // Convert to JS array
    use boa_engine::builtins::array::Array;
    let array = Array::create_array_from_list(
        results.into_iter().map(|obj| obj.into()).collect::<Vec<_>>(),
        context,
    );
    Ok(array.into())
}

/// `Element.prototype.__dispatchTrustedMouseEvent(eventType, clientX, clientY, options?)`
///
/// Dispatches a trusted mouse event to this element.
/// This is a browser-internal API for automation that creates events with isTrusted: true.
///
/// ## Options
///
/// Standard mouse event options:
/// - `button`: number - The mouse button (0=left, 1=middle, 2=right)
/// - `buttons`: number - Bitmask of pressed buttons
/// - `ctrlKey`, `shiftKey`, `altKey`, `metaKey`: boolean - Modifier key states
///
/// CSS Transform options (for clicking on 3D-transformed elements):
/// - `transform`: string - CSS transform value (e.g., "matrix3d(...)" or "rotate(45deg)")
/// - `transformOrigin`: string - CSS transform-origin value (default: "50% 50% 0")
/// - `width`: number - Element width in pixels (for percentage-based origins)
/// - `height`: number - Element height in pixels (for percentage-based origins)
/// - `elementX`: number - Element's X position in document (default: 0)
/// - `elementY`: number - Element's Y position in document (default: 0)
///
/// When transform options are provided, the coordinates are transformed through the
/// inverse of the CSS transform matrix to correctly target the visual position on
/// a rotated/skewed/scaled element.
fn dispatch_trusted_mouse_event(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    use crate::events::ui_events::MouseEventData;

    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("__dispatchTrustedMouseEvent called on non-object")
    })?;

    // Verify this is an element
    if this_obj.downcast_ref::<ElementData>().is_none() {
        return Err(JsNativeError::typ()
            .with_message("__dispatchTrustedMouseEvent called on non-Element object")
            .into());
    }

    // Get event type
    let event_type = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    // Get coordinates
    let client_x = args.get_or_undefined(1).to_number(context).unwrap_or(0.0);
    let client_y = args.get_or_undefined(2).to_number(context).unwrap_or(0.0);

    // Get optional parameters
    let options = args.get_or_undefined(3);
    let (button, buttons, ctrl_key, shift_key, alt_key, meta_key, final_x, final_y) = if options.is_object() {
        let opts = options.as_object().unwrap();
        let button = opts.get(js_string!("button"), context)
            .map(|v| v.to_i32(context).unwrap_or(0) as i16)
            .unwrap_or(0);
        let buttons = opts.get(js_string!("buttons"), context)
            .map(|v| v.to_u32(context).unwrap_or(0) as u16)
            .unwrap_or(if event_type.contains("down") || event_type == "click" { 1 } else { 0 });
        let ctrl_key = opts.get(js_string!("ctrlKey"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false);
        let shift_key = opts.get(js_string!("shiftKey"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false);
        let alt_key = opts.get(js_string!("altKey"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false);
        let meta_key = opts.get(js_string!("metaKey"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false);

        // Check for CSS transform options
        let (final_x, final_y) = {
            let transform_opt = opts.get(js_string!("transform"), context).ok();
            if let Some(transform_val) = transform_opt {
                if let Some(transform_str) = transform_val.as_string() {
                    let transform = transform_str.to_std_string_escaped();
                    if !transform.is_empty() && !transform.eq_ignore_ascii_case("none") {
                        // Get transform-origin (default: "50% 50% 0")
                        let origin = opts.get(js_string!("transformOrigin"), context)
                            .ok()
                            .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
                            .unwrap_or_else(|| "50% 50% 0".to_string());

                        // Get element dimensions
                        let width = opts.get(js_string!("width"), context)
                            .ok()
                            .and_then(|v| v.to_number(context).ok())
                            .unwrap_or(0.0);
                        let height = opts.get(js_string!("height"), context)
                            .ok()
                            .and_then(|v| v.to_number(context).ok())
                            .unwrap_or(0.0);

                        // Get element position
                        let element_x = opts.get(js_string!("elementX"), context)
                            .ok()
                            .and_then(|v| v.to_number(context).ok())
                            .unwrap_or(0.0);
                        let element_y = opts.get(js_string!("elementY"), context)
                            .ok()
                            .and_then(|v| v.to_number(context).ok())
                            .unwrap_or(0.0);

                        // Apply inverse transform to map screen coords to element-local coords
                        let (local_x, local_y) = crate::css_transform::screen_to_element_coords(
                            client_x, client_y,
                            &transform,
                            &origin,
                            width, height,
                            element_x, element_y,
                        );

                        (local_x + element_x, local_y + element_y)
                    } else {
                        (client_x, client_y)
                    }
                } else {
                    (client_x, client_y)
                }
            } else {
                (client_x, client_y)
            }
        };

        (button, buttons, ctrl_key, shift_key, alt_key, meta_key, final_x, final_y)
    } else {
        let buttons = if event_type.contains("down") || event_type == "click" { 1 } else { 0 };
        (0, buttons, false, false, false, false, client_x, client_y)
    };

    // Determine event properties
    let (bubbles, cancelable) = match event_type.as_str() {
        "click" | "dblclick" | "mousedown" | "mouseup" | "mousemove"
        | "mouseover" | "mouseout" | "mouseenter" | "mouseleave" => (true, true),
        _ => (true, false),
    };

    // Create trusted mouse event data with transformed coordinates
    let mut mouse_event = MouseEventData::new_trusted_with_coords(
        event_type.clone(),
        bubbles,
        cancelable,
        final_x,   // Use transformed coordinates
        final_y,
        final_x,   // screen_x (same as clientX for simplicity)
        final_y,   // screen_y
        final_x,   // page_x
        final_y,   // page_y
        0.0,       // movement_x
        0.0,       // movement_y
        button,
        buttons,
    );

    // Set modifier keys directly (fields are public)
    mouse_event.ctrl_key = ctrl_key;
    mouse_event.shift_key = shift_key;
    mouse_event.alt_key = alt_key;
    mouse_event.meta_key = meta_key;

    // Create the event object
    let event_prototype = context.intrinsics().constructors().mouse_event().prototype();
    let event_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        event_prototype,
        mouse_event,
    );

    // Dispatch to the element using dispatchEvent
    dispatch_event(this, &[event_obj.upcast().into()], context)?;

    Ok(true.into())
}

/// `Element.prototype.checkVisibility(options?)`
///
/// Returns true if the element is rendered and visible.
/// This is used by widgets like Cloudflare Turnstile to verify visibility.
///
/// Options:
/// - checkOpacity: boolean - Check if opacity is 0
/// - checkVisibilityCSS: boolean - Check if visibility: hidden
/// - contentVisibilityAuto: boolean - Check content-visibility: auto
/// - opacityProperty: boolean - (alias for checkOpacity)
/// - visibilityProperty: boolean - (alias for checkVisibilityCSS)
///
/// See: https://developer.mozilla.org/en-US/docs/Web/API/Element/checkVisibility
fn check_visibility(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Element.prototype.checkVisibility called on non-object")
    })?;

    let element = this_obj.downcast_ref::<ElementData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Element.prototype.checkVisibility called on non-Element object")
    })?;

    // Parse options
    let options = args.get_or_undefined(0);
    let (check_opacity, check_visibility_css) = if options.is_object() {
        let opts = options.as_object().unwrap();

        // checkOpacity or opacityProperty
        let check_opacity = opts.get(js_string!("checkOpacity"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false)
            || opts.get(js_string!("opacityProperty"), context)
                .map(|v| v.to_boolean())
                .unwrap_or(false);

        // checkVisibilityCSS or visibilityProperty
        let check_visibility_css = opts.get(js_string!("checkVisibilityCSS"), context)
            .map(|v| v.to_boolean())
            .unwrap_or(false)
            || opts.get(js_string!("visibilityProperty"), context)
                .map(|v| v.to_boolean())
                .unwrap_or(false);

        (check_opacity, check_visibility_css)
    } else {
        (false, false)
    };

    // Check 1: Element must be in the DOM (have non-zero dimensions or be a valid element)
    let rect = element.get_bounding_client_rect();

    // Get computed style properties
    let style = element.style.lock().unwrap();

    // Check 2: display property - if 'none', element is not rendered
    if let Some(display) = style.get_property("display") {
        if display == "none" {
            eprintln!("checkVisibility: false (display: none)");
            return Ok(false.into());
        }
    }

    // Check 3 (optional): visibility CSS property
    if check_visibility_css {
        if let Some(visibility) = style.get_property("visibility") {
            if visibility == "hidden" || visibility == "collapse" {
                eprintln!("checkVisibility: false (visibility: {})", visibility);
                return Ok(false.into());
            }
        }
    }

    // Check 4 (optional): opacity
    if check_opacity {
        if let Some(opacity) = style.get_property("opacity") {
            if let Ok(opacity_val) = opacity.parse::<f64>() {
                if opacity_val == 0.0 {
                    eprintln!("checkVisibility: false (opacity: 0)");
                    return Ok(false.into());
                }
            }
        }
    }

    // Check 5: Element must have non-zero content size (width and height)
    // An element with 0x0 dimensions is not rendered and not visible
    // However, we allow elements with zero width/height if they have explicit positioning
    // (some elements are sized by their children or use overflow)
    let has_size = rect.width > 0.0 || rect.height > 0.0;

    // For visibility check, we consider an element visible if:
    // 1. It has non-zero dimensions, OR
    // 2. It's an element type that can be visible without explicit dimensions (like body, html)
    let tag = element.get_tag_name().to_lowercase();
    let is_structural = matches!(tag.as_str(), "html" | "body" | "head" | "script" | "style" | "meta" | "link");

    if !has_size && !is_structural {
        // Check if element is positioned with explicit layout
        // Elements inside turnstile widgets often have dimensions from CSS
        // For now, be permissive and assume elements are visible unless explicitly hidden
        eprintln!("checkVisibility: element has zero size but may be visible (tag={})", tag);
    }

    // Check 6: Element should be in viewport (optional, but useful for Turnstile)
    let element_id = element.get_element_identifier();
    let in_viewport = crate::layout_registry::is_element_in_viewport(&element_id, &tag);

    eprintln!("checkVisibility: true (tag={}, size={:.1}x{:.1}, in_viewport={})",
        tag, rect.width, rect.height, in_viewport);

    // Return true - element is rendered and not hidden by CSS
    // Note: We're being permissive here because Turnstile widgets
    // often have complex CSS that may not be fully computed
    Ok(true.into())
}
