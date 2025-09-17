use thalora::RustRenderer;

#[tokio::test]
async fn test_modern_javascript_compatibility() {
    let mut renderer = RustRenderer::new();

    // Test modern JavaScript patterns that require comprehensive polyfills
    let result = renderer.execute_javascript_safely(r#"
        // Test modern fetch with async/await pattern
        async function testFetch() {
            try {
                const response = await fetch('https://api.example.com/data');
                return response.status === 200;
            } catch (error) {
                return false;
            }
        }

        // Test URL manipulation
        const url = new URL('https://example.com/api/v1/users?limit=10&offset=0');
        url.searchParams.set('limit', '20');
        const urlTest = url.searchParams.get('limit') === '20';

        // Test crypto for secure random generation
        const randomArray = new Uint8Array(16);
        crypto.getRandomValues(randomArray);
        const hasRandomData = randomArray.some(byte => byte > 0);

        // Test storage APIs
        localStorage.setItem('user_preferences', JSON.stringify({theme: 'dark', lang: 'en'}));
        const storedPrefs = JSON.parse(localStorage.getItem('user_preferences'));
        const storageTest = storedPrefs.theme === 'dark';

        // Test FormData for modern forms
        const formData = new FormData();
        formData.append('file', new Blob(['test content'], {type: 'text/plain'}), 'test.txt');
        formData.append('metadata', JSON.stringify({version: '1.0'}));
        const formTest = formData.has('file') && formData.has('metadata');

        JSON.stringify({
            fetchAvailable: typeof fetch === 'function',
            urlTest,
            hasRandomData,
            storageTest,
            formTest
        });
    "#).await;

    assert!(result.is_ok(), "Modern JavaScript compatibility test should execute without errors");
    let output = result.unwrap();

    let output_string = renderer.js_value_to_string(output);
    let json_result: serde_json::Value = serde_json::from_str(&output_string.replace("'", "\"")).unwrap();

    assert_eq!(json_result["fetchAvailable"], true, "fetch should be available for async operations");
    assert_eq!(json_result["urlTest"], true, "URL manipulation should work");
    assert_eq!(json_result["hasRandomData"], true, "crypto random generation should work");
    assert_eq!(json_result["storageTest"], true, "localStorage should work with JSON");
    assert_eq!(json_result["formTest"], true, "FormData should work with Blob");

    println!("✅ Modern JavaScript patterns work correctly with comprehensive polyfills!");
}
