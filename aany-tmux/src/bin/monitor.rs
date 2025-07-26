use aany_tmux::{SafeTmuxProxy, TmuxSession, TmuxProxy};
use std::time::Duration;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n👁️  TMUX SESSION MONITOR 👁️");
    println!("==========================\n");
    
    // Create a demo session with multiple scenarios
    let session = "monitor-demo";
    if TmuxSession::exists(session) {
        TmuxSession::kill(session)?;
    }
    TmuxSession::create(session, true)?;
    
    println!("📺 Session created: {}", session);
    println!("\n🎯 ATTACH NOW: tmux attach -t {}", session);
    println!("\n⏱️  Starting in 5 seconds...\n");
    thread::sleep(Duration::from_secs(5));
    
    let proxy = SafeTmuxProxy::new(session.to_string(), 0, 0)?;
    
    // Scenario 1: Normal operation
    println!("📍 Scenario 1: Normal Agent Operation");
    proxy.send_line("clear")?;
    proxy.send_line("echo '===== SCENARIO 1: NORMAL OPERATION ====='")?;
    proxy.send_line("echo ''")?;
    
    for i in 1..=5 {
        proxy.send_line(&format!("echo 'Agent working... step {}/5'", i))?;
        thread::sleep(Duration::from_millis(500));
    }
    
    proxy.send_line("echo '✅ Normal operation complete'")?;
    thread::sleep(Duration::from_secs(2));
    
    // Scenario 2: Pane split warning
    println!("\n📍 Scenario 2: Pane Split Protection");
    proxy.send_line("clear")?;
    proxy.send_line("echo '===== SCENARIO 2: PANE SPLIT PROTECTION ====='")?;
    proxy.send_line("echo ''")?;
    proxy.send_line("echo '🛡️  Agent attempts to split pane...'")?;
    thread::sleep(Duration::from_secs(1));
    
    match proxy.safe_split_pane(true, false) {
        Ok(_) => proxy.send_line("echo '❌ Split succeeded (unexpected)'")?,
        Err(_) => proxy.send_line("echo '✅ Split BLOCKED by safety system!'")?
    };
    
    proxy.send_line("echo ''")?;
    proxy.send_line("echo 'This prevents accidental pane proliferation'")?;
    thread::sleep(Duration::from_secs(3));
    
    // Scenario 3: Forced split and recovery
    println!("\n📍 Scenario 3: Split Recovery");
    proxy.send_line("clear")?;
    proxy.send_line("echo '===== SCENARIO 3: SPLIT RECOVERY ====='")?;
    proxy.send_line("echo ''")?;
    proxy.send_line("echo '⚠️  Forcing a pane split...'")?;
    thread::sleep(Duration::from_secs(1));
    
    proxy.safe_split_pane(true, true)?;
    proxy.send_line("echo '📊 Pane was split!'")?;
    
    // Show both panes
    proxy.send_line("echo 'This is pane 0'")?;
    
    // Send to other pane
    TmuxProxy::new(session.to_string(), 0, 1).send_line("echo 'This is pane 1 (unwanted)'")?;
    
    thread::sleep(Duration::from_secs(2));
    
    proxy.send_line("echo ''")?;
    proxy.send_line("echo '🧹 Cleaning up extra panes...'")?;
    thread::sleep(Duration::from_secs(1));
    
    let cleaned = proxy.cleanup_extra_panes()?;
    proxy.send_line(&format!("echo '✅ Cleaned {} pane(s)'", cleaned))?;
    proxy.send_line("echo 'Back to single pane!'")?;
    thread::sleep(Duration::from_secs(3));
    
    // Scenario 4: Health monitoring
    println!("\n📍 Scenario 4: Health Monitoring");
    proxy.send_line("clear")?;
    proxy.send_line("echo '===== SCENARIO 4: HEALTH MONITORING ====='")?;
    proxy.send_line("echo ''")?;
    
    let health = proxy.get_pane_health()?;
    proxy.send_line(&format!("echo '📊 Pane count: {}'"  , health.pane_count))?;
    proxy.send_line(&format!("echo '📐 Dimensions: {}x{}'", health.dimensions.0, health.dimensions.1))?;
    proxy.send_line(&format!("echo '🏥 Status: {:?}'", health.status))?;
    proxy.send_line("echo ''")?;
    
    match health.status {
        aany_tmux::HealthStatus::Healthy => {
            proxy.send_line("echo '✅ System is HEALTHY'")?;
        }
        _ => {
            proxy.send_line("echo '⚠️  Issues detected!'")?;
        }
    }
    thread::sleep(Duration::from_secs(3));
    
    // Scenario 5: Interference warning
    println!("\n📍 Scenario 5: Interference Warning");
    proxy.send_line("clear")?;
    
    let detector = proxy.with_interference_detection()?;
    detector.display_lock_message()?;
    thread::sleep(Duration::from_secs(3));
    
    // Final message
    detector.proxy.send_line("echo ''")?;
    detector.proxy.send_line("echo '🏁 DEMO COMPLETE!'")?;
    detector.proxy.send_line("echo ''")?;
    detector.proxy.send_line("echo 'This demonstrated:'")?;
    detector.proxy.send_line("echo '  ✅ Normal agent operation'")?;
    detector.proxy.send_line("echo '  ✅ Pane split protection'")?;
    detector.proxy.send_line("echo '  ✅ Automatic recovery'")?;
    detector.proxy.send_line("echo '  ✅ Health monitoring'")?;
    detector.proxy.send_line("echo '  ✅ Interference warnings'")?;
    
    println!("\n✅ All scenarios complete!");
    println!("📝 The agent demonstrated protection and recovery mechanisms");
    println!("\n🧹 To clean up: tmux kill-session -t {}", session);
    
    Ok(())
}