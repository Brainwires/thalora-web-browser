//! Visible text of a page, for agents that want to read it rather than parse
//! HTML: script/style/template contents and the head are dropped, block
//! elements become line breaks, and whitespace is collapsed.

use scraper::{Html, Node};

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

/// The visible text of `html`, one block per line.
pub fn visible_text(html: &str) -> String {
    let document = Html::parse_document(html);
    let mut raw = String::new();
    // Iterative walk (deep pages must not overflow the stack)
    let mut stack = vec![(document.tree.root(), false)];
    while let Some((node, closing)) = stack.pop() {
        match node.value() {
            Node::Element(element) => {
                let tag = element.name();
                if closing {
                    if is_block(tag) {
                        raw.push('\n');
                    } else if matches!(tag, "td" | "th") {
                        raw.push('\t');
                    }
                    continue;
                }
                if is_hidden_container(tag)
                    || element.attr("hidden").is_some()
                    || element.attr("aria-hidden") == Some("true")
                {
                    continue;
                }
                if is_block(tag) {
                    raw.push('\n');
                }
                stack.push((node, true));
            }
            Node::Text(text) => {
                raw.push_str(text);
                continue;
            }
            Node::Document | Node::Fragment => {}
            _ => continue,
        }
        let children: Vec<_> = node.children().collect();
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
    use super::visible_text;

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
