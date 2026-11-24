// End-to-end MCP tests: spawn the real `thalora` binary over stdio and drive
// it against the local fixture site (tests/fixtures/site).
//
// The binary is started with THALORA_ALLOW_LOOPBACK=1, which is honoured in
// debug builds (or with the `test-hooks` feature) so 127.0.0.1 is reachable.

use super::fixture_server::FixtureServer;
use super::mcp_harness::{McpTestHarness, McpToolResponse, create_harness_with_raw_env};
use serde_json::{Value, json};
use std::collections::HashMap;

fn fixture_harness() -> (McpTestHarness, FixtureServer) {
    let server = FixtureServer::start();
    let mut env = HashMap::new();
    env.insert("THALORA_PRESET".to_string(), "brainclaw".to_string());
    env.insert("THALORA_ALLOW_LOOPBACK".to_string(), "1".to_string());
    env.insert("THALORA_DISABLE_RATE_LIMIT".to_string(), "1".to_string());
    let harness = create_harness_with_raw_env(env).expect("Failed to create harness");
    (harness, server)
}

fn call(harness: &mut McpTestHarness, tool: &str, args: Value) -> McpToolResponse {
    harness
        .call_tool(tool, args)
        .unwrap_or_else(|e| panic!("{tool} failed at transport level: {e}"))
}

