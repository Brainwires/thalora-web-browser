//! History Web API implementation for Boa
//!
//! Native implementation of History standard
//! https://html.spec.whatwg.org/#the-history-interface

use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder, json::Json},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute
};

/// Safe JSON.stringify via the global JSON object (no eval injection risk)
fn stringify_json_safe(value: &JsValue, context: &mut Context) -> JsResult<String> {
    let global = context.global_object();
    let json_obj = global.get(js_string!("JSON"), context)?;
    let json_obj = json_obj.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("JSON is not an object")
    })?;
    let stringify_fn = json_obj.get(js_string!("stringify"), context)?;
    let stringify_fn = stringify_fn.as_callable().ok_or_else(|| {
        JsNativeError::typ().with_message("JSON.stringify is not callable")
    })?;
    let result = stringify_fn.call(&JsValue::undefined(), &[value.clone()], context)?;
    Ok(result.to_string(context)?.to_std_string_escaped())
}
use crate::browser::location::LocationData;
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};

/// JavaScript `History` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct History;

impl IntrinsicObject for History {
    fn init(realm: &Realm) {
        let length_func = BuiltInBuilder::callable(realm, get_length)
            .name(js_string!("get length"))
            .build();

        let state_func = BuiltInBuilder::callable(realm, get_state)
            .name(js_string!("get state"))
            .build();

        let scroll_restoration_func = BuiltInBuilder::callable(realm, get_scroll_restoration)
            .name(js_string!("get scrollRestoration"))
            .build();

        let scroll_restoration_setter_func = BuiltInBuilder::callable(realm, set_scroll_restoration)
            .name(js_string!("set scrollRestoration"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("length"),
                Some(length_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("state"),
                Some(state_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("scrollRestoration"),
                Some(scroll_restoration_func),
                Some(scroll_restoration_setter_func),
                Attribute::CONFIGURABLE,
            )
            .method(back, js_string!("back"), 0)
            .method(forward, js_string!("forward"), 0)
            .method(go, js_string!("go"), 1)
            .method(push_state, js_string!("pushState"), 3)
            .method(replace_state, js_string!("replaceState"), 3)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for History {
    const NAME: JsString = StaticJsStrings::HISTORY;
}

impl BuiltInConstructor for History {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::history;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::history,
            context,
        )?;

        let history_data = HistoryData::new();

        let history = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            history_data,
        );

        Ok(history.into())
    }
}

/// Internal data for History objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct HistoryData {
    #[unsafe_ignore_trace]
    entries: Arc<Mutex<Vec<HistoryEntry>>>,
    #[unsafe_ignore_trace]
    current_index: Arc<Mutex<i32>>,
    #[unsafe_ignore_trace]
    scroll_restoration: Arc<Mutex<String>>,
}

#[derive(Debug, Clone)]
struct HistoryEntry {
    url: String,
    title: String,
    state: Option<String>, // JSON-serialized state
}

impl HistoryData {
    pub fn new() -> Self {
        let initial_entry = HistoryEntry {
            url: "about:blank".to_string(),
            title: "".to_string(),
            state: None,
        };

        Self {
            entries: Arc::new(Mutex::new(vec![initial_entry])),
            current_index: Arc::new(Mutex::new(0)),
            scroll_restoration: Arc::new(Mutex::new("auto".to_string())),
        }
    }

    pub fn get_length(&self) -> i32 {
        self.entries.lock().unwrap().len() as i32
    }

    pub fn get_current_state(&self) -> Option<String> {
        let entries = self.entries.lock().unwrap();
        let index = *self.current_index.lock().unwrap() as usize;

        if index < entries.len() {
            entries[index].state.clone()
        } else {
            None
        }
    }

    pub fn get_scroll_restoration(&self) -> String {
        self.scroll_restoration.lock().unwrap().clone()
    }

    pub fn set_scroll_restoration(&self, value: String) {
        let normalized = if value == "manual" { "manual" } else { "auto" };
        *self.scroll_restoration.lock().unwrap() = normalized.to_string();
    }

    pub fn back(&self) -> bool {
        let mut index = self.current_index.lock().unwrap();
        if *index > 0 {
            *index -= 1;
            true
        } else {
            false
        }
    }

    pub fn forward(&self) -> bool {
        let mut index = self.current_index.lock().unwrap();
        let length = self.entries.lock().unwrap().len() as i32;
        if *index < length - 1 {
            *index += 1;
            true
        } else {
            false
        }
    }

