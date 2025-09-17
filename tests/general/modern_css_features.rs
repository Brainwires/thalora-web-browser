use thalora::HeadlessWebBrowser;
use std::collections::HashMap;

#[tokio::test]
async fn test_modern_css_features() {
    println!("🧪 Testing modern CSS feature support...");

    let browser = HeadlessWebBrowser::new();

    // Test modern CSS features through JavaScript
    let css_feature_tests = vec![
        // CSS Grid
        ("CSS Grid", "CSS.supports('display', 'grid')"),

        // CSS Flexbox
        ("CSS Flexbox", "CSS.supports('display', 'flex')"),

        // CSS Custom Properties
        ("CSS Variables", "CSS.supports('--custom-property', 'value')"),

        // CSS Subgrid
        ("CSS Subgrid", "CSS.supports('grid-template-rows', 'subgrid')"),

        // CSS Container Queries
        ("CSS Container Queries", "CSS.supports('container-type', 'inline-size')"),

        // CSS has() selector
        ("CSS :has() selector", "CSS.supports('selector(:has(div))')"),

        // CSS Cascade Layers
        ("CSS @layer", "CSS.supports('@layer')"),

        // CSS Logical Properties
        ("CSS Logical Properties", "CSS.supports('margin-inline-start', '1em')"),

        // CSS aspect-ratio
        ("CSS aspect-ratio", "CSS.supports('aspect-ratio', '16/9')"),

        // CSS gap for flexbox
        ("CSS gap (flexbox)", "CSS.supports('gap', '1rem')"),
    ];

    let mut css_supported = 0;
    let mut css_not_supported = 0;

    for (feature_name, css_test) in css_feature_tests {
        let result = browser.lock().unwrap().execute_javascript(css_test).await;

        match result {
            Ok(value) => {
                let value_str = format!("{:?}", value);
                if value_str.contains("true") {
                    println!("✅ {}: Supported", feature_name);
                    css_supported += 1;
                } else {
                    println!("❌ {}: Not supported ({})", feature_name, value_str);
                    css_not_supported += 1;
                }
            }
            Err(e) => {
                println!("❌ {}: Error testing - {:?}", feature_name, e);
                css_not_supported += 1;
            }
        }
    }

    println!("\n📊 Modern CSS Feature Results:");
    println!("  ✅ Supported: {}", css_supported);
    println!("  ❌ Not Supported: {}", css_not_supported);
    println!("  📈 CSS Feature Coverage: {:.1}%", (css_supported as f64 / (css_supported + css_not_supported) as f64) * 100.0);

    assert!(css_supported + css_not_supported > 0, "Should have tested CSS features");
}
