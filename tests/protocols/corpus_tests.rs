// Real-world pattern corpus (tests/corpus): navigate each page with
// JavaScript over MCP, check its expectations, and score it.
//
// Bundled pages (tests/corpus/pages) must all pass. Pages saved under
// tests/corpus/local (git-ignored) are scored but never fail the test.
// THALORA_CORPUS_WRITE=1 rewrites tests/corpus/scoreboard.json.

use super::fixture_server::{FixtureServer, corpus_root};
use super::mcp_harness::{McpTestHarness, create_harness_with_raw_env};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

fn corpus_harness() -> (McpTestHarness, FixtureServer) {
    let server = FixtureServer::start();
    let mut env = HashMap::new();
    env.insert("THALORA_PRESET".to_string(), "brainclaw".to_string());
    env.insert("THALORA_ALLOW_LOOPBACK".to_string(), "1".to_string());
    env.insert("THALORA_DISABLE_RATE_LIMIT".to_string(), "1".to_string());
    let harness = create_harness_with_raw_env(env).expect("Failed to create harness");
    (harness, server)
}

fn text_of(content: &[Value]) -> String {
    content
        .iter()
        .filter_map(|c| c["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// `(url path, expectations)` for every page with an `.expect.json` in `dir`.
fn pages_in(dir: &str) -> Vec<(String, Value)> {
    let root = corpus_root().join(dir);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut pages: Vec<(String, Value)> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .filter_map(|html| {
            let expect = html.with_extension("expect.json");
            let expect: Value =
                serde_json::from_str(&std::fs::read_to_string(expect).ok()?).ok()?;
            let name = html.file_name()?.to_str()?.to_string();
            Some((format!("/corpus/{dir}/{name}"), expect))
        })
        .collect();
    pages.sort_by(|a, b| a.0.cmp(&b.0));
    pages
}

fn score_page(h: &mut McpTestHarness, site: &FixtureServer, path: &str, expect: &Value) -> Value {
    let session = format!(
        "corpus-{}",
        Path::new(path).file_stem().unwrap().to_string_lossy()
    );
    let start = Instant::now();
    let nav = h.call_tool(
        "browser_navigate_to",
        json!({"url": site.url(path), "session_id": session, "wait_for_js": true}),
    );
    let nav_ms = start.elapsed().as_millis() as u64;
    let nav_error = match &nav {
        Err(e) => Some(e.to_string()),
        Ok(r) if r.is_error => Some(text_of(&r.content)),
        Ok(_) => None,
    };
    if let Some(error) = nav_error {
        return json!({"page": path, "ok": false, "nav_ms": nav_ms, "error": error});
    }

    let content = h
        .call_tool("browser_get_page_content", json!({"session_id": session}))
        .map(|r| text_of(&r.content))
        .unwrap_or_default();
    let snapshot = h
        .call_tool("browser_snapshot", json!({"session_id": session}))
        .map(|r| text_of(&r.content))
        .unwrap_or_default();
    let _ = h.call_tool(
        "browser_session_management",
        json!({"action": "close", "session_id": session}),
    );

    let strings = |key: &str| -> Vec<String> {
        expect[key]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    };
    let missing: Vec<String> = strings("text")
        .into_iter()
        .filter(|t| !content.contains(t.as_str()))
        .collect();
    let unexpected: Vec<String> = strings("absent")
        .into_iter()
        .filter(|t| content.contains(t.as_str()))
        .collect();
    let refs = snapshot.matches("[ref=").count() as u64;
    let min_refs = expect["min_refs"].as_u64().unwrap_or(0);
    let ok = missing.is_empty() && unexpected.is_empty() && refs >= min_refs;
    json!({
        "page": path,
        "ok": ok,
        "missing_text": missing,
        "unexpected_text": unexpected,
        "refs": refs,
        "min_refs": min_refs,
        "nav_ms": nav_ms,
    })
}

#[test]
fn corpus_pages_meet_their_expectations() {
    let (mut h, site) = corpus_harness();
    let bundled = pages_in("pages");
    assert!(!bundled.is_empty(), "no corpus pages found");
    let local = pages_in("local");

    let mut results = Vec::new();
    for (path, expect) in bundled.iter().chain(local.iter()) {
        let result = score_page(&mut h, &site, path, expect);
        eprintln!("corpus: {result}");
        results.push(result);
    }

    let passed = results.iter().filter(|r| r["ok"] == true).count();
    let scoreboard = json!({
        "passed": passed,
        "total": results.len(),
        "pages": results,
    });
    if std::env::var("THALORA_CORPUS_WRITE").is_ok_and(|v| v == "1") {
        // Timings vary run to run; keep them out of the committed file
        let mut stable = scoreboard.clone();
        for page in stable["pages"].as_array_mut().unwrap() {
            page.as_object_mut().unwrap().remove("nav_ms");
        }
        std::fs::write(
            corpus_root().join("scoreboard.json"),
            serde_json::to_string_pretty(&stable).unwrap() + "\n",
        )
        .expect("write scoreboard");
    }

    let failed_bundled: Vec<&Value> = results
        .iter()
        .take(bundled.len())
        .filter(|r| r["ok"] != true)
        .collect();
    assert!(
        failed_bundled.is_empty(),
        "corpus pages failed:\n{}",
        serde_json::to_string_pretty(&failed_bundled).unwrap()
    );
}
