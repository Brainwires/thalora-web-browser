use thalora::{CssProcessor, LayoutEngine, LayoutResult, ElementLayout};

/// Test CssProcessor creation
#[test]
fn test_css_processor_new() {
    let processor = CssProcessor::new();
    // Should not panic
}

/// Test CssProcessor default
#[test]
fn test_css_processor_default() {
    let processor = CssProcessor::default();
    // Should not panic
}

/// Test basic CSS processing
#[test]
fn test_css_processor_basic() {
    let processor = CssProcessor::new();
    let css = "body { color: red; }";

    let result = processor.process_css(css).unwrap();
    assert_eq!(result, css);
}

/// Test empty CSS
#[test]
fn test_css_processor_empty() {
    let processor = CssProcessor::new();
    let css = "";

    let result = processor.process_css(css).unwrap();
    assert_eq!(result, "");
}

/// Test CSS with comments
#[test]
fn test_css_processor_with_comments() {
    let processor = CssProcessor::new();
    let css = r#"
        /* This is a comment */
        body {
            color: red; /* inline comment */
        }
    "#;

    let result = processor.process_css(css).unwrap();
    assert!(result.contains("body"));
    assert!(result.contains("color: red"));
}

/// Test CSS with media queries
#[test]
fn test_css_processor_media_queries() {
    let processor = CssProcessor::new();
    let css = r#"
        @media (max-width: 768px) {
            body {
                font-size: 14px;
            }
        }
    "#;

    let result = processor.process_css(css).unwrap();
    assert!(result.contains("@media"));
    assert!(result.contains("max-width: 768px"));
}

/// Test CSS with keyframes
#[test]
fn test_css_processor_keyframes() {
    let processor = CssProcessor::new();
    let css = r#"
        @keyframes slidein {
            from { margin-left: 100%; }
            to { margin-left: 0%; }
        }
    "#;

    let result = processor.process_css(css).unwrap();
    assert!(result.contains("@keyframes"));
    assert!(result.contains("slidein"));
}

/// Test CSS with pseudo-classes
#[test]
fn test_css_processor_pseudo_classes() {
    let processor = CssProcessor::new();
    let css = r#"
        a:hover { color: blue; }
        input:focus { border-color: green; }
        li:nth-child(2n) { background: gray; }
    "#;

    let result = processor.process_css(css).unwrap();
    assert!(result.contains(":hover"));
    assert!(result.contains(":focus"));
    assert!(result.contains(":nth-child"));
}

/// Test CSS with pseudo-elements
#[test]
fn test_css_processor_pseudo_elements() {
    let processor = CssProcessor::new();
    let css = r#"
        p::before { content: ">> "; }
        p::after { content: " <<"; }
    "#;

    let result = processor.process_css(css).unwrap();
    assert!(result.contains("::before"));
    assert!(result.contains("::after"));
}

/// Test CSS with complex selectors
#[test]
fn test_css_processor_complex_selectors() {
    let processor = CssProcessor::new();
    let css = r#"
        div > p { margin: 10px; }
        div + p { margin-top: 20px; }
        div ~ p { color: blue; }
        div p { padding: 5px; }
    "#;

    let result = processor.process_css(css).unwrap();
    assert!(result.contains(">"));
    assert!(result.contains("+"));
    assert!(result.contains("~"));
}

/// Test CSS with attribute selectors
#[test]
fn test_css_processor_attribute_selectors() {
    let processor = CssProcessor::new();
    let css = r#"
        [type="text"] { border: 1px solid gray; }
        [class~="button"] { cursor: pointer; }
        [href^="https"] { color: green; }
        [href$=".pdf"] { color: red; }
        [href*="example"] { font-weight: bold; }
    "#;

    let result = processor.process_css(css).unwrap();
    assert!(result.contains("[type="));
    assert!(result.contains("[class~="));
    assert!(result.contains("[href^="));
    assert!(result.contains("[href$="));
    assert!(result.contains("[href*="));
}

/// Test very large CSS
#[test]
fn test_css_processor_large_css() {
    let processor = CssProcessor::new();
    let css = ".class { color: red; }\n".repeat(1000);

    let result = processor.process_css(&css).unwrap();
    assert_eq!(result.len(), css.len());
}

/// Test LayoutEngine creation
#[test]
fn test_layout_engine_new() {
    let engine = LayoutEngine::new();
    // Should not panic
}

