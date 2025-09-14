use synaptic::{ChallengeSolver, ChallengeType, SolverConfig};
use std::time::Duration;
use tokio;

#[tokio::test]
async fn test_google_challenge_detection() {
    let solver = ChallengeSolver::new();
    
    let google_challenge_html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Google</title>
        <meta content="0;url=/httpservice/retry/enablejs?sei=08vFaJ-rBbzEp84P_u6I6QY" http-equiv="refresh">
    </head>
    <body>
        <div style="display:block">Please click <a href="/httpservice/retry/enablejs?sei=08vFaJ-rBbzEp84P_u6I6QY">here</a> if you are not redirected within a few seconds.</div>
        <script>
        (function(){var sctm=false;(function(){sctm&&google.tick("load","pbsst");}).call(this);})();
        </script>
    </body>
    </html>
    "#;
    
    let challenge_type = solver.detect_challenge_type(google_challenge_html, "https://google.com/search?q=test");
    assert_eq!(challenge_type, ChallengeType::GoogleAntiBot);
}

#[tokio::test]
async fn test_recaptcha_v3_detection() {
    let solver = ChallengeSolver::new();
    
    let recaptcha_html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <script src="https://www.google.com/recaptcha/api.js?render=6LdyC2cUAAAAABbhGvHV7YQO1xYNWm7O2rPqOmJe"></script>
    </head>
    <body>
        <form>
            <button class="g-recaptcha" 
                    data-sitekey="6LdyC2cUAAAAABbhGvHV7YQO1xYNWm7O2rPqOmJe" 
                    data-callback="onSubmit">
                Submit
            </button>
        </form>
        <script>
        grecaptcha.ready(function() {
            grecaptcha.execute('6LdyC2cUAAAAABbhGvHV7YQO1xYNWm7O2rPqOmJe', {action: 'submit'});
        });
        </script>
    </body>
    </html>
    "#;
    
    let challenge_type = solver.detect_challenge_type(recaptcha_html, "https://example.com/form");
    assert_eq!(challenge_type, ChallengeType::GoogleRecaptchaV3);
}

#[tokio::test]
async fn test_cloudflare_challenge_detection() {
    let solver = ChallengeSolver::new();
    
    let cloudflare_html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Checking your browser before accessing example.com.</title>
    </head>
    <body>
        <div class="cf-browser-verification cf-im-under-attack">
            <div class="cf-wrapper">
                <div class="cf-challenge-running">Checking your browser</div>
            </div>
        </div>
        <form class="cf-challenge-form" action="/cdn-cgi/l/chk_captcha" method="get">
            <input type="hidden" name="s" value="abc123"/>
            <input type="hidden" name="jschl_vc" value="def456"/>
        </form>
        <script>
        setTimeout(function(){
            document.getElementById('challenge-form').submit();
        }, 4000);
        </script>
    </body>
    </html>
    "#;
    
    let challenge_type = solver.detect_challenge_type(cloudflare_html, "https://example.com");
    assert_eq!(challenge_type, ChallengeType::CloudflareJsChallenge);
}

#[tokio::test]
async fn test_turnstile_detection() {
    let solver = ChallengeSolver::new();
    
    let turnstile_html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <script src="https://challenges.cloudflare.com/turnstile/v0/api.js"></script>
    </head>
    <body>
        <form>
            <div class="cf-turnstile" 
                 data-sitekey="0x4AAAAAAABkMYinukE_cYqA" 
                 data-callback="onloadTurnstileCallback">
            </div>
            <button type="submit">Submit</button>
        </form>
        <script>
        window.turnstile.render('.cf-turnstile', {
            sitekey: '0x4AAAAAAABkMYinukE_cYqA',
            callback: function(token) {
                console.log('Turnstile token:', token);
            },
        });
        </script>
    </body>
    </html>
    "#;
    
    let challenge_type = solver.detect_challenge_type(turnstile_html, "https://example.com");
    assert_eq!(challenge_type, ChallengeType::CloudflareTurnstile);
}

