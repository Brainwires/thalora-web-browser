//! Page scripts: loading a page into its JavaScript runtime, collecting,
//! fetching and running its `<script>` elements, pumping its event loop and
//! performing the navigations its scripts ask for.
//!
//! The page runtime (`deno_web_page::PageRuntime`) never loads a script and
//! never navigates by itself. Fetching scripts — with the page's
//! Content-Security-Policy, Subresource Integrity, mixed-content and SSRF
//! checks and the resource cache — and navigation are done here, in Rust.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use deno_web_page::{FormSubmission, NavigationRequest};
use thalora_dom::{Document, NodeId};

use super::mixed_content::{MixedContentResult, ResourceType, should_block_mixed_content};
use crate::engine::browser::InteractionResponse;
use crate::engine::browser::page_js::{JsError, JsPage};
use crate::engine::security::SsrfProtection;

/// How long the event loop is pumped after `load` (timers, fetches,
/// promises) before navigation returns.
const LOAD_IDLE_BUDGET: Duration = Duration::from_millis(1000);
/// The same, when the caller asked to wait for JavaScript (`wait_for_js`).
const LOAD_IDLE_BUDGET_WAIT_FOR_JS: Duration = Duration::from_millis(5000);
/// How long the event loop is pumped after an interaction (click, typing,
/// evaluation) so that handlers' follow-up work shows in the result.
pub(super) const INTERACTION_SETTLE: Duration = Duration::from_millis(250);
/// The slice the event loop is pumped for between two checks of a wait
/// condition.
pub(super) const WAIT_POLL: Duration = Duration::from_millis(50);
/// How many navigations requested by scripts are followed in a row
/// (`location.href = …` on load, and so on) before the rest are dropped.
const MAX_SCRIPT_NAVIGATIONS: usize = 5;
/// The largest external script that is run.
const MAX_SCRIPT_BYTES: usize = 16 * 1024 * 1024;
/// After this long, the remaining scripts of a page are not started.
const PAGE_SCRIPTS_DEADLINE: Duration = Duration::from_secs(60);
/// How many times in a row scripts inserted by scripts are looked for (a
/// script that inserts a script that inserts a script…).
const MAX_INSERTED_SCRIPT_ROUNDS: usize = 8;
/// How many `<script>` elements of one page are ever considered.
const MAX_PAGE_SCRIPTS: usize = 512;

/// Fires `load` or `error` at the n-th `<script>` of the document.
const SCRIPT_EVENT_JS: &str = r#"function (index, type) {
  const script = document.querySelectorAll("script")[index];
  if (script) script.dispatchEvent(new Event(type));
}"#;

/// What happens to the history when a page is loaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HistoryMode {
    /// A new entry after the current one.
    Push,
    /// The current entry is replaced (`location.replace()`).
    Replace,
    /// The history is left alone (reload, back/forward).
    Keep,
}

/// When a classic script runs, relative to the others.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// In document order, before `DOMContentLoaded`: inline scripts and
    /// external scripts without `async`/`defer`.
    Blocking,
    /// `defer`: after the blocking ones, before `DOMContentLoaded`.
    Deferred,
    /// `async`: whenever it arrives; here, after `DOMContentLoaded` and
    /// before `load`.
    Async,
}

/// A `<script>` element as found in the document.
#[derive(Debug, PartialEq, Eq)]
struct PageScript {
    node: NodeId,
    kind: ScriptKind,
    /// The `src` attribute (an external script), unresolved.
    src: Option<String>,
    /// The element's text (the source of an inline script).
    text: String,
    phase: Phase,
    integrity: Option<String>,
    nonce: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScriptKind {
    Classic,
    Module,
    /// A data block (`application/json`, `application/ld+json`, templates):
    /// never executed.
    Data,
}

/// The kind of script a `type` attribute value names.
fn script_kind(type_attr: Option<&str>) -> ScriptKind {
    let Some(type_attr) = type_attr else {
        return ScriptKind::Classic;
    };
    let essence = type_attr
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    match essence.as_str() {
        "" => ScriptKind::Classic,
        "module" => ScriptKind::Module,
        "text/javascript"
        | "application/javascript"
        | "text/ecmascript"
        | "application/ecmascript"
        | "application/x-javascript"
        | "application/x-ecmascript"
        | "text/x-javascript"
        | "text/x-ecmascript"
        | "text/jscript"
        | "text/livescript" => ScriptKind::Classic,
        other if other.starts_with("text/javascript1.") => ScriptKind::Classic,
        _ => ScriptKind::Data,
    }
}

/// Every `<script>` element of the document, in document order.
fn script_nodes(doc: &Document) -> Vec<NodeId> {
    doc.query_selector_all(doc.document(), "script")
        .unwrap_or_default()
}

/// The `<script>` element `node` as the loader sees it.
fn describe_script(doc: &Document, node: NodeId) -> Option<PageScript> {
    let element = doc.as_element(node)?;
    let src = element.attr("src").map(|src| src.trim().to_string());
    // `async` and `defer` only apply to external scripts, and `async` wins.
    let phase = if src.is_none() {
        Phase::Blocking
    } else if element.has_attr("async") {
        Phase::Async
    } else if element.has_attr("defer") {
        Phase::Deferred
    } else {
        Phase::Blocking
    };
    Some(PageScript {
        node,
        kind: script_kind(element.attr("type")),
        src,
        text: doc.text_content(node).ok().flatten().unwrap_or_default(),
        phase,
        integrity: element
            .attr("integrity")
            .filter(|i| !i.trim().is_empty())
            .map(str::to_string),
        nonce: element
            .attr("nonce")
            .filter(|n| !n.is_empty())
            .map(str::to_string),
    })
}

/// A classic script that passed the checks: its name (the URL it came
/// from) and source.
struct RunnableScript {
    node: NodeId,
    name: String,
    /// `None`: an external script that could not be loaded (its element
    /// gets an `error` event).
    source: Option<String>,
    external: bool,
    phase: Phase,
}

/// The base64 SHA-256 of an inline script, as CSP hash sources give it.
fn sha256_base64(text: &str) -> String {
    use base64::Engine as _;
    use digest::Digest;
    base64::engine::general_purpose::STANDARD.encode(sha2::Sha256::digest(text.as_bytes()))
}

/// Fetch one external script. The URL has passed the CSP and mixed-content
/// checks; the SSRF guard and the response checks are here.
async fn fetch_script_source(client: reqwest::Client, url: String) -> Result<String> {
    SsrfProtection::new().is_safe_url(&url)?;
    let response = client
        .get(&url)
        .header(reqwest::header::ACCEPT, "*/*")
        .send()
        .await
        .map_err(|e| anyhow!("request failed: {e}"))?;
    if !response.status().is_success() {
        return Err(anyhow!("HTTP status {}", response.status()));
    }
    // `X-Content-Type-Options: nosniff` forbids running a response that is
    // not labelled as a script.
    let header = |name: &str| {
        response
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.trim().to_ascii_lowercase())
    };
    if header("x-content-type-options").as_deref() == Some("nosniff") {
        let content_type = header("content-type").unwrap_or_default();
        if script_kind(Some(&content_type)) != ScriptKind::Classic || content_type.is_empty() {
            return Err(anyhow!(
                "refused: MIME type '{content_type}' is not a script type and nosniff is set"
            ));
        }
    }
    let source = response
        .text()
        .await
        .map_err(|e| anyhow!("reading the response failed: {e}"))?;
    if source.len() > MAX_SCRIPT_BYTES {
        return Err(anyhow!("script is larger than {MAX_SCRIPT_BYTES} bytes"));
    }
    Ok(source)
}

