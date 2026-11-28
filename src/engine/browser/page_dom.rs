//! The current page's live document.
//!
//! [`PageDom`] is the single owner of the page the browser has loaded: a
//! [`thalora_dom::Document`] plus the page URL. Every reader (selectors,
//! snapshot, accessibility tree, visible text, form discovery) goes through
//! [`PageDom::with_doc`].
//!
//! # Two modes
//!
//! - **JavaScript mode** ([`PageDom::load_with_js`]): the document is owned by
//!   the page's V8 runtime (the Brainwires deno fork's
//!   `deno_web_page::PageRuntime`, wrapped in `JsPage`) and shared with the
//!   isolate, so `with_doc` sees every mutation page scripts made.
//!   Interaction goes through the runtime (`PageDom::js`: click, set value,
//!   evaluate), so events fire and form state is the IDL state
//!   (`value`/`checked`), not the markup.
//! - **Static mode** ([`PageDom::load`]): the document is owned here and
//!   interaction mutates it through [`PageDom::static_doc_mut`]. This is the
//!   mode when JavaScript is disabled, when the page runtime could not start
//!   ([`PageDom::js_note`] says why) and for pages loaded outside a browser
//!   thread.
//!
//! # Threading
//!
//! A `PageDom` lives inside a `HeadlessWebBrowser`, which lives on its
//! `BrowserThread` and is only touched by jobs running there. It is
//! deliberately not shared across threads (no `Arc<Mutex<_>>`): the page
//! runtime is `!Send`. Code that needs the page on another thread receives a
//! serialized snapshot ([`PageDom::to_html`]) or a value computed by a job on
//! the browser thread.
//!
//! # Static form state
//!
//! Without a script engine there is no IDL `value`/`checked`/`selected`
//! state separate from the markup, so static interaction writes content
//! attributes (`value`, `checked`, `selected`) and `<textarea>` text.
//!
//! # Secrets
//!
//! Secrets (stored passwords filled by `browser_fill_credential`) must never
//! appear in serialized HTML, the accessibility tree, snapshots or the
//! result of an evaluation, so in both modes they are kept in a side table
//! keyed by [`NodeId`] ([`PageDom::set_secret_value`]) that only form
//! submission reads. Page JavaScript therefore does not see them either.

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};

use thalora_dom::{Document, NodeId};

use super::types::ConsoleEntry;

/// How many console entries are kept per page; the oldest are dropped.
const MAX_CONSOLE_ENTRIES: usize = 1000;

/// The live document of the current page and its URL.
#[derive(Debug, Default)]
pub struct PageDom {
    /// The page's JavaScript runtime, which owns the document in JavaScript
    /// mode. Declared first: it is dropped before everything else.
    #[cfg(feature = "core")]
    js: Option<super::page_js::JsPage>,
    /// Navigations page scripts asked for and the host has not performed.
    #[cfg(feature = "core")]
    pending_navigations: VecDeque<deno_web_page::NavigationRequest>,
    /// The document in static mode (empty in JavaScript mode).
    doc: Document,
    url: Option<String>,
    /// Whether a non-empty response body was loaded.
    loaded: bool,
    /// Hash of the HTML the page was loaded from: with the URL, the identity
    /// of this page version (see [`PageDom::version_key`]).
    source_hash: u64,
    /// Values of fields filled with secrets; never serialized.
    secret_values: HashMap<NodeId, String>,
    /// Why the page is static although scripts were wanted.
    js_note: Option<String>,
    /// The `<script>` elements the browser has already considered.
    seen_scripts: std::collections::HashSet<NodeId>,
    /// What the page logged and the errors nothing caught, oldest first.
    console: VecDeque<ConsoleEntry>,
}

fn source_hash(html: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    html.hash(&mut hasher);
    hasher.finish()
}

