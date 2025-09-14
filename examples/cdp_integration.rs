use anyhow::Result;
use serde_json::json;
use synaptic::{HeadlessWebBrowser, CdpServer, CdpMessage, CdpCommand};
use tokio::time::{sleep, Duration};

/// Example demonstrating CDP integration with Synaptic for AI coding agents
/// This shows how an AI agent can debug a web application using Chrome DevTools Protocol
#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Synaptic CDP Integration Example for AI Coding Agents");
    println!("======================================================\n");

    // Initialize browser and CDP server
    let mut browser = HeadlessWebBrowser::new();
    let mut cdp_server = CdpServer::new();
    
    // Test with development server - now working with fixed ALPN negotiation!
    let test_url = "https://dev.brainwires.net";
    println!("🌐 Loading development server: {}", test_url);
    
    let scraped = browser.scrape(test_url, true, None, true, true).await?;
    println!("✅ Page loaded successfully");
    println!("📄 Title: {}", scraped.title.as_deref().unwrap_or("No title"));
    println!("🔗 Found {} links", scraped.links.len());
    println!("📝 Content length: {} characters", scraped.content.len());
    
    println!("🌟 Testing URL: {}", test_url);
    
    // Demonstrate CDP debugging workflow for AI agents
    demonstrate_runtime_debugging(&mut cdp_server).await?;
    demonstrate_dom_inspection(&mut cdp_server).await?;
    demonstrate_debugger_controls(&mut cdp_server).await?;
    demonstrate_network_monitoring(&mut cdp_server).await?;
    demonstrate_performance_analysis(&mut cdp_server).await?;
    
    // Run the AI agent workflow example
    ai_agent_debug_workflow(&mut cdp_server, &mut browser).await?;
    
    println!("\n🎉 CDP Integration demonstration complete!");
    println!("AI coding agents can now use these debugging capabilities to:");
    println!("• Inspect and manipulate DOM elements");
    println!("• Execute JavaScript and evaluate expressions");
    println!("• Set breakpoints and step through code");
    println!("• Monitor network requests and responses");
    println!("• Analyze performance metrics");
    println!("• Access storage and console information");
    
    Ok(())
}

async fn demonstrate_runtime_debugging(cdp_server: &mut CdpServer) -> Result<()> {
    println!("\n🔧 Runtime Domain - JavaScript Execution and Inspection");
    println!("-------------------------------------------------------");
    
    // Enable runtime
    let enable_cmd = CdpMessage::Command(CdpCommand {
        id: 1,
        method: "Runtime.enable".to_string(),
        params: None,
        session_id: None,
    });
    
    let response = cdp_server.handle_message(enable_cmd)?;
    println!("✅ Runtime enabled: {:?}", response);
    
    // Evaluate JavaScript expression
    let eval_cmd = CdpMessage::Command(CdpCommand {
        id: 2,
        method: "Runtime.evaluate".to_string(),
        params: Some(json!({
            "expression": "console.log('AI Agent Debug Session Started'); 2 + 2",
            "returnByValue": true
        })),
        session_id: None,
    });
    
    let response = cdp_server.handle_message(eval_cmd)?;
    println!("🧮 JavaScript evaluation result: {:?}", response);
    
    // Compile script for future execution
    let compile_cmd = CdpMessage::Command(CdpCommand {
        id: 3,
        method: "Runtime.compileScript".to_string(),
        params: Some(json!({
            "expression": "function debugHelperFunction() { return 'AI debugging helper loaded'; }",
            "sourceURL": "ai-debug-helper.js",
            "persistScript": true
        })),
        session_id: None,
    });
    
    let response = cdp_server.handle_message(compile_cmd)?;
    println!("📜 Script compilation result: {:?}", response);
    
    Ok(())
}

