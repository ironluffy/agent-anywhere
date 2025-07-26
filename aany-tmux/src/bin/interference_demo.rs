use aany_tmux::{SafeTmuxProxy, TmuxSession, InterferenceEvent};
use std::time::Duration;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Human Interference Detection Demo");
    println!("=================================\n");
    
    // Create test session
    let session = "interference-test";
    if !TmuxSession::exists(session) {
        TmuxSession::create(session, true)?;
    }
    
    // Create safe proxy with interference detection
    println!("1. Setting up interference detection...");
    let proxy = SafeTmuxProxy::new(session.to_string(), 0, 0)?;
    let mut detector = proxy.with_interference_detection()?;
    
    // Display warning message
    detector.display_lock_message()?;
    thread::sleep(Duration::from_millis(500));
    
    // Set up interference callbacks
    detector.on_interference(|event| {
        match event {
            InterferenceEvent::PaneCountChanged { before, after } => {
                println!("⚠️  INTERFERENCE: Pane count changed from {} to {}", before, after);
            }
            InterferenceEvent::ActivePaneChanged { before, after } => {
                println!("⚠️  INTERFERENCE: Active pane switched from {} to {}", before, after);
            }
            InterferenceEvent::UnexpectedInput { content_snippet } => {
                println!("⚠️  INTERFERENCE: Unexpected input detected: '{}'", content_snippet);
            }
            InterferenceEvent::PaneKilled { pane } => {
                println!("⚠️  INTERFERENCE: Pane {} was killed!", pane);
            }
            InterferenceEvent::HumanAttached { client_info, read_only } => {
                if *read_only {
                    println!("👁️  MONITORING: Read-only attachment: {}", client_info);
                } else {
                    println!("⚠️  INTERFERENCE: Interactive attachment: {}", client_info);
                }
            }
            _ => {
                println!("⚠️  INTERFERENCE: {:?}", event);
            }
        }
    });
    
    println!("\n2. Monitoring for interference...");
    println!("   Try these actions in another terminal:");
    println!("   - tmux attach -t {}", session);
    println!("   - Split the pane (Ctrl+B %)");
    println!("   - Type some commands");
    println!("   - Switch panes (Ctrl+B arrow)");
    println!("   - Kill a pane (Ctrl+B x)");
    println!("\n   Press Ctrl+C to stop monitoring\n");
    
    // Initial agent work
    detector.proxy.send_line("echo 'Agent starting work...'")?;
    let mut counter = 0;
    
    // Monitor loop
    loop {
        // Check for interference
        let events = detector.check_interference()?;
        
        if !events.is_empty() {
            println!("\n🚨 {} interference event(s) detected!", events.len());
            
            // Attempt recovery
            println!("🔧 Attempting recovery...");
            let report = detector.recover()?;
            println!("   {}", report.summary());
            
            // Re-display warning
            detector.display_lock_message()?;
        }
        
        // Simulate agent work
        if counter % 10 == 0 {
            detector.proxy.send_line(&format!("echo 'Agent working... ({})'", counter))?;
        }
        
        counter += 1;
        thread::sleep(Duration::from_millis(500));
        
        // Check if we should exit
        if counter > 120 {  // Run for about 60 seconds
            break;
        }
    }
    
    println!("\n3. Demo completed!");
    println!("   Run 'tmux kill-session -t {}' to clean up", session);
    
    Ok(())
}