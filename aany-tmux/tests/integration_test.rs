use aany_tmux::{TmuxProxy, TmuxSession};

#[test]
fn test_session_lifecycle() {
    let session_name = "test-lifecycle";
    
    // Ensure clean state
    if TmuxSession::exists(session_name) {
        TmuxSession::kill(session_name).unwrap();
    }
    
    // Create session
    assert!(!TmuxSession::exists(session_name));
    TmuxSession::create(session_name, true).unwrap();
    assert!(TmuxSession::exists(session_name));
    
    // List sessions should contain our session
    let sessions = TmuxSession::list().unwrap();
    assert!(sessions.contains(&session_name.to_string()));
    
    // Clean up
    TmuxSession::kill(session_name).unwrap();
    assert!(!TmuxSession::exists(session_name));
}

#[test]
fn test_proxy_operations() {
    let session_name = "test-proxy";
    
    // Setup
    if !TmuxSession::exists(session_name) {
        TmuxSession::create(session_name, true).unwrap();
    }
    
    let proxy = TmuxProxy::new(session_name.to_string(), 0, 0);
    
    // Test send_line
    proxy.send_line("echo 'test output'").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    // Test capture_pane
    let content = proxy.capture_pane().unwrap();
    assert!(content.contains("test output"));
    
    // Test wait_for_text
    proxy.send_line("echo 'MARKER'").unwrap();
    let found = proxy.wait_for_text("MARKER", 1000).unwrap();
    assert!(found);
    
    // Test interrupt
    proxy.send_line("sleep 10").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    proxy.interrupt().unwrap();
    
    // Clean up
    TmuxSession::kill(session_name).unwrap();
}

#[test]
fn test_window_and_pane_management() {
    let session_name = "test-windows";
    
    // Setup
    if !TmuxSession::exists(session_name) {
        TmuxSession::create(session_name, true).unwrap();
    }
    
    // Create new window
    let window_idx = TmuxSession::new_window(session_name, Some("test-window")).unwrap();
    assert!(window_idx > 0);
    
    // Split pane
    let pane_idx = TmuxSession::split_pane(session_name, 0, 0, true).unwrap();
    assert!(pane_idx > 0);
    
    // Test both panes work
    let proxy1 = TmuxProxy::new(session_name.to_string(), 0, 0);
    let proxy2 = TmuxProxy::new(session_name.to_string(), 0, pane_idx);
    
    proxy1.send_line("echo 'pane 1'").unwrap();
    proxy2.send_line("echo 'pane 2'").unwrap();
    
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    let content1 = proxy1.capture_pane().unwrap();
    let content2 = proxy2.capture_pane().unwrap();
    
    assert!(content1.contains("pane 1"));
    assert!(content2.contains("pane 2"));
    
    // Clean up
    TmuxSession::kill(session_name).unwrap();
}

#[test]
fn test_execute_and_wait() {
    let session_name = "test-execute";
    
    // Setup
    if !TmuxSession::exists(session_name) {
        TmuxSession::create(session_name, true).unwrap();
    }
    
    let proxy = TmuxProxy::new(session_name.to_string(), 0, 0);
    
    // Execute command and wait for prompt
    let output = proxy.execute_and_wait("echo 'executed'", "$", 2000).unwrap();
    assert!(output.contains("executed"));
    
    // Test timeout - use a command that won't return a prompt quickly
    proxy.send_line("cat").unwrap(); // cat without args waits for input
    std::thread::sleep(std::time::Duration::from_millis(100));
    let timeout_result = proxy.wait_for_text("WILL_NOT_APPEAR", 500);
    assert!(!timeout_result.unwrap()); // Should return Ok(false) on timeout
    
    // Clean up
    proxy.interrupt().unwrap();
    TmuxSession::kill(session_name).unwrap();
}