//! The current page's live document.
//!
//! [`PageDom`] is the single owner of the page the browser has loaded: a
//! [`thalora_dom::Document`] plus the page URL. Every reader (selectors,
//! snapshot, accessibility tree, visible text, form discovery) goes through
//! [`PageDom::with_doc`]; static-mode interaction (typing, selecting,
//! checking) mutates it through [`PageDom::with_doc_mut`], so a later
//! serialization or form submission sees the change.
//!
//! # Threading
//!
//! A `PageDom` lives inside a `HeadlessWebBrowser`, which lives on its
//! `BrowserThread` and is only touched by jobs running there. It is
//! deliberately not shared across threads (no `Arc<Mutex<_>>`): when page
//! JavaScript runs on V8, the document will be owned by the fork's
//! `deno_web_page::PageRuntime` (an `Rc<RefCell<Document>>` shared with the
//! isolate, which is `!Send`). Code that needs the page on another thread
//! receives a serialized snapshot ([`PageDom::to_html`]) or a value computed
//! by a job on the browser thread.
//!
//! # Seam for the page runtime
//!
//! Callers only use `with_doc`, `with_doc_mut`, [`PageDom::load`] and the
//! accessors below. When the runtime lands, `load` hands the HTML to the
//! runtime (which parses it and runs scripts), `with_doc` borrows the
//! runtime's document, and `with_doc_mut` is replaced by host calls such as
//! `set_value(NodeId, &str)` where events must fire; callers do not change.
//!
//! # Static form state
//!
//! Without a script engine there is no IDL `value`/`checked`/`selected`
//! state separate from the markup, so static interaction writes content
//! attributes (`value`, `checked`, `selected`) and `<textarea>` text. The
//! one exception is secrets (stored passwords filled by
//! `browser_fill_credential`): they must never appear in serialized HTML,
//! the accessibility tree or snapshots, so they are kept in a side table
//! keyed by [`NodeId`] ([`PageDom::set_secret_value`]) that only form
//! submission reads. This mirrors a real browser, where the IDL value of an
//! input is not reflected into its `value` attribute.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use thalora_dom::{Document, NodeId};

/// The live document of the current page and its URL.
#[derive(Debug, Default)]
pub struct PageDom {
    doc: Document,
    url: Option<String>,
    /// Whether a non-empty response body was loaded.
    loaded: bool,
    /// Hash of the HTML the page was loaded from: with the URL, the identity
    /// of this page version (see [`PageDom::version_key`]).
    source_hash: u64,
    /// Values of fields filled with secrets; never serialized.
    secret_values: HashMap<NodeId, String>,
}