/// Whether navigating from `current` to `target` stays in the same document
/// (only the fragment differs).
fn is_same_document_navigation(current: &str, target: &str) -> bool {
    let (Ok(mut current), Ok(mut target)) = (url::Url::parse(current), url::Url::parse(target))
    else {
        return false;
    };
    if target.fragment().is_none() {
        return false;
    }
    current.set_fragment(None);
    target.set_fragment(None);
    current == target
}

impl super::super::HeadlessWebBrowser {
    /// Make `html`, fetched from `url`, the current page.
    ///
    /// With JavaScript enabled the page runtime parses the markup, the
    /// page's classic scripts are fetched and run (inline and blocking ones
    /// in document order, then `defer`, `DOMContentLoaded`, `async`, `load`)
    /// and the event loop is pumped until it is idle or the budget is spent
    /// (1 s, 5 s with `wait_for_js`). Navigations the scripts ask for are
    /// left queued for [`follow_script_navigations`](Self::follow_script_navigations).
    ///
    /// The page is static (no script runs) when JavaScript is disabled, when
    /// the runtime cannot start, or when this is not a browser thread;
    /// [`PageDom::js_note`](crate::engine::browser::PageDom::js_note) says
    /// which.
    ///
    /// Returns whether any script ran.
    pub(crate) async fn load_page(&mut self, url: &str, html: &str, wait_for_js: bool) -> bool {
        if !self.js_settings.enabled {
            self.page.load(Some(url.to_string()), html);
            self.page
                .set_js_note("JavaScript is disabled for this browser (THALORA_JS=0)");
            return false;
        }
        if html.is_empty() {
            // No content, nothing to run: not worth an isolate
            self.page.load(Some(url.to_string()), html);
            return false;
        }
        if !self.page.load_with_js(url, html, &self.js_settings) {
            return false;
        }
        let budget = if wait_for_js {
            LOAD_IDLE_BUDGET_WAIT_FOR_JS
        } else {
            LOAD_IDLE_BUDGET
        };
        self.run_page_scripts(url, budget).await
    }

    /// Fetch and run the scripts of the page that was just loaded, fire the
    /// lifecycle events and let the page settle. Returns whether any script
    /// ran.
    async fn run_page_scripts(&mut self, page_url: &str, idle_budget: Duration) -> bool {
        let scripts = self.take_new_scripts();
        let runnable = self.prepare_scripts(page_url, scripts).await;
        let started = Instant::now();
        let mut ran = 0usize;

        let run_phase = |browser: &mut Self, phase: Phase, ran: &mut usize| -> bool {
            for script in runnable.iter().filter(|script| script.phase == phase) {
                if started.elapsed() > PAGE_SCRIPTS_DEADLINE {
                    browser.page.push_console(
                        "error",
                        format!(
                            "The page's scripts ran longer than {} s; {} was not started",
                            PAGE_SCRIPTS_DEADLINE.as_secs(),
                            script.name
                        ),
                    );
                    continue;
                }
                *ran += usize::from(script.source.is_some());
                if !browser.run_prepared_script(script) {
                    return false;
                }
            }
            true
        };

        // Each step only if the page's JavaScript is still alive
        let alive = run_phase(self, Phase::Blocking, &mut ran)
            && run_phase(self, Phase::Deferred, &mut ran)
            && {
                ran += self.run_inserted_scripts().await;
                self.lifecycle("DOMContentLoaded", JsPage::fire_dom_content_loaded)
            }
            && run_phase(self, Phase::Async, &mut ran)
            && {
                ran += self.run_inserted_scripts().await;
                self.lifecycle("load", JsPage::fire_load)
            };
        if alive {
            self.settle(idle_budget).await;
        }
        tracing::debug!(
            "page scripts: {} ran in {} ms for {}",
            ran,
            started.elapsed().as_millis(),
            page_url
        );
        ran > 0
    }

    /// The `<script>` elements of the document that were not seen before, in
    /// document order: all of them right after a load, later the ones
    /// scripts inserted.
    fn take_new_scripts(&mut self) -> Vec<PageScript> {
        // Ids first: the sources are only copied for the new ones
        let nodes = self.page.with_doc(script_nodes);
        let new: Vec<NodeId> = nodes
            .into_iter()
            .filter(|&node| self.page.mark_script_seen(node, MAX_PAGE_SCRIPTS))
            .collect();
        if new.is_empty() {
            return Vec::new();
        }
        self.page.with_doc(|doc| {
            new.into_iter()
                .filter_map(|node| describe_script(doc, node))
                .collect()
        })
    }

