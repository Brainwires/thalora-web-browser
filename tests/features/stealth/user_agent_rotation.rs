use thalora::HeadlessWebBrowser;
use std::time::Instant;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_user_agent_rotation() {
    let browser = HeadlessWebBrowser::new();
    
    // Generate multiple user agents and verify they're different
    let user_agents: Vec<String> = (0..10)
        .map(|_| browser.get_random_user_agent())
        .collect();
    
    // Should have some variety in user agents (not all the same)
    let unique_agents: std::collections::HashSet<&String> = user_agents.iter().collect();
    assert!(unique_agents.len() > 1, "User agents should vary");
    
    // All should be realistic browser user agents
    for agent in &user_agents {
        assert!(
            agent.contains("Chrome") || agent.contains("Firefox") || 
            agent.contains("Safari") || agent.contains("Edge"),
            "User agent should be realistic: {}", agent
        );
    }
}