async fn demonstrate_dom_inspection(cdp_server: &mut CdpServer) -> Result<()> {
    println!("\n🏗️  DOM Domain - Document Structure Inspection");
    println!("----------------------------------------------");
    
    // Enable DOM
    let enable_cmd = CdpMessage::Command(CdpCommand {
        id: 10,
        method: "DOM.enable".to_string(),
        params: None,
        session_id: None,
    });
    
    let response = cdp_server.handle_message(enable_cmd)?;
    println!("✅ DOM enabled: {:?}", response);
    
    // Get document root
    let doc_cmd = CdpMessage::Command(CdpCommand {
        id: 11,
        method: "DOM.getDocument".to_string(),
        params: None,
        session_id: None,
    });
    
    let response = cdp_server.handle_message(doc_cmd)?;
    println!("🌳 Document structure: {:?}", response);
    
    // Query selector for AI analysis
    let query_cmd = CdpMessage::Command(CdpCommand {
        id: 12,
        method: "DOM.querySelector".to_string(),
        params: Some(json!({
            "nodeId": 1,
            "selector": "button[data-test-id], .error-message, form input[type='email']"
        })),
        session_id: None,
    });
    
    let response = cdp_server.handle_message(query_cmd)?;
    println!("🔍 Query selector results: {:?}", response);
    
    // Query all elements for comprehensive analysis
    let query_all_cmd = CdpMessage::Command(CdpCommand {
        id: 13,
        method: "DOM.querySelectorAll".to_string(),
        params: Some(json!({
            "nodeId": 1,
            "selector": "a[href], img[src], script[src]"
        })),
        session_id: None,
    });
    
    let response = cdp_server.handle_message(query_all_cmd)?;
    println!("📋 All matching elements: {:?}", response);
    
    Ok(())
}

async fn demonstrate_debugger_controls(cdp_server: &mut CdpServer) -> Result<()> {
    println!("\n🐛 Debugger Domain - JavaScript Debugging Controls");
    println!("--------------------------------------------------");
    
    // Enable debugger
    let enable_cmd = CdpMessage::Command(CdpCommand {
        id: 20,
        method: "Debugger.enable".to_string(),
        params: None,
        session_id: None,
    });
    
    let response = cdp_server.handle_message(enable_cmd)?;
    println!("✅ Debugger enabled: {:?}", response);
    
    // Set breakpoint for AI analysis
    let breakpoint_cmd = CdpMessage::Command(CdpCommand {
        id: 21,
        method: "Debugger.setBreakpointByUrl".to_string(),
        params: Some(json!({
            "lineNumber": 10,
            "url": "https://dev.brainwires.net/app.js",
            "condition": "typeof error !== 'undefined'"
        })),
        session_id: None,
    });
    
    let response = cdp_server.handle_message(breakpoint_cmd)?;
    println!("🎯 Breakpoint set: {:?}", response);
    
    // Simulate stepping through code
    sleep(Duration::from_millis(100)).await;
    
    let step_cmd = CdpMessage::Command(CdpCommand {
        id: 22,
        method: "Debugger.stepOver".to_string(),
        params: None,
        session_id: None,
    });
    
    let response = cdp_server.handle_message(step_cmd)?;
    println!("👣 Step over executed: {:?}", response);
    
    // Resume execution
    let resume_cmd = CdpMessage::Command(CdpCommand {
        id: 23,
        method: "Debugger.resume".to_string(),
        params: None,
        session_id: None,
    });
    
    let response = cdp_server.handle_message(resume_cmd)?;
    println!("▶️  Execution resumed: {:?}", response);
    
    Ok(())
}

async fn demonstrate_network_monitoring(cdp_server: &mut CdpServer) -> Result<()> {
    println!("\n🌐 Network Domain - Request/Response Monitoring");
    println!("-----------------------------------------------");
    
    // Enable network monitoring
    let enable_cmd = CdpMessage::Command(CdpCommand {
        id: 30,
        method: "Network.enable".to_string(),
        params: None,
        session_id: None,
    });
    
    let response = cdp_server.handle_message(enable_cmd)?;
    println!("✅ Network monitoring enabled: {:?}", response);
    
    // Get cookies for session analysis
    let cookies_cmd = CdpMessage::Command(CdpCommand {
        id: 31,
        method: "Network.getAllCookies".to_string(),
        params: None,
        session_id: None,
    });
    
    let response = cdp_server.handle_message(cookies_cmd)?;
    println!("🍪 Cookies retrieved: {:?}", response);
    
    Ok(())
}

