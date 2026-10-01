//! HTML serialization (the HTML fragment serialization algorithm).

use super::{DomTree, NODE_ID_ATTR, NodeId, NodeKind, is_void_element};

/// Elements whose text children are emitted unescaped.
fn is_raw_text_parent(tag: &str) -> bool {
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "style" | "script" | "xmp" | "iframe" | "noembed" | "noframes" | "plaintext" | "noscript"
    )
}

fn escape_text(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '\u{a0}' => out.push_str("&nbsp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
}

fn escape_attr(out: &mut String, value: &str) {
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '\u{a0}' => out.push_str("&nbsp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
}

enum Step {
    Open(NodeId),
    Close(NodeId),
}

/// Serialize `id` (`outer`: including itself) without recursion.
/// `with_ids` stamps each element with [`NODE_ID_ATTR`].
pub(super) fn serialize(tree: &DomTree, id: NodeId, outer: bool, with_ids: bool) -> String {
    let mut out = String::new();
    let mut stack: Vec<Step> = Vec::new();
    if outer {
        stack.push(Step::Open(id));
    } else {
        stack.extend(tree.children(id).iter().rev().map(|&c| Step::Open(c)));
    }

    while let Some(step) = stack.pop() {
        match step {
            Step::Close(id) => {
                if let Some(tag) = tree.tag(id) {
                    out.push_str("</");
                    out.push_str(tag);
                    out.push('>');
                }
            }
            Step::Open(id) => {
                let Some(node) = tree.node(id) else {
                    continue;
                };
                match &node.kind {
                    NodeKind::Document => {
                        stack.extend(node.children.iter().rev().map(|&c| Step::Open(c)));
                    }
                    NodeKind::Doctype { name, .. } => {
                        out.push_str("<!DOCTYPE ");
                        out.push_str(name);
                        out.push('>');
                    }
                    NodeKind::Comment(data) => {
                        out.push_str("<!--");
                        out.push_str(data);
                        out.push_str("-->");
                    }
                    NodeKind::Text(data) => {
                        let raw = node
                            .parent
                            .and_then(|p| tree.tag(p))
                            .is_some_and(is_raw_text_parent);
                        if raw {
                            out.push_str(data);
                        } else {
                            escape_text(&mut out, data);
                        }
                    }
                    NodeKind::Element { tag, attrs } => {
                        out.push('<');
                        out.push_str(tag);
                        for (name, value) in attrs {
                            if with_ids && name == NODE_ID_ATTR {
                                continue;
                            }
                            out.push(' ');
                            out.push_str(name);
                            out.push_str("=\"");
                            escape_attr(&mut out, value);
                            out.push('"');
                        }
                        if with_ids {
                            out.push_str(&format!(" {NODE_ID_ATTR}=\"{id}\""));
                        }
                        out.push('>');
                        if is_void_element(tag) {
                            continue;
                        }
                        stack.push(Step::Close(id));
                        stack.extend(node.children.iter().rev().map(|&c| Step::Open(c)));
                    }
                }
            }
        }
    }
    out
}
