use anyhow::Result;
use serde_json::json;
use synaptic::{
    AiMemoryHeap, ResearchEntry, NotePriority, MemorySearchCriteria, 
    MemorySortBy, SessionStatus
};
use chrono::Utc;
use std::collections::HashMap;

/// Comprehensive demonstration of AI Memory Heap functionality
/// Shows how AI agents can store and retrieve persistent information across context compressions
#[tokio::main]
async fn main() -> Result<()> {
    println!("🧠 AI Memory Heap Demonstration");
    println!("===============================\n");

    // Initialize the AI memory heap (will create ~/.synaptic/ai_memory.json)
    let mut ai_memory = AiMemoryHeap::new_default()?;
    
    println!("📊 Initial Memory Statistics:");
    let stats = ai_memory.get_statistics();
    println!("   Research entries: {}", stats.research_count);
    println!("   Credentials: {}", stats.credential_count);
    println!("   Sessions: {}", stats.session_count);
    println!("   Bookmarks: {}", stats.bookmark_count);
    println!("   Notes: {}", stats.note_count);
    println!("   File size: {} bytes\n", stats.file_size_bytes);

    // === Demonstrate Research Storage ===
    println!("🔬 Storing Research Findings");
    println!("---------------------------");
    
    let research_entry = ResearchEntry {
        topic: "Chrome DevTools Protocol Implementation".to_string(),
        summary: "Successfully implemented CDP for Synaptic browser with 8 core domains".to_string(),
        findings: vec![
            "Runtime domain enables JavaScript execution and evaluation".to_string(),
            "Debugger domain provides breakpoint management and step controls".to_string(),
            "DOM domain offers document structure inspection".to_string(),
            "Network domain monitors requests and responses".to_string(),
        ],
        sources: vec![
            "https://chromedevtools.github.io/devtools-protocol/".to_string(),
            "https://dev.brainwires.net".to_string(),
        ],
        tags: vec!["CDP".to_string(), "debugging".to_string(), "web-standards".to_string()],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        confidence_score: 0.95,
        related_topics: vec!["browser-automation".to_string(), "AI-coding-agents".to_string()],
    };
    
    ai_memory.store_research("cdp_implementation", research_entry)?;
    println!("✅ Stored research on CDP implementation");

    // Store another research entry
    let web_scraping_research = ResearchEntry {
        topic: "Web Scraping Best Practices".to_string(),
        summary: "Advanced techniques for ethical and effective web scraping".to_string(),
        findings: vec![
            "Use proper HTTP headers to appear as legitimate browser traffic".to_string(),
            "Implement rate limiting to avoid overwhelming target servers".to_string(),
            "Handle JavaScript-heavy sites with proper rendering engines".to_string(),
            "Respect robots.txt and implement stealth features".to_string(),
        ],
        sources: vec![
            "https://httpbin.org/html".to_string(),
            "Web scraping documentation and best practices".to_string(),
        ],
        tags: vec!["scraping".to_string(), "ethics".to_string(), "automation".to_string()],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        confidence_score: 0.9,
        related_topics: vec!["data-extraction".to_string(), "browser-automation".to_string()],
    };
    
    ai_memory.store_research("web_scraping_practices", web_scraping_research)?;
    println!("✅ Stored research on web scraping practices\n");

    // === Demonstrate Credential Storage ===
    println!("🔐 Storing Credentials Securely");
    println!("-------------------------------");
    
    let mut additional_data = HashMap::new();
    additional_data.insert("api_endpoint".to_string(), "https://api.brainwires.net".to_string());
    additional_data.insert("client_id".to_string(), "brainwires_client_123".to_string());
    
    ai_memory.store_credentials(
        "brainwires_dev",
        "Brainwires Development Server",
        "ai_agent@brainwires.net",
        "secure_dev_password_2025",
        additional_data
    )?;
    println!("✅ Stored credentials for Brainwires Development Server");

    // Store another credential
    ai_memory.store_credentials(
        "github_api",
        "GitHub API",
        "synaptic-ai-agent",
        "github_personal_access_token_xyz",
        HashMap::from([
            ("scope".to_string(), "repo,user".to_string()),
            ("rate_limit".to_string(), "5000/hour".to_string()),
        ])
    )?;
    println!("✅ Stored GitHub API credentials\n");

    // === Demonstrate Session Tracking ===
    println!("📝 Starting Development Session");
    println!("------------------------------");
    
    ai_memory.start_session(
        "cdp_enhancement_session",
        "Enhancing Synaptic with Chrome DevTools Protocol support for AI coding agents",
        vec![
            "Research CDP domains and specifications".to_string(),
            "Design CDP implementation architecture".to_string(),
            "Implement core CDP domains".to_string(),
            "Test CDP integration with development server".to_string(),
            "Create AI memory heap for persistent data".to_string(),
        ]
    )?;
    println!("✅ Started development session");

    // Update session progress
    ai_memory.update_session_progress("cdp_enhancement_session", "current_phase", json!("Implementation Complete"))?;
    ai_memory.complete_session_task("cdp_enhancement_session", "Research CDP domains and specifications")?;
    ai_memory.complete_session_task("cdp_enhancement_session", "Design CDP implementation architecture")?;
    ai_memory.complete_session_task("cdp_enhancement_session", "Implement core CDP domains")?;
    ai_memory.complete_session_task("cdp_enhancement_session", "Test CDP integration with development server")?;
    println!("✅ Updated session progress\n");

    // === Demonstrate Bookmark Storage ===
    println!("🔖 Storing Important Resources");
    println!("-----------------------------");
    
    ai_memory.store_bookmark(
        "chrome_devtools_protocol",
        "https://chromedevtools.github.io/devtools-protocol/",
        "Chrome DevTools Protocol",
        "Official Chrome DevTools Protocol documentation and API reference",
        "The Chrome DevTools Protocol allows for tools to instrument, inspect, debug and profile Chromium, Chrome and other Blink-based browsers...",
        vec!["CDP".to_string(), "debugging".to_string(), "reference".to_string()]
    )?;
    println!("✅ Bookmarked CDP documentation");

    ai_memory.store_bookmark(
        "brainwires_dev",
        "https://dev.brainwires.net",
        "Brainwires Studio Development Server",
        "Development server for testing Synaptic browser capabilities",
        "Brainwires Studio - Advanced AI development platform with modern web technologies...",
        vec!["development".to_string(), "testing".to_string(), "AI".to_string()]
    )?;
    println!("✅ Bookmarked Brainwires development server\n");

    // === Demonstrate Note Storage ===
    println!("📋 Storing Development Notes");
    println!("---------------------------");
    
    ai_memory.store_note(
        "ssl_tls_fix",
        "Fixed SSL/TLS Protocol Negotiation Issue",
        "Resolved NoApplicationProtocol error with dev.brainwires.net by removing http2_prior_knowledge() from HTTP client configuration. This allows proper ALPN negotiation between HTTP/1.1 and HTTP/2.",
        "bug-fixes",
        vec!["SSL".to_string(), "TLS".to_string(), "ALPN".to_string(), "HTTP2".to_string()],
        NotePriority::High
    )?;
    println!("✅ Stored bug fix note");

    ai_memory.store_note(
        "memory_heap_design",
        "AI Memory Heap Architecture",
        "Implemented persistent storage system with categories: research, credentials, sessions, bookmarks, notes. Uses encrypted password storage and supports search/filtering. File location: ~/.synaptic/ai_memory.json",
        "architecture",
        vec!["memory".to_string(), "persistence".to_string(), "AI".to_string(), "storage".to_string()],
        NotePriority::Critical
    )?;
    println!("✅ Stored architecture note\n");

    // === Demonstrate Search Functionality ===
    println!("🔍 Searching Stored Information");
    println!("------------------------------");
    
    // Search research entries
    let research_criteria = MemorySearchCriteria {
        query: Some("CDP".to_string()),
        tags: None,
        date_range: None,
        category: None,
        limit: Some(5),
        sort_by: MemorySortBy::UpdatedAt,
    };
    
    let research_results = ai_memory.search_research(&research_criteria);
    println!("🔬 Found {} research entries matching 'CDP':", research_results.len());
    for (key, entry) in research_results {
        println!("   • {}: {} (confidence: {:.0}%)", key, entry.topic, entry.confidence_score * 100.0);
    }
    
    // Search notes by category
    let notes_criteria = MemorySearchCriteria {
        query: None,
        tags: None,
        date_range: None,
        category: Some("bug-fixes".to_string()),
        limit: None,
        sort_by: MemorySortBy::Priority,
    };
    
    let note_results = ai_memory.search_notes(&notes_criteria);
    println!("\n📋 Found {} notes in 'bug-fixes' category:", note_results.len());
    for (key, note) in note_results {
        println!("   • {}: {} (priority: {:?})", key, note.title, note.priority);
    }

    // Search bookmarks
    let bookmark_criteria = MemorySearchCriteria {
        query: Some("development".to_string()),
        tags: None,
        date_range: None,
        category: None,
        limit: None,
        sort_by: MemorySortBy::AccessCount,
    };
    
    let bookmark_results = ai_memory.search_bookmarks(&bookmark_criteria);
    println!("\n🔖 Found {} bookmarks matching 'development':", bookmark_results.len());
    for (key, bookmark) in bookmark_results {
        println!("   • {}: {} (accessed {} times)", key, bookmark.title, bookmark.access_count);
    }
    
    // === Demonstrate Credential Retrieval ===
    println!("\n🔐 Retrieving Stored Credentials");
    println!("-------------------------------");
    
    if let Some((service, username, password, additional_data)) = ai_memory.get_credentials("brainwires_dev")? {
        println!("✅ Retrieved credentials for {}", service);
        println!("   Username: {}", username);
        println!("   Password: {} (encrypted storage)", if password.len() > 8 { "***[SECURED]***" } else { "***" });
        println!("   Additional data: {} items", additional_data.len());
        for (key, value) in additional_data {
            println!("     - {}: {}", key, value);
        }
    }
    
    // === Final Statistics ===
    println!("\n📊 Final Memory Statistics:");
    let final_stats = ai_memory.get_statistics();
    println!("   Research entries: {}", final_stats.research_count);
    println!("   Credentials: {}", final_stats.credential_count);
    println!("   Active sessions: {}", ai_memory.get_active_sessions().len());
    println!("   Bookmarks: {}", final_stats.bookmark_count);
    println!("   Notes: {}", final_stats.note_count);
    println!("   Total entries: {}", final_stats.total_entries);
    println!("   File size: {} bytes", final_stats.file_size_bytes);
    println!("   Last updated: {}", final_stats.last_updated);

    println!("\n🎉 AI Memory Heap demonstration complete!");
    println!("💾 All data has been persisted to ~/.synaptic/ai_memory.json");
    println!("🔄 This information will survive context compressions and be available in future sessions");
    
    // === Demonstrate Context Compression Simulation ===
    println!("\n🔄 Simulating Context Compression Recovery");
    println!("------------------------------------------");
    
    // Simulate starting a new session by creating a new memory instance
    let recovered_memory = AiMemoryHeap::new_default()?;
    let recovered_stats = recovered_memory.get_statistics();
    
    println!("✅ Successfully recovered {} total entries from persistent storage", recovered_stats.total_entries);
    println!("   This demonstrates how AI agents can maintain persistent memory across:");
    println!("   • Context window resets");  
    println!("   • Session restarts");
    println!("   • System reboots");
    println!("   • Long-term project continuity");
    
    Ok(())
}

