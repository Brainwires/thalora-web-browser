//! HTML parsing via html5ever's `TreeSink`.

use std::borrow::Cow;
use std::cell::{Ref, RefCell};

use html5ever::interface::{ElementFlags, NodeOrText, TreeSink};
use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{Attribute, ParseOpts, QualName};

use crate::node::{Attr, NodeData, QuirksMode};
use crate::{Document, DomError, DomResult, NodeId};

/// Builds a [`Document`] from parser callbacks. Handles are arena indices.
/// The parser builds detached structure only: no mutation records and no
/// ID index maintenance (the index is rebuilt once parsing ends).
struct Sink {
    doc: RefCell<Document>,
}

fn convert_attrs(attrs: Vec<Attribute>) -> Vec<Attr> {
    attrs
        .into_iter()
        .map(|a| Attr {
            name: a.name,
            value: String::from(a.value),
        })
        .collect()
}

impl Sink {
    fn append_text_or_node(&self, parent: u32, before: Option<u32>, child: NodeOrText<u32>) {
        let mut doc = self.doc.borrow_mut();
        match child {
            NodeOrText::AppendNode(n) => {
                doc.unlink(n);
                doc.link_before(parent, n, before);
            }
            NodeOrText::AppendText(text) => {
                // Merge with an adjacent preceding text node.
                let prev = match before {
                    Some(b) => doc.node(b).prev_sibling,
                    None => doc.node(parent).last_child,
                };
                if let Some(p) = prev
                    && let NodeData::Text(s) = &mut doc.node_mut(p).data
                {
                    s.push_str(&text);
                    return;
                }
                let t = doc.alloc(NodeData::Text(String::from(text)));
                doc.link_before(parent, t, before);
            }
        }
    }
}

impl TreeSink for Sink {
    type Handle = u32;
    type Output = Document;
    type ElemName<'a> = Ref<'a, QualName>;

    fn finish(self) -> Document {
        self.doc.into_inner()
    }

