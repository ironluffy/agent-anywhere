use aany_tmux::{SafeTmuxProxy, TmuxSession, InterferenceEvent};
use std::time::{Duration, Instant};
use std::thread;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🤖 INTERACTIVE AGENT-HUMAN INTERFERENCE DEMO 🤖");
    println!("================================================\n");
    
    // Create test session
    let session = "interactive-demo";
    if TmuxSession::exists(session) {
        TmuxSession::kill(session)?;
    }
    TmuxSession::create(session, true)?;
    
    println!("📺 Session created: {}", session);
    println!("\n🎯 INSTRUCTIONS:");
    println!("1. Open a new terminal");
    println!("2. Run: tmux attach -t {}", session);
    println!("3. Try these actions:\n");
    
    println!("   ACTION                    | KEYS");
    println!("   --------------------------|------------------");
    println!("   Split pane vertically     | Ctrl+B then %");
    println!("   Split pane horizontally   | Ctrl+B then \"");
    println!("   Switch between panes      | Ctrl+B then arrow");
    println!("   Type some text           | Just type!");
    println!("   Kill current pane        | Ctrl+B then x");
    println!("   Detach from session      | Ctrl+B then d");
    println!("   Resize pane              | Ctrl+B then Ctrl+arrow\n");
    
    println!("Press ENTER when you're attached to the session...");
    io::stdout().flush()?;
    io::stdin().read_line(&mut String::new())?;
    
    // Create safe proxy with interference detection
    let proxy = SafeTmuxProxy::new(session.to_string(), 0, 0)?;
    let mut detector = proxy.with_interference_detection()?;
    
    // Display initial agent message
    detector.proxy.send_line("clear")?;
    detector.proxy.send_line("echo '🤖 AGENT STARTING...'")?;
    detector.proxy.send_line("echo '=================='")?;
    detector.proxy.send_line("echo ''")?;
    detector.proxy.send_line("echo 'This is an automated agent controlling this pane.'")?;
    detector.proxy.send_line("echo 'Try interfering with me!'")?;
    detector.proxy.send_line("echo ''")?;
    thread::sleep(Duration::from_millis(500));
    
    // Agent state
    let mut task_counter = 0;
    let mut last_interference = Instant::now();
    let mut total_interferences = 0;
    
    println!("\n🚀 AGENT RUNNING - Monitoring for interference...\n");
    println!("(Press Ctrl+C to stop)\n");
    
    // Main monitoring loop
    loop {
        // Check for interference
        let events = detector.check_interference()?;
        
        if !events.is_empty() {
            total_interferences += events.len();
            last_interference = Instant::now();
            
            println!("\n🚨 INTERFERENCE DETECTED! 🚨");
            for event in &events {
                match event {
                    InterferenceEvent::PaneCountChanged { before, after } => {
                        println!("   📊 Pane count: {} → {}", before, after);
                        detector.proxy.send_line(&format!(
                            "echo '⚠️  DETECTED: Pane count changed from {} to {}'", 
                            before, after
                        ))?;
                    }
                    InterferenceEvent::ActivePaneChanged { before, after } => {
                        println!("   👁️  Active pane: {} → {}", before, after);
                        detector.proxy.send_line(
                            "echo '⚠️  DETECTED: You switched panes!'"
                        )?;
                    }
                    InterferenceEvent::UnexpectedInput { content_snippet } => {
                        println!("   ⌨️  Unexpected input: '{}'", content_snippet);
                        detector.proxy.send_line(&format!(
                            "echo '⚠️  DETECTED: You typed: {}'", 
                            content_snippet.replace("'", "")
                        ))?;
                    }
                    InterferenceEvent::PaneKilled { pane } => {
                        println!("   💀 Pane {} was killed!", pane);
                        detector.proxy.send_line(&format!(
                            "echo '⚠️  DETECTED: Pane {} was killed!'", 
                            pane
                        ))?;
                    }
                    InterferenceEvent::PaneDimensionsChanged { pane, before, after } => {
                        println!("   📐 Pane {} resized: {:?} → {:?}", pane, before, after);
                        detector.proxy.send_line(&format!(
                            "echo '⚠️  DETECTED: Pane resized from {}x{} to {}x{}'",
                            before.0, before.1, after.0, after.1
                        ))?;
                    }
                    InterferenceEvent::HumanAttached { client_info, read_only } => {
                        if *read_only {
                            println!("   👁️  Monitor attached (read-only): {}", client_info);
                        } else {
                            println!("   👤 Human attached (interactive): {}", client_info);
                        }
                    }
                    _ => {
                        println!("   ❓ Other: {:?}", event);
                    }
                }
            }
            
            // Recovery actions
            println!("\n🔧 RECOVERING...");
            thread::sleep(Duration::from_secs(1));
            
            let report = detector.recover()?;
            println!("   {}", report.summary());
            
            // Visual recovery in tmux
            detector.proxy.send_line("echo ''")?;
            detector.proxy.send_line("echo '🔧 RECOVERY INITIATED:'")?;
            
            if report.panes_cleaned > 0 {
                detector.proxy.send_line(&format!(
                    "echo '   ✅ Cleaned {} extra pane(s)'",
                    report.panes_cleaned
                ))?;
            }
            if report.active_pane_restored {
                detector.proxy.send_line("echo '   ✅ Restored focus to agent pane'")?;
            }
            if report.pane_cleared {
                detector.proxy.send_line("echo '   ✅ Cleared unexpected content'")?;
            }
            
            detector.proxy.send_line("echo ''")?;
            detector.proxy.send_line("echo '✨ AGENT RESUMED CONTROL ✨'")?;
            detector.proxy.send_line("echo ''")?;
            
            thread::sleep(Duration::from_secs(1));
        }
        
        // Regular agent work
        if task_counter % 20 == 0 {  // Every 10 seconds
            let uptime = last_interference.elapsed().as_secs();
            detector.proxy.send_line(&format!(
                "echo '🤖 Agent task #{} | Uptime: {}s | Interferences: {}'",
                task_counter / 20,
                uptime,
                total_interferences
            ))?;
            
            // Simulate some work
            detector.proxy.send_line("echo -n '   Processing: '")?;
            for i in 0..10 {
                thread::sleep(Duration::from_millis(100));
                detector.proxy.send_keys(&format!("{}", i))?;
            }
            detector.proxy.send_line(" ✓")?;
        }
        
        task_counter += 1;
        thread::sleep(Duration::from_millis(500));
        
        // Stop after 2 minutes
        if task_counter > 240 {
            break;
        }
    }
    
    // Cleanup message
    detector.proxy.send_line("echo ''")?;
    detector.proxy.send_line("echo '🏁 DEMO COMPLETED!'")?;
    detector.proxy.send_line(&format!(
        "echo 'Total interferences handled: {}'",
        total_interferences
    ))?;
    
    println!("\n✅ Demo completed!");
    println!("📝 Total interferences handled: {}", total_interferences);
    println!("\n🧹 To clean up: tmux kill-session -t {}", session);
    
    Ok(())
}