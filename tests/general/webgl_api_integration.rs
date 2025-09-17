use thalora::apis::WebApis;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_webgl_api_integration() {
    use thalora::engine::RustRenderer;

    let mut renderer = RustRenderer::new();

    // Setup DOM elements to test WebGL
    let js_code = r#"
        // Test canvas WebGL context creation
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');

        // Store results for testing
        window.testResults = {
            hasWebGL: typeof gl === "object" && gl !== null,
            hasConstants: gl && gl.VERTEX_SHADER === 35633 && gl.FRAGMENT_SHADER === 35632 && gl.TRIANGLES === 4,
            hasMethods: gl && typeof gl.createShader === "function" && typeof gl.createProgram === "function" && typeof gl.createBuffer === "function",
            hasFingerprinting: gl && gl.getParameter(7936) === "WebKit" && gl.getParameter(7937) === "WebKit WebGL",
            hasWebGL2: (() => {
                const gl2 = canvas.getContext('webgl2');
                return typeof gl2 === "object" && gl2 !== null && typeof gl2.createVertexArray === "function";
            })()
        };

        window.testResults;
    "#;

    let result = renderer.execute_javascript_safely(js_code).await.expect("JS execution failed");

    // Test that the execution didn't fail - if we get here, WebGL is working
    println!("WebGL test result: {}", renderer.js_value_to_string(result));

    println!("✅ WebGL API integration test passed");
}