fn text(resp: &McpToolResponse) -> String {
    resp.content
        .iter()
        .filter_map(|c| c["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn call_ok(harness: &mut McpTestHarness, tool: &str, args: Value) -> String {
    let resp = call(harness, tool, args);
    assert!(!resp.content.is_empty(), "{tool} returned empty content");
    let body = text(&resp);
    assert!(!resp.is_error, "{tool} returned an error: {body}");
    body
}

fn page_content(harness: &mut McpTestHarness, session_id: &str) -> String {
    call_ok(
        harness,
        "browser_get_page_content",
        json!({"session_id": session_id}),
    )
}

#[test]
fn e2e_navigate_and_read_page_content() {
    let (mut h, site) = fixture_harness();
    let nav = call_ok(
        &mut h,
        "browser_navigate_to",
        json!({"url": site.url("/index.html"), "session_id": "e2e"}),
    );
    assert!(
        nav.contains("Thalora Fixture Site"),
        "navigate output: {nav}"
    );

    let content = page_content(&mut h, "e2e");
    assert!(
        content.contains("Thalora Fixture Site"),
        "page content: {content}"
    );
}

#[test]
fn e2e_snapshot_url_returns_page_text() {
    let (mut h, site) = fixture_harness();
    let snapshot = call_ok(
        &mut h,
        "snapshot_url",
        json!({"url": site.url("/index.html"), "wait_for_js": false}),
    );
    // Basic extraction: metadata and links
    assert!(
        snapshot.contains("Thalora Fixture Index") && snapshot.contains("Login form"),
        "snapshot output: {snapshot}"
    );
}

#[test]
fn e2e_fill_then_click_submit_posts_every_field() {
    let (mut h, site) = fixture_harness();
    call_ok(
        &mut h,
        "browser_navigate",
        json!({"url": site.url("/login.html"), "session_id": "login"}),
    );
    call_ok(
        &mut h,
        "browser_fill",
        json!({"selector": "#username", "value": "agent", "session_id": "login"}),
    );
    call_ok(
        &mut h,
        "browser_fill",
        json!({"selector": "#password", "value": "pw123", "session_id": "login"}),
    );
    call_ok(
        &mut h,
        "browser_click",
        json!({"selector": "#signin", "session_id": "login"}),
    );

    let echoed = page_content(&mut h, "login");
    for expected in [
        "csrf=fixture-csrf-token",
        "username=agent",
        "password=pw123",
        "remember=yes",
        "action=signin",
    ] {
        assert!(
            echoed.contains(expected),
            "missing {expected} in echoed POST: {echoed}"
        );
    }
    assert!(echoed.contains("POST"), "expected a POST: {echoed}");
}

#[test]
fn e2e_fill_form_without_submit_stays_on_page() {
    let (mut h, site) = fixture_harness();
    call_ok(
        &mut h,
        "browser_navigate_to",
        json!({"url": site.url("/login.html"), "session_id": "nosubmit"}),
    );
    let filled = call_ok(
        &mut h,
        "browser_fill_form",
        json!({
            "session_id": "nosubmit",
            "form_selector": "#search",
            "form_data": {"q": "hello"},
            "submit": false
        }),
    );
    assert!(
        filled.contains("\"submitted\": false"),
        "fill output: {filled}"
    );

    let content = page_content(&mut h, "nosubmit");
    assert!(
        content.contains("Sign in"),
        "should still be on login page: {content}"
    );
}

#[test]
fn e2e_fill_form_get_submits_query() {
    let (mut h, site) = fixture_harness();
    call_ok(
        &mut h,
        "browser_navigate_to",
        json!({"url": site.url("/login.html"), "session_id": "search"}),
    );
    call_ok(
        &mut h,
        "browser_fill_form",
        json!({
            "session_id": "search",
            "form_selector": "#search",
            "form_data": {"q": "rust"}
        }),
    );
    let echoed = page_content(&mut h, "search");
    assert!(echoed.contains("q=rust"), "expected GET query: {echoed}");
}

#[test]
fn e2e_click_link_navigates() {
    let (mut h, site) = fixture_harness();
    call_ok(
        &mut h,
        "browser_navigate_to",
        json!({"url": site.url("/index.html"), "session_id": "links"}),
    );
    call_ok(
        &mut h,
        "browser_click_element",
        json!({"selector": "#to-login", "session_id": "links"}),
    );
    let content = page_content(&mut h, "links");
    assert!(
        content.contains("login.html"),
        "expected login URL: {content}"
    );
    assert!(
        content.contains("Sign in"),
        "expected login page: {content}"
    );
}

#[test]
fn e2e_unknown_session_is_an_error() {
    let (mut h, _site) = fixture_harness();
    let resp = call(
        &mut h,
        "browser_get_page_content",
        json!({"session_id": "does-not-exist"}),
    );
    assert!(resp.is_error, "unknown session should error");
    assert!(text(&resp).contains("Unknown session"), "{}", text(&resp));
}

// ── Page JavaScript (V8) ────────────────────────────────────────────────────
//
// Pages run their scripts on V8 (the deno fork's page runtime). These tests
// drive js_app.html / js_app.js and the older script fixtures through the
// MCP tools.

/// A harness whose browser sessions keep `localStorage` in a scratch profile.
fn js_harness() -> (McpTestHarness, FixtureServer) {
    let server = FixtureServer::start();
    let mut env = HashMap::new();
    env.insert("THALORA_PRESET".to_string(), "brainclaw".to_string());
    env.insert("THALORA_ALLOW_LOOPBACK".to_string(), "1".to_string());
    env.insert("THALORA_DISABLE_RATE_LIMIT".to_string(), "1".to_string());
    env.insert(
        "THALORA_PROFILE_DIR".to_string(),
        std::env::temp_dir()
            .join(format!("thalora-e2e-profile-{}", std::process::id()))
            .to_string_lossy()
            .into_owned(),
    );
    let harness = create_harness_with_raw_env(env).expect("Failed to create harness");
    (harness, server)
}

fn open_js_app(h: &mut McpTestHarness, site: &FixtureServer, session: &str) -> String {
    call_ok(
        h,
        "browser_navigate_to",
        json!({"url": site.url("/js_app.html"), "session_id": session}),
    )
}

/// The JSON object inside a tool output that is wrapped in the
/// untrusted-page-content envelope.
fn json_of(body: &str) -> Value {
    let json = body
        .find('{')
        .zip(body.rfind('}'))
        .map(|(start, end)| &body[start..=end])
        .unwrap_or_else(|| panic!("no JSON object in: {body}"));
    serde_json::from_str(json).unwrap_or_else(|e| panic!("not JSON ({e}): {body}"))
}

/// Evaluate `expression` in `session` and return the tool's JSON output.
fn eval(h: &mut McpTestHarness, session: &str, expression: &str) -> Value {
    let body = call_ok(
        h,
        "browser_eval",
        json!({"expression": expression, "session_id": session}),
    );
    assert!(body.contains("<untrusted_page_content"), "{body}");
    json_of(&body)
}

#[test]
fn e2e_js_scripts_run_in_order_and_mutate_the_dom() {
    let (mut h, site) = js_harness();
    let nav = open_js_app(&mut h, &site, "js");
    assert!(nav.contains("\"javascript\": \"running\""), "{nav}");
    // The navigation result is the document as the scripts left it
    assert!(nav.contains("booted inline"), "{nav}");

    let content = page_content(&mut h, "js");
    assert!(content.contains("booted inline"), "{content}");
    // The deferred external script ran after the inline one
    assert!(
        content.contains("deferred script ran after booted inline"),
        "{content}"
    );
}

#[test]
fn e2e_js_dom_built_by_script_is_in_the_page() {
    let (mut h, site) = js_harness();
    call_ok(
        &mut h,
        "browser_navigate_to",
        json!({"url": site.url("/dom_mutation.html"), "session_id": "dom"}),
    );
    let content = page_content(&mut h, "dom");
    assert!(content.contains("added by script"), "{content}");
    assert!(content.contains("Inserted button"), "{content}");
    // A script element inserted by a script runs too, once
    assert_eq!(
        content.matches("from inserted script").count(),
        1,
        "{content}"
    );
}

#[test]
fn e2e_js_timers_microtasks_and_fetch_run_during_navigation() {
    let (mut h, site) = js_harness();
    call_ok(
        &mut h,
        "browser_navigate_to",
        json!({"url": site.url("/timers.html"), "session_id": "timers", "wait_for_js": true}),
    );
    let content = page_content(&mut h, "timers");
    for expected in ["timeout fired", "microtask ran", "raf fired"] {
        assert!(content.contains(expected), "missing {expected}: {content}");
    }

    // A fetch() the page starts on load; wait for what it renders
    call_ok(
        &mut h,
        "browser_navigate_to",
        json!({"url": site.url("/spa_fetch.html"), "session_id": "timers"}),
    );
    let waited = call_ok(
        &mut h,
        "browser_wait",
        json!({"session_id": "timers", "text": "fixture data loaded", "timeout_ms": 10000}),
    );
    assert!(waited.contains("\"met\": true"), "{waited}");
    let content = page_content(&mut h, "timers");
    assert!(content.contains("fixture data loaded"), "{content}");
}

#[test]
fn e2e_js_click_handler_runs() {
    let (mut h, site) = js_harness();
    open_js_app(&mut h, &site, "click");
    for expected in ["Clicked 1 times", "Clicked 2 times"] {
        let clicked = call_ok(
            &mut h,
            "browser_click",
            json!({"selector": "#counter", "session_id": "click"}),
        );
        assert!(clicked.contains("Clicked element"), "{clicked}");
        let content = page_content(&mut h, "click");
        assert!(content.contains(expected), "missing {expected}: {content}");
    }
}

#[test]
fn e2e_js_timer_content_and_wait_for_element() {
    let (mut h, site) = js_harness();
    open_js_app(&mut h, &site, "wait");

    // Nothing is pending: the answer comes at once, not after the timeout
    let started = std::time::Instant::now();
    let absent = call_ok(
        &mut h,
        "browser_wait_for_element",
        json!({"selector": "#more", "timeout": 30000, "session_id": "wait"}),
    );
    assert!(absent.contains("\"found\": false"), "{absent}");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(20),
        "an idle page must not be waited on: {:?}",
        started.elapsed()
    );

    // The click starts a 700 ms timer that adds #more
    call_ok(
        &mut h,
        "browser_click",
        json!({"selector": "#load-more", "session_id": "wait"}),
    );
    let found = call_ok(
        &mut h,
        "browser_wait_for_element",
        json!({"selector": "#more", "timeout": 10000, "session_id": "wait"}),
    );
    assert!(found.contains("\"found\": true"), "{found}");
    let content = page_content(&mut h, "wait");
    assert!(content.contains("more content from a timer"), "{content}");
}

#[test]
fn e2e_js_timers_keep_running_between_tool_calls() {
    let (mut h, site) = js_harness();
    open_js_app(&mut h, &site, "pump");
    call_ok(
        &mut h,
        "browser_click",
        json!({"selector": "#load-more", "session_id": "pump"}),
    );
    // No tool call pumps the page here: the browser thread does
    std::thread::sleep(std::time::Duration::from_millis(2500));
    let content = page_content(&mut h, "pump");
    assert!(content.contains("more content from a timer"), "{content}");
}

#[test]
fn e2e_js_submit_handler_and_form_submitted_by_script() {
    let (mut h, site) = js_harness();

    // A submit button: the submit handler runs, then the browser sends the form
    open_js_app(&mut h, &site, "forms");
    call_ok(
        &mut h,
        "browser_fill",
        json!({"selector": "#name", "value": "agent", "session_id": "forms"}),
    );
    let clicked = call_ok(
        &mut h,
        "browser_click",
        json!({"selector": "#send", "session_id": "forms"}),
    );
    assert!(clicked.contains("Form submitted successfully"), "{clicked}");
    let echoed = page_content(&mut h, "forms");
    assert!(echoed.contains("POST"), "{echoed}");
    assert!(echoed.contains("name=agent"), "{echoed}");
    assert!(echoed.contains("token=set-by-submit-handler"), "{echoed}");

    // form.submit() from a click handler
    open_js_app(&mut h, &site, "forms");
    call_ok(
        &mut h,
        "browser_fill",
        json!({"selector": "#name", "value": "scripted", "session_id": "forms"}),
    );
    call_ok(
        &mut h,
        "browser_click",
        json!({"selector": "#submit-by-script", "session_id": "forms"}),
    );
    let echoed = page_content(&mut h, "forms");
    assert!(echoed.contains("name=scripted"), "{echoed}");
    assert!(echoed.contains("token=set-by-click-handler"), "{echoed}");
}

#[test]
fn e2e_js_script_navigation_is_followed() {
    let (mut h, site) = js_harness();
    open_js_app(&mut h, &site, "nav");
    let clicked = call_ok(
        &mut h,
        "browser_click",
        json!({"selector": "#go-index", "session_id": "nav"}),
    );
    assert!(clicked.contains("the page navigated to"), "{clicked}");
    let content = page_content(&mut h, "nav");
    assert!(content.contains("Thalora Fixture Site"), "{content}");
}

#[test]
fn e2e_js_eval_returns_json() {
    let (mut h, site) = js_harness();
    open_js_app(&mut h, &site, "eval");

    let out = eval(&mut h, "eval", "document.title");
    assert_eq!(out["result"], json!("Fixture JS App"), "{out}");

    let out = eval(
        &mut h,
        "eval",
        "({ sum: 1 + 2, list: [1, 'two', null], nested: { ok: true }, \
          buttons: document.querySelectorAll('button').length })",
    );
    assert_eq!(
        out["result"],
        json!({"sum": 3, "list": [1, "two", null], "nested": {"ok": true}, "buttons": 5}),
        "{out}"
    );

    // undefined is null; a DOM node is its markup
    assert_eq!(eval(&mut h, "eval", "undefined")["result"], Value::Null);
    let node = eval(&mut h, "eval", "document.getElementById('boot')");
    assert_eq!(
        node["result"],
        json!("<div id=\"boot\">booted inline</div>"),
        "{node}"
    );

    // State persists between evaluations, and the DOM changes are real
    eval(
        &mut h,
        "eval",
        "var fromEval = 41; document.getElementById('boot').textContent = 'changed by eval'",
    );
    assert_eq!(eval(&mut h, "eval", "fromEval + 1")["result"], json!(42));
    let content = page_content(&mut h, "eval");
    assert!(content.contains("changed by eval"), "{content}");

    // The other name of the tool
    let cdp = call_ok(
        &mut h,
        "cdp_runtime_evaluate",
        json!({"expression": "6 * 7", "session_id": "eval"}),
    );
    assert!(cdp.contains("\"result\": 42"), "{cdp}");

    // An uncaught exception is an error, with the message
    let thrown = call(
        &mut h,
        "browser_eval",
        json!({"expression": "throw new Error('eval boom')", "session_id": "eval"}),
    );
    assert!(thrown.is_error, "{}", text(&thrown));
    assert!(text(&thrown).contains("eval boom"), "{}", text(&thrown));
}

#[test]
fn e2e_js_runaway_script_is_terminated() {
    let server = FixtureServer::start();
    let mut env = HashMap::new();
    for (k, v) in [
        ("THALORA_PRESET", "brainclaw"),
        ("THALORA_ALLOW_LOOPBACK", "1"),
        ("THALORA_DISABLE_RATE_LIMIT", "1"),
        ("THALORA_JS_SCRIPT_TIMEOUT_MS", "500"),
    ] {
        env.insert(k.to_string(), v.to_string());
    }
    let mut h = create_harness_with_raw_env(env).expect("Failed to create harness");
    open_js_app(&mut h, &server, "loop");

    let stuck = call(
        &mut h,
        "browser_eval",
        json!({"expression": "while (true) {}", "session_id": "loop"}),
    );
    assert!(stuck.is_error, "{}", text(&stuck));
    assert!(text(&stuck).contains("terminated"), "{}", text(&stuck));

    // The page is still usable
    let out = eval(&mut h, "loop", "1 + 1");
    assert_eq!(out["result"], json!(2), "{out}");
}

#[test]
fn e2e_js_console_messages_are_captured() {
    let (mut h, site) = js_harness();
    open_js_app(&mut h, &site, "console");
    call_ok(
        &mut h,
        "browser_click",
        json!({"selector": "#counter", "session_id": "console"}),
    );
    eval(
        &mut h,
        "console",
        "console.error('from eval', { n: 1 }); setTimeout(function () { throw new Error('late boom'); }, 0)",
    );

    let out = call_ok(
        &mut h,
        "browser_console_messages",
        json!({"session_id": "console", "clear": true}),
    );
    let parsed = json_of(&out);
    assert_eq!(parsed["javascript"], json!("running"), "{out}");
    let messages = parsed["messages"].as_array().expect("messages");
    let has = |level: &str, needle: &str| {
        messages
            .iter()
            .any(|m| m["level"] == level && m["text"].as_str().is_some_and(|t| t.contains(needle)))
    };
    assert!(has("log", "js app inline script ran"), "{out}");
    assert!(has("warning", "counter clicked 1"), "{out}");
    assert!(has("error", "from eval"), "{out}");
    // An error nothing caught, in a timer
    assert!(has("error", "late boom"), "{out}");

    // Cleared by the read above
    let again = call_ok(
        &mut h,
        "browser_console_messages",
        json!({"session_id": "console"}),
    );
    assert!(again.contains("\"count\": 0"), "{again}");
}

#[test]
fn e2e_js_keyboard_hover_and_scroll_dispatch_events() {
    let (mut h, site) = js_harness();
    open_js_app(&mut h, &site, "input");

    for key in ["h", "i"] {
        let pressed = call_ok(
            &mut h,
            "browser_press_key",
            json!({"selector": "#keys", "key": key, "session_id": "input"}),
        );
        assert!(pressed.contains("typed into the field"), "{pressed}");
    }
    let state = eval(
        &mut h,
        "input",
        "({ value: document.getElementById('keys').value, \
          log: document.getElementById('keylog').textContent })",
    );
    assert_eq!(
        state["result"],
        json!({"value": "hi", "log": "[h][i]"}),
        "{state}"
    );

    call_ok(
        &mut h,
        "browser_hover",
        json!({"selector": "#hover-target", "session_id": "input"}),
    );
    call_ok(
        &mut h,
        "browser_scroll",
        json!({"delta_y": 200, "session_id": "input"}),
    );
    let content = page_content(&mut h, "input");
    assert!(content.contains("hovered"), "{content}");
    assert!(content.contains("scrolled"), "{content}");

    // What was typed shows in the snapshot (the live value, not markup)
    let snapshot = call_ok(&mut h, "browser_snapshot", json!({"session_id": "input"}));
    assert!(snapshot.contains("value=\"hi\""), "{snapshot}");
}

#[test]
fn e2e_js_disabled_pages_are_static() {
    let server = FixtureServer::start();
    let mut env = HashMap::new();
    for (k, v) in [
        ("THALORA_PRESET", "brainclaw"),
        ("THALORA_ALLOW_LOOPBACK", "1"),
        ("THALORA_DISABLE_RATE_LIMIT", "1"),
        ("THALORA_JS", "0"),
    ] {
        env.insert(k.to_string(), v.to_string());
    }
    let mut h = create_harness_with_raw_env(env).expect("Failed to create harness");

    // timers.html fills its elements from script; with JavaScript off the
    // page still loads, as static HTML, and the tool output says so
    let nav = call_ok(
        &mut h,
        "browser_navigate_to",
        json!({"url": server.url("/timers.html"), "session_id": "static", "wait_for_js": true}),
    );
    assert!(nav.contains("static: JavaScript is disabled"), "{nav}");
    let content = page_content(&mut h, "static");
    assert!(
        content.contains("pending") && !content.contains("timeout fired"),
        "page scripts must not run: {content}"
    );

    // Tools that need script fail with a clear error, never silently succeed
    for (tool, args) in [
        ("browser_eval", json!({"expression": "document.title"})),
        ("cdp_runtime_evaluate", json!({"expression": "1 + 1"})),
        ("browser_hover", json!({"selector": "h1"})),
        ("browser_scroll", json!({"delta_y": 200})),
        ("browser_click", json!({"selector": "h1"})),
    ] {
        let mut args = args;
        args["session_id"] = json!("static");
        let resp = call(&mut h, tool, args);
        let body = text(&resp);
        assert!(
            resp.is_error,
            "{tool} should fail without JavaScript: {body}"
        );
        assert!(
            body.contains("JavaScript execution is unavailable")
                && body.contains("JavaScript is disabled"),
            "{tool} error: {body}"
        );
    }

    // Default actions still work on a static page
    call_ok(
        &mut h,
        "browser_navigate_to",
        json!({"url": server.url("/index.html"), "session_id": "static"}),
    );
    call_ok(
        &mut h,
        "browser_click",
        json!({"selector": "#to-login", "session_id": "static"}),
    );
    assert!(page_content(&mut h, "static").contains("Sign in"));

    // The console is empty and says why
    let console = call_ok(
        &mut h,
        "browser_console_messages",
        json!({"session_id": "static"}),
    );
    assert!(console.contains("\"count\": 0"), "{console}");
    assert!(
        console.contains("static: JavaScript is disabled"),
        "{console}"
    );
}

// ── Snapshot refs (Phase 2) ─────────────────────────────────────────────────

/// Find the ref on the snapshot line that contains `needle`.
fn ref_for(snapshot: &str, needle: &str) -> String {
    let line = snapshot
        .lines()
        .find(|l| l.contains(needle) && l.contains("[ref="))
        .unwrap_or_else(|| panic!("no ref line containing {needle:?} in:\n{snapshot}"));
    line.rsplit("[ref=")
        .next()
        .unwrap()
        .trim_end_matches(']')
        .to_string()
}

#[test]
fn e2e_snapshot_refs_drive_a_login_flow() {
    let (mut h, site) = fixture_harness();
    call_ok(
        &mut h,
        "browser_navigate_to",
        json!({"url": site.url("/login.html"), "session_id": "refs"}),
    );
    let snapshot = call_ok(&mut h, "browser_snapshot", json!({"session_id": "refs"}));
    assert!(snapshot.contains("<untrusted_page_content"), "{snapshot}");
    assert!(snapshot.contains("heading \"Sign in\""), "{snapshot}");
    assert!(
        !snapshot.contains("fixture-csrf-token"),
        "hidden field leaked"
    );

    // Same page, same refs
    let again = call_ok(&mut h, "browser_snapshot", json!({"session_id": "refs"}));
    assert_eq!(snapshot, again);

    let username = ref_for(&snapshot, "textbox \"Username\"");
    let password = ref_for(&snapshot, "textbox \"Password\"");
    let sign_in = ref_for(&snapshot, "button \"Sign in\"");

    call_ok(
        &mut h,
        "browser_fill",
        json!({"ref": username, "value": "agent", "session_id": "refs"}),
    );
    call_ok(
        &mut h,
        "browser_fill",
        json!({"ref": password, "value": "pw123", "session_id": "refs"}),
    );
    call_ok(
        &mut h,
        "browser_click",
        json!({"ref": sign_in, "session_id": "refs"}),
    );

    let echoed = page_content(&mut h, "refs");
    assert!(echoed.contains("username=agent"), "{echoed}");
    assert!(echoed.contains("password=pw123"), "{echoed}");

    // The page changed, so the old ref is stale
    let stale = call(
        &mut h,
        "browser_click",
        json!({"ref": sign_in, "session_id": "refs"}),
    );
    assert!(stale.is_error);
    assert!(text(&stale).contains("stale_ref"), "{}", text(&stale));
}

#[test]
fn e2e_browser_wait_conditions() {
    let (mut h, site) = fixture_harness();
    call_ok(
        &mut h,
        "browser_navigate_to",
        json!({"url": site.url("/index.html"), "session_id": "wait"}),
    );
    for condition in [
        json!({"text": "Thalora Fixture Site"}),
        json!({"selector": "#to-login"}),
        json!({"url_contains": "index.html"}),
        json!({"network_idle": true}),
    ] {
        let mut args = condition.clone();
        args["session_id"] = json!("wait");
        args["timeout_ms"] = json!(2000);
        let out = call_ok(&mut h, "browser_wait", args);
        assert!(out.contains("\"met\": true"), "{condition}: {out}");
    }

    let missing = call_ok(
        &mut h,
        "browser_wait",
        json!({"session_id": "wait", "text": "never on this page", "timeout_ms": 300}),
    );
    assert!(missing.contains("\"met\": false"), "{missing}");

    let both = call(
        &mut h,
        "browser_wait",
        json!({"session_id": "wait", "text": "a", "selector": "b"}),
    );
    assert!(both.is_error, "two conditions should be rejected");
}

#[test]
fn e2e_check_and_enter_submit() {
    let (mut h, site) = fixture_harness();
    call_ok(
        &mut h,
        "browser_navigate_to",
        json!({"url": site.url("/login.html"), "session_id": "keys"}),
    );
    call_ok(
        &mut h,
        "browser_fill",
        json!({"selector": "#username", "value": "keyboard", "session_id": "keys"}),
    );
    call_ok(
        &mut h,
        "browser_check",
        json!({"selector": "#remember", "checked": false, "session_id": "keys"}),
    );
    let pressed = call_ok(
        &mut h,
        "browser_press_key",
        json!({"selector": "#username", "key": "Enter", "session_id": "keys"}),
    );
    assert!(
        pressed.contains("submitted"),
        "Enter should submit: {pressed}"
    );

    let echoed = page_content(&mut h, "keys");
    assert!(echoed.contains("username=keyboard"), "{echoed}");
    assert!(
        !echoed.contains("remember=yes"),
        "unchecked box was submitted: {echoed}"
    );
}

// ── Credential fill-by-reference ────────────────────────────────────────────

#[test]
fn e2e_fill_credential_never_reveals_the_secret() {
    let server = FixtureServer::start();
    let vault_path = std::env::temp_dir().join(format!(
        "thalora-e2e-vault-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut env = HashMap::new();
    env.insert(
        "THALORA_VAULT_PATH".to_string(),
        vault_path.to_string_lossy().into_owned(),
    );
    for (k, v) in [
        ("THALORA_PRESET", "brainclaw"),
        ("THALORA_ALLOW_LOOPBACK", "1"),
        ("THALORA_ENABLE_VAULT", "true"),
        (
            "THALORA_MASTER_PASSWORD",
            "test_master_password_min_32chars_secure",
        ),
    ] {
        env.insert(k.to_string(), v.to_string());
    }
    let mut h = create_harness_with_raw_env(env).expect("Failed to create harness");

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let service = format!("fixture-login-{unique}");
    let secret = format!("s3cret-{unique}");

    call_ok(
        &mut h,
        "vault_store_credential",
        json!({
            "service": service,
            "username": "agent",
            "password": secret,
            "origin": server.url("/"),
        }),
    );
    let looked_up = call_ok(
        &mut h,
        "vault_list_credentials",
        json!({"service": service}),
    );
    assert!(!looked_up.contains(&secret), "secret returned: {looked_up}");
    assert!(
        looked_up.contains("agent"),
        "username should be listed: {looked_up}"
    );

    call_ok(
        &mut h,
        "browser_navigate_to",
        json!({"url": server.url("/login.html"), "session_id": "cred"}),
    );
    let filled = call_ok(
        &mut h,
        "browser_fill_credential",
        json!({
            "service": service,
            "username_selector": "#username",
            "password_selector": "#password",
            "session_id": "cred"
        }),
    );
    assert!(!filled.contains(&secret), "secret returned: {filled}");

    call_ok(
        &mut h,
        "browser_click",
        json!({"selector": "#signin", "session_id": "cred"}),
    );
    let echoed = page_content(&mut h, "cred");
    assert!(
        echoed.contains(&format!("password={secret}")),
        "the server should receive the password: {echoed}"
    );

    // A credential for another origin is refused
    let other = format!("other-site-{unique}");
    call_ok(
        &mut h,
        "vault_store_credential",
        json!({
            "service": other,
            "username": "x",
            "password": "y",
            "origin": "https://example.com",
        }),
    );
    call_ok(
        &mut h,
        "browser_navigate_to",
        json!({"url": server.url("/login.html"), "session_id": "cred"}),
    );
    let refused = call(
        &mut h,
        "browser_fill_credential",
        json!({"service": other, "password_selector": "#password", "session_id": "cred"}),
    );
    assert!(refused.is_error);
    assert!(
        text(&refused).contains("Refusing to fill"),
        "{}",
        text(&refused)
    );
}

#[test]
fn e2e_screenshot_returns_a_png_image() {
    use base64::Engine;

    let (mut h, site) = fixture_harness();
    call_ok(
        &mut h,
        "browser_navigate_to",
        json!({"url": site.url("/index.html"), "session_id": "shot"}),
    );
    let resp = call(
        &mut h,
        "browser_screenshot",
        json!({"session_id": "shot", "width": 800, "height": 600}),
    );
    assert!(!resp.is_error, "{}", text(&resp));
    let image = resp
        .content
        .iter()
        .find(|c| c["type"] == "image")
        .expect("an image content block");
    assert_eq!(image["mimeType"], "image/png");
    let png = base64::engine::general_purpose::STANDARD
        .decode(image["data"].as_str().unwrap())
        .expect("valid base64");
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
}

// ── SSRF hardening ──────────────────────────────────────────────────────────

#[test]
fn e2e_redirects_into_private_networks_are_blocked() {
    let (mut h, site) = fixture_harness();
    for target in [
        "http%3A%2F%2F169.254.169.254%2Flatest%2Fmeta-data",
        "http%3A%2F%2F10.0.0.1%2F",
        "http%3A%2F%2F%5Bfd00%3A%3A1%5D%2F",
    ] {
        let resp = call(
            &mut h,
            "browser_navigate_to",
            json!({"url": site.url(&format!("/redirect?to={target}")), "session_id": "ssrf"}),
        );
        assert!(
            resp.is_error,
            "redirect to {target} should be blocked: {}",
            text(&resp)
        );
    }

    // A redirect to another public-or-allowed page still works
    let ok = call_ok(
        &mut h,
        "browser_navigate_to",
        json!({
            "url": site.url("/redirect?to=%2Findex.html"),
            "session_id": "ssrf"
        }),
    );
    assert!(ok.contains("Thalora Fixture Site"), "{ok}");
}
