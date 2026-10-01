//! Agent-facing page snapshot with stable element refs.
//!
//! Renders the current page as a compact, indented outline of interactive
//! elements (with refs such as `e12`) plus enough structure and text to
//! orient an agent:
//!
//! ```text
//! - heading "Sign in" [level=1]
//! - form
//!   - textbox "Username" [ref=e1]
//!   - textbox "Password" (password) [ref=e2]
//!   - checkbox "Remember me" [checked] [ref=e3]
//!   - button "Sign in" [ref=e4]
//! - link "About" -> /about [ref=e5]
//! ```
//!
//! Refs resolve to an exact structural CSS selector, so action tools can take
//! a `ref` instead of a hand-written selector. A ref stays the same across
//! snapshots while the page content is unchanged, and becomes stale (an
//! error, never a wrong element) once the page navigates or its content is
//! replaced.

use super::accessibility::{compute_accessible_name, heading_level, implicit_role};
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Tags whose subtree never appears in a snapshot.
const SKIP_TAGS: &[&str] = &[
    "head", "script", "style", "template", "noscript", "svg", "math", "iframe", "object",
];

/// Interactive tags whose descendants are summarised by the element itself.
const LEAF_TAGS: &[&str] = &[
    "a", "button", "input", "select", "textarea", "option", "summary",
];

/// Roles shown as structure (without refs unless also interactive).
const STRUCTURE_ROLES: &[&str] = &[
    "heading",
    "navigation",
    "main",
    "form",
    "dialog",
    "banner",
    "contentinfo",
    "complementary",
    "search",
    "region",
    "article",
];

/// Block elements whose own text is shown as `- text "..."` lines.
const TEXT_TAGS: &[&str] = &[
    "p",
    "li",
    "td",
    "th",
    "dd",
    "dt",
    "blockquote",
    "figcaption",
    "pre",
    "caption",
];

const MAX_NAME_CHARS: usize = 100;
const MAX_TEXT_CHARS: usize = 160;

/// Options for [`build_snapshot`].
#[derive(Debug, Clone, Copy)]
pub struct SnapshotOptions {
    /// Approximate output budget in tokens (~4 characters per token).
    pub max_tokens: usize,
    /// Only list interactive elements (no headings, landmarks or text).
    pub interactive_only: bool,
}

impl Default for SnapshotOptions {
    fn default() -> Self {
        Self {
            max_tokens: 4000,
            interactive_only: false,
        }
    }
}

/// A rendered snapshot.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub text: String,
    /// Interactive elements included in `text`.
    pub ref_count: usize,
    /// Whether lines were dropped to fit the budget.
    pub truncated: bool,
}

#[derive(Debug, Clone)]
struct RefTarget {
    selector: String,
    tag: String,
}

/// Refs issued for one version of a page.
#[derive(Debug, Default, Clone)]
pub struct RefTable {
    /// Page version the refs belong to (URL + content hash).
    page: Option<String>,
    by_path: HashMap<String, String>,
    by_ref: HashMap<String, RefTarget>,
    next: u32,
}

impl RefTable {
    /// Start a new ref namespace if `page` differs from the current one.
    fn enter_page(&mut self, page: &str) {
        if self.page.as_deref() != Some(page) {
            *self = Self {
                page: Some(page.to_string()),
                ..Self::default()
            };
        }
    }

    fn ref_for(&mut self, path: &str, tag: &str) -> String {
        if let Some(existing) = self.by_path.get(path) {
            return existing.clone();
        }
        self.next += 1;
        let r = format!("e{}", self.next);
        self.by_path.insert(path.to_string(), r.clone());
        self.by_ref.insert(
            r.clone(),
            RefTarget {
                selector: path.to_string(),
                tag: tag.to_string(),
            },
        );
        r
    }

    /// Resolve `r` to a CSS selector for the page version `page` with HTML
    /// `html`. Fails with a `stale_ref` message if the ref wasn't issued for
    /// this page version or no longer matches an element of the same type.
    pub fn resolve(&self, page: &str, r: &str, html: &str) -> Result<String, String> {
        let stale = || {
            format!(
                "stale_ref: '{r}' is not a ref on the current page (the page changed or the \
                 ref came from an older snapshot). Take a new browser_snapshot."
            )
        };
        if self.page.as_deref() != Some(page) {
            return Err(stale());
        }
        let target = self.by_ref.get(r).ok_or_else(stale)?;
        let selector = Selector::parse(&target.selector).map_err(|_| stale())?;
        let doc = Html::parse_document(html);
        match doc.select(&selector).next() {
            Some(el) if el.value().name() == target.tag => Ok(target.selector.clone()),
            _ => Err(stale()),
        }
    }
}