    /// Run the scripts that scripts inserted into the document
    /// (`document.createElement("script")` + `appendChild`), in document
    /// order, until no new one appears. Returns how many ran.
    ///
    /// The page runtime does not tell the host when a script element is
    /// inserted, so they are found by looking at the document after scripts
    /// and event loop tasks ran. Differences from a browser: an inserted
    /// inline script runs after the script that inserted it has finished
    /// (not during the insertion), a script removed again before the host
    /// looks never runs, and scripts created by `innerHTML` run too.
    async fn run_inserted_scripts(&mut self) -> usize {
        let mut ran = 0;
        for _ in 0..MAX_INSERTED_SCRIPT_ROUNDS {
            if !self.page.has_js() {
                break;
            }
            let scripts = self.take_new_scripts();
            if scripts.is_empty() {
                break;
            }
            let Some(page_url) = self.get_current_url() else {
                break;
            };
            for script in self.prepare_scripts(&page_url, scripts).await {
                ran += usize::from(script.source.is_some());
                if !self.run_prepared_script(&script) {
                    return ran;
                }
            }
        }
        ran
    }

    /// Run a prepared script, then fire `load` at its element if it is
    /// external (`error` if it could not be loaded). `false` means the
    /// page's JavaScript is finished.
    fn run_prepared_script(&mut self, script: &RunnableScript) -> bool {
        let alive = match &script.source {
            Some(source) => self.run_script(&script.name, source.clone()),
            None => true,
        };
        if alive && script.external {
            let index = self.page.with_doc(|doc| {
                doc.query_selector_all(doc.document(), "script")
                    .unwrap_or_default()
                    .iter()
                    .position(|&node| node == script.node)
            });
            if let Some(index) = index {
                let event = if script.source.is_some() {
                    "load"
                } else {
                    "error"
                };
                // A throwing listener is reported like any uncaught error
                if let Err(e) = self.call_js(SCRIPT_EVENT_JS, serde_json::json!([index, event])) {
                    self.page.push_console("error", e.to_string());
                }
            }
        }
        alive
    }

    /// Run one script of the page. An exception or a timeout is reported on
    /// the page's console and the page goes on; `false` means the page's
    /// JavaScript is finished (out of memory).
    fn run_script(&mut self, name: &str, source: String) -> bool {
        let Some(js) = self.page.js() else {
            return false;
        };
        let result = js.execute_script(name, source);
        self.page.collect_js_output();
        match result {
            Ok(()) => true,
            Err(JsError::Exception(message)) => {
                let text = if message.starts_with("Uncaught") {
                    message
                } else {
                    format!("Uncaught {message}")
                };
                self.page.push_console("error", text);
                true
            }
            Err(error) => {
                self.page.push_console("error", format!("{error} ({name})"));
                error != JsError::OutOfMemory
            }
        }
    }

    /// Fire a lifecycle event. `false`: the page's JavaScript is finished.
    fn lifecycle(&mut self, event: &str, fire: fn(&mut JsPage) -> Result<(), JsError>) -> bool {
        let Some(js) = self.page.js() else {
            return false;
        };
        let result = fire(js);
        self.page.collect_js_output();
        match result {
            Ok(()) => true,
            Err(error) => {
                self.page
                    .push_console("error", format!("{error} (in a {event} listener)"));
                error != JsError::OutOfMemory
            }
        }
    }

