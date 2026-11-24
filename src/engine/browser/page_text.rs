//! Visible text of a page, for agents that want to read it rather than parse
//! HTML: script/style/template contents and the head are dropped, block
//! elements become line breaks, and whitespace is collapsed.

use thalora_dom::{Document, NodeData};

/// Elements whose contents are never visible text.
fn is_hidden_container(tag: &str) -> bool {
    matches!(
        tag,
        "script" | "style" | "noscript" | "template" | "head" | "title" | "svg" | "iframe"
    )
}

/// Elements that start a new line.
fn is_block(tag: &str) -> bool {
    matches!(
        tag,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "br"
            | "dd"
            | "details"
            | "dialog"
            | "div"
            | "dl"
            | "dt"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hr"
            | "li"
            | "main"
            | "nav"
            | "ol"
            | "option"
            | "p"
            | "pre"
            | "section"
            | "summary"
            | "table"
            | "tr"
            | "ul"
    )
}

/// The visible text of a page's document, one block per line.
pub fn visible_text(doc: &Document) -> String {
    let mut raw = String::new();
    // Iterative walk (deep pages must not overflow the stack)
    let mut stack = vec![(doc.document(), false)];
    while let Some((node, closing)) = stack.pop() {
        let Ok(data) = doc.data(node) else {
            continue;
        };
        match data {
            NodeData::Element(element) => {
                let tag = &**element.local_name();
                if closing {
                    if is_block(tag) {
                        raw.push('\n');
                    } else if matches!(tag, "td" | "th") {
                        raw.push('\t');
                    }
                    continue;
                }
                if is_hidden_container(tag)
                    || element.has_attr("hidden")
                    || element.attr("aria-hidden") == Some("true")
                {
                    continue;
                }
                if is_block(tag) {
                    raw.push('\n');
                }
                stack.push((node, true));
            }
            NodeData::Text(text) => {
                raw.push_str(text);
                continue;
            }
            NodeData::Document | NodeData::DocumentFragment => {}
            _ => continue,
        }
        let Ok(children) = doc.children(node) else {
            continue;
        };
        let children: Vec<_> = children.collect();
        stack.extend(children.into_iter().rev().map(|child| (child, false)));
    }

    // Collapse whitespace within lines and drop empty lines
    raw.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use thalora_dom::Document;

    fn visible_text(html: &str) -> String {
        super::visible_text(&Document::parse_html(html))
    }

    #[test]
    fn drops_scripts_styles_and_head_and_keeps_blocks() {
        let html = r#"<html><head><title>T</title><style>p{}</style></head><body>
            <h1>Title</h1><p>One <b>bold</b>   word</p>
            <script>var hidden = "Error: no";</script>
            <ul><li>a</li><li>b</li></ul><p hidden>secret</p>
            <table><tr><td>x</td><td>y</td></tr></table></body></html>"#;
        let text = visible_text(html);
        assert_eq!(text, "Title\nOne bold word\na\nb\nx y");
        assert!(!text.contains("Error"));
    }
}
