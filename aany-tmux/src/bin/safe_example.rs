use aany_tmux::{SafeTmuxProxy, SafetyConfig, TmuxSession};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Safe TMux Proxy Example");
    println!("======================\n");
    
    // Create test session
    let session = "safe-test";
    if !TmuxSession::exists(session) {
        TmuxSession::create(session, true)?;
    }
    
    // Example 1: Create safe proxy with default settings
    println!("1. Creating safe proxy with default settings:");
    let safe_proxy = SafeTmuxProxy::new(session.to_string(), 0, 0)?;
    
    // Check health
    let health = safe_proxy.get_pane_health()?;
    health.report();
    
    // Example 2: Try to split pane (will be blocked by default)
    println!("\n2. Attempting to split pane (should be blocked):");
    match safe_proxy.safe_split_pane(true, false) {
        Ok(_) => println!("   Unexpected: Split succeeded"),
        Err(e) => println!("   ✅ Expected: {}", e),
    }
    
    // Example 3: Force split to demonstrate warnings
    println!("\n3. Force splitting pane:");
    let new_pane = safe_proxy.safe_split_pane(true, true)?;
    println!("   Created pane {}", new_pane);
    
    // Create another safe proxy to see the warning
    println!("\n4. Creating new proxy to see multi-pane warning:");
    let _warned_proxy = SafeTmuxProxy::new(session.to_string(), 0, 0)?;
    
    // Check health again
    println!("\n5. Health check after split:");
    let health = safe_proxy.get_pane_health()?;
    health.report();
    
    // Example 4: Clean up extra panes
    println!("\n6. Cleaning up extra panes:");
    let cleaned = safe_proxy.cleanup_extra_panes()?;
    println!("   Cleaned {} panes", cleaned);
    
    // Check health after cleanup
    println!("\n7. Health check after cleanup:");
    let health = safe_proxy.get_pane_health()?;
    health.report();
    
    // Example 5: Using windows instead of panes (recommended)
    println!("\n8. Creating new window (recommended approach):");
    let (window_idx, new_proxy) = safe_proxy.create_window(Some("worker"))?;
    println!("   Created window {} with its own safe proxy", window_idx);
    
    // Send commands to both windows
    safe_proxy.send_line("echo 'Main window'")?;
    new_proxy.send_line("echo 'Worker window'")?;
    
    std::thread::sleep(std::time::Duration::from_millis(200));
    
    println!("\n9. Content from main window:");
    let content = safe_proxy.capture_pane()?;
    println!("   Last line: {}", content.lines().rev().nth(1).unwrap_or(""));
    
    println!("\n10. Content from worker window:");
    let content = new_proxy.capture_pane()?;
    println!("   Last line: {}", content.lines().rev().nth(1).unwrap_or(""));
    
    // Example 6: Custom safety config
    println!("\n11. Creating proxy with custom safety config:");
    let mut config = SafetyConfig::default();
    config.max_panes_per_window = 2;
    config.auto_cleanup_panes = true;
    
    let _custom_proxy = SafeTmuxProxy::with_config(
        session.to_string(),
        0,
        0,
        config
    )?;
    println!("   Created proxy with max 2 panes per window");
    
    // Cleanup
    println!("\n12. Cleanup:");
    println!("   Run 'tmux kill-session -t {}' to clean up", session);
    
    Ok(())
}