async fn demonstrate_performance_analysis(cdp_server: &mut CdpServer) -> Result<()> {
    println!("\n📊 Performance Domain - Metrics and Analysis");
    println!("--------------------------------------------");
    
    // Enable performance monitoring
    let enable_cmd = CdpMessage::Command(CdpCommand {
        id: 40,
        method: "Performance.enable".to_string(),
        params: None,
        session_id: None,
    });
    
    let response = cdp_server.handle_message(enable_cmd)?;
    println!("✅ Performance monitoring enabled: {:?}", response);
    
    // Get current metrics
    let metrics_cmd = CdpMessage::Command(CdpCommand {
        id: 41,
        method: "Performance.getMetrics".to_string(),
        params: None,
        session_id: None,
    });
    
    let response = cdp_server.handle_message(metrics_cmd)?;
    println!("📈 Performance metrics: {:?}", response);
    
    Ok(())
}

/// Example AI agent workflow using CDP
pub async fn ai_agent_debug_workflow(cdp_server: &mut CdpServer, browser: &mut HeadlessWebBrowser) -> Result<()> {
    println!("\n🤖 AI Agent Debugging Workflow Example");
    println!("=====================================");
    
    // Step 1: Load and analyze page  
    let url = "https://dev.brainwires.net";
    let scraped = browser.scrape(url, true, None, true, true).await?;
    println!("📄 Loaded page: {}", scraped.title.as_deref().unwrap_or("No title"));
    
    // Step 2: Enable all debugging domains
    for domain in &["Runtime", "DOM", "Debugger", "Network", "Console", "Performance"] {
        let cmd = CdpMessage::Command(CdpCommand {
            id: 100,
            method: format!("{}.enable", domain),
            params: None,
            session_id: None,
        });
        cdp_server.handle_message(cmd)?;
        println!("✅ {} domain enabled", domain);
    }
    
    // Step 3: Analyze DOM structure for potential issues
    let dom_cmd = CdpMessage::Command(CdpCommand {
        id: 101,
        method: "DOM.querySelectorAll".to_string(),
        params: Some(json!({
            "nodeId": 1,
            "selector": ".error, [data-error], .warning, .exception"
        })),
        session_id: None,
    });
    
    let dom_result = cdp_server.handle_message(dom_cmd)?;
    println!("🔍 Error element analysis: {:?}", dom_result);
    
    // Step 4: Execute diagnostic JavaScript
    let diag_cmd = CdpMessage::Command(CdpCommand {
        id: 102,
        method: "Runtime.evaluate".to_string(),
        params: Some(json!({
            "expression": r#"
                (function() {
                    const diagnostics = {
                        errorCount: document.querySelectorAll('.error').length,
                        formCount: document.forms.length,
                        scriptCount: document.scripts.length,
                        hasReact: typeof window.React !== 'undefined',
                        hasJQuery: typeof window.$ !== 'undefined',
                        viewport: {
                            width: window.innerWidth,
                            height: window.innerHeight
                        }
                    };
                    return JSON.stringify(diagnostics, null, 2);
                })()
            "#,
            "returnByValue": true
        })),
        session_id: None,
    });
    
    let diag_result = cdp_server.handle_message(diag_cmd)?;
    println!("🧪 Page diagnostics: {:?}", diag_result);
    
    // Step 5: Set strategic breakpoints
    let bp_cmd = CdpMessage::Command(CdpCommand {
        id: 103,
        method: "Debugger.setBreakpointByUrl".to_string(),
        params: Some(json!({
            "lineNumber": 1,
            "url": url,
            "condition": "console.error.arguments.length > 0"
        })),
        session_id: None,
    });
    
    let bp_result = cdp_server.handle_message(bp_cmd)?;
    println!("🎯 Strategic breakpoint set: {:?}", bp_result);
    
    println!("🎉 AI Agent debugging workflow completed!");
    println!("   The agent can now intelligently debug web applications");
    
    Ok(())
}