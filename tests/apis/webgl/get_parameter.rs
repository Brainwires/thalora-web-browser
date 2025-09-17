use thalora::engine::HeadlessWebBrowser;
use boa_engine::{Context, Source};

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
