// Basic readability extraction tests
//
// Tests the core functionality of the reading mode feature to ensure
// it can extract clean, readable content from web pages.

use thalora::features::readability::{ReadabilityEngine, ReadabilityConfig, OutputFormat};

#[cfg(test)]
mod tests {
    use super::*;

    // Sample HTML for testing
    const SAMPLE_ARTICLE_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Test Article - News Website</title>
    <meta name="description" content="This is a test article for readability extraction">
    <meta name="author" content="Test Author">
</head>
<body>
    <header>
        <nav>
            <a href="/">Home</a>
            <a href="/news">News</a>
            <a href="/about">About</a>
        </nav>
    </header>

    <main>
        <article>
            <h1>Revolutionary AI Technology Changes Everything</h1>
            <p class="byline">By Test Author | Published on January 15, 2025</p>

            <p>This is the first paragraph of our test article. It contains meaningful content that should be extracted by the readability algorithm. The content is substantial enough to meet quality thresholds.</p>

            <p>This is the second paragraph, providing more context and depth to the article. It includes <strong>important information</strong> and <em>emphasizes key points</em> that readers need to understand.</p>

            <h2>Important Subsection</h2>
            <p>Here we dive deeper into the topic with a detailed explanation that spans multiple sentences. This section should also be included in the extraction as it's part of the main content flow.</p>

            <blockquote>
                "This is a meaningful quote that adds value to the article and should be preserved in the extracted content."
            </blockquote>

            <p>The final paragraph wraps up the article with concluding thoughts and provides a satisfying end to the content.</p>
        </article>
    </main>

    <aside>
        <div class="sidebar">
            <h3>Related Articles</h3>
            <ul>
                <li><a href="/article1">Article 1</a></li>
                <li><a href="/article2">Article 2</a></li>
            </ul>
        </div>
        <div class="advertisement">
            <p>This is an advertisement that should be filtered out</p>
        </div>
    </aside>

    <footer>
        <p>Copyright 2025 News Website. All rights reserved.</p>
    </footer>
</body>
</html>
"#;

    // HTML with poor content structure for testing edge cases
    const POOR_CONTENT_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Navigation Heavy Page</title>
</head>
<body>
    <nav>
        <a href="/">Home</a>
        <a href="/about">About</a>
        <a href="/contact">Contact</a>
        <a href="/services">Services</a>
        <a href="/products">Products</a>
        <a href="/blog">Blog</a>
        <a href="/help">Help</a>
    </nav>

    <div>
        <p>Very short content.</p>
    </div>

    <footer>
        <p>Footer content</p>
    </footer>
</body>
</html>
"#;

    #[test]
    fn test_readability_engine_creation() {
        let engine = ReadabilityEngine::new();
        assert_eq!(engine.config().min_content_score, 0.3);
        assert_eq!(engine.config().output_format, OutputFormat::Markdown);
    }

    #[test]
    fn test_basic_content_extraction() {
        let mut engine = ReadabilityEngine::new();

        let result = engine.extract(SAMPLE_ARTICLE_HTML, "https://example.com/article")
            .expect("Extraction should succeed");

        assert!(result.success, "Extraction should be successful");
        assert!(!result.content.content.is_empty(), "Should extract some content");

        // Verify that main content is included
        assert!(result.content.content.contains("Revolutionary AI Technology"));
        assert!(result.content.content.contains("first paragraph"));
        assert!(result.content.content.contains("Important Subsection"));

        // Verify that navigation and ads are excluded
        assert!(!result.content.content.contains("Home"));
        assert!(!result.content.content.contains("advertisement"));
        assert!(!result.content.content.contains("Copyright 2025"));
    }

    #[test]
    fn test_markdown_output_format() {
        let mut engine = ReadabilityEngine::new();

        let result = engine.extract(SAMPLE_ARTICLE_HTML, "https://example.com/article")
            .expect("Extraction should succeed");

        // Check for markdown formatting
        assert!(result.content.content.contains("# Revolutionary AI Technology"));
        assert!(result.content.content.contains("## Important Subsection"));
        assert!(result.content.content.contains("**important information**"));
        assert!(result.content.content.contains("*emphasizes key points*"));
        assert!(result.content.content.contains("> "));
    }

