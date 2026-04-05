// ES Module integration tests
// Tests Module::parse, evaluate_module, and HttpModuleLoader

use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_inline_module_basic() {
    let browser = HeadlessWebBrowser::new();
    let result = browser
        .lock()
        .unwrap()
        .execute_module(
            r#"
            const x = 42;
            globalThis.__moduleTestResult = x;
            "#,
            "https://example.com/test.mjs",
        )
        .await;
    assert!(
        result.is_ok(),
        "Module should parse and evaluate: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_module_export_import_syntax() {
    let browser = HeadlessWebBrowser::new();
    // Module with export syntax should parse (even if no importer)
    let result = browser
        .lock()
        .unwrap()
        .execute_module(
            r#"
            export const value = 123;
            export function add(a, b) { return a + b; }
            "#,
            "https://example.com/lib.mjs",
        )
        .await;
    assert!(
        result.is_ok(),
        "Module with exports should parse: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_module_strict_mode() {
    let browser = HeadlessWebBrowser::new();
    // Modules are always strict mode — `arguments` in top level should work,
    // but undeclared variable assignment should fail
    let result = browser
        .lock()
        .unwrap()
        .execute_module(
            r#"
            // This is valid in strict mode
            const arr = [1, 2, 3];
            globalThis.__moduleStrictTest = arr.length;
            "#,
            "https://example.com/strict.mjs",
        )
        .await;
    assert!(
        result.is_ok(),
        "Strict mode module should work: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_module_base_url_update() {
    let browser = HeadlessWebBrowser::new();
    // Set base URL and verify it updates the module loader
    browser
        .lock()
        .unwrap()
        .set_module_base_url("https://myapp.com/app/");

    // Module should be parseable (we're testing the URL update doesn't crash)
    let result = browser
        .lock()
        .unwrap()
        .execute_module(
            "globalThis.__baseUrlTest = true;",
            "https://myapp.com/app/main.mjs",
        )
        .await;
    assert!(
        result.is_ok(),
        "Module after base URL update should work: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_module_url_resolution_unit() {
    // Test the URL resolution logic directly via the public module_loader
    use thalora::engine::browser::module_loader::HttpModuleLoader;

    let loader = HttpModuleLoader::new("https://example.com/app/index.html");

    // Absolute URL
    let result = loader.resolve_url_pub("https://cdn.example.com/lib.js", None);
    assert_eq!(result.unwrap(), "https://cdn.example.com/lib.js");

    // Relative
    let result = loader.resolve_url_pub("./utils.js", None);
    assert_eq!(result.unwrap(), "https://example.com/app/utils.js");

    // Root-relative
    let result = loader.resolve_url_pub("/static/mod.js", None);
    assert_eq!(result.unwrap(), "https://example.com/static/mod.js");

    // Bare specifier
    let result = loader.resolve_url_pub("lodash-es", None);
    assert_eq!(result.unwrap(), "https://esm.sh/lodash-es");
}