impl PageDom {
    /// No page loaded (an empty document, no URL).
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the page with `html` loaded from `url` (navigation, form
    /// submission, history traversal), in static mode: no script runs.
    /// Drops all state of the old page.
    pub fn load(&mut self, url: Option<String>, html: &str) {
        // The old page's runtime goes first (see `load_with_js`).
        self.clear();
        *self = Self {
            doc: Document::parse_html(html),
            url,
            loaded: !html.is_empty(),
            source_hash: source_hash(html),
            ..Self::default()
        };
    }

    /// Replace the page with `html` loaded from `url`, in JavaScript mode:
    /// the page runtime parses the markup and owns the document. No script
    /// has run when this returns; the caller executes the page's scripts
    /// through [`PageDom::js`].
    ///
    /// If the runtime cannot start, the page is loaded in static mode,
    /// [`PageDom::js_note`] says why and `false` is returned.
    #[cfg(feature = "core")]
    pub(crate) fn load_with_js(
        &mut self,
        url: &str,
        html: &str,
        settings: &super::page_js::JsSettings,
    ) -> bool {
        // V8 isolates must be dropped in reverse order of creation: the old
        // page's runtime goes before the new one is created.
        self.clear();
        let booted = url::Url::parse(url)
            .map_err(|e| format!("the page URL is not valid ({e})"))
            .and_then(|parsed| {
                super::page_js::JsPage::new(
                    settings.page_options(parsed, html),
                    settings.script_timeout,
                )
            });
        match booted {
            Ok(js) => {
                *self = Self {
                    js: Some(js),
                    url: Some(url.to_string()),
                    loaded: !html.is_empty(),
                    source_hash: source_hash(html),
                    ..Self::default()
                };
                true
            }
            Err(reason) => {
                tracing::warn!("page runtime did not start for {url}: {reason}; static mode");
                self.load(Some(url.to_string()), html);
                self.js_note = Some(format!("the page runtime did not start: {reason}"));
                false
            }
        }
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
    /// interaction with the page (filling fields) and mutations by page
    /// scripts do not change it.
    pub fn version_key(&self) -> String {
        format!(
            "{}#{:016x}",
            self.url.as_deref().unwrap_or("about:blank"),
            self.source_hash
        )
    }

    /// Whether the page runs JavaScript (JavaScript mode).
    pub fn has_js(&self) -> bool {
        #[cfg(feature = "core")]
        {
            self.js.is_some()
        }
        #[cfg(not(feature = "core"))]
        {
            false
        }
    }

    /// Why the page is static although JavaScript was wanted (it is
    /// disabled, or the page runtime failed to start). `None` in JavaScript
    /// mode and for pages loaded with [`PageDom::load`].
    pub fn js_note(&self) -> Option<&str> {
        self.js_note.as_deref()
    }

    /// Record why the page is static (see [`PageDom::js_note`]).
    pub(crate) fn set_js_note(&mut self, note: impl Into<String>) {
        self.js_note = Some(note.into());
    }

    /// Record that the browser considered the `<script>` element `node`.
    /// Returns whether it is new (and the page has not reached `limit`
    /// scripts).
    pub(crate) fn mark_script_seen(&mut self, node: NodeId, limit: usize) -> bool {
        self.seen_scripts.len() < limit && self.seen_scripts.insert(node)
    }

    /// The page's JavaScript runtime (JavaScript mode only).
    #[cfg(feature = "core")]
    pub(crate) fn js(&mut self) -> Option<&mut super::page_js::JsPage> {
        self.js.as_mut()
    }

    /// The live value and checkedness of every form control (`<input>`,
    /// `<textarea>`, `<select>`, `<option>`) of a page running JavaScript,
    /// where they are state of the controls and not markup. Empty in static
    /// mode, where the attributes are the state. Never contains secrets.
    pub fn live_form_states(&mut self) -> HashMap<NodeId, (String, bool)> {
        #[cfg(feature = "core")]
        if let Some(js) = self.js.as_mut() {
            let controls = js.with_document(|doc| {
                doc.query_selector_all(doc.document(), "input, textarea, select, option")
                    .unwrap_or_default()
            });
            return controls
                .into_iter()
                .filter_map(|node| {
                    let state = js.form_state(node).ok().flatten()?;
                    Some((node, (state.value, state.checked)))
                })
                .collect();
        }
        HashMap::new()
    }

    /// Run `f` with read access to the document.
    pub fn with_doc<R>(&self, f: impl FnOnce(&Document) -> R) -> R {
        #[cfg(feature = "core")]
        if let Some(js) = &self.js {
            return js.with_document(f);
        }
        f(&self.doc)
    }

    /// Write access to the document for static-mode interaction. `None` in
    /// JavaScript mode, where the runtime owns the document and changes go
    /// through it (so events fire).
    pub fn static_doc_mut(&mut self) -> Option<&mut Document> {
        if self.has_js() {
            None
        } else {
            Some(&mut self.doc)
        }
    }

    /// The page serialized as HTML (doctype plus the document element), or
    /// the empty string when [`is_empty`](Self::is_empty). Reflects every
    /// mutation; never contains secrets.
    pub fn to_html(&self) -> String {
        if self.is_empty() {
            return String::new();
        }
        self.with_doc(|doc| doc.inner_html(doc.document()).unwrap_or_default())
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

    /// Every control that has a secret value.
    pub(crate) fn secret_nodes(&self) -> Vec<NodeId> {
        self.secret_values.keys().copied().collect()
    }

    /// What the page wrote to its console and the errors nothing caught
    /// since it was loaded (or since [`clear_console`](Self::clear_console)),
    /// oldest first, at most 1000.
    pub fn console_messages(&self) -> impl Iterator<Item = &ConsoleEntry> {
        self.console.iter()
    }

    /// Empty the console buffer.
    pub fn clear_console(&mut self) {
        self.console.clear();
    }

    /// Append a console entry (also used for what the host reports about
    /// the page's scripts: blocked, failed to load, timed out).
    pub(crate) fn push_console(&mut self, level: &'static str, text: impl Into<String>) {
        if self.console.len() >= MAX_CONSOLE_ENTRIES {
            self.console.pop_front();
        }
        self.console.push_back(ConsoleEntry {
            level,
            text: text.into(),
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or_default(),
        });
    }

    /// Move what the runtime buffered (console messages, uncaught errors,
    /// navigation requests) into this page. Call after running JavaScript.
    #[cfg(feature = "core")]
    pub(crate) fn collect_js_output(&mut self) {
        let Some(js) = self.js.as_mut() else {
            return;
        };
        let output = js.drain();
        for (level, text) in output.console {
            self.push_console(level, text);
        }
        self.pending_navigations.extend(output.navigations);
    }

    /// Queue a navigation for [`take_navigation_request`](Self::take_navigation_request).
    #[cfg(feature = "core")]
    pub(crate) fn push_navigation_request(&mut self, request: deno_web_page::NavigationRequest) {
        self.pending_navigations.push_back(request);
    }

    /// Whether page scripts asked for a navigation the host has not
    /// performed yet.
    #[cfg(feature = "core")]
    pub(crate) fn has_navigation_request(&self) -> bool {
        !self.pending_navigations.is_empty()
    }

    /// The navigation page scripts asked for, if any. Of several requests
    /// the last one wins, as in a browser; the others are dropped.
    #[cfg(feature = "core")]
    pub(crate) fn take_navigation_request(&mut self) -> Option<deno_web_page::NavigationRequest> {
        let last = self.pending_navigations.pop_back();
        self.pending_navigations.clear();
        last
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
        page.static_doc_mut()
            .unwrap()
            .set_attribute(u, "value", "ann")
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
        page.static_doc_mut()
            .unwrap()
            .set_attribute(a, "value", "x")
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
    fn a_page_loaded_here_is_static() {
        let mut page = PageDom::new();
        page.load(Some("u".into()), "<script>document.title = 'x'</script>");
        assert!(!page.has_js());
        assert_eq!(page.js_note(), None);
        assert!(page.static_doc_mut().is_some());
        assert_eq!(page.console_messages().count(), 0);
    }

    #[test]
    fn console_keeps_the_newest_entries() {
        let mut page = PageDom::new();
        page.load(Some("u".into()), "<p>");
        for i in 0..(MAX_CONSOLE_ENTRIES + 5) {
            page.push_console("log", format!("m{i}"));
        }
        assert_eq!(page.console_messages().count(), MAX_CONSOLE_ENTRIES);
        assert_eq!(page.console_messages().next().unwrap().text, "m5");
        page.clear_console();
        assert_eq!(page.console_messages().count(), 0);
        page.push_console("error", "kept until the next load");
        page.load(Some("u".into()), "<p>");
        assert_eq!(page.console_messages().count(), 0);
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