    #[test]
    fn test_text_output_format() {
        let config = ReadabilityConfig {
            output_format: OutputFormat::Text,
            ..Default::default()
        };
        let mut engine = ReadabilityEngine::with_config(config);

        let result = engine.extract(SAMPLE_ARTICLE_HTML, "https://example.com/article")
            .expect("Extraction should succeed");

        // Text format should not contain markdown
        assert!(!result.content.content.contains("#"));
        assert!(!result.content.content.contains("**"));
        assert!(!result.content.content.contains("*"));

        // But should still contain the text content
        assert!(result.content.content.contains("Revolutionary AI Technology"));
        assert!(result.content.content.contains("Important Subsection"));
    }

    #[test]
    fn test_metadata_extraction() {
        let mut engine = ReadabilityEngine::new();

        let result = engine.extract(SAMPLE_ARTICLE_HTML, "https://example.com/article")
            .expect("Extraction should succeed");

        // Check metadata extraction
        assert_eq!(result.content.metadata.title, Some("Test Article - News Website".to_string()));
        assert_eq!(result.content.metadata.author, Some("Test Author".to_string()));
        assert!(result.content.metadata.word_count > 0);
        assert!(result.content.metadata.reading_time_minutes > 0);
    }

    #[test]
    fn test_quality_metrics() {
        let mut engine = ReadabilityEngine::new();

        let result = engine.extract(SAMPLE_ARTICLE_HTML, "https://example.com/article")
            .expect("Extraction should succeed");

        // Check quality metrics
        assert!(result.quality.readability_score > 50, "Should have good readability score");
        assert!(result.quality.completeness > 0.1, "Should have some content completeness");
        assert!(result.quality.noise_level < 0.5, "Should have low noise level");
        assert!(result.quality.structure_quality > 0.3, "Should have reasonable structure quality");
    }

    #[test]
    fn test_poor_content_handling() {
        let mut engine = ReadabilityEngine::new();

        let result = engine.extract(POOR_CONTENT_HTML, "https://example.com/poor")
            .expect("Extraction should complete");

        // Should either fail gracefully or extract minimal content
        if result.success {
            assert!(result.quality.readability_score < 50, "Should have low readability score");
            assert!(result.quality.noise_level > 0.5, "Should have high noise level");
        } else {
            assert!(result.error.is_some(), "Should have error message");
        }
    }

    #[test]
    fn test_custom_content_threshold() {
        let config = ReadabilityConfig {
            min_content_score: 0.8, // Very high threshold
            ..Default::default()
        };
        let mut engine = ReadabilityEngine::with_config(config);

        let result = engine.extract(POOR_CONTENT_HTML, "https://example.com/poor")
            .expect("Extraction should complete");

        // Should fail with high threshold
        assert!(!result.success, "Should fail with high threshold");
        assert!(result.error.is_some(), "Should have error message");
    }

    #[test]
    fn test_processing_time_tracking() {
        let mut engine = ReadabilityEngine::new();

        let result = engine.extract(SAMPLE_ARTICLE_HTML, "https://example.com/article")
            .expect("Extraction should succeed");

        assert!(result.processing_time_ms > 0, "Should track processing time");
        assert!(result.processing_time_ms < 5000, "Should complete reasonably quickly");
    }

    #[test]
    fn test_empty_html_handling() {
        let mut engine = ReadabilityEngine::new();

        let result = engine.extract("", "https://example.com/empty")
            .expect("Should handle empty HTML gracefully");

        assert!(!result.success, "Should fail on empty HTML");
        assert!(result.error.is_some(), "Should have error message");
    }

    #[test]
    fn test_invalid_html_handling() {
        let mut engine = ReadabilityEngine::new();

        let invalid_html = "<html><body><p>Unclosed paragraph<div>Invalid nesting</body>";
        let result = engine.extract(invalid_html, "https://example.com/invalid")
            .expect("Should handle invalid HTML gracefully");

        // HTML5 parser should handle this, but content might be minimal
        assert!(result.processing_time_ms > 0, "Should still process");
    }
}