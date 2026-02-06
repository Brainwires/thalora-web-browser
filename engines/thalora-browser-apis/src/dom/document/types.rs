//! Document type definitions
//!
//! Contains the Document struct, ScriptEntry, and DocumentData types.

use boa_engine::{
    object::JsObject,
    JsData,
    string::StaticJsStrings,
    JsString,
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// JavaScript `Document` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Document;

/// Represents a loaded script element with all its attributes
/// This is used to track dynamically loaded scripts that need to be
/// visible in document.scripts and getElementsByTagName("script")
#[derive(Debug, Clone)]
pub struct ScriptEntry {
    pub src: Option<String>,
    pub script_type: Option<String>,
    pub async_: bool,
    pub defer: bool,
    pub text: String,
    pub attributes: HashMap<String, String>,
}

impl ScriptEntry {
    pub fn new() -> Self {
        Self {
            src: None,
            script_type: None,
            async_: false,
            defer: false,
            text: String::new(),
            attributes: HashMap::new(),
        }
    }

    /// Create a ScriptEntry with a source URL
    pub fn with_src(src: String) -> Self {
        Self {
            src: Some(src),
            script_type: None,
            async_: false,
            defer: false,
            text: String::new(),
            attributes: HashMap::new(),
        }
    }

    /// Set all attributes from an iterator (typically from HTML parsing)
    pub fn with_attributes<I, K, V>(mut self, attrs: I) -> Self
    where
        I: Iterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        for (key, value) in attrs {
            let key_str = key.as_ref().to_string();
            let value_str = value.as_ref().to_string();

            // Also set known fields from attributes
            match key_str.as_str() {
                "src" => self.src = Some(value_str.clone()),
                "type" => self.script_type = Some(value_str.clone()),
                "async" => self.async_ = true,
                "defer" => self.defer = true,
                _ => {}
            }

            self.attributes.insert(key_str, value_str);
        }
        self
    }
}

/// Internal data for Document objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct DocumentData {
    #[unsafe_ignore_trace]
    pub(super) ready_state: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub(super) url: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub(super) title: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub(super) cookie: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub(super) referrer: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub(super) domain: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub(super) character_set: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub(super) content_type: Arc<Mutex<String>>,
    #[unsafe_ignore_trace]
    pub(super) elements: Arc<Mutex<HashMap<String, JsObject>>>,
    #[unsafe_ignore_trace]
    pub(super) event_listeners: Arc<Mutex<HashMap<String, Vec<boa_engine::value::JsValue>>>>,
    #[unsafe_ignore_trace]
    pub(super) html_content: Arc<Mutex<String>>,
    /// Registry of loaded scripts (both from HTML and dynamically created)
    /// This allows document.scripts and getElementsByTagName("script") to find
    /// scripts that were executed but not statically present in the current HTML
    #[unsafe_ignore_trace]
    loaded_scripts: Arc<Mutex<Vec<ScriptEntry>>>,
}

impl DocumentData {
    pub(super) fn new() -> Self {
        let doc_data = Self {
            ready_state: Arc::new(Mutex::new("loading".to_string())),
            url: Arc::new(Mutex::new("about:blank".to_string())),
            title: Arc::new(Mutex::new("".to_string())),
            cookie: Arc::new(Mutex::new("".to_string())),
            referrer: Arc::new(Mutex::new("".to_string())),
            domain: Arc::new(Mutex::new("".to_string())),
            character_set: Arc::new(Mutex::new("UTF-8".to_string())),
            content_type: Arc::new(Mutex::new("text/html".to_string())),
            elements: Arc::new(Mutex::new(HashMap::new())),
            event_listeners: Arc::new(Mutex::new(HashMap::new())),
            html_content: Arc::new(Mutex::new("".to_string())),
            loaded_scripts: Arc::new(Mutex::new(Vec::new())),
        };

        // Set up DOM sync bridge - connect Element changes to Document updates
        use crate::dom::element::GLOBAL_DOM_SYNC;
        let html_content_ref = doc_data.html_content.clone();
        GLOBAL_DOM_SYNC.get_or_init(|| crate::dom::element::DomSync::new())
            .set_updater(Box::new(move |html| {
                *html_content_ref.lock().unwrap() = html.to_string();
            }));

        doc_data
    }

    pub fn set_ready_state(&self, state: &str) {
        *self.ready_state.lock().unwrap() = state.to_string();
    }

    pub fn set_url(&self, url: &str) {
        *self.url.lock().unwrap() = url.to_string();
    }

    pub fn set_title(&self, title: &str) {
        *self.title.lock().unwrap() = title.to_string();
    }

    pub fn set_html_content(&self, html: &str) {
        *self.html_content.lock().unwrap() = html.to_string();

        // Process all forms in the HTML and prepare them for DOM access
        self.process_forms_in_html(html);
    }

    pub fn update_html_from_dom(&self, html: &str) {
        *self.html_content.lock().unwrap() = html.to_string();
    }

