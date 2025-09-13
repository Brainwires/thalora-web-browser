use anyhow::{anyhow, Result};
use boa_engine::{Context, JsObject, JsValue, NativeFunction};
use scraper::{Html, Selector, ElementRef};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct EnhancedDom {
    document: Html,
    element_cache: Arc<Mutex<HashMap<String, ElementRef<'static>>>>,
    event_listeners: Arc<Mutex<HashMap<String, Vec<EventListener>>>>,
    next_element_id: Arc<Mutex<u32>>,
}

#[derive(Debug, Clone)]
struct EventListener {
    event_type: String,
    callback: JsValue,
    element_id: String,
}

#[derive(Debug, Clone)]
pub struct DomElement {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub text_content: String,
    pub inner_html: String,
    pub children: Vec<DomElement>,
    pub id: String,
}

impl EnhancedDom {
    pub fn new(html_content: &str) -> Result<Self> {
        let document = Html::parse_document(html_content);
        
        Ok(Self {
            document,
            element_cache: Arc::new(Mutex::new(HashMap::new())),
            event_listeners: Arc::new(Mutex::new(HashMap::new())),
            next_element_id: Arc::new(Mutex::new(1)),
        })
    }

    pub fn setup_dom_globals(&self, context: &mut Context) -> Result<()> {
        let document_obj = JsObject::default();
        
        // document.getElementById
        let get_element_by_id_fn = NativeFunction::from_fn_ptr(move |_, args, context| {
            if args.is_empty() {
                return Ok(JsValue::null());
            }

            let id = args[0].to_string(context)?.to_std_string_escaped();
            
            // Create mock element object
            let element = Self::create_element_object(&id, "div", context)?;
            Ok(JsValue::from(element))
        });
        document_obj.set("getElementById", get_element_by_id_fn, false, context)?;

        // document.querySelector
        let query_selector_fn = NativeFunction::from_fn_ptr(move |_, args, context| {
            if args.is_empty() {
                return Ok(JsValue::null());
            }

            let selector = args[0].to_string(context)?.to_std_string_escaped();
            
            // Create mock element based on selector
            let element_id = format!("element_{}", selector.replace(&['#', '.', ' '][..], "_"));
            let element = Self::create_element_object(&element_id, "div", context)?;
            Ok(JsValue::from(element))
        });
        document_obj.set("querySelector", query_selector_fn, false, context)?;

        // document.querySelectorAll
        let query_selector_all_fn = NativeFunction::from_fn_ptr(move |_, args, context| {
            if args.is_empty() {
                return Ok(JsValue::from(context.construct_array(&[]).unwrap()));
            }

            let selector = args[0].to_string(context)?.to_std_string_escaped();
            
            // Create mock NodeList
            let elements = vec![
                Self::create_element_object(&format!("el1_{}", selector), "div", context)?,
                Self::create_element_object(&format!("el2_{}", selector), "div", context)?,
            ];
            
            let js_elements: Vec<JsValue> = elements.into_iter().map(JsValue::from).collect();
            Ok(JsValue::from(context.construct_array(&js_elements).unwrap()))
        });
        document_obj.set("querySelectorAll", query_selector_all_fn, false, context)?;

        // document.createElement
        let create_element_fn = NativeFunction::from_fn_ptr(move |_, args, context| {
            let tag_name = if args.is_empty() {
                "div".to_string()
            } else {
                args[0].to_string(context)?.to_std_string_escaped()
            };

            let element_id = format!("created_{}", tag_name);
            let element = Self::create_element_object(&element_id, &tag_name, context)?;
            Ok(JsValue::from(element))
        });
        document_obj.set("createElement", create_element_fn, false, context)?;

        // document.body
        let body_element = Self::create_element_object("body", "body", context)?;
        document_obj.set("body", body_element, false, context)?;

        // document.head
        let head_element = Self::create_element_object("head", "head", context)?;
        document_obj.set("head", head_element, false, context)?;

        context.register_global_property("document", document_obj, boa_engine::property::Attribute::all())?;

        Ok(())
    }