    pub fn go(&self, delta: i32) -> bool {
        let mut index = self.current_index.lock().unwrap();
        let length = self.entries.lock().unwrap().len() as i32;
        let new_index = *index + delta;

        if new_index >= 0 && new_index < length {
            *index = new_index;
            true
        } else {
            false
        }
    }

    pub fn push_state(&self, state: Option<String>, title: String, url: Option<String>) {
        let mut entries = self.entries.lock().unwrap();
        let mut index = self.current_index.lock().unwrap();

        // Get current URL if not provided
        let current_url = if *index >= 0 && (*index as usize) < entries.len() {
            entries[*index as usize].url.clone()
        } else {
            "about:blank".to_string()
        };

        let new_url = url.unwrap_or(current_url);

        let new_entry = HistoryEntry {
            url: new_url,
            title,
            state,
        };

        // Remove any entries after current index
        entries.truncate(*index as usize + 1);

        // Add new entry
        entries.push(new_entry);
        *index += 1;
    }

    pub fn replace_state(&self, state: Option<String>, title: String, url: Option<String>) {
        let mut entries = self.entries.lock().unwrap();
        let index = *self.current_index.lock().unwrap();

        if index >= 0 && (index as usize) < entries.len() {
            let current_entry = &mut entries[index as usize];

            current_entry.state = state;
            current_entry.title = title;

            if let Some(new_url) = url {
                current_entry.url = new_url;
            }
        }
    }

    pub fn get_current_url(&self) -> String {
        let entries = self.entries.lock().unwrap();
        let index = *self.current_index.lock().unwrap() as usize;

        if index < entries.len() {
            entries[index].url.clone()
        } else {
            "about:blank".to_string()
        }
    }

    pub fn set_current_url(&self, url: String) {
        let mut entries = self.entries.lock().unwrap();
        let index = *self.current_index.lock().unwrap() as usize;

        if index < entries.len() {
            entries[index].url = url;
        }
    }
}

/// `History.prototype.length` getter
fn get_length(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.length called on non-object")
    })?;

    let value = {
            let history = this_obj.downcast_ref::<HistoryData>().ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("History.prototype.length called on non-History object")
            })?;
            history.get_length()
        };
    Ok(JsValue::from(value))
}

/// `History.prototype.state` getter
fn get_state(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.state called on non-object")
    })?;

    let history = this_obj.downcast_ref::<HistoryData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("History.prototype.state called on non-History object")
    })?;

    if let Some(state_json) = history.get_current_state() {
        // Parse JSON state using Boa's JSON intrinsic directly (no eval injection risk)
        match Json::parse(&JsValue::undefined(), &[JsValue::from(js_string!(state_json))], context) {
            Ok(value) => Ok(value),
            Err(_) => Ok(JsValue::null()),
        }
    } else {
        Ok(JsValue::null())
    }
}

/// `History.prototype.scrollRestoration` getter
fn get_scroll_restoration(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.scrollRestoration called on non-object")
    })?;

    let value = {
            let history = this_obj.downcast_ref::<HistoryData>().ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("History.prototype.scrollRestoration called on non-History object")
            })?;
            history.get_scroll_restoration()
        };
    Ok(JsString::from(value).into())
}

/// `History.prototype.scrollRestoration` setter
fn set_scroll_restoration(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.scrollRestoration setter called on non-object")
    })?;

    let history = this_obj.downcast_ref::<HistoryData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("History.prototype.scrollRestoration setter called on non-History object")
    })?;

    let value = args.get_or_undefined(0).to_string(context)?;
    history.set_scroll_restoration(value.to_std_string_escaped());
    Ok(JsValue::undefined())
}

/// `History.prototype.back()`
fn back(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.back called on non-object")
    })?;

    let history = this_obj.downcast_ref::<HistoryData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("History.prototype.back called on non-History object")
    })?;

    history.back();
    // In a real implementation, this would trigger pageswap event and navigation
    Ok(JsValue::undefined())
}

/// `History.prototype.forward()`
fn forward(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.forward called on non-object")
    })?;

    let history = this_obj.downcast_ref::<HistoryData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("History.prototype.forward called on non-History object")
    })?;

    history.forward();
    // In a real implementation, this would trigger pageswap event and navigation
    Ok(JsValue::undefined())
}