/// Test LayoutEngine default
#[test]
fn test_layout_engine_default() {
    let engine = LayoutEngine::default();
    // Should not panic
}

/// Test basic layout calculation
#[test]
fn test_layout_engine_calculate_basic() {
    let engine = LayoutEngine::new();
    let html = "<html><body><h1>Test</h1></body></html>";
    let css = "";

    let result = engine.calculate_layout(html, css).unwrap();

    assert_eq!(result.width, 1024.0);
    assert_eq!(result.height, 768.0);
    assert!(result.elements.len() > 0);
}

/// Test layout with empty HTML
#[test]
fn test_layout_engine_empty_html() {
    let engine = LayoutEngine::new();
    let html = "";
    let css = "";

    let result = engine.calculate_layout(html, css).unwrap();
    assert!(result.elements.len() > 0);
}

/// Test layout result structure
#[test]
fn test_layout_result_structure() {
    let engine = LayoutEngine::new();
    let html = "<html><body></body></html>";
    let css = "";

    let result = engine.calculate_layout(html, css).unwrap();

    // Check root element
    assert!(result.elements.len() > 0);
    let root = &result.elements[0];
    assert_eq!(root.tag, "html");
    assert_eq!(root.x, 0.0);
    assert_eq!(root.y, 0.0);
}

/// Test layout with nested elements
#[test]
fn test_layout_nested_elements() {
    let engine = LayoutEngine::new();
    let html = r#"
        <html>
            <body>
                <div>
                    <p>Paragraph</p>
                </div>
            </body>
        </html>
    "#;
    let css = "";

    let result = engine.calculate_layout(html, css).unwrap();

    // Root should have children
    assert!(result.elements.len() > 0);
    let root = &result.elements[0];
    assert!(root.children.len() > 0);
}

/// Test LayoutResult serialization
#[test]
fn test_layout_result_serialization() {
    let layout = LayoutResult {
        width: 800.0,
        height: 600.0,
        elements: vec![
            ElementLayout {
                id: "root".to_string(),
                tag: "div".to_string(),
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
                children: vec![],
            }
        ],
    };

    let json = serde_json::to_string(&layout).unwrap();
    assert!(json.contains("\"width\":800"));
    assert!(json.contains("\"height\":600"));
    assert!(json.contains("\"tag\":\"div\""));
}

/// Test LayoutResult deserialization
#[test]
fn test_layout_result_deserialization() {
    let json = r#"{
        "width": 1024.0,
        "height": 768.0,
        "elements": [
            {
                "id": "root",
                "tag": "html",
                "x": 0.0,
                "y": 0.0,
                "width": 1024.0,
                "height": 768.0,
                "children": []
            }
        ]
    }"#;

    let layout: LayoutResult = serde_json::from_str(json).unwrap();
    assert_eq!(layout.width, 1024.0);
    assert_eq!(layout.height, 768.0);
    assert_eq!(layout.elements.len(), 1);
    assert_eq!(layout.elements[0].tag, "html");
}

/// Test ElementLayout serialization
#[test]
fn test_element_layout_serialization() {
    let element = ElementLayout {
        id: "elem1".to_string(),
        tag: "div".to_string(),
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
        children: vec![],
    };

    let json = serde_json::to_string(&element).unwrap();
    assert!(json.contains("\"id\":\"elem1\""));
    assert!(json.contains("\"tag\":\"div\""));
    assert!(json.contains("\"x\":10"));
    assert!(json.contains("\"y\":20"));
}

/// Test ElementLayout with children
#[test]
fn test_element_layout_with_children() {
    let element = ElementLayout {
        id: "parent".to_string(),
        tag: "div".to_string(),
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
        children: vec![
            ElementLayout {
                id: "child1".to_string(),
                tag: "p".to_string(),
                x: 10.0,
                y: 10.0,
                width: 180.0,
                height: 30.0,
                children: vec![],
            },
            ElementLayout {
                id: "child2".to_string(),
                tag: "p".to_string(),
                x: 10.0,
                y: 50.0,
                width: 180.0,
                height: 30.0,
                children: vec![],
            },
        ],
    };

    assert_eq!(element.children.len(), 2);
    assert_eq!(element.children[0].id, "child1");
    assert_eq!(element.children[1].id, "child2");
}