    fn parse_error(&self, _msg: Cow<'static, str>) {}

    fn get_document(&self) -> u32 {
        self.doc.borrow().root_index()
    }

    fn elem_name<'a>(&'a self, target: &'a u32) -> Ref<'a, QualName> {
        Ref::map(self.doc.borrow(), |d| {
            &d.element(*target)
                .expect("html5ever only asks element names of elements")
                .name
        })
    }

    fn create_element(&self, name: QualName, attrs: Vec<Attribute>, _flags: ElementFlags) -> u32 {
        // Template contents are created for every HTML <template>, which is
        // exactly when html5ever sets `flags.template`.
        self.doc
            .borrow_mut()
            .alloc_element(name, convert_attrs(attrs))
    }

    fn create_comment(&self, text: StrTendril) -> u32 {
        self.doc
            .borrow_mut()
            .alloc(NodeData::Comment(String::from(text)))
    }

    fn create_pi(&self, target: StrTendril, data: StrTendril) -> u32 {
        self.doc
            .borrow_mut()
            .alloc(NodeData::ProcessingInstruction {
                target: String::from(target),
                data: String::from(data),
            })
    }

    fn append(&self, parent: &u32, child: NodeOrText<u32>) {
        self.append_text_or_node(*parent, None, child);
    }

    fn append_based_on_parent_node(
        &self,
        element: &u32,
        prev_element: &u32,
        child: NodeOrText<u32>,
    ) {
        let has_parent = self.doc.borrow().node(*element).parent.is_some();
        if has_parent {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    fn append_doctype_to_document(
        &self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        let mut doc = self.doc.borrow_mut();
        let d = doc.alloc(NodeData::Doctype {
            name: String::from(name),
            public_id: String::from(public_id),
            system_id: String::from(system_id),
        });
        let root = doc.root_index();
        doc.link_before(root, d, None);
    }

    fn get_template_contents(&self, target: &u32) -> u32 {
        self.doc
            .borrow()
            .element(*target)
            .and_then(|e| e.template_contents)
            .expect("html5ever only asks template contents of templates")
    }

    fn same_node(&self, x: &u32, y: &u32) -> bool {
        x == y
    }

    fn set_quirks_mode(&self, mode: html5ever::interface::QuirksMode) {
        self.doc.borrow_mut().set_quirks(from_h5e_quirks(mode));
    }

    fn append_before_sibling(&self, sibling: &u32, new_node: NodeOrText<u32>) {
        let parent = self.doc.borrow().node(*sibling).parent;
        if let Some(parent) = parent {
            self.append_text_or_node(parent, Some(*sibling), new_node);
        }
    }

    fn add_attrs_if_missing(&self, target: &u32, attrs: Vec<Attribute>) {
        let mut doc = self.doc.borrow_mut();
        if let NodeData::Element(e) = &mut doc.node_mut(*target).data {
            for a in convert_attrs(attrs) {
                if !e.attrs.iter().any(|x| x.name == a.name) {
                    e.attrs.push(a);
                }
            }
        }
    }

    fn remove_from_parent(&self, target: &u32) {
        self.doc.borrow_mut().unlink(*target);
    }

    fn reparent_children(&self, node: &u32, new_parent: &u32) {
        let mut doc = self.doc.borrow_mut();
        for c in doc.child_indices(*node) {
            doc.unlink(c);
            doc.link_before(*new_parent, c, None);
        }
    }
}

fn from_h5e_quirks(mode: html5ever::interface::QuirksMode) -> QuirksMode {
    use html5ever::interface::QuirksMode as Q;
    match mode {
        Q::Quirks => QuirksMode::Quirks,
        Q::LimitedQuirks => QuirksMode::LimitedQuirks,
        Q::NoQuirks => QuirksMode::NoQuirks,
    }
}

fn to_h5e_quirks(mode: QuirksMode) -> html5ever::interface::QuirksMode {
    use html5ever::interface::QuirksMode as Q;
    match mode {
        QuirksMode::Quirks => Q::Quirks,
        QuirksMode::LimitedQuirks => Q::LimitedQuirks,
        QuirksMode::NoQuirks => Q::NoQuirks,
    }
}

fn parse_opts(quirks: QuirksMode) -> ParseOpts {
    ParseOpts {
        tree_builder: TreeBuilderOpts {
            // Thalora runs page scripts, so parse as a scripting-enabled UA
            // (`<noscript>` is raw text). Serialization matches.
            scripting_enabled: true,
            quirks_mode: to_h5e_quirks(quirks),
            ..TreeBuilderOpts::default()
        },
        ..ParseOpts::default()
    }
}

impl Document {
    /// Parses a full HTML document (the HTML Standard's parser, via
    /// html5ever). Never fails: HTML parsing recovers from every error.
    pub fn parse_html(html: &str) -> Document {
        let sink = Sink {
            doc: RefCell::new(Document::new()),
        };
        let mut doc = html5ever::parse_document(sink, parse_opts(QuirksMode::NoQuirks)).one(html);
        doc.rebuild_id_index();
        doc
    }

    /// The HTML fragment parsing algorithm with `context` as the context
    /// element: returns a new detached `DocumentFragment` holding the
    /// parsed nodes.
    pub fn parse_fragment(&mut self, context: NodeId, html: &str) -> DomResult<NodeId> {
        let frag = self.parse_fragment_index(context, html)?;
        Ok(self.id_of(frag))
    }

    fn parse_fragment_index(&mut self, context: NodeId, html: &str) -> DomResult<u32> {
        let ctx = self.element_data(context)?;
        let name = ctx.name.clone();
        let attrs = ctx
            .attrs
            .iter()
            .map(|a| Attribute {
                name: a.name.clone(),
                value: StrTendril::from(a.value.as_str()),
            })
            .collect();
        let sink = Sink {
            doc: RefCell::new(Document::new()),
        };
        let tmp =
            html5ever::parse_fragment(sink, parse_opts(self.quirks_mode()), name, attrs, true)
                .one(html);
        // The fragment parser puts the result under an <html> root element.
        let tmp_root = tmp.root_index();
        let html_root = tmp
            .child_indices(tmp_root)
            .into_iter()
            .find(|&c| tmp.element(c).is_some())
            .ok_or_else(|| DomError::Syntax("fragment parser produced no root".into()))?;
        let frag = self.alloc(NodeData::DocumentFragment);
        for c in tmp.child_indices(html_root) {
            let copy = self.copy_subtree(Some(&tmp), c, true);
            self.link_before(frag, copy, None);
        }
        Ok(frag)
    }

    /// The `innerHTML` setter: parses `html` as a fragment in the context of
    /// `element` and replaces its children (for `<template>`, the template
    /// contents' children) with the result, as one `childList` record.
    pub fn set_inner_html(&mut self, element: NodeId, html: &str) -> DomResult<()> {
        let i = self.check(element)?;
        let frag = self.parse_fragment_index(element, html)?;
        let target = self
            .element(i)
            .and_then(|e| e.template_contents)
            .unwrap_or(i);
        self.replace_all(Some(frag), target, false);
        self.free_detached(frag);
        Ok(())
    }
}
