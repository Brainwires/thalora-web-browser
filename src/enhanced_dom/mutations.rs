use super::element::DomElement;

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
    StyleChanged {
        element_id: String,
        property: String,
        new_value: String,
    },
    ClassListChanged {
        element_id: String,
        action: ClassListAction,
        class_name: String,
    },
}

#[derive(Debug, Clone)]
pub enum ClassListAction {
    Add,
    Remove,
    Toggle,
}

impl DomMutation {
    pub fn apply_to_element(&self, element: &mut DomElement) -> bool {
        match self {
            DomMutation::AttributeChanged { element_id, attribute_name, new_value } => {
                if element.id == *element_id {
                    element.attributes.insert(attribute_name.clone(), new_value.clone());
                    return true;
                }
            }
            DomMutation::ContentChanged { element_id, new_content } => {
                if element.id == *element_id {
                    element.text_content = new_content.clone();
                    element.inner_html = new_content.clone();
                    return true;
                }
            }
            DomMutation::ChildAdded { parent_id, child_element } => {
                if element.id == *parent_id {
                    element.children.push(child_element.clone());
                    return true;
                }
            }
            DomMutation::ChildRemoved { parent_id, child_id } => {
                if element.id == *parent_id {
                    element.children.retain(|child| child.id != *child_id);
                    return true;
                }
            }
            DomMutation::StyleChanged { element_id, property, new_value } => {
                if element.id == *element_id {
                    let style_attr = format!("{}:{};", property, new_value);
                    element.attributes.insert("style".to_string(), style_attr);
                    return true;
                }
            }
            DomMutation::ClassListChanged { element_id, action, class_name } => {
                if element.id == *element_id {
                    let current_class = element.attributes.get("class").cloned().unwrap_or_default();
                    let mut classes: Vec<&str> = current_class.split_whitespace().collect();
                    
                    match action {
                        ClassListAction::Add => {
                            if !classes.contains(&class_name.as_str()) {
                                classes.push(class_name);
                            }
                        }
                        ClassListAction::Remove => {
                            classes.retain(|&c| c != class_name);
                        }
                        ClassListAction::Toggle => {
                            if let Some(pos) = classes.iter().position(|&c| c == class_name) {
                                classes.remove(pos);
                            } else {
                                classes.push(class_name);
                            }
                        }
                    }
                    
                    element.attributes.insert("class".to_string(), classes.join(" "));
                    return true;
                }
            }
        }
        
        // Apply to children recursively
        for child in &mut element.children {
            if self.apply_to_element(child) {
                return true;
            }
        }
        
        false
    }
}