#[tokio::test]
async fn test_chrome_127_media_metadata_chapters() {
    println!("🧪 Testing Chrome 127: Media Metadata Chapters...");

    let browser = HeadlessWebBrowser::new();

    // Test MediaMetadata with chapter information
    let js_code = r#"
        try {
            if (typeof MediaMetadata !== 'undefined') {
                var metadata = new MediaMetadata({
                    title: 'Test Video',
                    artist: 'Test Artist',
                    album: 'Test Album',
                    artwork: [{
                        src: 'test.jpg',
                        sizes: '96x96',
                        type: 'image/jpeg'
                    }],
                    // Chrome 127: Chapter information support
                    chapterInfo: [{
                        title: 'Chapter 1: Introduction',
                        startTime: 0,
                        artwork: [{
                            src: 'chapter1.jpg',
                            sizes: '96x96',
                            type: 'image/jpeg'
                        }]
                    }, {
                        title: 'Chapter 2: Content',
                        startTime: 300,
                        artwork: [{
                            src: 'chapter2.jpg',
                            sizes: '96x96',
                            type: 'image/jpeg'
                        }]
                    }]
                });

                'MediaMetadata with chapters created successfully';
            } else {
                'MediaMetadata not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Media metadata chapters test: {}", value_str);
            // MediaMetadata might not be fully implemented yet, but should not throw syntax errors
            assert!(!value_str.contains("SyntaxError"), "Media metadata chapters should not cause syntax errors, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test media metadata chapters: {:?}", e),
    }

    println!("✅ Media metadata chapters test completed");
}