/// `History.prototype.go(delta)`
fn go(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.go called on non-object")
    })?;

    let history = this_obj.downcast_ref::<HistoryData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("History.prototype.go called on non-History object")
    })?;

    let delta = args.get_or_undefined(0).to_i32(context)?;
    history.go(delta);
    // In a real implementation, this would trigger pageswap event and navigation
    Ok(JsValue::undefined())
}

/// `History.prototype.pushState(state, title, url)`
fn push_state(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.pushState called on non-object")
    })?;

    let history = this_obj.downcast_ref::<HistoryData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("History.prototype.pushState called on non-History object")
    })?;

    let state = args.get_or_undefined(0);
    let title = args.get_or_undefined(1).to_string(context)?;
    let url = args.get(2);

    // Serialize state to JSON if not null/undefined
    let state_json = if state.is_null() || state.is_undefined() {
        None
    } else {
        // Serialize via JSON.stringify intrinsic (no eval injection risk)
        match stringify_json_safe(state, context) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    };

    let url_string = if let Some(url_val) = url {
        if !url_val.is_null() && !url_val.is_undefined() {
            Some(url_val.to_string(context)?.to_std_string_escaped())
        } else {
            None
        }
    } else {
        None
    };

    history.push_state(state_json, title.to_std_string_escaped(), url_string.clone());

    // Update window.location to reflect the new URL (per WHATWG spec)
    if let Some(ref new_url) = url_string {
        update_window_location(new_url, context);
    }

    Ok(JsValue::undefined())
}

/// `History.prototype.replaceState(state, title, url)`
fn replace_state(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("History.prototype.replaceState called on non-object")
    })?;

    let history = this_obj.downcast_ref::<HistoryData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("History.prototype.replaceState called on non-History object")
    })?;

    let state = args.get_or_undefined(0);
    let title = args.get_or_undefined(1).to_string(context)?;
    let url = args.get(2);

    // Serialize state to JSON if not null/undefined
    let state_json = if state.is_null() || state.is_undefined() {
        None
    } else {
        // Serialize via JSON.stringify intrinsic (no eval injection risk)
        match stringify_json_safe(state, context) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    };

    let url_string = if let Some(url_val) = url {
        if !url_val.is_null() && !url_val.is_undefined() {
            Some(url_val.to_string(context)?.to_std_string_escaped())
        } else {
            None
        }
    } else {
        None
    };

    history.replace_state(state_json, title.to_std_string_escaped(), url_string.clone());

    // Update window.location to reflect the new URL (per WHATWG spec)
    if let Some(ref new_url) = url_string {
        update_window_location(new_url, context);
    }

    Ok(JsValue::undefined())
}

/// Resolve a URL passed to pushState/replaceState against the current location.
/// Per WHATWG spec, the URL must be same-origin. Absolute paths are resolved
/// against the current origin; relative paths against the current URL directory.
fn resolve_pushstate_url(current_href: &str, new_url: &str) -> String {
    // Already absolute URL
    if new_url.starts_with("http://") || new_url.starts_with("https://") {
        return new_url.to_string();
    }

    // Parse current URL to get origin
    if let Some(protocol_end) = current_href.find("://") {
        let rest = &current_href[protocol_end + 3..];
        let host_end = rest.find('/').unwrap_or(rest.len());
        let origin = &current_href[..protocol_end + 3 + host_end];

        if new_url.starts_with('/') {
            // Absolute path — prepend origin
            format!("{}{}", origin, new_url)
        } else {
            // Relative path — resolve against current URL's directory
            let current_path_end = current_href.rfind('/').unwrap_or(origin.len());
            let base = &current_href[..current_path_end + 1];
            format!("{}{}", base, new_url)
        }
    } else {
        new_url.to_string()
    }
}

/// Update window.location.href after pushState/replaceState changes the URL.
/// Accesses the global object to find the location and update its href.
fn update_window_location(new_url: &str, context: &mut Context) {
    let global = context.global_object().clone();
    if let Ok(location_val) = global.get(js_string!("location"), context) {
        if let Some(location_obj) = location_val.as_object() {
            if let Some(location_data) = location_obj.downcast_ref::<LocationData>() {
                let current_href = location_data.get_href();
                let resolved = resolve_pushstate_url(&current_href, new_url);
                location_data.set_href(&resolved);
            }
        }
    }
}