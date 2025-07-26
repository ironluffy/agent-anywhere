use aany_tmux::{SafeTmuxProxy, SafetyConfig, TmuxSession, HealthStatus};

#[test]
fn test_safe_proxy_blocks_splits() {
    let session = "test-safe-splits";
    
    // Setup
    if TmuxSession::exists(session) {
        TmuxSession::kill(session).unwrap();
    }
    TmuxSession::create(session, true).unwrap();
    
    let proxy = SafeTmuxProxy::new(session.to_string(), 0, 0).unwrap();
    
    // Should block split without force
    assert!(proxy.safe_split_pane(true, false).is_err());
    
    // Should allow with force
    assert!(proxy.safe_split_pane(true, true).is_ok());
    
    // Cleanup
    TmuxSession::kill(session).unwrap();
}

#[test]
fn test_pane_health_monitoring() {
    let session = "test-health";
    
    // Setup
    if TmuxSession::exists(session) {
        TmuxSession::kill(session).unwrap();
    }
    TmuxSession::create(session, true).unwrap();
    
    let proxy = SafeTmuxProxy::new(session.to_string(), 0, 0).unwrap();
    
    // Check initial health
    let health = proxy.get_pane_health().unwrap();
    assert_eq!(health.pane_count, 1);
    assert_eq!(health.status, HealthStatus::Healthy);
    
    // Split pane
    proxy.safe_split_pane(true, true).unwrap();
    
    // Check health after split
    let health = proxy.get_pane_health().unwrap();
    assert_eq!(health.pane_count, 2);
    assert_eq!(health.status, HealthStatus::Critical);
    
    // Cleanup
    TmuxSession::kill(session).unwrap();
}

#[test]
fn test_pane_cleanup() {
    let session = "test-cleanup";
    
    // Setup
    if TmuxSession::exists(session) {
        TmuxSession::kill(session).unwrap();
    }
    TmuxSession::create(session, true).unwrap();
    
    let proxy = SafeTmuxProxy::new(session.to_string(), 0, 0).unwrap();
    
    // Create one split pane
    proxy.safe_split_pane(true, true).unwrap();
    
    // Verify we have 2 panes
    let health = proxy.get_pane_health().unwrap();
    assert_eq!(health.pane_count, 2);
    
    // Cleanup
    let cleaned = proxy.cleanup_extra_panes().unwrap();
    assert_eq!(cleaned, 1);
    
    // Verify only one pane remains
    let health = proxy.get_pane_health().unwrap();
    assert_eq!(health.pane_count, 1);
    assert_eq!(health.status, HealthStatus::Healthy);
    
    // Cleanup
    TmuxSession::kill(session).unwrap();
}

#[test]
fn test_custom_safety_config() {
    let session = "test-custom-config";
    
    // Setup
    if TmuxSession::exists(session) {
        TmuxSession::kill(session).unwrap();
    }
    TmuxSession::create(session, true).unwrap();
    
    // Create config allowing 2 panes
    let mut config = SafetyConfig::default();
    config.max_panes_per_window = 2;
    config.require_split_confirmation = false;
    
    let proxy = SafeTmuxProxy::with_config(
        session.to_string(),
        0,
        0,
        config
    ).unwrap();
    
    // First split should succeed without force
    assert!(proxy.safe_split_pane(true, false).is_ok());
    
    // Second split should fail (would create 3rd pane)
    assert!(proxy.safe_split_pane(true, false).is_err());
    
    // Cleanup
    TmuxSession::kill(session).unwrap();
}

#[test]
fn test_window_creation() {
    let session = "test-windows";
    
    // Setup
    if TmuxSession::exists(session) {
        TmuxSession::kill(session).unwrap();
    }
    TmuxSession::create(session, true).unwrap();
    
    let proxy = SafeTmuxProxy::new(session.to_string(), 0, 0).unwrap();
    
    // Create new window
    let (window_idx, new_proxy) = proxy.create_window(Some("test-window")).unwrap();
    assert!(window_idx > 0);
    
    // Both proxies should be healthy
    let health1 = proxy.get_pane_health().unwrap();
    let health2 = new_proxy.get_pane_health().unwrap();
    
    assert_eq!(health1.status, HealthStatus::Healthy);
    assert_eq!(health2.status, HealthStatus::Healthy);
    
    // Cleanup
    TmuxSession::kill(session).unwrap();
}