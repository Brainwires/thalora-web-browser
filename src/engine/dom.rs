use anyhow::{anyhow, Result};
use scraper::{Html, Selector, ElementRef};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::apis::events::EventListener;

#[derive(Debug, Clone)]
pub struct DomElement {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub text_content: String,
    pub inner_html: String,
    pub children: Vec<DomElement>,
    pub id: String,
}

#[derive(Debug, Clone)]
pub enum DomMutation {
    ContentChanged { element_id: String, new_content: String },
    AttributeChanged { element_id: String, attribute: String, new_value: String },
    ElementAdded { element_id: String, parent_id: String, element: DomElement },
    ElementRemoved { element_id: String },
}

pub struct EnhancedDom {
    document: Html,
    element_cache: Arc<Mutex<HashMap<String, ElementRef<'static>>>>,
    event_listeners: Arc<Mutex<HashMap<String, Vec<EventListener>>>>,
    next_element_id: Arc<Mutex<u32>>,
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
            mutations.push(DomMutation::ElementAdded {
                element_id: "new-element".to_string(),
                parent_id: "body".to_string(),
                element: DomElement {
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
                attribute: "class".to_string(),
                new_value: "dynamic-class".to_string(),
            });
        }

        Ok(mutations)
    }
}