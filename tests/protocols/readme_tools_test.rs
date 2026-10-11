// Keep README's MCP tool tables honest: every tool the server lists (in full
// mode and in the brainclaw preset) must be documented, and every tool name
// documented between the `tools:start` / `tools:end` markers must exist.

use super::mcp_harness::{create_harness_with_env, create_harness_with_raw_env};
use std::collections::{BTreeSet, HashMap};

fn readme_tool_names() -> BTreeSet<String> {
    let readme = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))
        .expect("README.md readable");
    let start = readme
        .find("<!-- tools:start -->")
        .expect("tools:start marker");
    let end = readme.find("<!-- tools:end -->").expect("tools:end marker");
    let section = &readme[start..end];

    // Tool names appear as `code` spans in the first column of table rows.
    let mut names = BTreeSet::new();
    for line in section.lines().filter(|l| l.starts_with("| `")) {
        let first_cell = line.split('|').nth(1).unwrap_or_default();
        for part in first_cell.split('`').skip(1).step_by(2) {
            names.insert(part.to_string());
        }
    }
    names
}

fn listed_tool_names(env: HashMap<String, String>, raw: bool) -> BTreeSet<String> {
    let mut harness = if raw {
        create_harness_with_raw_env(env)
    } else {
        create_harness_with_env(env)
    }
    .expect("Failed to create harness");
    harness
        .list_tools()
        .expect("tools/list should succeed")
        .iter()
        .filter_map(|t| t["name"].as_str().map(String::from))
        .collect()
}

#[test]
fn readme_documents_every_listed_tool() {
    let documented = readme_tool_names();

    // Full mode without the opt-in experimental/advanced tools
    let mut full = HashMap::new();
    full.insert(
        "THALORA_ENABLE_CDP_EXPERIMENTAL".to_string(),
        "false".to_string(),
    );
    let mut listed = listed_tool_names(full, false);
    let mut brainclaw = HashMap::new();
    brainclaw.insert("THALORA_PRESET".to_string(), "brainclaw".to_string());
    listed.extend(listed_tool_names(brainclaw, true));

    let undocumented: Vec<_> = listed.difference(&documented).collect();
    assert!(
        undocumented.is_empty(),
        "Tools listed by the server but missing from README: {undocumented:?}"
    );

    let unknown: Vec<_> = documented.difference(&listed).collect();
    assert!(
        unknown.is_empty(),
        "Tools documented in README that the server does not list: {unknown:?}"
    );
}
