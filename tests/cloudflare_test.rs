use synaptic::{ChallengeSolver, ChallengeType};
use tracing_subscriber;

#[tokio::test]
async fn test_cloudflare_challenge_solver() -> Result<(), Box<dyn std::error::Error>> {
    // Enable debug logging
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();
        
    println!("🧪 Testing Enhanced Cloudflare Challenge Solver");
    
    let mut solver = ChallengeSolver::new();
    
    let enhanced_cloudflare_html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Just a moment...</title>
    </head>
    <body>
        <div class="cf-browser-verification cf-im-under-attack">
            <div class="cf-wrapper">
                <div class="cf-challenge-running" id="cf-challenge-running">
                    Checking your browser before accessing the website.
                </div>
            </div>
        </div>
        <form id="challenge-form" action="/cdn-cgi/l/chk_jschl" method="get">
            <input type="hidden" name="jschl_vc" value="a1b2c3d4e5f6"/>
            <input type="hidden" name="pass" value="1640123456.789-AbCdEfGhIjKl"/>
            <input type="hidden" name="s" value="xyz789"/>
            <input type="hidden" name="r" value="fedcba9876543210"/>
        </form>
        <script>
            var a = 25;
            var t = 4;
            var s = "example.com";
            s = s.length;
            var result = a + t + s;
            
            setTimeout(function() {
                document.getElementById('challenge-form').submit();
                window.cf_challenge_complete = true;
            }, 4000);
            
            // Mock Cloudflare challenge completion
            window.jschl_answer = result;
            window.jschl_vc = "a1b2c3d4e5f6";
        </script>
    </body>
    </html>
    "#;
    
    println!("🔍 Detecting challenge type...");
    let challenge_type = solver.detect_challenge_type(enhanced_cloudflare_html, "https://example.com");
    println!("✅ Detected: {:?}", challenge_type);
    
    if challenge_type == ChallengeType::CloudflareJsChallenge {
        println!("🚀 Solving Cloudflare challenge...");
        let result = solver.solve_challenges(enhanced_cloudflare_html, "https://example.com").await?;
        
        println!("📊 Challenge Result:");
        println!("  - Solved: {}", result.solved);
        println!("  - Solve time: {:?}", result.solve_time);
        println!("  - Challenge type: {:?}", result.challenge_type);
        println!("  - Solution data keys: {:?}", result.solution_data.keys().collect::<Vec<_>>());
        
        if result.solved {
            if let Some(clearance) = result.solution_data.get("cf_clearance") {
                println!("  - CF Clearance: {}", clearance);
            }
            if let Some(ray) = result.solution_data.get("cf_ray") {
                println!("  - CF Ray: {}", ray);
            }
            println!("🎉 Enhanced Cloudflare challenge solver working correctly!");
        } else {
            println!("❌ Challenge not solved");
        }
    } else {
        println!("❌ Challenge type not detected correctly");
    }
    
    Ok(())
}