/// Identity of a page version: its URL plus a hash of its content.
pub fn page_key(url: Option<&str>, html: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    html.hash(&mut hasher);
    format!("{}#{:016x}", url.unwrap_or("about:blank"), hasher.finish())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineKind {
    Interactive,
    Structure,
    Text,
}

struct Line {
    depth: usize,
    kind: LineKind,
    text: String,
}

/// Build a snapshot of `html`, issuing refs in `table` for page `page`.
/// With `focus`, only the subtree of the element matching that selector is
/// rendered.
pub fn build_snapshot(
    html: &str,
    page: &str,
    table: &mut RefTable,
    options: SnapshotOptions,
    focus: Option<&str>,
) -> Result<Snapshot, String> {
    table.enter_page(page);
    let doc = Html::parse_document(html);

    let root = match focus {
        Some(selector) => {
            let sel =
                Selector::parse(selector).map_err(|_| "invalid focus selector".to_string())?;
            doc.select(&sel)
                .next()
                .ok_or_else(|| "focus element not found".to_string())?
        }
        None => doc.root_element(),
    };

    let mut lines = Vec::new();
    walk(root, &doc, 0, table, options, &mut lines);
    Ok(render(lines, options.max_tokens.saturating_mul(4)))
}

fn walk(
    el: ElementRef,
    doc: &Html,
    depth: usize,
    table: &mut RefTable,
    options: SnapshotOptions,
    lines: &mut Vec<Line>,
) {
    let tag = el.value().name();
    if SKIP_TAGS.contains(&tag) || is_hidden(&el) {
        return;
    }
    let attrs: HashMap<&str, &str> = el.value().attrs().collect();
    let role = attrs
        .get("role")
        .copied()
        .filter(|r| !r.is_empty())
        .or_else(|| implicit_role(tag, &attrs));
    let interactive = is_interactive(tag, &attrs, role);
    let structural =
        !options.interactive_only && role.is_some_and(|r| STRUCTURE_ROLES.contains(&r));

    let mut child_depth = depth;
    if interactive || structural {
        let role_label = role.unwrap_or(if interactive { "generic" } else { tag });
        let mut line = format!("- {role_label}");
        let mut name = clean(&compute_accessible_name(&el, doc), MAX_NAME_CHARS);
        if name.is_empty() && interactive && !matches!(tag, "input" | "select" | "textarea") {
            // e.g. <div role="button">Save</div>: fall back to the text content
            name = clean(&el.text().collect::<String>(), MAX_NAME_CHARS);
        }
        if !name.is_empty() {
            line.push_str(&format!(" \"{name}\""));
        }
        line.push_str(&describe(tag, &attrs, &el));
        if interactive {
            let r = table.ref_for(&css_path(el), tag);
            line.push_str(&format!(" [ref={r}]"));
        }
        lines.push(Line {
            depth,
            kind: if interactive {
                LineKind::Interactive
            } else {
                LineKind::Structure
            },
            text: line,
        });
        child_depth = depth + 1;
        if interactive && LEAF_TAGS.contains(&tag) {
            return;
        }
    } else if !options.interactive_only && TEXT_TAGS.contains(&tag) {
        let own_text: String = el
            .children()
            .filter_map(|c| c.value().as_text().map(|t| t.to_string()))
            .collect();
        let text = clean(&own_text, MAX_TEXT_CHARS);
        if !text.is_empty() {
            lines.push(Line {
                depth,
                kind: LineKind::Text,
                text: format!("- text \"{text}\""),
            });
        }
    }

    for child in el.children().filter_map(ElementRef::wrap) {
        walk(child, doc, child_depth, table, options, lines);
    }
}

fn is_hidden(el: &ElementRef) -> bool {
    let v = el.value();
    if v.attr("hidden").is_some() || v.attr("aria-hidden") == Some("true") {
        return true;
    }
    if v.name() == "input"
        && v.attr("type")
            .is_some_and(|t| t.eq_ignore_ascii_case("hidden"))
    {
        return true;
    }
    if let Some(style) = v.attr("style") {
        let style: String = style
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .to_ascii_lowercase();
        if style.contains("display:none") || style.contains("visibility:hidden") {
            return true;
        }
    }
    false
}

fn is_interactive(tag: &str, attrs: &HashMap<&str, &str>, role: Option<&str>) -> bool {
    match tag {
        "a" => attrs.contains_key("href"),
        "button" | "select" | "textarea" | "summary" => true,
        "input" => attrs
            .get("type")
            .is_none_or(|t| !t.eq_ignore_ascii_case("hidden")),
        _ => {
            role.is_some_and(|r| {
                matches!(
                    r,
                    "button"
                        | "link"
                        | "checkbox"
                        | "radio"
                        | "switch"
                        | "tab"
                        | "menuitem"
                        | "menuitemcheckbox"
                        | "menuitemradio"
                        | "option"
                        | "textbox"
                        | "searchbox"
                        | "combobox"
                        | "slider"
                        | "spinbutton"
                        | "treeitem"
                )
            }) || attrs.contains_key("onclick")
                || attrs.get("contenteditable").is_some_and(|v| *v != "false")
                || attrs
                    .get("tabindex")
                    .and_then(|t| t.trim().parse::<i32>().ok())
                    .is_some_and(|t| t >= 0)
        }
    }
}

/// Extra state shown after the name: level, input type, value, checked, …
fn describe(tag: &str, attrs: &HashMap<&str, &str>, el: &ElementRef) -> String {
    let mut out = String::new();
    if let Some(level) = heading_level(tag) {
        out.push_str(&format!(" [level={level}]"));
    }
    if tag == "input" {
        let input_type = attrs.get("type").map(|t| t.to_ascii_lowercase());
        let is_password = input_type.as_deref() == Some("password");
        if let Some(t) = input_type.as_deref()
            && !matches!(
                t,
                "text" | "checkbox" | "radio" | "submit" | "button" | "search" | "range" | "number"
            )
        {
            out.push_str(&format!(" ({t})"));
        }
        if let Some(value) = attrs.get("value").filter(|v| !v.is_empty())
            && !matches!(
                input_type.as_deref(),
                Some("checkbox" | "radio" | "submit" | "button" | "image" | "reset")
            )
        {
            if is_password {
                out.push_str(" value=\"•••\"");
            } else {
                out.push_str(&format!(" value=\"{}\"", clean(value, 60)));
            }
        }
    }
    if tag == "select" {
        let option_selector = Selector::parse("option").expect("valid selector");
        let options: Vec<_> = el.select(&option_selector).collect();
        let chosen = options
            .iter()
            .find(|o| o.value().attr("selected").is_some())
            .or(options.first());
        if let Some(chosen) = chosen {
            let text: String = chosen.text().collect();
            out.push_str(&format!(" value=\"{}\"", clean(&text, 60)));
        }
        out.push_str(&format!(" ({} options)", options.len()));
    }
    if tag == "a"
        && let Some(href) = attrs.get("href")
    {
        out.push_str(&format!(" -> {}", clean(href, 80)));
    }
    for (attr, label) in [
        ("checked", "checked"),
        ("disabled", "disabled"),
        ("required", "required"),
        ("readonly", "readonly"),
    ] {
        if attrs.contains_key(attr) {
            out.push_str(&format!(" [{label}]"));
        }
    }
    match attrs.get("aria-expanded").copied() {
        Some("true") => out.push_str(" [expanded]"),
        Some("false") => out.push_str(" [collapsed]"),
        _ => {}
    }
    if attrs.get("aria-checked") == Some(&"true") {
        out.push_str(" [checked]");
    }
    if attrs.get("aria-selected") == Some(&"true") {
        out.push_str(" [selected]");
    }
    out
}

/// Collapse whitespace, escape quotes and truncate to `max` characters.
fn clean(text: &str, max: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let escaped = collapsed.replace('"', "\\\"");
    if escaped.chars().count() > max {
        format!("{}…", escaped.chars().take(max).collect::<String>())
    } else {
        escaped
    }
}

/// Exact structural selector: `html > body:nth-child(2) > div:nth-child(1) …`.
fn css_path(el: ElementRef) -> String {
    let mut parts = Vec::new();
    let mut current = Some(el);
    while let Some(e) = current {
        let tag = e.value().name();
        let parent = e.parent().and_then(ElementRef::wrap);
        match parent {
            Some(p) => {
                let index = p
                    .children()
                    .filter_map(ElementRef::wrap)
                    .position(|c| c == e)
                    .map_or(1, |i| i + 1);
                parts.push(format!("{tag}:nth-child({index})"));
            }
            None => parts.push(tag.to_string()),
        }
        current = parent;
    }
    parts.reverse();
    parts.join(" > ")
}

/// Join lines within `max_chars`, dropping text, then structure, then
/// trailing interactive lines if needed.
fn render(mut lines: Vec<Line>, max_chars: usize) -> Snapshot {
    let size =
        |lines: &[Line]| -> usize { lines.iter().map(|l| l.depth * 2 + l.text.len() + 1).sum() };
    let mut truncated = false;

    for kind in [LineKind::Text, LineKind::Structure] {
        while size(&lines) > max_chars {
            let Some(pos) = lines.iter().rposition(|l| l.kind == kind) else {
                break;
            };
            lines.remove(pos);
            truncated = true;
        }
    }

    let total_interactive = lines
        .iter()
        .filter(|l| l.kind == LineKind::Interactive)
        .count();
    let mut dropped = 0;
    while size(&lines) > max_chars && !lines.is_empty() {
        lines.pop();
        dropped += 1;
        truncated = true;
    }
    let ref_count = lines
        .iter()
        .filter(|l| l.kind == LineKind::Interactive)
        .count();

    let mut text = String::new();
    for line in &lines {
        text.push_str(&"  ".repeat(line.depth));
        text.push_str(&line.text);
        text.push('\n');
    }
    if dropped > 0 {
        text.push_str(&format!(
            "… {} more interactive elements not shown (raise max_tokens or use focus_ref)\n",
            total_interactive - ref_count
        ));
    } else if truncated {
        text.push_str("… some text and structure omitted to fit max_tokens\n");
    }
    Snapshot {
        text,
        ref_count,
        truncated,
    }
}

#[cfg(feature = "core")]
impl super::HeadlessWebBrowser {
    /// Snapshot the current page, issuing refs for its interactive elements.
    /// `focus_ref` limits the snapshot to that element's subtree.
    pub fn snapshot(
        &mut self,
        options: SnapshotOptions,
        focus_ref: Option<&str>,
    ) -> anyhow::Result<Snapshot> {
        if self.current_content.is_empty() {
            return Err(anyhow::anyhow!("No page loaded. Navigate to a page first."));
        }
        // Let pending microtasks/timers that are already due run first
        self.pump_event_loop(thalora_browser_apis::event_loop::PumpBudget::no_wait());

        let page = page_key(self.current_url.as_deref(), &self.current_content);
        let focus = match focus_ref {
            Some(r) => Some(self.resolve_ref(r)?),
            None => None,
        };
        build_snapshot(
            &self.current_content,
            &page,
            &mut self.snapshot_refs,
            options,
            focus.as_deref(),
        )
        .map_err(|e| anyhow::anyhow!(e))
    }

    /// Resolve a ref from [`snapshot`](Self::snapshot) to a CSS selector.
    pub fn resolve_ref(&self, r: &str) -> anyhow::Result<String> {
        let page = page_key(self.current_url.as_deref(), &self.current_content);
        self.snapshot_refs
            .resolve(&page, r, &self.current_content)
            .map_err(|e| anyhow::anyhow!(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE: &str = r#"<!DOCTYPE html><html><head><title>T</title><script>var x;</script></head>
    <body>
      <h1>Sign in</h1>
      <p>Welcome back. Please log in.</p>
      <form id="login">
        <input type="hidden" name="csrf" value="secret">
        <label for="u">Username</label><input id="u" name="user" value="ann">
        <label for="p">Password</label><input id="p" type="password" name="pass" value="hunter2">
        <input type="checkbox" name="remember" checked aria-label="Remember me">
        <select name="lang"><option>English</option><option selected>French</option></select>
        <button type="submit">Sign <b>in</b></button>
      </form>
      <div style="display: none"><a href="/hidden">Hidden</a></div>
      <a href="/about"><span>About us</span></a>
      <div role="button" tabindex="0">Custom</div>
    </body></html>"#;

    fn snap(table: &mut RefTable, options: SnapshotOptions) -> Snapshot {
        build_snapshot(PAGE, "page-1", table, options, None).unwrap()
    }

    #[test]
    fn renders_interactive_elements_with_refs_and_structure() {
        let mut table = RefTable::default();
        let s = snap(&mut table, SnapshotOptions::default());
        let text = &s.text;
        assert!(text.contains("- heading \"Sign in\" [level=1]"), "{text}");
        assert!(
            text.contains("- text \"Welcome back. Please log in.\""),
            "{text}"
        );
        assert!(
            text.contains("textbox \"Username\" value=\"ann\" [ref="),
            "{text}"
        );
        assert!(
            text.contains("textbox \"Password\" (password) value=\"•••\" [ref="),
            "{text}"
        );
        assert!(!text.contains("hunter2"), "password leaked: {text}");
        assert!(!text.contains("secret"), "hidden input shown: {text}");
        assert!(
            text.contains("checkbox \"Remember me\" [checked] [ref="),
            "{text}"
        );
        assert!(
            text.contains("combobox value=\"French\" (2 options) [ref="),
            "{text}"
        );
        assert!(text.contains("button \"Sign in\" [ref="), "{text}");
        assert!(text.contains("link \"About us\" -> /about [ref="), "{text}");
        assert!(text.contains("button \"Custom\" [ref="), "{text}");
        assert!(!text.contains("Hidden"), "display:none shown: {text}");
        assert!(!text.contains("var x"), "script shown: {text}");
        // form controls are nested under the form
        assert!(text.contains("\n  - textbox \"Username\""), "{text}");
        assert_eq!(s.ref_count, 7, "{text}");
        assert!(!s.truncated);
    }

    #[test]
    fn refs_are_stable_and_resolve_to_the_element() {
        let mut table = RefTable::default();
        let first = snap(&mut table, SnapshotOptions::default()).text;
        let second = snap(&mut table, SnapshotOptions::default()).text;
        assert_eq!(first, second);

        let line = first
            .lines()
            .find(|l| l.contains("button \"Sign in\""))
            .unwrap();
        let r = line.rsplit("[ref=").next().unwrap().trim_end_matches(']');
        let selector = table.resolve("page-1", r, PAGE).unwrap();
        let doc = Html::parse_document(PAGE);
        let el = doc
            .select(&Selector::parse(&selector).unwrap())
            .next()
            .unwrap();
        assert_eq!(el.value().name(), "button");
    }

    #[test]
    fn refs_from_another_page_version_are_stale() {
        let mut table = RefTable::default();
        snap(&mut table, SnapshotOptions::default());
        let err = table.resolve("page-2", "e1", PAGE).unwrap_err();
        assert!(err.starts_with("stale_ref"), "{err}");
        let err = table.resolve("page-1", "e999", PAGE).unwrap_err();
        assert!(err.starts_with("stale_ref"), "{err}");
    }

    #[test]
    fn interactive_only_drops_text_and_structure() {
        let mut table = RefTable::default();
        let s = snap(
            &mut table,
            SnapshotOptions {
                interactive_only: true,
                ..SnapshotOptions::default()
            },
        );
        assert!(!s.text.contains("heading"));
        assert!(!s.text.contains("- text"));
        assert_eq!(s.ref_count, 7);
    }

    #[test]
    fn budget_drops_text_before_interactive_elements() {
        let mut table = RefTable::default();
        let s = snap(
            &mut table,
            SnapshotOptions {
                max_tokens: 100,
                interactive_only: false,
            },
        );
        assert!(s.truncated);
        assert!(!s.text.contains("- text"), "{}", s.text);
        assert!(s.text.len() <= 100 * 4 + 120, "{}", s.text);
    }

    #[test]
    fn page_key_changes_with_content() {
        assert_eq!(page_key(Some("u"), "a"), page_key(Some("u"), "a"));
        assert_ne!(page_key(Some("u"), "a"), page_key(Some("u"), "b"));
        assert_ne!(page_key(Some("u"), "a"), page_key(Some("v"), "a"));
    }
}
