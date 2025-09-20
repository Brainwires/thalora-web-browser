use thalora::RustRenderer;

#[tokio::test]
async fn test_comprehensive_polyfills_integration() {
    let mut renderer = RustRenderer::new();

    // Test that all missing Web APIs are now available
    let result = renderer.execute_javascript_safely(r#"
        // Test fetch API
        const hasFetch = typeof fetch === 'function';

        // Test URL API
        const hasURL = typeof URL === 'function';
        const url = new URL('https://example.com/path?query=value#hash');
        const correctURL = url.hostname === 'example.com';

        // Test URLSearchParams
        const hasURLSearchParams = typeof URLSearchParams === 'function';
        const params = new URLSearchParams('key1=value1&key2=value2');
        const correctParams = params.get('key1') === 'value1';

        // Test timers
        const hasSetTimeout = typeof setTimeout === 'function';
        const hasSetInterval = typeof setInterval === 'function';
        const hasClearTimeout = typeof clearTimeout === 'function';
        const hasClearInterval = typeof clearInterval === 'function';

        // Test storage
        const hasLocalStorage = typeof localStorage === 'object';
        const hasSessionStorage = typeof sessionStorage === 'object';
        localStorage.setItem('test', 'value');
        const storageWorks = localStorage.getItem('test') === 'value';

        // Test base64
        const hasAtob = typeof atob === 'function';
        const hasBtoa = typeof btoa === 'function';
        const base64Works = atob(btoa('hello')) === 'hello';

        // Test crypto
        const hasCrypto = typeof crypto === 'object';
        const hasRandomUUID = typeof crypto.randomUUID === 'function';
        const uuid = crypto.randomUUID();
        const validUUID = uuid.includes('-') && uuid.length === 36;

        // Test TextEncoder/TextDecoder
        const hasTextEncoder = typeof TextEncoder === 'function';
        const hasTextDecoder = typeof TextDecoder === 'function';
        const encoder = new TextEncoder();
        const encoded = encoder.encode('hello');
        const encoderWorks = encoded.length === 5;

        // Test Blob
        const hasBlob = typeof Blob === 'function';
        const blob = new Blob(['hello'], { type: 'text/plain' });
        const blobWorks = blob.size === 5 && blob.type === 'text/plain';

        // Test FormData
        const hasFormData = typeof FormData === 'function';
        const formData = new FormData();
        formData.set('key', 'value');
        const formDataWorks = formData.get('key') === 'value';

        // Test Headers
        const hasHeaders = typeof Headers === 'function';
        const headers = new Headers({ 'Content-Type': 'application/json' });
        const headersWork = headers.get('content-type') === 'application/json';

        // Test Request/Response
        const hasRequest = typeof Request === 'function';
        const hasResponse = typeof Response === 'function';
        const request = new Request('https://example.com', { method: 'POST' });
        const response = new Response('{"test": true}', { status: 200 });
        const requestResponseWork = request.method === 'POST' && response.status === 200;

        // Test AbortController
        const hasAbortController = typeof AbortController === 'function';
        const controller = new AbortController();
        const abortWorks = !controller.signal.aborted;

        // Return comprehensive test results
        JSON.stringify({
            hasFetch,
            hasURL, correctURL,
            hasURLSearchParams, correctParams,
            hasSetTimeout, hasSetInterval, hasClearTimeout, hasClearInterval,
            hasLocalStorage, hasSessionStorage, storageWorks,
            hasAtob, hasBtoa, base64Works,
            hasCrypto, hasRandomUUID, validUUID,
            hasTextEncoder, hasTextDecoder, encoderWorks,
            hasBlob, blobWorks,
            hasFormData, formDataWorks,
            hasHeaders, headersWork,
            hasRequest, hasResponse, requestResponseWork,
            hasAbortController, abortWorks
        });
    "#).await;

    assert!(result.is_ok(), "Polyfills test should execute without errors");
    let output = result.unwrap();

    // Parse the JSON result
    let output_string = renderer.js_value_to_string(output);
    let json_result: serde_json::Value = serde_json::from_str(&output_string.replace("'", "\"")).unwrap();

    // Verify all Web APIs are available
    assert_eq!(json_result["hasFetch"], true, "fetch should be available");
    assert_eq!(json_result["hasURL"], true, "URL should be available");
    assert_eq!(json_result["correctURL"], true, "URL should work correctly");
    assert_eq!(json_result["hasURLSearchParams"], true, "URLSearchParams should be available");
    assert_eq!(json_result["correctParams"], true, "URLSearchParams should work correctly");

    assert_eq!(json_result["hasSetTimeout"], true, "setTimeout should be available");
    assert_eq!(json_result["hasSetInterval"], true, "setInterval should be available");
    assert_eq!(json_result["hasClearTimeout"], true, "clearTimeout should be available");
    assert_eq!(json_result["hasClearInterval"], true, "clearInterval should be available");

    assert_eq!(json_result["hasLocalStorage"], true, "localStorage should be available");
    assert_eq!(json_result["hasSessionStorage"], true, "sessionStorage should be available");
    assert_eq!(json_result["storageWorks"], true, "localStorage should work correctly");

    assert_eq!(json_result["hasAtob"], true, "atob should be available");
    assert_eq!(json_result["hasBtoa"], true, "btoa should be available");
    assert_eq!(json_result["base64Works"], true, "base64 encoding should work correctly");

    assert_eq!(json_result["hasCrypto"], true, "crypto should be available");
    assert_eq!(json_result["hasRandomUUID"], true, "crypto.randomUUID should be available");
    assert_eq!(json_result["validUUID"], true, "UUID generation should work correctly");

    assert_eq!(json_result["hasTextEncoder"], true, "TextEncoder should be available");
    assert_eq!(json_result["hasTextDecoder"], true, "TextDecoder should be available");
    assert_eq!(json_result["encoderWorks"], true, "TextEncoder should work correctly");

    assert_eq!(json_result["hasBlob"], true, "Blob should be available");
    assert_eq!(json_result["blobWorks"], true, "Blob should work correctly");

    assert_eq!(json_result["hasFormData"], true, "FormData should be available");
    assert_eq!(json_result["formDataWorks"], true, "FormData should work correctly");

    assert_eq!(json_result["hasHeaders"], true, "Headers should be available");
    assert_eq!(json_result["headersWork"], true, "Headers should work correctly");

    assert_eq!(json_result["hasRequest"], true, "Request should be available");
    assert_eq!(json_result["hasResponse"], true, "Response should be available");
    assert_eq!(json_result["requestResponseWork"], true, "Request/Response should work correctly");

    assert_eq!(json_result["hasAbortController"], true, "AbortController should be available");
    assert_eq!(json_result["abortWorks"], true, "AbortController should work correctly");

    println!("✅ All comprehensive Web API polyfills are working correctly!");
}

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