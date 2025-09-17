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
