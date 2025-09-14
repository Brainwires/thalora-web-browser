use anyhow::Result;
use synaptic::HeadlessWebBrowser;

/// Test connection to dev.brainwires.net with fixed HTTP client configuration
#[tokio::main]
async fn main() -> Result<()> {
    println!("🔧 Testing connection to dev.brainwires.net with fixed HTTP client...");
    
    let mut browser = HeadlessWebBrowser::new();
    
    match browser.scrape("https://dev.brainwires.net", true, None, true, true).await {
        Ok(scraped) => {
            println!("✅ Successfully connected to dev.brainwires.net!");
            println!("📄 Title: {}", scraped.title.as_deref().unwrap_or("No title"));
            println!("🔗 Found {} links", scraped.links.len());
            println!("🖼️  Found {} images", scraped.images.len());
            println!("📝 Content length: {} characters", scraped.content.len());
            
            // Show first 200 characters of content
            let preview = if scraped.content.len() > 200 {
                format!("{}...", &scraped.content[..200])
            } else {
                scraped.content.clone()
            };
            println!("📋 Content preview: {}", preview);
            
            Ok(())
        }
        Err(e) => {
            println!("❌ Connection failed: {}", e);
            println!("🔍 Error details: {:#?}", e);
            Err(e)
        }
    }
}