impl PageDom {
    /// No page loaded (an empty document, no URL).
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the page with `html` loaded from `url` (navigation, form
    /// submission, history traversal). Drops all state of the old page.
    pub fn load(&mut self, url: Option<String>, html: &str) {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        html.hash(&mut hasher);
        *self = Self {
            doc: Document::parse_html(html),
            url,
            loaded: !html.is_empty(),
            source_hash: hasher.finish(),
            secret_values: HashMap::new(),
        };
    }

    /// Forget the page (back to the state of [`PageDom::new`]).
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// The page URL, if a page was loaded.
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    /// Whether there is no page content: nothing was loaded, or the loaded
    /// response body was empty.
    pub fn is_empty(&self) -> bool {
        !self.loaded
    }

    /// Identity of the loaded page version: its URL plus a hash of the HTML
    /// it was loaded from. Changes on every navigation to different content;
    /// interaction with the page (filling fields) does not change it.
    pub fn version_key(&self) -> String {
        format!(
            "{}#{:016x}",
            self.url.as_deref().unwrap_or("about:blank"),
            self.source_hash
        )
    }

    /// Run `f` with read access to the document.
    pub fn with_doc<R>(&self, f: impl FnOnce(&Document) -> R) -> R {
        f(&self.doc)
    }

    /// Run `f` with write access to the document (static-mode interaction).
    pub fn with_doc_mut<R>(&mut self, f: impl FnOnce(&mut Document) -> R) -> R {
        f(&mut self.doc)
    }

    /// The page serialized as HTML (doctype plus the document element), or
    /// the empty string when [`is_empty`](Self::is_empty). Reflects every
    /// mutation; never contains secrets.
    pub fn to_html(&self) -> String {
        if self.is_empty() {
            return String::new();
        }
        let doc = &self.doc;
        doc.inner_html(doc.document()).unwrap_or_default()
    }

    /// Give the control `node` a secret value, sent on form submission but
    /// never serialized (see the module docs).
    pub fn set_secret_value(&mut self, node: NodeId, value: &str) {
        self.secret_values.insert(node, value.to_string());
    }

    /// Drop the secret value of `node` (its markup value applies again).
    pub fn clear_secret_value(&mut self, node: NodeId) {
        self.secret_values.remove(&node);
    }

    /// The secret value of `node`, if one was set.
    pub fn secret_value(&self, node: NodeId) -> Option<&str> {
        self.secret_values.get(&node).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_until_loaded() {
        let mut page = PageDom::new();
        assert!(page.is_empty());
        assert_eq!(page.to_html(), "");
        assert_eq!(page.url(), None);

        page.load(Some("https://e.com/".into()), "<p>hi</p>");
        assert!(!page.is_empty());
        assert_eq!(page.url(), Some("https://e.com/"));
        assert_eq!(
            page.to_html(),
            "<html><head></head><body><p>hi</p></body></html>"
        );
        let p = page.with_doc(|d| d.query_selector(d.document(), "p").unwrap().unwrap());
        assert_eq!(
            page.with_doc(|d| d.text_content(p).unwrap()).as_deref(),
            Some("hi")
        );

        page.clear();
        assert!(page.is_empty());
        assert_eq!(page.url(), None);
    }

    #[test]
    fn empty_body_counts_as_no_content() {
        let mut page = PageDom::new();
        page.load(Some("https://e.com/".into()), "");
        assert!(page.is_empty());
        assert_eq!(page.to_html(), "");
    }

    #[test]
    fn serialization_reflects_mutations_but_not_secrets() {
        let mut page = PageDom::new();
        page.load(
            Some("https://e.com/".into()),
            "<!DOCTYPE html><input id=u><input id=p type=password>",
        );
        let (u, p) = page.with_doc(|d| {
            (
                d.get_element_by_id("u").unwrap(),
                d.get_element_by_id("p").unwrap(),
            )
        });
        page.with_doc_mut(|d| d.set_attribute(u, "value", "ann"))
            .unwrap();
        page.set_secret_value(p, "hunter2");
        let html = page.to_html();
        assert!(html.starts_with("<!DOCTYPE html>"), "{html}");
        assert!(html.contains(r#"<input id="u" value="ann">"#), "{html}");
        assert!(!html.contains("hunter2"), "{html}");
        assert_eq!(page.secret_value(p), Some("hunter2"));
        page.clear_secret_value(p);
        assert_eq!(page.secret_value(p), None);
    }

    #[test]
    fn version_key_tracks_loads_not_mutations() {
        let mut page = PageDom::new();
        page.load(Some("u".into()), "<input id=a>");
        let before = page.version_key();
        let a = page.with_doc(|d| d.get_element_by_id("a").unwrap());
        page.with_doc_mut(|d| d.set_attribute(a, "value", "x"))
            .unwrap();
        assert_eq!(page.version_key(), before);

        page.load(Some("u".into()), "<input id=a>");
        assert_eq!(page.version_key(), before, "same URL and content");
        page.load(Some("u".into()), "<input id=b>");
        assert_ne!(page.version_key(), before);
        page.load(Some("v".into()), "<input id=a>");
        assert_ne!(page.version_key(), before);
    }

    #[test]
    fn load_drops_secrets_of_the_old_page() {
        let mut page = PageDom::new();
        page.load(Some("u".into()), "<input id=a>");
        let a = page.with_doc(|d| d.get_element_by_id("a").unwrap());
        page.set_secret_value(a, "s");
        page.load(Some("u".into()), "<input id=a>");
        let a2 = page.with_doc(|d| d.get_element_by_id("a").unwrap());
        assert_eq!(page.secret_value(a2), None);
    }
}
