//! IntrinsicObject, BuiltInObject, and BuiltInConstructor implementations for Document

use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsData, JsResult, js_string,
    JsString, realm::Realm, property::Attribute
};

use super::types::{Document, DocumentData};
use super::properties::*;
use super::creation::*;
use super::query::*;
use super::events::*;
use super::canvas::*;
use super::collections::*;

impl IntrinsicObject for Document {
    fn init(realm: &Realm) {
        let ready_state_func = BuiltInBuilder::callable(realm, get_ready_state)
            .name(js_string!("get readyState"))
            .build();

        let url_func = BuiltInBuilder::callable(realm, get_url)
            .name(js_string!("get URL"))
            .build();

        let title_func = BuiltInBuilder::callable(realm, get_title)
            .name(js_string!("get title"))
            .build();

        let title_setter_func = BuiltInBuilder::callable(realm, set_title)
            .name(js_string!("set title"))
            .build();

        let body_func = BuiltInBuilder::callable(realm, get_body)
            .name(js_string!("get body"))
            .build();

        let head_func = BuiltInBuilder::callable(realm, get_head)
            .name(js_string!("get head"))
            .build();

        let document_element_func = BuiltInBuilder::callable(realm, get_document_element)
            .name(js_string!("get documentElement"))
            .build();

        let forms_func = BuiltInBuilder::callable(realm, get_forms)
            .name(js_string!("get forms"))
            .build();

        let images_func = BuiltInBuilder::callable(realm, get_images)
            .name(js_string!("get images"))
            .build();

        let links_func = BuiltInBuilder::callable(realm, get_links)
            .name(js_string!("get links"))
            .build();

        let scripts_func = BuiltInBuilder::callable(realm, get_scripts)
            .name(js_string!("get scripts"))
            .build();

        let cookie_func = BuiltInBuilder::callable(realm, get_cookie)
            .name(js_string!("get cookie"))
            .build();

        let cookie_setter_func = BuiltInBuilder::callable(realm, set_cookie)
            .name(js_string!("set cookie"))
            .build();

        let referrer_func = BuiltInBuilder::callable(realm, get_referrer)
            .name(js_string!("get referrer"))
            .build();

        let domain_func = BuiltInBuilder::callable(realm, get_domain)
            .name(js_string!("get domain"))
            .build();

        let character_set_func = BuiltInBuilder::callable(realm, get_character_set)
            .name(js_string!("get characterSet"))
            .build();

        let content_type_func = BuiltInBuilder::callable(realm, get_content_type)
            .name(js_string!("get contentType"))
            .build();

        let visibility_state_func = BuiltInBuilder::callable(realm, get_visibility_state)
            .name(js_string!("get visibilityState"))
            .build();

        let hidden_func = BuiltInBuilder::callable(realm, get_hidden)
            .name(js_string!("get hidden"))
            .build();

        let active_element_func = BuiltInBuilder::callable(realm, get_active_element)
            .name(js_string!("get activeElement"))
            .build();

        let current_script_func = BuiltInBuilder::callable(realm, get_current_script)
            .name(js_string!("get currentScript"))
            .build();

        let scrolling_element_func = BuiltInBuilder::callable(realm, get_scrolling_element)
            .name(js_string!("get scrollingElement"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Set up prototype chain: Document -> Node -> EventTarget
            .inherits(Some(realm.intrinsics().constructors().node().prototype()))
            .accessor(
                js_string!("readyState"),
                Some(ready_state_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("URL"),
                Some(url_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("title"),
                Some(title_func),
                Some(title_setter_func),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("body"),
                Some(body_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("head"),
                Some(head_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("documentElement"),
                Some(document_element_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("forms"),
                Some(forms_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("images"),
                Some(images_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("links"),
                Some(links_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("scripts"),
                Some(scripts_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("cookie"),
                Some(cookie_func),
                Some(cookie_setter_func),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("referrer"),
                Some(referrer_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("domain"),
                Some(domain_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("characterSet"),
                Some(character_set_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("contentType"),
                Some(content_type_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("visibilityState"),
                Some(visibility_state_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("hidden"),
                Some(hidden_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("activeElement"),
                Some(active_element_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("currentScript"),
                Some(current_script_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("scrollingElement"),
                Some(scrolling_element_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(create_element, js_string!("createElement"), 1)
            .method(create_element_ns, js_string!("createElementNS"), 2)
            .method(create_text_node, js_string!("createTextNode"), 1)
            .method(create_document_fragment, js_string!("createDocumentFragment"), 0)
            .method(create_range, js_string!("createRange"), 0)
            .method(create_event, js_string!("createEvent"), 1)
            .method(get_element_by_id, js_string!("getElementById"), 1)
            .method(query_selector, js_string!("querySelector"), 1)
            .method(query_selector_all, js_string!("querySelectorAll"), 1)
            .method(add_event_listener, js_string!("addEventListener"), 2)
            .method(remove_event_listener, js_string!("removeEventListener"), 2)
            .method(dispatch_event, js_string!("dispatchEvent"), 1)
            .method(start_view_transition, js_string!("startViewTransition"), 0)
            // New DOM query methods
            .method(get_elements_by_class_name, js_string!("getElementsByClassName"), 1)
            .method(get_elements_by_tag_name, js_string!("getElementsByTagName"), 1)
            .method(get_elements_by_name, js_string!("getElementsByName"), 1)
            .method(create_comment, js_string!("createComment"), 1)
            .method(create_attribute, js_string!("createAttribute"), 1)
            .method(has_focus, js_string!("hasFocus"), 0)
            .method(exec_command, js_string!("execCommand"), 3)
            // DOM Traversal methods
            .method(create_tree_walker, js_string!("createTreeWalker"), 1)
            .method(create_node_iterator, js_string!("createNodeIterator"), 1)
            // CSSOM View methods (used by Cloudflare Turnstile for bot detection)
            .method(element_from_point, js_string!("elementFromPoint"), 2)
            .method(elements_from_point, js_string!("elementsFromPoint"), 2)
            // Scroll methods (Document delegates to Window)
            .method(scroll_to_document, js_string!("scrollTo"), 2)
            .method(scroll_to_document, js_string!("scroll"), 2)  // alias for scrollTo
            // Internal trusted event dispatch (for Cloudflare etc.)
            .method(dispatch_trusted_mouse_event_document, js_string!("__dispatchTrustedMouseEvent"), 3)
            // Static method: Document.parseHTMLUnsafe(html) - Chrome 124+
            .static_method(super::document_parse::parse_html_unsafe, js_string!("parseHTMLUnsafe"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Document {
    const NAME: JsString = StaticJsStrings::DOCUMENT;
}

impl BuiltInConstructor for Document {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 68; // Accessors and methods on prototype (added scrollTo, scroll, __dispatchTrustedMouseEvent)
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 2;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::document;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::document,
            context,
        )?;

        let document_data = DocumentData::new();

        let document = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            document_data,
        );

        Ok(document.into())
    }
}