    pub fn get_html_content(&self) -> String {
        self.html_content.lock().unwrap().clone()
    }

    pub fn get_ready_state(&self) -> String {
        self.ready_state.lock().unwrap().clone()
    }

    pub fn get_url(&self) -> String {
        self.url.lock().unwrap().clone()
    }

    pub fn get_title(&self) -> String {
        self.title.lock().unwrap().clone()
    }

    pub fn add_element(&self, id: String, element: JsObject) {
        self.elements.lock().unwrap().insert(id, element);
    }

    pub fn get_element(&self, id: &str) -> Option<JsObject> {
        self.elements.lock().unwrap().get(id).cloned()
    }

    /// Process all forms in HTML content and prepare elements collections
    /// This ensures that forms accessed via DOM events have proper elements collections
    fn process_forms_in_html(&self, html_content: &str) {
        eprintln!("🔍 DEBUG: process_forms_in_html called with {} characters of HTML", html_content.len());

        // Parse the HTML content to find all forms
        let document = scraper::Html::parse_document(html_content);

        // Find all form elements
        if let Ok(form_selector) = scraper::Selector::parse("form") {
            let form_count = document.select(&form_selector).count();
            eprintln!("🔍 DEBUG: Found {} forms in HTML", form_count);

            for (form_index, form_element) in document.select(&form_selector).enumerate() {
                // Create a unique ID for this form if it doesn't have one
                let form_id = if let Some(id) = form_element.value().attr("id") {
                    id.to_string()
                } else {
                    format!("auto_form_{}", form_index)
                };

                // Store form metadata for later DOM access
                let mut form_inputs = Vec::new();

                // Parse form's inner HTML to find input elements
                let form_inner_html = form_element.inner_html();
                let form_doc = scraper::Html::parse_fragment(&form_inner_html);

                if let Ok(input_selector) = scraper::Selector::parse("input") {
                    for input_element in form_doc.select(&input_selector) {
                        if let Some(input_name) = input_element.value().attr("name") {
                            let input_value = input_element.value().attr("value").unwrap_or("").to_string();
                            let input_type = input_element.value().attr("type").unwrap_or("text").to_string();

                            form_inputs.push((input_name.to_string(), input_value, input_type));
                        }
                    }
                }

                // Store the form metadata for later JavaScript access
                // We'll use this when DOM queries ask for this form
                self.add_form_metadata(form_id, form_inputs);
            }
        }
    }

    /// Add form metadata that can be used when creating form elements in JavaScript
    fn add_form_metadata(&self, form_id: String, inputs: Vec<(String, String, String)>) {
        // Create an HTMLFormElement with proper elements collection
        use crate::misc::form::{HTMLFormElement, HTMLInputElement, HTMLFormControlsCollection};
        use boa_engine::{Context, object::ObjectInitializer, js_string};

        // For now, store the metadata - we'll need a context to create the actual objects
        // This processing happens at document level so all forms are known before JavaScript queries them
        // TODO: This needs to be enhanced to create actual JavaScript objects when we have a context
        eprintln!("🔍 DEBUG: Found form '{}' with {} inputs", form_id, inputs.len());
        for (name, value, input_type) in &inputs {
            eprintln!("🔍 DEBUG: - Input '{}' = '{}' (type: {})", name, value, input_type);
        }
    }

    pub fn add_event_listener(&self, event_type: String, listener: boa_engine::value::JsValue) {
        self.event_listeners.lock().unwrap()
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(listener);
    }

    pub fn remove_event_listener(&self, event_type: &str, listener: &boa_engine::value::JsValue) {
        if let Some(listeners) = self.event_listeners.lock().unwrap().get_mut(event_type) {
            listeners.retain(|l| !boa_engine::value::JsValue::same_value(l, listener));
        }
    }

    pub fn get_event_listeners(&self, event_type: &str) -> Vec<boa_engine::value::JsValue> {
        self.event_listeners.lock().unwrap()
            .get(event_type)
            .cloned()
            .unwrap_or_default()
    }

    /// Register a script that has been loaded/executed
    /// This makes the script visible in document.scripts and getElementsByTagName("script")
    pub fn register_script(&self, entry: ScriptEntry) {
        self.loaded_scripts.lock().unwrap().push(entry);
    }

    /// Get all registered loaded scripts
    pub fn get_loaded_scripts(&self) -> Vec<ScriptEntry> {
        self.loaded_scripts.lock().unwrap().clone()
    }

    /// Clear all registered scripts (typically when navigating to a new page)
    pub fn clear_scripts(&self) {
        self.loaded_scripts.lock().unwrap().clear();
    }

    /// Get a reference to the loaded_scripts Arc for sharing with other components
    pub fn get_loaded_scripts_ref(&self) -> Arc<Mutex<Vec<ScriptEntry>>> {
        self.loaded_scripts.clone()
    }
}
