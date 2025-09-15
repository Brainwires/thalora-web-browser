use synaptic::engine::HeadlessWebBrowser;
use boa_engine::{Context, Source};

#[tokio::test]
async fn test_canvas_get_context_2d() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test canvas element with 2D context
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        typeof ctx === "object" && ctx !== null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_canvas_2d_context_properties() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test 2D context properties
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        ctx.fillStyle === "#000000" &&
        ctx.strokeStyle === "#000000" &&
        ctx.lineWidth === 1 &&
        ctx.font === "10px sans-serif";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_canvas_2d_context_methods() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test 2D context drawing methods exist
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        typeof ctx.fillRect === "function" &&
        typeof ctx.fillText === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_canvas_get_context_webgl() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test canvas element with WebGL context
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        typeof gl === "object" && gl !== null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test experimental-webgl alias
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('experimental-webgl');
        typeof gl === "object" && gl !== null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_canvas_get_context_webgl2() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test canvas element with WebGL2 context
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl2');
        typeof gl === "object" && gl !== null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_webgl_constants() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test WebGL constants are defined
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        gl.VERTEX_SHADER === 35633 &&
        gl.FRAGMENT_SHADER === 35632 &&
        gl.ARRAY_BUFFER === 34962 &&
        gl.STATIC_DRAW === 35044 &&
        gl.COLOR_BUFFER_BIT === 16384 &&
        gl.TRIANGLES === 4 &&
        gl.FLOAT === 5126;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_webgl_shader_methods() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test WebGL shader creation methods
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        typeof gl.createShader === "function" &&
        typeof gl.shaderSource === "function" &&
        typeof gl.compileShader === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test shader creation returns objects
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const shader = gl.createShader(gl.VERTEX_SHADER);
        typeof shader === "object" && shader !== null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_webgl_program_methods() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test WebGL program methods
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        typeof gl.createProgram === "function" &&
        typeof gl.attachShader === "function" &&
        typeof gl.linkProgram === "function" &&
        typeof gl.useProgram === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test program creation returns object
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const program = gl.createProgram();
        typeof program === "object" && program !== null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_webgl_buffer_methods() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test WebGL buffer methods
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        typeof gl.createBuffer === "function" &&
        typeof gl.bindBuffer === "function" &&
        typeof gl.bufferData === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test buffer creation returns object
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const buffer = gl.createBuffer();
        typeof buffer === "object" && buffer !== null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_webgl_rendering_methods() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test WebGL rendering methods
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        typeof gl.viewport === "function" &&
        typeof gl.clearColor === "function" &&
        typeof gl.clear === "function" &&
        typeof gl.drawArrays === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_webgl_get_parameter() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test WebGL getParameter method for fingerprinting info
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const vendor = gl.getParameter(7936); // GL_VENDOR
        const renderer = gl.getParameter(7937); // GL_RENDERER
        const version = gl.getParameter(7938); // GL_VERSION
        const shadingLanguage = gl.getParameter(35724); // GL_SHADING_LANGUAGE_VERSION

        vendor === "WebKit" &&
        renderer === "WebKit WebGL" &&
        version === "WebGL 1.0 (OpenGL ES 2.0 Chromium)" &&
        shadingLanguage === "WebGL GLSL ES 1.0 (OpenGL ES GLSL ES 1.0 Chromium)";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test numeric parameters
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const maxTextureSize = gl.getParameter(3379); // GL_MAX_TEXTURE_SIZE
        const maxViewportDims = gl.getParameter(3386); // GL_MAX_VIEWPORT_DIMS
        const maxTextureUnits = gl.getParameter(34024); // GL_MAX_TEXTURE_IMAGE_UNITS

        maxTextureSize === 16384 &&
        maxViewportDims === 30 &&
        maxTextureUnits === 16;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_webgl_extensions() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test getSupportedExtensions method
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const extensions = gl.getSupportedExtensions();
        typeof extensions === "object" && extensions.length === 5;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test getExtension method for debug renderer info
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const debugInfo = gl.getExtension('WEBGL_debug_renderer_info');
        typeof debugInfo === "object" &&
        debugInfo.UNMASKED_VENDOR_WEBGL === 37445 &&
        debugInfo.UNMASKED_RENDERER_WEBGL === 37446;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test getExtension method for lose context
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const loseContext = gl.getExtension('WEBGL_lose_context');
        typeof loseContext === "object" &&
        typeof loseContext.loseContext === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test getExtension returns null for unsupported extensions
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const unsupported = gl.getExtension('UNSUPPORTED_EXTENSION');
        unsupported === null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_webgl_texture_methods() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test WebGL texture methods
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        typeof gl.createTexture === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test texture creation returns object
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        const texture = gl.createTexture();
        typeof texture === "object" && texture !== null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}

#[tokio::test]
async fn test_webgl2_specific_methods() {
    let mut browser = HeadlessWebBrowser::new();
    let mut context = Context::default();
    browser.setup_browser_environment(&mut context).await.expect("Failed to setup browser");

    // Test WebGL2 specific methods exist when getting webgl2 context
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl2');
        typeof gl.createVertexArray === "function" &&
        typeof gl.bindVertexArray === "function";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test WebGL2 methods don't exist on WebGL1 context
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl');
        typeof gl.createVertexArray === "undefined" &&
        typeof gl.bindVertexArray === "undefined";
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");

    // Test WebGL2 VAO creation
    let result = context.eval(Source::from_bytes(r#"
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl2');
        const vao = gl.createVertexArray();
        typeof vao === "object" && vao !== null;
    "#)).unwrap();
    assert_eq!(result.to_string(&mut context).unwrap().to_std_string_escaped(), "true");
}