use thalora::engine::EngineConfig;
use thalora::gui::{TabManager, Tab};

/// Test creating a new TabManager
#[tokio::test]
async fn test_create_tab_manager() {
    let config = EngineConfig::default();
    let result = TabManager::new(config).await;
    assert!(result.is_ok());
}

/// Test creating a new tab
#[tokio::test]
async fn test_create_tab() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    let tab_id = tab_manager.create_tab("about:blank".to_string()).await.unwrap();

    assert!(tab_id > 0);
}

/// Test creating multiple tabs
#[tokio::test]
async fn test_create_multiple_tabs() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    let tab_id1 = tab_manager.create_tab("about:blank".to_string()).await.unwrap();
    let tab_id2 = tab_manager.create_tab("about:blank".to_string()).await.unwrap();
    let tab_id3 = tab_manager.create_tab("about:blank".to_string()).await.unwrap();

    assert_ne!(tab_id1, tab_id2);
    assert_ne!(tab_id2, tab_id3);
    assert_ne!(tab_id1, tab_id3);

    assert_eq!(tab_manager.tab_count(), 3);
}

/// Test setting active tab
#[tokio::test]
async fn test_set_active_tab() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    let tab_id = tab_manager.create_tab("about:blank".to_string()).await.unwrap();

    let result = tab_manager.set_active_tab(tab_id);
    assert!(result.is_ok());

    assert_eq!(tab_manager.get_active_tab_id(), Some(tab_id));
}

/// Test setting non-existent tab as active
#[tokio::test]
async fn test_set_invalid_active_tab() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    // Try to set a non-existent tab as active
    let result = tab_manager.set_active_tab(999);
    assert!(result.is_err());
}

/// Test getting active tab
#[tokio::test]
async fn test_get_active_tab() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    let tab_id = tab_manager.create_tab("https://example.com".to_string()).await.unwrap();
    tab_manager.set_active_tab(tab_id).unwrap();

    let active_tab = tab_manager.get_active_tab();
    assert!(active_tab.is_some());

    let tab = active_tab.unwrap();
    assert_eq!(tab.id(), tab_id);
}

/// Test closing a tab
#[tokio::test]
async fn test_close_tab() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    let tab_id = tab_manager.create_tab("about:blank".to_string()).await.unwrap();

    assert_eq!(tab_manager.tab_count(), 1);

    let result = tab_manager.close_tab(tab_id);
    assert!(result.is_ok());

    assert_eq!(tab_manager.tab_count(), 0);
}

/// Test closing non-existent tab
#[tokio::test]
async fn test_close_nonexistent_tab() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    let result = tab_manager.close_tab(999);
    assert!(result.is_err());
}

/// Test closing active tab switches to another tab
#[tokio::test]
async fn test_close_active_tab_switches() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    let tab_id1 = tab_manager.create_tab("about:blank".to_string()).await.unwrap();
    let tab_id2 = tab_manager.create_tab("about:blank".to_string()).await.unwrap();

    tab_manager.set_active_tab(tab_id1).unwrap();

    // Close active tab
    tab_manager.close_tab(tab_id1).unwrap();

    // Should automatically switch to remaining tab
    assert_eq!(tab_manager.get_active_tab_id(), Some(tab_id2));
}

/// Test tab count
#[tokio::test]
async fn test_tab_count() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    assert_eq!(tab_manager.tab_count(), 0);

    tab_manager.create_tab("about:blank".to_string()).await.unwrap();
    assert_eq!(tab_manager.tab_count(), 1);

    tab_manager.create_tab("about:blank".to_string()).await.unwrap();
    assert_eq!(tab_manager.tab_count(), 2);

    tab_manager.create_tab("about:blank".to_string()).await.unwrap();
    assert_eq!(tab_manager.tab_count(), 3);
}

/// Test getting tab by ID
#[tokio::test]
async fn test_get_tab_by_id() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    let tab_id = tab_manager.create_tab("https://example.com".to_string()).await.unwrap();

    let tab = tab_manager.get_tab(tab_id);
    assert!(tab.is_some());
    assert_eq!(tab.unwrap().id(), tab_id);
}

/// Test getting non-existent tab
#[tokio::test]
async fn test_get_nonexistent_tab() {
    let config = EngineConfig::default();
    let tab_manager = TabManager::new(config).await.unwrap();

    let tab = tab_manager.get_tab(999);
    assert!(tab.is_none());
}

/// Test tab initial state
#[tokio::test]
async fn test_tab_initial_state() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    let tab_id = tab_manager.create_tab("about:blank".to_string()).await.unwrap();
    let tab = tab_manager.get_tab(tab_id).unwrap();

    // Initial state checks
    assert_eq!(tab.url(), "about:blank");
    assert_eq!(tab.is_loading(), false);
    assert_eq!(tab.can_go_back(), false);
    assert_eq!(tab.can_go_forward(), false);
}

