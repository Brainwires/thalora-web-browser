// Tests for src/engine/dom.rs
#[cfg(test)]
mod dom_tests {
    use thalora::engine::dom::*;

    #[test]
    fn test_dom_element_creation() {
        let element = DomElement {
            tag_name: "div".to_string(),
            attributes: std::collections::HashMap::new(),
            text_content: "Hello World".to_string(),
            inner_html: "<span>test</span>".to_string(),
            children: vec![],
            id: "element1".to_string(),
        };

        assert_eq!(element.tag_name, "div");
        assert_eq!(element.text_content, "Hello World");
        assert_eq!(element.id, "element1");
    }

    #[test]
    fn test_dom_mutation_creation() {
        let mutation = DomMutation::ContentChanged {
            element_id: "test_element".to_string(),
            new_content: "New content".to_string(),
        };

        match mutation {
            DomMutation::ContentChanged { element_id, new_content } => {
                assert_eq!(element_id, "test_element");
                assert_eq!(new_content, "New content");
            },
            _ => panic!("Wrong mutation type"),
        }
    }

    #[test]
    fn test_enhanced_dom_creation() {
        let html = "<html><body><h1>Test</h1></body></html>";
        let dom = EnhancedDom::new(html);
        assert!(dom.is_ok());
    }
}