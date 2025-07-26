use aany_tmux::{SafeTmuxProxy, TmuxSession, InterferenceEvent};
use std::time::Duration;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n👁️  READ-ONLY MONITORING DEMO 👁️");
    println!("=================================\n");
    
    // Create test session
    let session = "readonly-demo";
    if TmuxSession::exists(session) {
        TmuxSession::kill(session)?;
    }
    TmuxSession::create(session, true)?;
    
    println!("📺 Session created: {}", session);
    println!("\n🎯 INSTRUCTIONS FOR MONITORING:");
    println!("1. Open a new terminal");
    println!("2. Run: tmux attach -r -t {} (note the -r flag!)", session);
    println!("3. You can watch without interfering\n");
    
    println!("For comparison, try WITHOUT -r flag:");
    println!("   tmux attach -t {} (interactive mode)\n", session);
    
    println!("Starting agent in 3 seconds...\n");
    thread::sleep(Duration::from_secs(3));
    
    // Create safe proxy with interference detection
    let proxy = SafeTmuxProxy::new(session.to_string(), 0, 0)?;
    let mut detector = proxy.with_interference_detection()?;
    
    // Display initial message
    detector.proxy.send_line("clear")?;
    detector.display_lock_message()?;
    detector.proxy.send_line("echo ''")?;
    detector.proxy.send_line("echo '🤖 Agent starting work...'")?;
    detector.proxy.send_line("echo ''")?;
    
    // Track different types of attachments
    let mut readonly_monitors = 0;
    let mut interactive_attachments = 0;
    let mut monitor_welcomed = false;
    
    println!("🚀 Agent running - Detecting attachment types...\n");
    
    // Main loop
    let mut task_counter = 0;
    loop {
        // Check for interference
        let events = detector.check_interference()?;
        
        for event in &events {
            match event {
                InterferenceEvent::HumanAttached { client_info, read_only } => {
                    if *read_only {
                        readonly_monitors += 1;
                        println!("✅ READ-ONLY monitor attached: {}", client_info);
                        detector.proxy.send_line(&format!(
                            "echo '👁️  Welcome monitor! ({})'", 
                            client_info
                        ))?;
                        
                        // Show welcome message once
                        if !monitor_welcomed {
                            thread::sleep(Duration::from_millis(500));
                            detector.display_monitor_welcome()?;
                            monitor_welcomed = true;
                        }
                    } else {
                        interactive_attachments += 1;
                        println!("⚠️  INTERACTIVE attachment detected: {}", client_info);
                        detector.proxy.send_line(&format!(
                            "echo '⚠️  Interactive attachment detected! ({})'", 
                            client_info
                        ))?;
                        detector.proxy.send_line(&format!("echo '⚠️  Consider using: tmux attach -r -t {}'", session))?;
                    }
                }
                InterferenceEvent::PaneCountChanged { before, after } => {
                    println!("🚨 INTERFERENCE: Pane count changed {} → {}", before, after);
                    detector.proxy.send_line("echo '🚨 Pane modification detected!'")?;
                    
                    // Recover
                    let report = detector.recover()?;
                    println!("   Recovery: {}", report.summary());
                }
                InterferenceEvent::UnexpectedInput { content_snippet } => {
                    println!("🚨 INTERFERENCE: Input detected: '{}'", content_snippet);
                    detector.proxy.send_line("echo '🚨 Manual input detected!'")?;
                    detector.proxy.send_line("echo '   Tip: Use -r flag for read-only mode'")?;
                }
                _ => {
                    println!("   Other event: {:?}", event);
                }
            }
        }
        
        // Regular agent work
        if task_counter % 10 == 0 {
            detector.proxy.send_line(&format!(
                "echo '🤖 Task #{} | Monitors: {} | Interactive: {}'",
                task_counter / 10,
                readonly_monitors,
                interactive_attachments
            ))?;
            
            // Show different messages based on attachment type
            if readonly_monitors > 0 && interactive_attachments == 0 {
                detector.proxy.send_line("echo '   ✅ All attachments are read-only - safe!'")?;
            } else if interactive_attachments > 0 {
                detector.proxy.send_line("echo '   ⚠️  Interactive attachments may cause issues'")?;
            }
        }
        
        task_counter += 1;
        thread::sleep(Duration::from_millis(500));
        
        // Run for 60 seconds
        if task_counter > 120 {
            break;
        }
    }
    
    // Summary
    detector.proxy.send_line("echo ''")?;
    detector.proxy.send_line("echo '📊 SESSION SUMMARY:'")?;
    detector.proxy.send_line(&format!(
        "echo '   Read-only monitors: {}'",
        readonly_monitors
    ))?;
    detector.proxy.send_line(&format!(
        "echo '   Interactive attachments: {}'",
        interactive_attachments
    ))?;
    detector.proxy.send_line("echo ''")?;
    detector.proxy.send_line("echo '✅ Read-only monitoring is safe!'")?;
    detector.proxy.send_line("echo '⚠️  Interactive mode can interfere'")?;
    
    println!("\n✅ Demo completed!");
    println!("📊 Summary:");
    println!("   - Read-only monitors: {}", readonly_monitors);
    println!("   - Interactive attachments: {}", interactive_attachments);
    println!("\n🧹 To clean up: tmux kill-session -t {}", session);
    
    Ok(())
}