    fn create_element_object(id: &str, tag_name: &str, context: &mut Context) -> Result<JsObject, boa_engine::JsError> {
        let element = JsObject::default();
        
        // Basic properties
        element.set("id", JsValue::from(id), true, context)?;
        element.set("tagName", JsValue::from(tag_name.to_uppercase()), false, context)?;
        element.set("nodeName", JsValue::from(tag_name.to_uppercase()), false, context)?;
        element.set("innerHTML", JsValue::from(""), true, context)?;
        element.set("textContent", JsValue::from(""), true, context)?;
        element.set("className", JsValue::from(""), true, context)?;

        // Style object
        let style_obj = JsObject::default();
        element.set("style", style_obj, false, context)?;

        // getAttribute method
        let get_attribute_fn = NativeFunction::from_fn_ptr(move |this, args, context| {
            if args.is_empty() {
                return Ok(JsValue::null());
            }
            
            let attr_name = args[0].to_string(context)?.to_std_string_escaped();
            
            // Try to get from element's attributes
            if let Some(this_obj) = this.as_object() {
                if let Ok(value) = this_obj.get(&attr_name, context) {
                    if !value.is_undefined() {
                        return Ok(value);
                    }
                }
            }
            
            Ok(JsValue::null())
        });
        element.set("getAttribute", get_attribute_fn, false, context)?;

        // setAttribute method
        let set_attribute_fn = NativeFunction::from_fn_ptr(move |this, args, context| {
            if args.len() < 2 {
                return Ok(JsValue::undefined());
            }
            
            let attr_name = args[0].to_string(context)?.to_std_string_escaped();
            let attr_value = args[1].to_string(context)?.to_std_string_escaped();
            
            if let Some(this_obj) = this.as_object() {
                this_obj.set(&attr_name, JsValue::from(attr_value), true, context)?;
            }
            
            Ok(JsValue::undefined())
        });
        element.set("setAttribute", set_attribute_fn, false, context)?;

        // addEventListener method
        let add_event_listener_fn = NativeFunction::from_fn_ptr(move |this, args, context| {
            if args.len() < 2 {
                return Ok(JsValue::undefined());
            }
            
            let event_type = args[0].to_string(context)?.to_std_string_escaped();
            let callback = args[1].clone();
            
            // Store event listener (simplified)
            tracing::debug!("Event listener added: {} on element", event_type);
            
            Ok(JsValue::undefined())
        });
        element.set("addEventListener", add_event_listener_fn, false, context)?;

        // appendChild method
        let append_child_fn = NativeFunction::from_fn_ptr(move |this, args, context| {
            if args.is_empty() {
                return Ok(JsValue::undefined());
            }
            
            let child = args[0].clone();
            tracing::debug!("Child element appended");
            
            Ok(child)
        });
        element.set("appendChild", append_child_fn, false, context)?;

        // querySelector method for elements
        let element_query_selector_fn = NativeFunction::from_fn_ptr(move |this, args, context| {
            if args.is_empty() {
                return Ok(JsValue::null());
            }
            
            let selector = args[0].to_string(context)?.to_std_string_escaped();
            let child_element = Self::create_element_object(&format!("child_{}", selector), "div", context)?;
            Ok(JsValue::from(child_element))
        });
        element.set("querySelector", element_query_selector_fn, false, context)?;

        Ok(element)
    }

    pub fn extract_enhanced_content(&self, selector: Option<&str>) -> Result<DomElement> {
        let root_selector = selector.unwrap_or("body");
        let selector_obj = Selector::parse(root_selector)
            .map_err(|e| anyhow!("Invalid CSS selector: {}", e))?;

        if let Some(element_ref) = self.document.select(&selector_obj).next() {
            Ok(self.element_to_dom_element(element_ref))
        } else {
            // Return empty body if selector not found
            Ok(DomElement {
                tag_name: "body".to_string(),
                attributes: HashMap::new(),
                text_content: String::new(),
                inner_html: String::new(),
                children: Vec::new(),
                id: "body".to_string(),
            })
        }
    }

    fn element_to_dom_element(&self, element: ElementRef) -> DomElement {
        let tag_name = element.value().name().to_string();
        let mut attributes = HashMap::new();

        for (name, value) in element.value().attrs() {
            attributes.insert(name.to_string(), value.to_string());
        }

        let id = attributes.get("id").cloned().unwrap_or_else(|| {
            format!("element_{}", attributes.get("class").unwrap_or(&tag_name))
        });

        let text_content = element.text().collect::<Vec<_>>().join("");
        let inner_html = element.inner_html();

        let children = element
            .children()
            .filter_map(|child| {
                if let Some(child_element) = ElementRef::wrap(child) {
                    Some(self.element_to_dom_element(child_element))
                } else {
                    None
                }
            })
            .collect();

        DomElement {
            tag_name,
            attributes,
            text_content,
            inner_html,
            children,
            id,
        }
    }

