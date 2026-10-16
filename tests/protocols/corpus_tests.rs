// Real-world pattern corpus (tests/corpus): navigate each page with
// JavaScript over MCP, check its expectations, optionally drive it through
// interactive `steps`, and score it.
//
// Bundled pages (tests/corpus/pages) must all pass, except pages whose
// expect file sets `known_failing` (scored and reported, never fail the
// test). Pages saved under tests/corpus/local (git-ignored) are scored but
// never fail the test. THALORA_CORPUS_WRITE=1 rewrites
// tests/corpus/scoreboard.json. See tests/corpus/README.md for the schema.

use super::fixture_server::{FixtureServer, corpus_root};
use super::mcp_harness::{McpTestHarness, create_harness_with_raw_env};
use serde_json::{Map, Value, json};
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

/// String array at `value[key]` (missing or malformed -> empty).
fn strings(value: &Value, key: &str) -> Vec<String> {
    value[key]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// `(missing, unexpected)` for `expect`'s `text`/`absent` lists against `haystack`.
fn check_text(expect: &Value, haystack: &str) -> (Vec<String>, Vec<String>) {
    let missing = strings(expect, "text")
        .into_iter()
        .filter(|t| !haystack.contains(t.as_str()))
        .collect();
    let unexpected = strings(expect, "absent")
        .into_iter()
        .filter(|t| haystack.contains(t.as_str()))
        .collect();
    (missing, unexpected)
}

/// `known_failing` from an expect file: `true` or a reason string.
fn known_failing(expect: &Value) -> Option<String> {
    match &expect["known_failing"] {
        Value::Bool(true) => Some("known failing".to_string()),
        Value::String(reason) if !reason.is_empty() => Some(reason.clone()),
        _ => None,
    }
}

fn page_content(h: &mut McpTestHarness, session: &str) -> String {
    h.call_tool("browser_get_page_content", json!({"session_id": session}))
        .map(|r| text_of(&r.content))
        .unwrap_or_default()
}

fn page_snapshot(h: &mut McpTestHarness, session: &str) -> String {
    h.call_tool("browser_snapshot", json!({"session_id": session}))
        .map(|r| text_of(&r.content))
        .unwrap_or_default()
}

/// One element line of a `browser_snapshot` outline.
struct SnapshotEntry {
    role: String,
    name: String,
    reference: String,
}

/// Parse `- role "name" ... [ref=eN]` lines of a snapshot.
fn snapshot_entries(snapshot: &str) -> Vec<SnapshotEntry> {
    snapshot
        .lines()
        .filter_map(|line| {
            let line = line.trim_start().strip_prefix("- ")?;
            let ref_start = line.rfind("[ref=")?;
            let reference = line[ref_start + 5..].split(']').next()?.to_string();
            let role = line.split_whitespace().next()?.to_string();
            let name = line[role.len()..]
                .trim_start()
                .strip_prefix('"')
                .and_then(|rest| rest.split('"').next())
                .unwrap_or("")
                .to_string();
            Some(SnapshotEntry {
                role,
                name,
                reference,
            })
        })
        .collect()
}

/// Resolve a step `target` to tool arguments (`ref` or `selector`).
///
/// `{"selector": "css"}` passes through. Otherwise a fresh snapshot is taken
/// and the first element whose accessible name equals `text`/`label`/`name`
/// (case-insensitive; falling back to a substring match) and, if given,
/// whose role equals `role` supplies the ref.
fn resolve_target(
    h: &mut McpTestHarness,
    session: &str,
    target: &Value,
) -> Result<Map<String, Value>, String> {
    let mut args = Map::new();
    if let Some(selector) = target["selector"].as_str() {
        args.insert("selector".into(), json!(selector));
        return Ok(args);
    }
    let wanted = ["text", "label", "name"]
        .iter()
        .find_map(|k| target[*k].as_str())
        .map(|s| s.trim().to_lowercase());
    let role = target["role"].as_str();
    if wanted.is_none() && role.is_none() {
        return Err(format!(
            "target has no selector/text/label/name/role: {target}"
        ));
    }

    let snapshot = page_snapshot(h, session);
    let entries = snapshot_entries(&snapshot);
    let role_ok = |e: &SnapshotEntry| role.is_none_or(|r| e.role == r);
    let name = |e: &SnapshotEntry| e.name.trim().to_lowercase();
    let found = match &wanted {
        None => entries.iter().find(|e| role_ok(e)),
        Some(w) => entries
            .iter()
            .find(|e| role_ok(e) && name(e) == *w)
            .or_else(|| {
                entries
                    .iter()
                    .find(|e| role_ok(e) && name(e).contains(w.as_str()))
            }),
    };
    match found {
        Some(e) => {
            args.insert("ref".into(), json!(e.reference));
            Ok(args)
        }
        None => Err(format!(
            "no snapshot element matches {target} ({} refs in snapshot)",
            entries.len()
        )),
    }
}

/// Run one `{"action": ...}` step. Returns the step record.
fn run_action(h: &mut McpTestHarness, session: &str, index: usize, step: &Value) -> Value {
    let action = step["action"].as_str().unwrap_or("");
    let mut record = json!({"step": index, "action": action});
    let value = step["value"].as_str().unwrap_or("");

    let call: Result<(&str, Map<String, Value>), String> = (|| {
        let mut args = match action {
            "wait" => Map::new(),
            "press" if step["target"].is_null() => Map::new(),
            _ => resolve_target(h, session, &step["target"])?,
        };
        args.insert("session_id".into(), json!(session));
        let tool = match action {
            "click" => "browser_click",
            "type" => {
                args.insert("text".into(), json!(value));
                args.insert("clear_first".into(), json!(true));
                "browser_type_text"
            }
            "fill" => {
                args.insert("value".into(), json!(value));
                "browser_fill"
            }
            "select" => {
                args.insert("value".into(), json!(value));
                "browser_select_option"
            }
            "check" => {
                let checked = step["checked"].as_bool().unwrap_or(true);
                args.insert("checked".into(), json!(checked));
                "browser_check"
            }
            "press" => {
                args.insert("key".into(), json!(step["key"].as_str().unwrap_or(value)));
                "browser_press_key"
            }
            "wait" => {
                if let Some(text) = step["text"].as_str() {
                    args.insert("text".into(), json!(text));
                } else if let Some(selector) = step["selector"].as_str() {
                    args.insert("selector".into(), json!(selector));
                } else {
                    // Plain delay: wait on a selector that never matches, which
                    // runs page timers and network for `ms`.
                    args.insert("selector".into(), json!("#corpus-wait-never-matches"));
                }
                let ms = step["ms"].as_u64().unwrap_or(1000);
                args.insert("timeout_ms".into(), json!(ms));
                "browser_wait"
            }
            other => return Err(format!("unknown action {other:?}")),
        };
        Ok((tool, args))
    })();

    let (tool, args) = match call {
        Ok(call) => call,
        Err(error) => {
            record["ok"] = json!(false);
            record["error"] = json!(error);
            return record;
        }
    };
    record["tool"] = json!(tool);
    if let Some(r) = args.get("ref").or_else(|| args.get("selector")) {
        record["target"] = r.clone();
    }
    match h.call_tool(tool, Value::Object(args)) {
        Err(e) => {
            record["ok"] = json!(false);
            record["error"] = json!(e.to_string());
        }
        Ok(r) if r.is_error => {
            record["ok"] = json!(false);
            record["error"] = json!(text_of(&r.content));
        }
        Ok(_) => record["ok"] = json!(true),
    }
    record
}

/// Run one `{"expect": ...}` step against both page content and snapshot.
fn run_expect(h: &mut McpTestHarness, session: &str, index: usize, expect: &Value) -> Value {
    let content = page_content(h, session);
    let snapshot = page_snapshot(h, session);
    let (content_missing, content_unexpected) = check_text(expect, &content);
    let (snapshot_missing, snapshot_unexpected) = check_text(expect, &snapshot);
    let mut failed_in = Vec::new();
    if !content_missing.is_empty() || !content_unexpected.is_empty() {
        failed_in.push("page_content");
    }
    if !snapshot_missing.is_empty() || !snapshot_unexpected.is_empty() {
        failed_in.push("snapshot");
    }
    json!({
        "step": index,
        "expect": true,
        "ok": failed_in.is_empty(),
        "failed_in": failed_in,
        "page_content": {"missing_text": content_missing, "unexpected_text": content_unexpected},
        "snapshot": {"missing_text": snapshot_missing, "unexpected_text": snapshot_unexpected},
    })
}

fn run_steps(h: &mut McpTestHarness, session: &str, steps: &[Value]) -> Vec<Value> {
    steps
        .iter()
        .enumerate()
        .map(|(i, step)| {
            if step.get("expect").is_some() {
                run_expect(h, session, i, &step["expect"])
            } else {
                run_action(h, session, i, step)
            }
        })
        .collect()
}

fn score_page(h: &mut McpTestHarness, site: &FixtureServer, path: &str, expect: &Value) -> Value {
    let session = format!(
        "corpus-{}",
        Path::new(path).file_stem().unwrap().to_string_lossy()
    );
    let known = known_failing(expect);
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
        let _ = h.call_tool(
            "browser_session_management",
            json!({"action": "close", "session_id": session}),
        );
        return json!({
            "page": path, "ok": false, "nav_ms": nav_ms, "error": error, "known_failing": known,
        });
    }

    let content = page_content(h, &session);
    let snapshot = page_snapshot(h, &session);
    let (missing, unexpected) = check_text(expect, &content);
    let refs = snapshot.matches("[ref=").count() as u64;
    let min_refs = expect["min_refs"].as_u64().unwrap_or(0);
    let load_ok = missing.is_empty() && unexpected.is_empty() && refs >= min_refs;

    let steps = expect["steps"].as_array().cloned().unwrap_or_default();
    let step_results = run_steps(h, &session, &steps);
    let _ = h.call_tool(
        "browser_session_management",
        json!({"action": "close", "session_id": session}),
    );
    let steps_passed = step_results.iter().filter(|s| s["ok"] == true).count();
    let first_failed_step = step_results
        .iter()
        .find(|s| s["ok"] != true)
        .and_then(|s| s["step"].as_u64());

    let mut result = json!({
        "page": path,
        "ok": load_ok && first_failed_step.is_none(),
        "missing_text": missing,
        "unexpected_text": unexpected,
        "refs": refs,
        "min_refs": min_refs,
        "nav_ms": nav_ms,
    });
    if !steps.is_empty() {
        result["steps_passed"] = json!(steps_passed);
        result["steps_total"] = json!(step_results.len());
        result["first_failed_step"] = json!(first_failed_step);
        result["steps"] = json!(step_results);
    }
    if let Some(reason) = known {
        result["known_failing"] = json!(reason);
    }
    result
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
    let known_failing_count = results
        .iter()
        .filter(|r| r["known_failing"].is_string())
        .count();
    let scoreboard = json!({
        "passed": passed,
        "total": results.len(),
        "known_failing": known_failing_count,
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

    eprintln!(
        "corpus: {passed}/{} pages passed ({known_failing_count} marked known_failing)",
        results.len()
    );
    let bundled_results = &results[..bundled.len()];
    for r in bundled_results
        .iter()
        .filter(|r| r["known_failing"].is_string())
    {
        if r["ok"] == true {
            eprintln!(
                "corpus: NOTICE {} now passes; remove \"known_failing\" from its expect file",
                r["page"].as_str().unwrap_or("?")
            );
        } else {
            eprintln!(
                "corpus: known failing {} (first failed step: {}): {}",
                r["page"].as_str().unwrap_or("?"),
                r["first_failed_step"],
                r["known_failing"].as_str().unwrap_or("")
            );
        }
    }

    let failed_bundled: Vec<&Value> = bundled_results
        .iter()
        .filter(|r| r["ok"] != true && !r["known_failing"].is_string())
        .collect();
    assert!(
        failed_bundled.is_empty(),
        "corpus pages failed:\n{}",
        serde_json::to_string_pretty(&failed_bundled).unwrap()
    );
}
