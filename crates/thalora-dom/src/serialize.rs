//! HTML fragment serialization
//! (<https://html.spec.whatwg.org/multipage/parsing.html#serialising-html-fragments>).

use html5ever::ns;

use crate::node::{ElementData, NodeData, qualified_name};
use crate::{Document, DomError, DomResult, NodeId};

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "basefont", "bgsound", "br", "col", "embed", "frame", "hr", "img", "input",
    "keygen", "link", "meta", "param", "source", "track", "wbr",
];

/// Elements whose text children serialize literally. `noscript` is included
/// because documents are parsed with scripting enabled.
const RAW_TEXT_PARENTS: &[&str] = &[
    "style",
    "script",
    "xmp",
    "iframe",
    "noembed",
    "noframes",
    "plaintext",
    "noscript",
];

fn serializes_as_void(e: &ElementData) -> bool {
    e.is_html() && VOID_ELEMENTS.contains(&&*e.name.local)
}

fn escape(out: &mut String, s: &str, attribute_mode: bool) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '\u{a0}' => out.push_str("&nbsp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' if attribute_mode => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
}

fn tag_name(e: &ElementData) -> String {
    if e.name.ns == ns!(html) || e.name.ns == ns!(svg) || e.name.ns == ns!(mathml) {
        e.name.local.to_string()
    } else {
        qualified_name(&e.name)
    }
}

fn attr_name(a: &crate::Attr) -> String {
    let local = &*a.name.local;
    if a.name.ns.is_empty() {
        local.to_string()
    } else if a.name.ns == ns!(xml) {
        format!("xml:{local}")
    } else if a.name.ns == ns!(xmlns) {
        if local == "xmlns" {
            "xmlns".to_string()
        } else {
            format!("xmlns:{local}")
        }
    } else if a.name.ns == ns!(xlink) {
        format!("xlink:{local}")
    } else {
        qualified_name(&a.name)
    }
}

impl Document {
    /// The children an element serializes: template contents for
    /// `<template>`, its own children otherwise.
    fn serialization_children_parent(&self, i: u32) -> u32 {
        self.element(i)
            .and_then(|e| e.template_contents)
            .unwrap_or(i)
    }

    fn write_start_tag(&self, out: &mut String, e: &ElementData) {
        out.push('<');
        out.push_str(&tag_name(e));
        for a in &e.attrs {
            out.push(' ');
            out.push_str(&attr_name(a));
            out.push_str("=\"");
            escape(out, &a.value, true);
            out.push('"');
        }
        out.push('>');
    }

    /// Serializes `nodes` and their subtrees in order (iteratively: deep
    /// trees do not grow the call stack). Document and fragment nodes
    /// serialize as their children.
    fn serialize_nodes(&self, out: &mut String, nodes: Vec<u32>) {
        enum Step {
            Open(u32),
            Close(u32),
        }
        let mut stack: Vec<Step> = nodes.into_iter().rev().map(Step::Open).collect();
        while let Some(step) = stack.pop() {
            let i = match step {
                Step::Close(i) => {
                    if let Some(e) = self.element(i) {
                        out.push_str("</");
                        out.push_str(&tag_name(e));
                        out.push('>');
                    }
                    continue;
                }
                Step::Open(i) => i,
            };
            let node = self.node(i);
            match &node.data {
                NodeData::Element(e) => {
                    self.write_start_tag(out, e);
                    if serializes_as_void(e) {
                        continue;
                    }
                    stack.push(Step::Close(i));
                    let kids = self.child_indices(self.serialization_children_parent(i));
                    stack.extend(kids.into_iter().rev().map(Step::Open));
                }
                NodeData::Text(s) => {
                    let raw = node
                        .parent
                        .and_then(|p| self.element(p))
                        .is_some_and(|p| p.is_html() && RAW_TEXT_PARENTS.contains(&&*p.name.local));
                    if raw {
                        out.push_str(s);
                    } else {
                        escape(out, s, false);
                    }
                }
                NodeData::Comment(s) => {
                    out.push_str("<!--");
                    out.push_str(s);
                    out.push_str("-->");
                }
                NodeData::ProcessingInstruction { target, data } => {
                    out.push_str("<?");
                    out.push_str(target);
                    out.push(' ');
                    out.push_str(data);
                    out.push('>');
                }
                NodeData::Doctype { name, .. } => {
                    out.push_str("<!DOCTYPE ");
                    out.push_str(name);
                    out.push('>');
                }
                NodeData::Document | NodeData::DocumentFragment => {
                    let kids = self.child_indices(i);
                    stack.extend(kids.into_iter().rev().map(Step::Open));
                }
            }
        }
    }

    /// The `innerHTML` getter: serializes the node's children (template
    /// contents for `<template>`; empty for void elements). Valid for
    /// elements, documents and fragments.
    pub fn inner_html(&self, id: NodeId) -> DomResult<String> {
        let i = self.check(id)?;
        let mut out = String::new();
        match &self.node(i).data {
            NodeData::Element(e) => {
                if !serializes_as_void(e) {
                    self.serialize_nodes(
                        &mut out,
                        self.child_indices(self.serialization_children_parent(i)),
                    );
                }
            }
            NodeData::Document | NodeData::DocumentFragment => {
                self.serialize_nodes(&mut out, self.child_indices(i))
            }
            _ => return Err(DomError::InvalidNodeType),
        }
        Ok(out)
    }

    /// The `outerHTML` getter generalized to any node: serializes the node
    /// itself (for the document and fragments, their children).
    pub fn outer_html(&self, id: NodeId) -> DomResult<String> {
        let i = self.check(id)?;
        let mut out = String::new();
        self.serialize_nodes(&mut out, vec![i]);
        Ok(out)
    }
}
