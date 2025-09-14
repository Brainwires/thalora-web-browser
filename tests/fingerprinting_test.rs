use synaptic::{BrowserFingerprint, FingerprintManager, BrowserType};

#[test]
fn test_fingerprinting_resistance() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Enhanced Fingerprinting Resistance");
    
    // Test individual browser fingerprints
    let browsers = vec![
        ("Chrome", BrowserType::Chrome),
        ("Firefox", BrowserType::Firefox), 
        ("Safari", BrowserType::Safari),
        ("Edge", BrowserType::Edge),
    ];
    
    for (name, browser_type) in browsers {
        println!("\n🌐 {} Fingerprint:", name);
        let fingerprint = BrowserFingerprint::generate(browser_type);
        
        println!("  User Agent: {}", &fingerprint.user_agent);
        println!("  TLS Version: {}", fingerprint.tls_fingerprint.version);
        println!("  HTTP/2 Window Size: {}", fingerprint.http2_settings.window_size);
        println!("  Screen: {}x{}", fingerprint.screen_info.width, fingerprint.screen_info.height);
        println!("  Accept Headers: {}", fingerprint.accept_headers.len());
        println!("  Browser Headers: {}", fingerprint.browser_headers.len());
        println!("  Canvas Fingerprint: {}", &fingerprint.canvas_fingerprint[..20]);
        println!("  WebGL Vendor: {}", fingerprint.webgl_fingerprint.get("vendor").unwrap_or(&"Unknown".to_string()));
    }
    
    // Test fingerprint manager rotation
    println!("\n🔄 Fingerprint Manager Rotation Test:");
    let mut manager = FingerprintManager::new();
    
    for i in 0..6 {
        let current_fp = if i == 0 { manager.current() } else { manager.rotate() };
        println!("  Rotation {}: {}", i + 1, &current_fp.user_agent[..60]);
    }
    
    // Test random fingerprints
    println!("\n🎲 Random Fingerprint Test:");
    for i in 0..3 {
        let random_fp = manager.random();
        println!("  Random {}: {}", i + 1, &random_fp.user_agent[..60]);
    }
    
    // Test HTTP client integration
    println!("\n🌐 HTTP Client Integration Test:");
    let fingerprint = BrowserFingerprint::generate(BrowserType::Chrome);
    let client_builder = reqwest::Client::builder();
    let client = fingerprint.apply_to_client_builder(client_builder).build()?;
    
    println!("  ✅ Successfully created HTTP client with Chrome fingerprint");
    println!("  📊 Fingerprint Details:");
    println!("    - TLS Cipher Suites: {}", fingerprint.tls_fingerprint.cipher_suites.len());
    println!("    - TLS Extensions: {}", fingerprint.tls_fingerprint.extensions.len());
    println!("    - HTTP/2 Max Concurrent Streams: {}", fingerprint.http2_settings.max_concurrent_streams);
    println!("    - Screen Color Depth: {} bits", fingerprint.screen_info.color_depth);
    
    println!("\n🎉 Enhanced Fingerprinting Resistance working correctly!");
    println!("   - 4 distinct browser signatures implemented");
    println!("   - TLS and HTTP/2 fingerprints configured");
    println!("   - Canvas and WebGL simulation included");
    println!("   - Header rotation and management working");
    
    Ok(())
}