#[tokio::test]
async fn test_google_challenge_solving() {
    let mut solver = ChallengeSolver::new();
    
    let google_challenge_html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta content="0;url=/httpservice/retry/enablejs?sei=test123" http-equiv="refresh">
    </head>
    <body>
        <div>Please click here if you are not redirected within a few seconds.</div>
        <script>
        (function(){
            var sctm=false;
            (function(){
                sctm&&google.tick("load","pbsst");
            }).call(this);
        })();
        window.challenge_completed = true;
        </script>
    </body>
    </html>
    "#;
    
    let result = solver.solve_challenges(google_challenge_html, "https://google.com/search?q=test").await;
    assert!(result.is_ok());
    
    let challenge_result = result.unwrap();
    assert_eq!(challenge_result.challenge_type, ChallengeType::GoogleAntiBot);
    // Note: In a real test environment, we might not achieve full solving without proper execution
}

#[tokio::test]
async fn test_recaptcha_v3_solving() {
    let mut solver = ChallengeSolver::new();
    
    let recaptcha_html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <script src="https://www.google.com/recaptcha/api.js?render=6LdyC2cUAAAAABbhGvHV7YQO1xYNWm7O2rPqOmJe"></script>
    </head>
    <body>
        <form>
            <div class="g-recaptcha" data-sitekey="6LdyC2cUAAAAABbhGvHV7YQO1xYNWm7O2rPqOmJe"></div>
        </form>
        <script>
            grecaptcha.execute('6LdyC2cUAAAAABbhGvHV7YQO1xYNWm7O2rPqOmJe', {action: 'submit'});
        </script>
    </body>
    </html>
    "#;
    
    let result = solver.solve_challenges(recaptcha_html, "https://example.com/form").await;
    assert!(result.is_ok());
    
    let challenge_result = result.unwrap();
    assert_eq!(challenge_result.challenge_type, ChallengeType::GoogleRecaptchaV3);
    assert!(challenge_result.solved);
    
    // Check that we got a token
    assert!(challenge_result.solution_data.contains_key("recaptcha_token"));
    assert!(challenge_result.solution_data.contains_key("site_key"));
}

#[tokio::test]
async fn test_cloudflare_challenge_solving() {
    let mut solver = ChallengeSolver::new();
    
    let cloudflare_html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Checking your browser</title>
    </head>
    <body>
        <form class="challenge-form" action="/cdn-cgi/l/chk_captcha" method="get">
            <input type="hidden" name="s" value="abc123"/>
            <input type="hidden" name="jschl_vc" value="def456"/>
            <input type="hidden" name="pass" value="1234567890.123-abc123def"/>
        </form>
        <script>
        setTimeout(function(){
            document.querySelector('.challenge-form').submit();
        }, 4000);
        </script>
    </body>
    </html>
    "#;
    
    let result = solver.solve_challenges(cloudflare_html, "https://example.com").await;
    assert!(result.is_ok());
    
    let challenge_result = result.unwrap();
    assert_eq!(challenge_result.challenge_type, ChallengeType::CloudflareJsChallenge);
    assert!(challenge_result.solved);
    
    // Check that we got clearance data
    assert!(challenge_result.solution_data.contains_key("cf_clearance"));
}