    pub fn simulate_mutations(&mut self, js_code: &str) -> Result<Vec<DomMutation>> {
        let mut mutations = Vec::new();

        // Detect common DOM manipulation patterns
        if js_code.contains("appendChild") {
            mutations.push(DomMutation::ChildAdded {
                parent_id: "body".to_string(),
                child_element: DomElement {
                    tag_name: "div".to_string(),
                    attributes: HashMap::new(),
                    text_content: "Dynamically added content".to_string(),
                    inner_html: "Dynamically added content".to_string(),
                    children: Vec::new(),
                    id: "dynamic_element".to_string(),
                },
            });
        }

        if js_code.contains("innerHTML") || js_code.contains("textContent") {
            mutations.push(DomMutation::ContentChanged {
                element_id: "main".to_string(),
                new_content: "Content modified by JavaScript".to_string(),
            });
        }

        if js_code.contains("className") || js_code.contains("classList") {
            mutations.push(DomMutation::AttributeChanged {
                element_id: "main".to_string(),
                attribute_name: "class".to_string(),
                new_value: "dynamic-class".to_string(),
            });
        }

        Ok(mutations)
    }
}

#[derive(Debug, Clone)]
pub enum DomMutation {
    ChildAdded {
        parent_id: String,
        child_element: DomElement,
    },
    ChildRemoved {
        parent_id: String,
        child_id: String,
    },
    AttributeChanged {
        element_id: String,
        attribute_name: String,
        new_value: String,
    },
    ContentChanged {
        element_id: String,
        new_content: String,
    },
}

pub struct WebStorage {
    local_storage: Arc<Mutex<HashMap<String, String>>>,
    session_storage: Arc<Mutex<HashMap<String, String>>>,
}

impl WebStorage {
    pub fn new() -> Self {
        Self {
            local_storage: Arc::new(Mutex::new(HashMap::new())),
            session_storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn setup_storage_globals(&self, context: &mut Context) -> Result<()> {
        let local_storage = self.create_storage_object(self.local_storage.clone(), context)?;
        let session_storage = self.create_storage_object(self.session_storage.clone(), context)?;

        context.register_global_property("localStorage", local_storage, boa_engine::property::Attribute::all())?;
        context.register_global_property("sessionStorage", session_storage, boa_engine::property::Attribute::all())?;

        Ok(())
    }

    fn create_storage_object(
        &self,
        storage: Arc<Mutex<HashMap<String, String>>>,
        context: &mut Context,
    ) -> Result<JsObject, boa_engine::JsError> {
        let storage_obj = JsObject::default();

        // getItem method
        let storage_clone = storage.clone();
        let get_item_fn = NativeFunction::from_fn_ptr(move |_, args, context| {
            if args.is_empty() {
                return Ok(JsValue::null());
            }

            let key = args[0].to_string(context)?.to_std_string_escaped();
            let storage_guard = storage_clone.lock().unwrap();
            
            match storage_guard.get(&key) {
                Some(value) => Ok(JsValue::from(value.clone())),
                None => Ok(JsValue::null()),
            }
        });
        storage_obj.set("getItem", get_item_fn, false, context)?;

        // setItem method
        let storage_clone = storage.clone();
        let set_item_fn = NativeFunction::from_fn_ptr(move |_, args, context| {
            if args.len() < 2 {
                return Ok(JsValue::undefined());
            }

            let key = args[0].to_string(context)?.to_std_string_escaped();
            let value = args[1].to_string(context)?.to_std_string_escaped();
            
            let mut storage_guard = storage_clone.lock().unwrap();
            storage_guard.insert(key, value);
            
            Ok(JsValue::undefined())
        });
        storage_obj.set("setItem", set_item_fn, false, context)?;

        // removeItem method
        let storage_clone = storage.clone();
        let remove_item_fn = NativeFunction::from_fn_ptr(move |_, args, context| {
            if args.is_empty() {
                return Ok(JsValue::undefined());
            }

            let key = args[0].to_string(context)?.to_std_string_escaped();
            let mut storage_guard = storage_clone.lock().unwrap();
            storage_guard.remove(&key);
            
            Ok(JsValue::undefined())
        });
        storage_obj.set("removeItem", remove_item_fn, false, context)?;

        // clear method
        let storage_clone = storage.clone();
        let clear_fn = NativeFunction::from_fn_ptr(move |_, _, _| {
            let mut storage_guard = storage_clone.lock().unwrap();
            storage_guard.clear();
            Ok(JsValue::undefined())
        });
        storage_obj.set("clear", clear_fn, false, context)?;

        Ok(storage_obj)
    }
}