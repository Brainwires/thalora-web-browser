use thalora::RustRenderer;

#[tokio::test]
async fn test_polyfills_vs_original_missing_features() {
    let mut renderer = RustRenderer::new();

    // Test the exact features that were previously missing
    let missing_features = vec![
        "fetch", "URL", "URLSearchParams", "setTimeout", "setInterval",
        "clearTimeout", "clearInterval", "localStorage", "sessionStorage",
        "atob", "btoa", "crypto", "TextEncoder", "TextDecoder",
        "AbortController", "Blob", "File", "FormData", "Headers", "Request", "Response"
    ];

    for feature in missing_features {
        let test_code = format!("typeof {} !== 'undefined'", feature);
        let result = renderer.execute_javascript_safely(&test_code).await;

        assert!(result.is_ok(), "Feature {} test should execute without errors", feature);
        let output = result.unwrap();
        assert_eq!(renderer.js_value_to_string(output), "true", "Feature {} should now be available", feature);
    }

    println!("✅ All previously missing features are now available!");
}