#[tokio::test]
async fn test_enhanced_cloudflare_challenge_solving() {
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
    
    let result = solver.solve_challenges(enhanced_cloudflare_html, "https://example.com").await;
    assert!(result.is_ok());
    
    let challenge_result = result.unwrap();
    assert_eq!(challenge_result.challenge_type, ChallengeType::CloudflareJsChallenge);
    assert!(challenge_result.solved);
    
    // Check that we got comprehensive solution data
    assert!(challenge_result.solution_data.contains_key("cf_clearance"));
    assert!(challenge_result.solution_data.contains_key("cf_ray"));
    assert!(challenge_result.solution_data.contains_key("jschl_vc"));
    assert!(challenge_result.solution_data.contains_key("pass"));
    assert!(challenge_result.solution_data.contains_key("s"));
    assert!(challenge_result.solution_data.contains_key("r"));
    
    // Verify clearance cookie format
    let clearance = challenge_result.solution_data.get("cf_clearance").unwrap().as_str().unwrap();
    assert!(clearance.contains("-")); // Should have timestamp separators
    assert!(clearance.len() > 20); // Should be reasonably long
    
    // Verify ray ID format
    let ray_id = challenge_result.solution_data.get("cf_ray").unwrap().as_str().unwrap();
    assert!(ray_id.contains("-"));
    assert!(ray_id.ends_with("DFW")); // Should have datacenter suffix
}

#[tokio::test]
async fn test_solver_configuration() {
    let config = SolverConfig {
        timeout: Duration::from_secs(10),
        enable_caching: false,
        human_delays: false,
        aggressiveness: 10,
    };
    
    let mut solver = ChallengeSolver::with_config(config.clone());
    
    // Test that configuration is applied
    let simple_html = "<html><body>Hello World</body></html>";
    let result = solver.solve_challenges(simple_html, "https://example.com").await;
    
    assert!(result.is_ok());
    let challenge_result = result.unwrap();
    assert_eq!(challenge_result.challenge_type, ChallengeType::Unknown);
    assert!(!challenge_result.solved);
}

#[tokio::test]
async fn test_challenge_caching() {
    let mut solver = ChallengeSolver::new();
    
    let html = r#"
    <html>
    <body>
        <div class="g-recaptcha" data-sitekey="test123"></div>
    </body>
    </html>
    "#;
    
    // First solve
    let result1 = solver.solve_challenges(html, "https://example.com").await;
    assert!(result1.is_ok());
    
    // Second solve should use cache
    let result2 = solver.solve_challenges(html, "https://example.com").await;
    assert!(result2.is_ok());
    
    // Check cache stats
    let cache_stats = solver.get_cache_stats();
    assert!(cache_stats.get("total").unwrap_or(&0) > &0);
}

#[tokio::test]
async fn test_challenge_patterns() {
    let solver = ChallengeSolver::new();
    
    // Test various challenge indicators
    let test_cases = vec![
        ("httpservice/retry/enablejs", ChallengeType::GoogleAntiBot),
        ("grecaptcha.execute", ChallengeType::GoogleRecaptchaV3),
        ("cf-challenge", ChallengeType::CloudflareJsChallenge),
        ("turnstile.render", ChallengeType::CloudflareTurnstile),
        ("bot detected", ChallengeType::Generic),
        ("verify you are human", ChallengeType::Generic),
    ];
    
    for (indicator, expected_type) in test_cases {
        let html = format!("<html><body>{}</body></html>", indicator);
        let detected_type = solver.detect_challenge_type(&html, "https://example.com");
        assert_eq!(detected_type, expected_type, "Failed to detect challenge for indicator: {}", indicator);
    }
}

#[tokio::test]
async fn test_performance_and_timing() {
    let mut solver = ChallengeSolver::new();
    
    let html = r#"
    <html>
    <head>
        <script src="https://www.google.com/recaptcha/api.js"></script>
    </head>
    <body>
        <div class="g-recaptcha" data-sitekey="test123"></div>
        <script>
        // Simulate a complex challenge
        for (let i = 0; i < 1000; i++) {
            Math.random();
        }
        window.challenge_complete = true;
        </script>
    </body>
    </html>
    "#;
    
    let start = std::time::Instant::now();
    let result = solver.solve_challenges(html, "https://example.com").await;
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration < Duration::from_secs(30)); // Should complete within timeout
    
    let challenge_result = result.unwrap();
    assert!(challenge_result.solve_time > Duration::from_millis(0)); // Should have measurable time
}