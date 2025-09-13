use anyhow::Result;
use regex::Regex;
use serde_json::{Map, Value};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ReactProcessor {
    next_data: Option<Value>,
    streaming_data: Vec<StreamingChunk>,
    hydration_data: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
struct StreamingChunk {
    id: String,
    data: Value,
    chunk_type: ChunkType,
}

#[derive(Debug, Clone)]
enum ChunkType {
    Component,
    Metadata,
    Error,
    Navigation,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ReactElement {
    pub component_type: String,
    pub props: HashMap<String, Value>,
    pub children: Vec<ReactElement>,
    pub text_content: Option<String>,
    pub key: Option<String>,
}

impl ReactProcessor {
    pub fn new() -> Self {
        Self {
            next_data: None,
            streaming_data: Vec::new(),
            hydration_data: HashMap::new(),
        }
    }

    pub fn process_next_streaming(&mut self, html_content: &str) -> Result<ProcessedReactData> {
        // Extract Next.js streaming data
        self.extract_next_streaming_data(html_content)?;
        
        // Parse __NEXT_DATA__ if present
        self.extract_next_data(html_content)?;
        
        // Process React Server Components
        let components = self.process_server_components()?;
        
        // Extract hydration information
        let hydration_props = self.extract_hydration_props()?;
        
        Ok(ProcessedReactData {
            components,
            hydration_props,
            metadata: self.extract_metadata(),
            navigation_data: self.extract_navigation_data(),
        })
    }

    fn extract_next_streaming_data(&mut self, html: &str) -> Result<()> {
        // Limit processing to avoid hangs on massive HTML
        if html.len() > 1_000_000 { // 1MB limit
            tracing::warn!("HTML too large for React processing ({}MB), skipping", html.len() / 1_000_000);
            return Ok(());
        }
        
        // Pattern to match Next.js streaming data: self.__next_f.push([1, "..."])
        let streaming_regex = Regex::new(r#"self\.__next_f\.push\(\[(\d+),\s*"([^"]*)"?\]\)"#)?;
        
        let mut processed_count = 0;
        for caps in streaming_regex.captures_iter(html) {
            processed_count += 1;
            if processed_count > 100 { // Limit number of chunks processed
                tracing::warn!("Reached processing limit (100 chunks), stopping React processing");
                break;
            }
            let chunk_id = caps.get(1).unwrap().as_str();
            let data_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            
            // Unescape the JSON string
            let unescaped = self.unescape_streaming_data(data_str);
            
            if let Ok(parsed_data) = serde_json::from_str::<Value>(&unescaped) {
                let chunk_type = self.determine_chunk_type(&parsed_data);
                
                self.streaming_data.push(StreamingChunk {
                    id: chunk_id.to_string(),
                    data: parsed_data,
                    chunk_type,
                });
            } else {
                // Handle non-JSON streaming data (like component definitions)
                self.streaming_data.push(StreamingChunk {
                    id: chunk_id.to_string(),
                    data: Value::String(unescaped),
                    chunk_type: ChunkType::Component,
                });
            }
        }

        Ok(())
    }

    fn unescape_streaming_data(&self, data: &str) -> String {
        // Limit individual chunk size to prevent excessive processing
        if data.len() > 50_000 {
            tracing::warn!("Individual React chunk too large ({}KB), truncating", data.len() / 1000);
            let truncated = &data[..50_000];
            return truncated.replace("\\\"", "\"")
                .replace("\\\\", "\\")
                .replace("\\n", "\n")
                .replace("\\r", "\r")
                .replace("\\t", "\t");
        }
        
        data.replace("\\\"", "\"")
            .replace("\\\\", "\\")
            .replace("\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\t", "\t")
    }

    fn determine_chunk_type(&self, data: &Value) -> ChunkType {
        if let Some(obj) = data.as_object() {
            if obj.contains_key("metadata") {
                ChunkType::Metadata
            } else if obj.contains_key("error") || obj.contains_key("digest") {
                ChunkType::Error
            } else if obj.contains_key("pathname") || obj.contains_key("searchParams") {
                ChunkType::Navigation
            } else {
                ChunkType::Component
            }
        } else if data.is_string() {
            let content = data.as_str().unwrap_or("");
            if content.contains("$L") || content.contains("React") {
                ChunkType::Component
            } else {
                ChunkType::Unknown
            }
        } else {
            ChunkType::Unknown
        }
    }

    fn extract_next_data(&mut self, html: &str) -> Result<()> {
        // Extract __NEXT_DATA__ script content
        let next_data_regex = Regex::new(r#"<script[^>]*id="__NEXT_DATA__"[^>]*>([^<]*)</script>"#)?;
        
        if let Some(caps) = next_data_regex.captures(html) {
            let json_str = caps.get(1).unwrap().as_str();
            if let Ok(data) = serde_json::from_str::<Value>(json_str) {
                self.next_data = Some(data);
            }
        }

        Ok(())
    }

    fn process_server_components(&self) -> Result<Vec<ReactElement>> {
        let mut components = Vec::new();

        for chunk in &self.streaming_data {
            if matches!(chunk.chunk_type, ChunkType::Component) {
                if let Some(element) = self.parse_react_element(&chunk.data)? {
                    components.push(element);
                }
            }
        }

        Ok(components)
    }

    fn parse_react_element(&self, data: &Value) -> Result<Option<ReactElement>> {
        match data {
            Value::String(s) => self.parse_react_element_from_string(s),
            Value::Object(obj) => self.parse_react_element_from_object(obj),
            Value::Array(arr) => {
                // Handle array of React elements
                if let Some(first) = arr.first() {
                    self.parse_react_element(first)
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn parse_react_element_from_string(&self, s: &str) -> Result<Option<ReactElement>> {
        // Parse React element descriptors like ["$", "div", null, {...}]
        if s.starts_with("[\"$\",") {
            // This is a React element descriptor
            if let Ok(parsed) = serde_json::from_str::<Value>(s) {
                if let Value::Array(arr) = parsed {
                    if arr.len() >= 4 {
                        let component_type = arr[1].as_str().unwrap_or("div").to_string();
                        let props = self.extract_props(&arr[3])?;
                        
                        return Ok(Some(ReactElement {
                            component_type,
                            props,
                            children: Vec::new(),
                            text_content: None,
                            key: arr.get(2).and_then(|v| v.as_str()).map(|s| s.to_string()),
                        }));
                    }
                }
            }
        }

        // Look for component references
        if s.contains("$L") {
            // Component reference - create placeholder
            return Ok(Some(ReactElement {
                component_type: "Component".to_string(),
                props: HashMap::new(),
                children: Vec::new(),
                text_content: Some(s.to_string()),
                key: None,
            }));
        }

        Ok(None)
    }

    fn parse_react_element_from_object(&self, obj: &Map<String, Value>) -> Result<Option<ReactElement>> {
        // Handle various object types that might represent React elements
        if let Some(element_type) = obj.get("type") {
            let component_type = element_type.as_str().unwrap_or("div").to_string();
            let props = if let Some(props_obj) = obj.get("props") {
                self.extract_props(props_obj)?
            } else {
                HashMap::new()
            };

            return Ok(Some(ReactElement {
                component_type,
                props,
                children: Vec::new(),
                text_content: None,
                key: obj.get("key").and_then(|v| v.as_str()).map(|s| s.to_string()),
            }));
        }

        Ok(None)
    }

    fn extract_props(&self, props_value: &Value) -> Result<HashMap<String, Value>> {
        let mut props = HashMap::new();

        if let Value::Object(props_obj) = props_value {
            for (key, value) in props_obj {
                props.insert(key.clone(), value.clone());
            }
        }

        Ok(props)
    }

    fn extract_hydration_props(&self) -> Result<HashMap<String, Value>> {
        let mut hydration_props = HashMap::new();

        // Extract props from streaming data that look like hydration data
        for chunk in &self.streaming_data {
            if let Value::Object(obj) = &chunk.data {
                if obj.contains_key("props") || obj.contains_key("pageProps") {
                    hydration_props.insert(chunk.id.clone(), chunk.data.clone());
                }
            }
        }

        // Add Next.js page props if available
        if let Some(next_data) = &self.next_data {
            if let Some(props) = next_data.get("props") {
                hydration_props.insert("pageProps".to_string(), props.clone());
            }
        }

        Ok(hydration_props)
    }

    fn extract_metadata(&self) -> HashMap<String, Value> {
        let mut metadata = HashMap::new();

        for chunk in &self.streaming_data {
            if matches!(chunk.chunk_type, ChunkType::Metadata) {
                if let Value::Object(obj) = &chunk.data {
                    for (key, value) in obj {
                        metadata.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        metadata
    }

    fn extract_navigation_data(&self) -> Option<NavigationData> {
        for chunk in &self.streaming_data {
            if matches!(chunk.chunk_type, ChunkType::Navigation) {
                if let Value::Object(obj) = &chunk.data {
                    return Some(NavigationData {
                        pathname: obj.get("pathname")
                            .and_then(|v| v.as_str())
                            .unwrap_or("/")
                            .to_string(),
                        search_params: obj.get("searchParams")
                            .and_then(|v| v.as_object())
                            .cloned()
                            .unwrap_or_default(),
                    });
                }
            }
        }

        None
    }

    pub fn render_to_html(&self, components: &[ReactElement]) -> String {
        let mut html = String::new();

        for component in components {
            html.push_str(&self.render_component_to_html(component));
        }

        html
    }

    fn render_component_to_html(&self, component: &ReactElement) -> String {
        if let Some(text) = &component.text_content {
            return text.clone();
        }

        let mut html = format!("<{}", component.component_type);

        // Add attributes from props
        for (key, value) in &component.props {
            if let Some(string_value) = value.as_str() {
                html.push_str(&format!(" {}=\"{}\"", key, html_escape::encode_text(string_value)));
            } else if value.is_boolean() && value.as_bool().unwrap_or(false) {
                html.push_str(&format!(" {}", key));
            }
        }

        html.push('>');

        // Render children
        for child in &component.children {
            html.push_str(&self.render_component_to_html(child));
        }

        html.push_str(&format!("</{}>", component.component_type));
        html
    }
}

#[derive(Debug)]
pub struct ProcessedReactData {
    pub components: Vec<ReactElement>,
    pub hydration_props: HashMap<String, Value>,
    pub metadata: HashMap<String, Value>,
    pub navigation_data: Option<NavigationData>,
}

#[derive(Debug)]
pub struct NavigationData {
    pub pathname: String,
    pub search_params: Map<String, Value>,
}

impl ProcessedReactData {
    pub fn to_enhanced_html(&self, base_html: &str) -> String {
        let mut enhanced_html = base_html.to_string();

        // Insert React-rendered content
        let react_content = self.render_react_content();
        
        // Try to inject into body
        if let Some(body_end) = enhanced_html.find("</body>") {
            enhanced_html.insert_str(body_end, &format!("<div id=\"react-content\">{}</div>", react_content));
        } else {
            enhanced_html.push_str(&react_content);
        }

        enhanced_html
    }

    fn render_react_content(&self) -> String {
        let mut content = String::new();

        for component in &self.components {
            content.push_str(&self.render_single_component(component));
        }

        content
    }

    fn render_single_component(&self, component: &ReactElement) -> String {
        if let Some(text) = &component.text_content {
            return text.clone();
        }

        let mut html = format!("<{}", component.component_type);

        // Add props as attributes
        for (key, value) in &component.props {
            if key == "children" {
                continue; // Handle children separately
            }

            match value {
                Value::String(s) => {
                    html.push_str(&format!(" {}=\"{}\"", key, html_escape::encode_text(s)));
                }
                Value::Bool(true) => {
                    html.push_str(&format!(" {}", key));
                }
                Value::Number(n) => {
                    html.push_str(&format!(" {}=\"{}\"", key, n));
                }
                _ => {}
            }
        }

        html.push('>');

        // Handle children from props
        if let Some(children_value) = component.props.get("children") {
            html.push_str(&self.render_children_value(children_value));
        }

        // Handle direct children
        for child in &component.children {
            html.push_str(&self.render_single_component(child));
        }

        html.push_str(&format!("</{}>", component.component_type));
        html
    }

    fn render_children_value(&self, children: &Value) -> String {
        match children {
            Value::String(s) => s.clone(),
            Value::Array(arr) => {
                arr.iter()
                    .map(|child| self.render_children_value(child))
                    .collect::<Vec<_>>()
                    .join("")
            }
            _ => String::new(),
        }
    }
}