/// Test layout with CSS styling
#[test]
fn test_layout_with_css_styling() {
    let engine = LayoutEngine::new();
    let html = "<html><body><div class='container'></div></body></html>";
    let css = r#"
        .container {
            width: 500px;
            height: 300px;
            margin: 20px;
            padding: 10px;
        }
    "#;

    let result = engine.calculate_layout(html, css).unwrap();
    assert!(result.width > 0.0);
}

/// Test layout with flexbox CSS
#[test]
fn test_layout_with_flexbox() {
    let engine = LayoutEngine::new();
    let html = r#"
        <html><body>
            <div class="flex-container">
                <div class="item">1</div>
                <div class="item">2</div>
                <div class="item">3</div>
            </div>
        </body></html>
    "#;
    let css = r#"
        .flex-container {
            display: flex;
            justify-content: space-between;
        }
        .item {
            flex: 1;
        }
    "#;

    let result = engine.calculate_layout(html, css).unwrap();
    assert!(result.elements.len() > 0);
}

/// Test layout with grid CSS
#[test]
fn test_layout_with_grid() {
    let engine = LayoutEngine::new();
    let html = r#"
        <html><body>
            <div class="grid">
                <div>1</div>
                <div>2</div>
                <div>3</div>
                <div>4</div>
            </div>
        </body></html>
    "#;
    let css = r#"
        .grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            grid-gap: 10px;
        }
    "#;

    let result = engine.calculate_layout(html, css).unwrap();
    assert!(result.elements.len() > 0);
}

/// Test layout with positioning
#[test]
fn test_layout_with_positioning() {
    let engine = LayoutEngine::new();
    let html = r#"
        <html><body>
            <div class="relative">
                <div class="absolute"></div>
            </div>
            <div class="fixed"></div>
        </body></html>
    "#;
    let css = r#"
        .relative { position: relative; }
        .absolute { position: absolute; top: 10px; left: 20px; }
        .fixed { position: fixed; top: 0; right: 0; }
    "#;

    let result = engine.calculate_layout(html, css).unwrap();
    assert!(result.elements.len() > 0);
}

/// Test layout dimensions
#[test]
fn test_layout_dimensions() {
    let result = LayoutResult {
        width: 1920.0,
        height: 1080.0,
        elements: vec![],
    };

    assert_eq!(result.width, 1920.0);
    assert_eq!(result.height, 1080.0);
}

/// Test element position
#[test]
fn test_element_position() {
    let element = ElementLayout {
        id: "test".to_string(),
        tag: "div".to_string(),
        x: 100.5,
        y: 200.75,
        width: 300.0,
        height: 150.0,
        children: vec![],
    };

    assert_eq!(element.x, 100.5);
    assert_eq!(element.y, 200.75);
}

/// Test element dimensions
#[test]
fn test_element_dimensions() {
    let element = ElementLayout {
        id: "test".to_string(),
        tag: "div".to_string(),
        x: 0.0,
        y: 0.0,
        width: 640.0,
        height: 480.0,
        children: vec![],
    };

    assert_eq!(element.width, 640.0);
    assert_eq!(element.height, 480.0);
}

/// Test deeply nested layout
#[test]
fn test_deeply_nested_layout() {
    let engine = LayoutEngine::new();
    let html = r#"
        <html>
            <body>
                <div>
                    <div>
                        <div>
                            <p>Deep content</p>
                        </div>
                    </div>
                </div>
            </body>
        </html>
    "#;
    let css = "";

    let result = engine.calculate_layout(html, css).unwrap();
    assert!(result.elements.len() > 0);
}

/// Test CSS with imports
#[test]
fn test_css_processor_imports() {
    let processor = CssProcessor::new();
    let css = r#"
        @import url("https://fonts.googleapis.com/css?family=Roboto");
        body { font-family: 'Roboto', sans-serif; }
    "#;

    let result = processor.process_css(css).unwrap();
    assert!(result.contains("@import"));
}

/// Test CSS with variables
#[test]
fn test_css_processor_variables() {
    let processor = CssProcessor::new();
    let css = r#"
        :root {
            --main-color: #06c;
            --accent-color: #ff6347;
        }
        body {
            color: var(--main-color);
        }
    "#;

    let result = processor.process_css(css).unwrap();
    assert!(result.contains("--main-color"));
    assert!(result.contains("var("));
}