    /// Decide which scripts run (type, CSP, mixed content), fetch the
    /// external ones (cache, SSRF guard, SRI) and return them in document
    /// order. What is refused or fails is reported on the page's console.
    async fn prepare_scripts(
        &mut self,
        page_url: &str,
        scripts: Vec<PageScript>,
    ) -> Vec<RunnableScript> {
        /// A classic script that passed the checks, before its source is known.
        enum Pending {
            Inline(String),
            External {
                url: String,
                integrity: Option<String>,
            },
        }

        let mut pending: Vec<(NodeId, Pending, Phase)> = Vec::new();
        let mut runnable = Vec::new();
        // An external script that is not loaded: its element gets `error`
        let failed = |node: NodeId, name: &str, phase: Phase| RunnableScript {
            node,
            name: name.to_string(),
            source: None,
            external: true,
            phase,
        };
        for script in scripts {
            match script.kind {
                ScriptKind::Data => continue,
                ScriptKind::Module => {
                    // TODO(a3): module scripts need a module loader in the
                    // page runtime (fork: deno_web_page has none yet).
                    let what = script.src.as_deref().unwrap_or("an inline module script");
                    tracing::debug!("skipping module script {what} on {page_url}");
                    self.page.push_console(
                        "warning",
                        format!("Module scripts are not supported yet; skipped {what}"),
                    );
                    continue;
                }
                ScriptKind::Classic => {}
            }

            let Some(src) = script.src else {
                if script.text.trim().is_empty() {
                    continue;
                }
                if let Some(csp) = &self.csp_policy
                    && !csp.allows_inline_script(
                        script.nonce.as_deref(),
                        Some(&sha256_base64(&script.text)),
                    )
                {
                    self.page.push_console(
                        "error",
                        "Refused to execute an inline script: it violates the page's \
                         Content-Security-Policy (script-src)",
                    );
                    continue;
                }
                pending.push((script.node, Pending::Inline(script.text), script.phase));
                continue;
            };

            if src.is_empty() {
                continue;
            }
            let url = match self.resolve_script_url(page_url, &src) {
                Ok(url) => url,
                Err(e) => {
                    self.page
                        .push_console("error", format!("Failed to load script '{src}': {e}"));
                    runnable.push(failed(script.node, &src, script.phase));
                    continue;
                }
            };
            if !(url.starts_with("http://") || url.starts_with("https://")) {
                self.page.push_console(
                    "warning",
                    format!("Skipped script {url}: only http(s) scripts are loaded"),
                );
                continue;
            }
            if should_block_mixed_content(page_url, &url, ResourceType::Script)
                == MixedContentResult::Block
            {
                self.page.push_console(
                    "error",
                    format!("Mixed content: blocked insecure script {url} on a secure page"),
                );
                runnable.push(failed(script.node, &url, script.phase));
                continue;
            }
            if let Some(csp) = &self.csp_policy
                && !csp.allows_external_script(&url, Some(page_url))
                && !csp.script_nonce_matches(script.nonce.as_deref())
            {
                self.page.push_console(
                    "error",
                    format!(
                        "Refused to load the script {url}: it violates the page's \
                         Content-Security-Policy (script-src)"
                    ),
                );
                runnable.push(failed(script.node, &url, script.phase));
                continue;
            }
            pending.push((
                script.node,
                Pending::External {
                    url,
                    integrity: script.integrity,
                },
                script.phase,
            ));
        }

        // Sources of the external scripts: the cache first, then one
        // concurrent fetch per distinct URL.
        let mut sources: HashMap<String, Result<String, String>> = HashMap::new();
        let mut to_fetch: Vec<String> = Vec::new();
        for (_, script, _) in &pending {
            let Pending::External { url, .. } = script else {
                continue;
            };
            if sources.contains_key(url) || to_fetch.contains(url) {
                continue;
            }
            if !self.bypass_cache
                && let Some(cached) = self.resource_cache.get(url)
            {
                tracing::debug!("CACHE HIT (script): {}", url);
                sources.insert(url.clone(), Ok(cached.content.clone()));
            } else {
                to_fetch.push(url.clone());
            }
        }
        let fetched = futures::future::join_all(
            to_fetch
                .iter()
                .map(|url| fetch_script_source(self.client.clone(), url.clone())),
        )
        .await;
        for (url, result) in to_fetch.into_iter().zip(fetched) {
            tracing::debug!("CACHE MISS (script): {} ok={}", url, result.is_ok());
            sources.insert(url, result.map_err(|e| e.to_string()));
        }

        for (node, script, phase) in pending {
            match script {
                Pending::Inline(source) => runnable.push(RunnableScript {
                    node,
                    name: page_url.to_string(),
                    source: Some(source),
                    external: false,
                    phase,
                }),
                Pending::External { url, integrity } => match sources.get(&url) {
                    Some(Ok(source)) => {
                        if let Some(integrity) = &integrity
                            && !super::sri::verify_integrity(source.as_bytes(), integrity)
                        {
                            self.page.push_console(
                                "error",
                                format!(
                                    "Failed to find a valid digest in the 'integrity' attribute \
                                     for {url}; the script was blocked"
                                ),
                            );
                            runnable.push(failed(node, &url, phase));
                            continue;
                        }
                        self.resource_cache.insert(url.clone(), source.clone());
                        runnable.push(RunnableScript {
                            node,
                            name: url,
                            source: Some(source.clone()),
                            external: true,
                            phase,
                        });
                    }
                    Some(Err(reason)) => {
                        self.page.push_console(
                            "error",
                            format!("Failed to load script {url}: {reason}"),
                        );
                        runnable.push(failed(node, &url, phase));
                    }
                    None => {}
                },
            }
        }
        // Back in document order (the refused ones were pushed first)
        let order: HashMap<NodeId, usize> = self.page.with_doc(|doc| {
            doc.query_selector_all(doc.document(), "script")
                .unwrap_or_default()
                .into_iter()
                .enumerate()
                .map(|(index, node)| (node, index))
                .collect()
        });
        runnable.sort_by_key(|script| order.get(&script.node).copied().unwrap_or(usize::MAX));
        runnable
    }

    /// Pump the page's event loop for up to `budget` (timers, fetches,
    /// promise jobs) and run the scripts that were inserted meanwhile.
    /// Returns whether the page is idle: nothing is pending any more. A
    /// static page is always idle.
    pub(crate) async fn settle(&mut self, budget: Duration) -> bool {
        let deadline = Instant::now() + budget;
        loop {
            let Some(js) = self.page.js() else {
                return true;
            };
            let remaining = deadline.saturating_duration_since(Instant::now());
            let result = js.run_until_idle(remaining).await;
            self.page.collect_js_output();
            let idle = match result {
                Ok(idle) => idle,
                Err(error) => {
                    self.page.push_console("error", error.to_string());
                    // Out of memory: nothing will ever run again.
                    return error == JsError::OutOfMemory;
                }
            };
            if self.run_inserted_scripts().await == 0 {
                return idle;
            }
            // The inserted scripts may have started new work
            if Instant::now() >= deadline {
                return false;
            }
        }
    }

    /// One slice of pumping between two tool calls, for the browser thread's
    /// idle loop. Returns whether the page still has pending work.
    ///
    /// Cancel-safe (the loop drops it when a job arrives). Navigations the
    /// page asked for are only queued: see
    /// [`has_script_navigation`](Self::has_script_navigation).
    pub(crate) async fn pump_background(&mut self, budget: Duration) -> bool {
        let Some(js) = self.page.js() else {
            return false;
        };
        let result = js.run_until_idle(budget).await;
        self.page.collect_js_output();
        match result {
            Ok(idle) => !idle,
            Err(error) => {
                self.page.push_console("error", error.to_string());
                false
            }
        }
    }

    /// Whether page scripts asked for a navigation that has not been
    /// performed yet.
    pub(crate) fn has_script_navigation(&self) -> bool {
        self.page.has_navigation_request()
    }