/// Test switching between tabs
#[tokio::test]
async fn test_switch_tabs() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    let tab_id1 = tab_manager.create_tab("about:blank".to_string()).await.unwrap();
    let tab_id2 = tab_manager.create_tab("https://example.com".to_string()).await.unwrap();

    // Set first tab as active
    tab_manager.set_active_tab(tab_id1).unwrap();
    assert_eq!(tab_manager.get_active_tab_id(), Some(tab_id1));

    // Switch to second tab
    tab_manager.set_active_tab(tab_id2).unwrap();
    assert_eq!(tab_manager.get_active_tab_id(), Some(tab_id2));

    // Switch back to first tab
    tab_manager.set_active_tab(tab_id1).unwrap();
    assert_eq!(tab_manager.get_active_tab_id(), Some(tab_id1));
}

/// Test closing all tabs
#[tokio::test]
async fn test_close_all_tabs() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    let tab_id1 = tab_manager.create_tab("about:blank".to_string()).await.unwrap();
    let tab_id2 = tab_manager.create_tab("about:blank".to_string()).await.unwrap();
    let tab_id3 = tab_manager.create_tab("about:blank".to_string()).await.unwrap();

    assert_eq!(tab_manager.tab_count(), 3);

    tab_manager.close_tab(tab_id1).unwrap();
    tab_manager.close_tab(tab_id2).unwrap();
    tab_manager.close_tab(tab_id3).unwrap();

    assert_eq!(tab_manager.tab_count(), 0);
    assert_eq!(tab_manager.get_active_tab_id(), None);
}

/// Test no active tab initially
#[tokio::test]
async fn test_no_active_tab_initially() {
    let config = EngineConfig::default();
    let tab_manager = TabManager::new(config).await.unwrap();

    assert_eq!(tab_manager.get_active_tab_id(), None);
    assert!(tab_manager.get_active_tab().is_none());
}

/// Test tab IDs are sequential
#[tokio::test]
async fn test_tab_ids_sequential() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    let tab_id1 = tab_manager.create_tab("about:blank".to_string()).await.unwrap();
    let tab_id2 = tab_manager.create_tab("about:blank".to_string()).await.unwrap();
    let tab_id3 = tab_manager.create_tab("about:blank".to_string()).await.unwrap();

    // IDs should be sequential (or at least incrementing)
    assert!(tab_id2 > tab_id1);
    assert!(tab_id3 > tab_id2);
}

/// Test tab with different engine configurations
#[tokio::test]
async fn test_tab_with_different_engines() {
    use thalora::engine::EngineType;

    // Test with Boa engine
    let config_boa = EngineConfig {
        engine_type: EngineType::Boa,
    };
    let mut tab_manager_boa = TabManager::new(config_boa).await.unwrap();
    let tab_id_boa = tab_manager_boa.create_tab("about:blank".to_string()).await.unwrap();
    assert!(tab_id_boa > 0);

    // Test with V8 engine
    let config_v8 = EngineConfig {
        engine_type: EngineType::V8,
    };
    let mut tab_manager_v8 = TabManager::new(config_v8).await.unwrap();
    let tab_id_v8 = tab_manager_v8.create_tab("about:blank".to_string()).await.unwrap();
    assert!(tab_id_v8 > 0);
}

/// Test maximum reasonable number of tabs
#[tokio::test]
async fn test_many_tabs() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    // Create 10 tabs
    for i in 0..10 {
        let url = format!("https://example.com/page{}", i);
        let result = tab_manager.create_tab(url).await;
        assert!(result.is_ok());
    }

    assert_eq!(tab_manager.tab_count(), 10);
}

/// Test tab list retrieval
#[tokio::test]
async fn test_get_all_tabs() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    let tab_id1 = tab_manager.create_tab("https://example1.com".to_string()).await.unwrap();
    let tab_id2 = tab_manager.create_tab("https://example2.com".to_string()).await.unwrap();
    let tab_id3 = tab_manager.create_tab("https://example3.com".to_string()).await.unwrap();

    let all_tabs = tab_manager.get_all_tabs();

    assert_eq!(all_tabs.len(), 3);

    let tab_ids: Vec<u32> = all_tabs.iter().map(|t| t.id()).collect();
    assert!(tab_ids.contains(&tab_id1));
    assert!(tab_ids.contains(&tab_id2));
    assert!(tab_ids.contains(&tab_id3));
}

/// Test tab ordering preserved
#[tokio::test]
async fn test_tab_order_preserved() {
    let config = EngineConfig::default();
    let mut tab_manager = TabManager::new(config).await.unwrap();

    let tab_id1 = tab_manager.create_tab("https://first.com".to_string()).await.unwrap();
    let tab_id2 = tab_manager.create_tab("https://second.com".to_string()).await.unwrap();
    let tab_id3 = tab_manager.create_tab("https://third.com".to_string()).await.unwrap();

    let all_tabs = tab_manager.get_all_tabs();

    // Verify order is preserved
    assert_eq!(all_tabs[0].id(), tab_id1);
    assert_eq!(all_tabs[1].id(), tab_id2);
    assert_eq!(all_tabs[2].id(), tab_id3);
}
