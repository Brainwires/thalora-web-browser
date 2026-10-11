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

#[test]
fn e2e_eval_runs_in_default_session() {
    let (mut h, site) = fixture_harness();
    call_ok(
        &mut h,
        "browser_navigate",
        json!({"url": site.url("/index.html")}),
    );
    let result = call_ok(
        &mut h,
        "browser_eval",
        json!({"expression": "document.title"}),
    );
    assert!(
        result.contains("Thalora Fixture Index"),
        "eval should see the navigated page: {result}"
    );
}

// ── Event loop (Phase 1) ────────────────────────────────────────────────────

fn navigate_with_js(h: &mut McpTestHarness, url: String, session_id: &str) {
    call_ok(
        h,
        "browser_navigate_to",
        json!({"url": url, "session_id": session_id, "wait_for_js": true}),
    );
}

fn eval_in(h: &mut McpTestHarness, session_id: &str, expression: &str) -> String {
    call_ok(
        h,
        "cdp_runtime_evaluate",
        json!({"expression": expression, "session_id": session_id}),
    )
}

#[test]
fn e2e_timers_microtasks_and_animation_frames_run_during_navigation() {
    let (mut h, site) = fixture_harness();
    navigate_with_js(&mut h, site.url("/timers.html"), "timers");
    let log = eval_in(&mut h, "timers", "fixtureLog.join(',')");
    for entry in ["microtask", "raf", "timeout"] {
        assert!(log.contains(entry), "{entry} did not run: {log}");
    }
    // ...and their DOM writes persist
    let text = eval_in(
        &mut h,
        "timers",
        "document.getElementById('timeout').textContent",
    );
    assert!(text.contains("timeout fired"), "timeout text: {text}");
}

#[test]
fn e2e_fetch_settles_during_navigation() {
    let (mut h, site) = fixture_harness();
    navigate_with_js(&mut h, site.url("/spa_fetch.html"), "fetch");
    let result = eval_in(&mut h, "fetch", "fetchResult");
    assert!(
        result.contains("fixture data loaded"),
        "fetch did not settle: {result}"
    );
}

#[test]
fn e2e_bundled_code_runs() {
    let (mut h, site) = fixture_harness();
    navigate_with_js(&mut h, site.url("/webpack_bundle.html"), "bundle");
    let result = eval_in(&mut h, "bundle", "String(window.bundleResult)");
    assert!(
        result.contains("bundle rendered (production)"),
        "bundle did not run: {result}"
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

#[test]
fn e2e_console_messages_are_captured() {
    let (mut h, site) = fixture_harness();
    navigate_with_js(&mut h, site.url("/timers.html"), "console");
    let messages = call_ok(
        &mut h,
        "browser_console_messages",
        json!({"session_id": "console"}),
    );
    assert!(
        messages.contains("timers fixture loaded"),
        "console.log not captured: {messages}"
    );
}

// ── Credential fill-by-reference ────────────────────────────────────────────

#[test]
fn e2e_fill_credential_never_reveals_the_secret() {
    let server = FixtureServer::start();
    let mut env = HashMap::new();
    for (k, v) in [
        ("THALORA_PRESET", "brainclaw"),
        ("THALORA_ALLOW_LOOPBACK", "1"),
        ("THALORA_ENABLE_AI_MEMORY", "true"),
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
        "ai_memory_store_credentials",
        json!({
            "service": service,
            "username": "agent",
            "password": secret,
            "origin": server.url("/"),
        }),
    );
    let looked_up = call_ok(
        &mut h,
        "ai_memory_get_credentials",
        json!({"service": service}),
    );
    assert!(!looked_up.contains(&secret), "secret returned: {looked_up}");

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
        "ai_memory_store_credentials",
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

#[test]
fn e2e_page_fetch_to_metadata_endpoint_is_blocked() {
    let (mut h, site) = fixture_harness();
    navigate_with_js(&mut h, site.url("/index.html"), "ssrf-fetch");
    let result = eval_in(
        &mut h,
        "ssrf-fetch",
        "(function () { try { fetch('http://169.254.169.254/latest/meta-data').then(function () { window.ssrfResult = 'fetched'; }, function (e) { window.ssrfResult = 'blocked: ' + e.message; }); return 'started'; } catch (e) { return 'threw: ' + e.message; } })()",
    );
    assert!(
        result.contains("started") || result.contains("threw"),
        "{result}"
    );
    call_ok(
        &mut h,
        "browser_wait",
        json!({"session_id": "ssrf-fetch", "network_idle": true, "timeout_ms": 2000}),
    );
    let outcome = eval_in(&mut h, "ssrf-fetch", "String(window.ssrfResult)");
    assert!(
        outcome.contains("blocked") || result.contains("threw"),
        "metadata fetch was not blocked: {outcome}"
    );
}

// ── Persistent DOM (Phase 3 R) ──────────────────────────────────────────────

#[test]
fn e2e_dom_mutations_reach_page_content_and_snapshot() {
    let (mut h, site) = fixture_harness();
    navigate_with_js(&mut h, site.url("/dom_mutation.html"), "dom");

    let content = page_content(&mut h, "dom");
    for expected in [
        "static item",
        "added by script",
        "Inserted button",
        "from inserted script",
    ] {
        assert!(
            content.contains(expected),
            "{expected:?} missing: {content}"
        );
    }

    let snapshot = call_ok(&mut h, "browser_snapshot", json!({"session_id": "dom"}));
    assert!(
        snapshot.contains("Inserted button") && snapshot.contains("[ref="),
        "snapshot: {snapshot}"
    );

    // Node identity holds across lookups
    let same = eval_in(
        &mut h,
        "dom",
        "document.getElementById('added') === document.querySelector('#items > #added')",
    );
    assert!(same.contains("true"), "identity: {same}");
    let count = eval_in(
        &mut h,
        "dom",
        "document.querySelectorAll('#items li').length",
    );
    assert!(count.contains('3'), "items: {count}");
}