    /// Perform the navigations page scripts asked for (`location.href = …`,
    /// `location.reload()`, `form.submit()`), up to a few in a row. Returns
    /// the content of the last page loaded, if any navigation happened. A
    /// failed navigation is reported on the console of the page that stays.
    pub(crate) async fn follow_script_navigations(&mut self) -> Option<String> {
        let mut content = None;
        for _ in 0..MAX_SCRIPT_NAVIGATIONS {
            let Some(request) = self.page.take_navigation_request() else {
                return content;
            };
            match self.perform_navigation_request(request).await {
                Ok(Some(loaded)) => content = Some(loaded),
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!("navigation requested by the page failed: {e}");
                    self.page.push_console(
                        "error",
                        format!("A navigation requested by the page failed: {e}"),
                    );
                    return content;
                }
            }
        }
        if self.page.take_navigation_request().is_some() {
            tracing::warn!("page keeps navigating; stopped after {MAX_SCRIPT_NAVIGATIONS} hops");
            self.page.push_console(
                "warning",
                format!(
                    "The page kept navigating; stopped following after {MAX_SCRIPT_NAVIGATIONS} navigations"
                ),
            );
        }
        content
    }

    /// Perform one navigation a page asked for. `Ok(None)`: nothing to load
    /// (a fragment change, a non-http(s) URL, a `dialog` form).
    pub(super) async fn perform_navigation_request(
        &mut self,
        request: NavigationRequest,
    ) -> Result<Option<String>> {
        let (url, history) = match request {
            NavigationRequest::Form(submission) => {
                return Ok(self.send_form_submission(submission).await?.new_content);
            }
            NavigationRequest::Reload => {
                let url = self
                    .get_current_url()
                    .ok_or_else(|| anyhow!("No current page to reload"))?;
                (url, HistoryMode::Keep)
            }
            NavigationRequest::Link { url, .. } => (url, HistoryMode::Push),
            NavigationRequest::Location { url, replace } => (
                url,
                if replace {
                    HistoryMode::Replace
                } else {
                    HistoryMode::Push
                },
            ),
        };
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            tracing::debug!("not navigating to {url}: only http(s) URLs are loaded");
            return Ok(None);
        }
        if history != HistoryMode::Keep
            && self
                .page
                .url()
                .is_some_and(|current| is_same_document_navigation(current, &url))
        {
            // TODO(a3): the page runtime has no way to be told its URL
            // changed, so `location.hash` keeps its old value.
            tracing::debug!("same-document navigation to {url}: nothing to load");
            return Ok(None);
        }
        self.fetch_and_load(&url, false, history).await.map(Some)
    }

    /// Submit a form the page asked to submit (a clicked submit button,
    /// `form.submit()`, `requestSubmit()`): the entry list is the page's, as
    /// of the submission, plus the secrets filled into that form.
    pub(super) async fn send_form_submission(
        &mut self,
        submission: FormSubmission,
    ) -> Result<InteractionResponse> {
        if submission.method == "dialog" {
            return Ok(InteractionResponse {
                success: true,
                message: "Form with method=dialog closed its dialog; nothing was sent".to_string(),
                redirect_url: None,
                new_content: None,
            });
        }
        let action = url::Url::parse(&submission.action)
            .map_err(|e| anyhow!("Invalid form action '{}': {e}", submission.action))?;
        let mut entries = submission.entries;
        self.overlay_secrets(&action, &mut entries);
        self.send_form(action, &submission.method, &submission.enctype, entries)
            .await
    }

    /// Put the secrets filled into the form that submits to `action` into
    /// its entry list. Page JavaScript never sees a secret, so the entries
    /// it built carry the fields' visible (empty) values.
    fn overlay_secrets(&self, action: &url::Url, entries: &mut Vec<(String, String)>) {
        let page = &self.page;
        let Some(base) = page.url().and_then(|url| url::Url::parse(url).ok()) else {
            return;
        };
        let mut action = action.clone();
        action.set_fragment(None);
        let secrets: Vec<(String, String)> = page.with_doc(|doc| {
            page.secret_nodes()
                .into_iter()
                .filter_map(|node| {
                    let name = doc
                        .as_element(node)?
                        .attr("name")
                        .filter(|n| !n.is_empty())?;
                    // Only into the form the field belongs to: a script must
                    // not be able to route a secret to another action.
                    let form = super::forms::form_of(doc, node)?;
                    let form_action = doc.as_element(form)?.attr("action").unwrap_or("");
                    let mut resolved = base.join(form_action).ok()?;
                    resolved.set_fragment(None);
                    (resolved == action).then(|| {
                        (
                            name.to_string(),
                            page.secret_value(node).unwrap_or_default().to_string(),
                        )
                    })
                })
                .collect()
        });
        super::forms::merge_entries(entries, secrets);
    }

    /// Evaluate JavaScript in the current page and return the completion
    /// value as JSON (`undefined` is `null`; a value JSON cannot express is
    /// its string form; a DOM node is its markup). Afterwards the event loop
    /// is pumped briefly and navigations the script asked for are performed.
    ///
    /// A promise is not awaited: the page runtime has no asynchronous
    /// evaluation yet. TODO(a3): use `evaluate_json_async` once the pinned
    /// fork revision has it. Until then, store the settled value in a
    /// global and read it with a second evaluation.
    pub async fn evaluate_javascript(&mut self, source: &str) -> Result<serde_json::Value> {
        if self.page.is_empty() {
            return Err(anyhow!("No page loaded. Navigate to a page first."));
        }
        let value = self.with_js(|js| js.evaluate_json(source.to_string()))?;
        self.settle(INTERACTION_SETTLE).await;
        self.follow_script_navigations().await;
        Ok(value)
    }

    /// What the current page logged (`console.*`), the errors nothing
    /// caught and the browser's notes about its scripts, oldest first, with
    /// [`javascript_status`](Self::javascript_status). `clear` empties the
    /// buffer after reading.
    pub fn console_messages(
        &mut self,
        clear: bool,
    ) -> (Vec<crate::engine::browser::types::ConsoleEntry>, String) {
        self.page.collect_js_output();
        let messages = self.page.console_messages().cloned().collect();
        if clear {
            self.page.clear_console();
        }
        (messages, self.javascript_status())
    }

    /// Whether the current page runs JavaScript, for tool output:
    /// `"running"`, or `"static: <why>"`.
    pub fn javascript_status(&self) -> String {
        if self.page.has_js() {
            "running".to_string()
        } else {
            match self.page.js_note() {
                Some(note) => format!("static: {note}"),
                None => "static: no script ran on this page".to_string(),
            }
        }
    }

    /// Call into the page's JavaScript runtime and collect what the call
    /// logged and requested. Fails when the page is static.
    pub(super) fn with_js<R>(
        &mut self,
        call: impl FnOnce(&mut JsPage) -> Result<R, JsError>,
    ) -> Result<R> {
        let unavailable = self.js_unavailable();
        let Some(js) = self.page.js() else {
            return Err(unavailable);
        };
        let result = call(js);
        self.page.collect_js_output();
        Ok(result?)
    }

    /// Call the JavaScript function expression `function` in the page with
    /// the JSON array `arguments` as its arguments.
    pub(super) fn call_js(
        &mut self,
        function: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let source = format!("({function}).apply(null, {arguments})");
        self.with_js(|js| js.evaluate_json(source))
    }

    /// The error of an operation that needs page JavaScript on a static
    /// page, with the reason the page is static when there is one.
    pub(crate) fn js_unavailable(&self) -> anyhow::Error {
        match self.page.js_note() {
            Some(note) => anyhow!("{} ({note})", crate::engine::JS_UNAVAILABLE),
            None => crate::engine::js_unavailable(),
        }
    }
}

