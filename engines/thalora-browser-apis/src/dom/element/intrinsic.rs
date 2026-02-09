//! Element intrinsic object registration (Boa builtins)

use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsData, JsResult, js_string,
    JsString, realm::Realm, property::Attribute
};

use super::types::ElementData;
use super::properties::*;
use super::dom_manipulation::*;
use super::layout::*;
use super::query_and_events::*;
use super::scripts::*;
use super::automation::*;

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

        let dataset_func = BuiltInBuilder::callable(realm, get_dataset)
            .name(js_string!("get dataset"))
            .build();

        let attributes_func = BuiltInBuilder::callable(realm, get_attributes)
            .name(js_string!("get attributes"))
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

        // Element-only traversal accessors
        let next_element_sibling_func = BuiltInBuilder::callable(realm, get_next_element_sibling)
            .name(js_string!("get nextElementSibling"))
            .build();

        let previous_element_sibling_func = BuiltInBuilder::callable(realm, get_previous_element_sibling)
            .name(js_string!("get previousElementSibling"))
            .build();

        let first_element_child_func = BuiltInBuilder::callable(realm, get_first_element_child)
            .name(js_string!("get firstElementChild"))
            .build();

        let last_element_child_func = BuiltInBuilder::callable(realm, get_last_element_child)
            .name(js_string!("get lastElementChild"))
            .build();

        let child_element_count_func = BuiltInBuilder::callable(realm, get_child_element_count)
            .name(js_string!("get childElementCount"))
            .build();

        let parent_element_func = BuiltInBuilder::callable(realm, get_parent_element)
            .name(js_string!("get parentElement"))
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
                js_string!("dataset"),
                Some(dataset_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("attributes"),
                Some(attributes_func),
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
            // Element-only traversal accessors
            .accessor(
                js_string!("nextElementSibling"),
                Some(next_element_sibling_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("previousElementSibling"),
                Some(previous_element_sibling_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("firstElementChild"),
                Some(first_element_child_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("lastElementChild"),
                Some(last_element_child_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("childElementCount"),
                Some(child_element_count_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("parentElement"),
                Some(parent_element_func),
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
            .method(set_attribute_js, js_string!("setAttribute"), 2)
            .method(get_attribute_js, js_string!("getAttribute"), 1)
            .method(has_attribute_js, js_string!("hasAttribute"), 1)
            .method(remove_attribute_js, js_string!("removeAttribute"), 1)
            .method(get_attribute_names, js_string!("getAttributeNames"), 0)
            .method(toggle_attribute, js_string!("toggleAttribute"), 1)
            .method(insert_adjacent_html, js_string!("insertAdjacentHTML"), 2)
            .method(insert_adjacent_element, js_string!("insertAdjacentElement"), 2)
            .method(get_elements_by_class_name, js_string!("getElementsByClassName"), 1)
            .method(get_elements_by_tag_name, js_string!("getElementsByTagName"), 1)
            .method(append_child, js_string!("appendChild"), 1)
            .method(remove_child, js_string!("removeChild"), 1)
            .method(insert_before_js, js_string!("insertBefore"), 2)
            .method(replace_child_js, js_string!("replaceChild"), 2)
            .method(clone_node, js_string!("cloneNode"), 1)
            .method(contains_js, js_string!("contains"), 1)
            .method(closest_js, js_string!("closest"), 1)
            .method(matches_js, js_string!("matches"), 1)
            .method(get_bounding_client_rect_js, js_string!("getBoundingClientRect"), 0)
            .method(scroll_into_view, js_string!("scrollIntoView"), 1)
            .method(focus, js_string!("focus"), 0)
            .method(blur, js_string!("blur"), 0)
            .method(click, js_string!("click"), 0)
            .method(set_html, js_string!("setHTML"), 1)
            .method(set_html_unsafe, js_string!("setHTMLUnsafe"), 1)
            // EventTarget methods - CRITICAL for form automation
            .method(add_event_listener_js, js_string!("addEventListener"), 2)
            .method(remove_event_listener_js, js_string!("removeEventListener"), 2)
            .method(dispatch_event_js, js_string!("dispatchEvent"), 1)
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
    const PROTOTYPE_STORAGE_SLOTS: usize = 150;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::element;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
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

        Ok(element.upcast().into())
    }
}