/// Example helper function showing how AI agents might use the memory system
/// in practice during development workflows
async fn ai_agent_workflow_example() -> Result<()> {
    println!("\n🤖 AI Agent Workflow Example");
    println!("============================");
    
    let mut memory = AiMemoryHeap::new_default()?;
    
    // Agent encounters a bug
    println!("🐛 AI Agent encounters SSL/TLS issue with dev.brainwires.net");
    
    // Store research about the problem
    memory.store_research("ssl_tls_investigation", ResearchEntry {
        topic: "NoApplicationProtocol SSL Error Investigation".to_string(),
        summary: "HTTP client fails to connect due to ALPN negotiation issues".to_string(),
        findings: vec![
            "Error: received fatal alert: NoApplicationProtocol".to_string(),
            "Caused by http2_prior_knowledge() forcing HTTP/2 without negotiation".to_string(),
            "Solution: Remove http2_prior_knowledge() to allow ALPN".to_string(),
        ],
        sources: vec!["Rust reqwest documentation".to_string()],
        tags: vec!["debug".to_string(), "networking".to_string()],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        confidence_score: 0.9,
        related_topics: vec!["HTTP".to_string(), "TLS".to_string()],
    })?;
    
    // Store the solution as a note
    memory.store_note(
        "http_client_fix",
        "HTTP Client Configuration Fix",
        "Fixed by removing .http2_prior_knowledge() and using .http1_title_case_headers() instead",
        "solutions",
        vec!["fix".to_string(), "HTTP".to_string()],
        NotePriority::High
    )?;
    
    println!("✅ AI Agent stored problem analysis and solution");
    println!("💡 This information is now available for future debugging sessions");
    
    Ok(())
}