#[cfg(test)]
// A test's browser is only used by that test's job, on its browser thread,
// so holding its MutexGuard across .await cannot contend or deadlock.
#[allow(clippy::await_holding_lock)]
mod tests {
    use super::*;
    use crate::engine::browser::session_thread::{BrowserHandle, BrowserThread};
    use futures::FutureExt;
    use futures::future::LocalBoxFuture;

    /// The URL of the test pages: nothing is fetched from it.
    const PAGE_URL: &str = "https://app.example.test/page";

    /// Run `job` with a fresh browser on its own browser thread (where page
    /// JavaScript runs) and return its result.
    fn on_browser_thread<R: Send + 'static>(
        job: impl FnOnce(BrowserHandle) -> LocalBoxFuture<'static, R> + Send + 'static,
    ) -> R {
        let thread = BrowserThread::spawn("js-test").expect("browser thread");
        let result = thread.call_blocking(job).expect("job");
        thread.shutdown_blocking();
        result
    }

    /// Load `html` as the current page, scripts included. No storage on
    /// disk, nothing from the network (inline scripts only).
    async fn load(browser: &BrowserHandle, html: &str) {
        let mut b = browser.lock().unwrap();
        b.js_settings_mut().storage_dir = None;
        b.load_page(PAGE_URL, html, false).await;
    }

    #[test]
    fn inline_scripts_run_in_order_and_lifecycle_events_fire() {
        let (html, has_js, status) = on_browser_thread(|browser| {
            async move {
                load(
                    &browser,
                    r#"<!DOCTYPE html><title>t</title><ol id="log"></ol>
                    <script>
                      function log(text) {
                        var li = document.createElement('li');
                        li.textContent = text;
                        document.getElementById('log').appendChild(li);
                      }
                      document.addEventListener('DOMContentLoaded', function () { log('dcl ' + document.readyState); });
                      window.addEventListener('load', function () { log('load ' + document.readyState); });
                      log('first');
                    </script>
                    <script type="application/json">{"not": "run"}</script>
                    <script>log('second ' + location.href);</script>"#,
                )
                .await;
                let b = browser.lock().unwrap();
                (
                    b.get_current_content(),
                    b.page().has_js(),
                    b.javascript_status(),
                )
            }
            .boxed_local()
        });
        assert!(has_js);
        assert_eq!(status, "running");
        let expected = format!(
            "<li>first</li><li>second {PAGE_URL}</li><li>dcl interactive</li><li>load complete</li>"
        );
        assert!(html.contains(&expected), "{html}");
    }

    #[test]
    fn a_script_error_is_logged_and_the_next_script_still_runs() {
        let (html, console) = on_browser_thread(|browser| {
            async move {
                load(
                    &browser,
                    r#"<div id="out">before</div>
                    <script>console.log('one'); missingFunction();</script>
                    <script>document.getElementById('out').textContent = 'after';</script>
                    <script type="module">document.getElementById('out').textContent = 'module';</script>"#,
                )
                .await;
                let mut b = browser.lock().unwrap();
                (b.get_current_content(), b.console_messages(false).0)
            }
            .boxed_local()
        });
        assert!(html.contains(r#"<div id="out">after</div>"#), "{html}");
        let has = |level: &str, needle: &str| {
            console
                .iter()
                .any(|m| m.level == level && m.text.contains(needle))
        };
        assert!(has("log", "one"), "{console:?}");
        assert!(has("error", "missingFunction"), "{console:?}");
        assert!(
            has("warning", "Module scripts are not supported"),
            "{console:?}"
        );
    }

    #[test]
    fn scripts_inserted_by_scripts_run_once() {
        let html = on_browser_thread(|browser| {
            async move {
                load(
                    &browser,
                    r#"<ul id="items"></ul>
                    <script>
                      var inserted = document.createElement('script');
                      inserted.textContent =
                        "document.getElementById('items').insertAdjacentHTML('beforeend', '<li>inserted ran</li>');";
                      document.body.appendChild(inserted);
                    </script>"#,
                )
                .await;
                let mut b = browser.lock().unwrap();
                // Looking again finds nothing new
                b.settle(Duration::from_millis(50)).await;
                b.get_current_content()
            }
            .boxed_local()
        });
        assert_eq!(html.matches("<li>inserted ran</li>").count(), 1, "{html}");
    }

    #[test]
    fn click_handlers_timers_and_waiting_for_an_element() {
        let (clicked, early, found, never, waited) = on_browser_thread(|browser| {
            async move {
                load(
                    &browser,
                    r#"<button id="b" type="button">Go</button><div id="out"></div>
                    <script>
                      document.getElementById('b').addEventListener('click', function () {
                        document.getElementById('out').textContent = 'clicked';
                        setTimeout(function () {
                          var p = document.createElement('p');
                          p.id = 'late';
                          document.body.appendChild(p);
                        }, 600);
                      });
                    </script>"#,
                )
                .await;
                let mut b = browser.lock().unwrap();
                let clicked = b.click_element("#b").await.unwrap();
                // The click settles for less than the timer
                let early = b.element_exists("#late").unwrap();
                let found = b.wait_for_element("#late", 10_000).await.unwrap();
                // Nothing is pending any more: no waiting for the timeout
                let started = Instant::now();
                let never = b.wait_for_element("#never", 30_000).await.unwrap();
                (
                    (clicked.success, clicked.message, b.get_current_content()),
                    early,
                    found,
                    never,
                    started.elapsed(),
                )
            }
            .boxed_local()
        });
        assert!(clicked.0);
        assert_eq!(clicked.1, "Clicked element");
        assert!(
            clicked.2.contains(r#"<div id="out">clicked</div>"#),
            "{}",
            clicked.2
        );
        assert!(!early, "the timer must not have fired yet");
        assert!(found);
        assert!(!never);
        assert!(waited < Duration::from_secs(10), "{waited:?}");
    }

    #[test]
    fn the_page_keeps_running_between_jobs() {
        let thread = BrowserThread::spawn("js-pump-test").expect("browser thread");
        thread
            .call_blocking(|browser| {
                async move {
                    load(
                        &browser,
                        r#"<div id="out">waiting</div><button id="b" type="button">Go</button>
                        <script>
                          document.getElementById('b').onclick = function () {
                            setTimeout(function () {
                              document.getElementById('out').textContent = 'timer fired';
                            }, 600);
                          };
                        </script>"#,
                    )
                    .await;
                    browser.lock().unwrap().click_element("#b").await.unwrap();
                }
                .boxed_local()
            })
            .unwrap();
        // No job runs meanwhile: the browser thread pumps the page
        std::thread::sleep(Duration::from_millis(2500));
        let html = thread
            .call_blocking(|browser| {
                async move { browser.lock().unwrap().get_current_content() }.boxed_local()
            })
            .unwrap();
        thread.shutdown_blocking();
        assert!(html.contains("timer fired"), "{html}");
    }

    #[test]
    fn evaluation_returns_json_and_reports_exceptions() {
        let (object, undefined, node, kept, thrown, console) = on_browser_thread(|browser| {
            async move {
                load(&browser, r#"<p id="p">hello</p>"#).await;
                let mut b = browser.lock().unwrap();
                let object = b
                    .evaluate_javascript("({ n: 1 + 2, list: [1, 'two', null], text: document.getElementById('p').textContent })")
                    .await
                    .unwrap();
                let undefined = b.evaluate_javascript("undefined").await.unwrap();
                let node = b
                    .evaluate_javascript("document.getElementById('p')")
                    .await
                    .unwrap();
                b.evaluate_javascript("var kept = 41; console.warn('from eval')")
                    .await
                    .unwrap();
                let kept = b.execute_javascript("kept + 1").await.unwrap();
                let thrown = b
                    .evaluate_javascript("throw new TypeError('eval boom')")
                    .await
                    .unwrap_err()
                    .to_string();
                (object, undefined, node, kept, thrown, b.console_messages(true).0)
            }
            .boxed_local()
        });
        assert_eq!(
            object,
            serde_json::json!({"n": 3, "list": [1, "two", null], "text": "hello"})
        );
        assert_eq!(undefined, serde_json::Value::Null);
        assert_eq!(node, serde_json::json!(r#"<p id="p">hello</p>"#));
        assert_eq!(kept, "42");
        assert!(thrown.contains("eval boom"), "{thrown}");
        assert!(
            console
                .iter()
                .any(|m| m.level == "warning" && m.text.contains("from eval")),
            "{console:?}"
        );
    }

    #[test]
    fn a_runaway_script_is_terminated_and_the_page_survives() {
        let (stuck, after, took) = on_browser_thread(|browser| {
            async move {
                {
                    let mut b = browser.lock().unwrap();
                    b.js_settings_mut().script_timeout = Duration::from_millis(300);
                }
                load(&browser, "<p>page</p>").await;
                let mut b = browser.lock().unwrap();
                let started = Instant::now();
                let stuck = b
                    .evaluate_javascript("while (true) {}")
                    .await
                    .unwrap_err()
                    .to_string();
                let took = started.elapsed();
                let after = b.evaluate_javascript("6 * 7").await.unwrap();
                (stuck, after, took)
            }
            .boxed_local()
        });
        assert!(stuck.contains("was terminated"), "{stuck}");
        assert_eq!(after, serde_json::json!(42));
        assert!(took < Duration::from_secs(20), "{took:?}");
    }

    #[test]
    fn typing_sets_live_state_and_secrets_stay_out_of_the_page() {
        let (value, events, html, snapshot, secret_seen) = on_browser_thread(|browser| {
            async move {
                load(
                    &browser,
                    r#"<form action="/login" method="post">
                      <input id="user" name="user">
                      <input id="pass" name="pass" type="password">
                    </form>
                    <script>
                      var events = [];
                      document.getElementById('user').addEventListener('input', function (e) {
                        events.push('input:' + e.target.value);
                      });
                      document.getElementById('user').addEventListener('change', function () {
                        events.push('change');
                      });
                    </script>"#,
                )
                .await;
                let mut b = browser.lock().unwrap();
                b.type_text_into_element("#user", "ann", true)
                    .await
                    .unwrap();
                b.fill_secret("#pass", "hunter2").await.unwrap();
                let value = b
                    .evaluate_javascript("document.getElementById('user').value")
                    .await
                    .unwrap();
                let events = b.evaluate_javascript("events").await.unwrap();
                let secret_seen = b
                    .evaluate_javascript("document.getElementById('pass').value")
                    .await
                    .unwrap();
                let snapshot = b.snapshot(Default::default(), None).unwrap().text;
                (
                    value,
                    events,
                    b.get_current_content(),
                    snapshot,
                    secret_seen,
                )
            }
            .boxed_local()
        });
        assert_eq!(value, serde_json::json!("ann"));
        assert_eq!(events, serde_json::json!(["input:ann", "change"]));
        // The value is state of the control, not markup
        assert!(html.contains(r#"<input id="user" name="user">"#), "{html}");
        assert!(snapshot.contains(r#"value="ann""#), "{snapshot}");
        // A secret is never given to the page, nor shown
        assert_eq!(secret_seen, serde_json::json!(""));
        assert!(!html.contains("hunter2") && !snapshot.contains("hunter2"));
    }

    #[test]
    fn keys_hover_and_scroll_dispatch_events() {
        use crate::engine::browser::types::ElementAction;
        let (messages, state) = on_browser_thread(|browser| {
            async move {
                load(
                    &browser,
                    r#"<input id="k"><div id="h">away</div>
                    <script>
                      var seen = [];
                      document.getElementById('k').addEventListener('keydown', function (e) { seen.push('down:' + e.key); });
                      document.getElementById('k').addEventListener('keyup', function (e) { seen.push('up:' + e.key); });
                      document.getElementById('h').addEventListener('mouseover', function () { seen.push('over'); });
                      window.addEventListener('scroll', function () { seen.push('scroll'); });
                    </script>"#,
                )
                .await;
                let mut b = browser.lock().unwrap();
                let mut messages = Vec::new();
                for (selector, action) in [
                    (Some("#k"), ElementAction::PressKey("a".to_string())),
                    (Some("#h"), ElementAction::Hover),
                    (None, ElementAction::Scroll(300)),
                ] {
                    messages.push(b.perform_action(selector, &action).await.unwrap().message);
                }
                let state = b
                    .evaluate_javascript("({ seen: seen, value: document.getElementById('k').value })")
                    .await
                    .unwrap();
                (messages, state)
            }
            .boxed_local()
        });
        assert_eq!(messages[0], "Pressed a; typed into the field");
        assert_eq!(
            state,
            serde_json::json!({"seen": ["down:a", "up:a", "over", "scroll"], "value": "a"})
        );
    }

    #[test]
    fn javascript_can_be_disabled_per_browser() {
        let (has_js, status, html, error) = on_browser_thread(|browser| {
            async move {
                browser.lock().unwrap().set_javascript_enabled(false);
                load(
                    &browser,
                    r#"<div id="out">static</div>
                    <script>document.getElementById('out').textContent = 'ran';</script>"#,
                )
                .await;
                let mut b = browser.lock().unwrap();
                let error = b.evaluate_javascript("1").await.unwrap_err().to_string();
                (
                    b.page().has_js(),
                    b.javascript_status(),
                    b.get_current_content(),
                    error,
                )
            }
            .boxed_local()
        });
        assert!(!has_js);
        assert!(
            status.starts_with("static: JavaScript is disabled"),
            "{status}"
        );
        assert!(html.contains(r#"<div id="out">static</div>"#), "{html}");
        assert!(error.starts_with(crate::engine::JS_UNAVAILABLE), "{error}");
    }

    #[tokio::test]
    async fn a_page_loaded_outside_a_browser_thread_is_static() {
        let browser = crate::engine::browser::HeadlessWebBrowser::new();
        let mut b = browser.lock().unwrap();
        b.js_settings_mut().storage_dir = None;
        let ran = b
            .load_page(
                PAGE_URL,
                r#"<div id="out">static</div><script>document.getElementById('out').textContent = 'ran';</script>"#,
                false,
            )
            .await;
        assert!(!ran);
        assert!(!b.page().has_js());
        assert!(
            b.javascript_status().contains("browser thread"),
            "{}",
            b.javascript_status()
        );
        assert!(b.get_current_content().contains(">static<"));
        // Static interaction still works
        assert!(b.wait_for_element("#out", 1000).await.unwrap());
        let error = b.evaluate_javascript("1").await.unwrap_err().to_string();
        assert!(error.starts_with(crate::engine::JS_UNAVAILABLE), "{error}");
    }

    #[test]
    fn script_types() {
        assert_eq!(script_kind(None), ScriptKind::Classic);
        assert_eq!(script_kind(Some("")), ScriptKind::Classic);
        assert_eq!(script_kind(Some(" Text/JavaScript ")), ScriptKind::Classic);
        assert_eq!(
            script_kind(Some("application/javascript; charset=utf-8")),
            ScriptKind::Classic
        );
        assert_eq!(script_kind(Some("text/javascript1.5")), ScriptKind::Classic);
        assert_eq!(script_kind(Some("module")), ScriptKind::Module);
        assert_eq!(script_kind(Some("application/ld+json")), ScriptKind::Data);
        assert_eq!(script_kind(Some("text/template")), ScriptKind::Data);
        assert_eq!(script_kind(Some("importmap")), ScriptKind::Data);
    }

    #[test]
    fn scripts_are_collected_in_document_order_with_their_phase() {
        let doc = Document::parse_html(
            r#"<head>
                <script src="/a.js" defer></script>
                <script src="/b.js" async defer integrity="sha256-x" nonce="n1"></script>
                <script>inline()</script>
                <script async>inlineAsync()</script>
                <script type="application/json">{"a":1}</script>
                <script type="module" src="/m.js"></script>
            </head><body><script src=" /c.js "></script></body>"#,
        );
        let scripts: Vec<PageScript> = script_nodes(&doc)
            .into_iter()
            .filter_map(|node| describe_script(&doc, node))
            .collect();
        let summary: Vec<(ScriptKind, Option<&str>, Phase)> = scripts
            .iter()
            .map(|s| (s.kind, s.src.as_deref(), s.phase))
            .collect();
        assert_eq!(
            summary,
            vec![
                (ScriptKind::Classic, Some("/a.js"), Phase::Deferred),
                (ScriptKind::Classic, Some("/b.js"), Phase::Async),
                (ScriptKind::Classic, None, Phase::Blocking),
                // `async` means nothing on an inline script
                (ScriptKind::Classic, None, Phase::Blocking),
                (ScriptKind::Data, None, Phase::Blocking),
                (ScriptKind::Module, Some("/m.js"), Phase::Blocking),
                (ScriptKind::Classic, Some("/c.js"), Phase::Blocking),
            ]
        );
        assert_eq!(scripts[1].integrity.as_deref(), Some("sha256-x"));
        assert_eq!(scripts[1].nonce.as_deref(), Some("n1"));
        assert_eq!(scripts[2].text, "inline()");
    }

    #[test]
    fn fragment_changes_are_same_document_navigations() {
        let page = "https://e.com/a?x=1";
        assert!(is_same_document_navigation(page, "https://e.com/a?x=1#top"));
        assert!(is_same_document_navigation(
            "https://e.com/a?x=1#one",
            "https://e.com/a?x=1#two"
        ));
        // The same URL without a fragment is a reload
        assert!(!is_same_document_navigation(page, page));
        assert!(!is_same_document_navigation(
            page,
            "https://e.com/a?x=2#top"
        ));
        assert!(!is_same_document_navigation(page, "https://e.com/b#top"));
        assert!(!is_same_document_navigation(
            "not a url",
            "https://e.com/#a"
        ));
    }

    #[test]
    fn inline_script_hash_is_the_csp_form() {
        // echo -n "alert(1)" | openssl dgst -sha256 -binary | base64
        assert_eq!(
            sha256_base64("alert(1)"),
            "bhHHL3z2vDgxUt0W3dWQOrprscmda2Y5pLsLg4GF+pI="
        );
    }
}
