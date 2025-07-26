use aany_tmux::{SafeTmuxProxy, TmuxSession};
use std::time::Duration;
use std::thread;
use std::io::{self, Write};

fn pause_for_user(message: &str) {
    println!("\n{}", message);
    println!("Press ENTER to continue...");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut String::new()).unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎓 GUIDED INTERFERENCE DEMO 🎓");
    println!("==============================\n");
    
    // Setup
    let session = "guided-demo";
    if TmuxSession::exists(session) {
        TmuxSession::kill(session)?;
    }
    TmuxSession::create(session, true)?;
    
    println!("📺 Created tmux session: {}", session);
    println!("\n📋 This demo will show you exactly what happens when:");
    println!("   1. An agent controls a tmux pane");
    println!("   2. A human interferes");
    println!("   3. The agent detects and recovers\n");
    
    pause_for_user("First, let's see normal agent operation");
    
    // Normal operation
    let proxy = SafeTmuxProxy::new(session.to_string(), 0, 0)?;
    
    println!("\n1️⃣ NORMAL AGENT OPERATION");
    println!("   The agent is sending commands...\n");
    
    proxy.send_line("echo '🤖 Agent: Starting automated tasks...'")?;
    thread::sleep(Duration::from_millis(500));
    
    for i in 1..=3 {
        proxy.send_line(&format!("echo '   Task {}: Processing data...'", i))?;
        thread::sleep(Duration::from_millis(300));
    }
    
    proxy.send_line("echo '✅ Agent: Tasks completed!'")?;
    
    println!("   ✅ Agent working normally");
    
    // Demonstrate pane safety
    pause_for_user("\n2️⃣ PANE SAFETY: Let's try to split a pane");
    
    println!("   Attempting to split pane without force...\n");
    match proxy.safe_split_pane(true, false) {
        Ok(_) => println!("   ❌ Unexpected: Split succeeded"),
        Err(e) => println!("   ✅ Safety system blocked it: {}", e),
    }
    
    pause_for_user("\n   Now let's force a split to see what happens");
    
    proxy.safe_split_pane(true, true)?;
    println!("   ⚠️  Pane was split (forced)");
    
    // Check health
    let health = proxy.get_pane_health()?;
    println!("\n   Health check:");
    health.report();
    
    // Show the warning when creating new proxy
    println!("\n   Creating new proxy to see warning...");
    let proxy2 = SafeTmuxProxy::new(session.to_string(), 0, 0)?;
    
    pause_for_user("\n3️⃣ RECOVERY: Let's clean up the extra pane");
    
    let cleaned = proxy2.cleanup_extra_panes()?;
    println!("   🧹 Cleaned {} pane(s)", cleaned);
    
    let health = proxy2.get_pane_health()?;
    println!("\n   Health after cleanup:");
    health.report();
    
    // Interference detection demo
    pause_for_user("\n4️⃣ INTERFERENCE DETECTION: Let's simulate human typing");
    
    let mut detector = proxy2.with_interference_detection()?;
    
    // Display lock message
    println!("\n   Displaying lock message in tmux...");
    detector.display_lock_message()?;
    thread::sleep(Duration::from_secs(1));
    
    // Initial state
    detector.check_interference()?;
    
    // Simulate human typing
    println!("\n   Simulating human typing 'hello world'...");
    detector.proxy.send_line("hello world")?;
    thread::sleep(Duration::from_millis(500));
    
    // Check for interference
    let events = detector.check_interference()?;
    if !events.is_empty() {
        println!("   🚨 Interference detected!");
        for event in &events {
            println!("      {:?}", event);
        }
    }
    
    // Window management demo
    pause_for_user("\n5️⃣ RECOMMENDED APPROACH: Using separate windows");
    
    println!("\n   Creating a new window for parallel work...");
    let (window_idx, worker_proxy) = detector.proxy.create_window(Some("worker"))?;
    println!("   ✅ Created window {} named 'worker'", window_idx);
    
    // Send commands to both
    detector.proxy.send_line("echo '🤖 Main: I work in window 0'")?;
    worker_proxy.send_line("echo '🤖 Worker: I work in window 1'")?;
    
    println!("\n   Both agents working in separate windows:");
    println!("   - No pane size issues");
    println!("   - No interference between agents");
    println!("   - Easy to manage");
    
    // Summary
    println!("\n📊 DEMO SUMMARY:");
    println!("   ✅ Safety controls prevent accidental pane splits");
    println!("   ✅ Health monitoring tracks pane status");
    println!("   ✅ Automatic cleanup removes extra panes");
    println!("   ✅ Interference detection catches human input");
    println!("   ✅ Window-based approach is recommended");
    
    println!("\n🎯 TO SEE IT IN ACTION:");
    println!("   1. Open new terminal");
    println!("   2. Run: tmux attach -t {}", session);
    println!("   3. Navigate windows with: Ctrl+B then 0/1");
    println!("   4. Try interfering and watch the agent respond!");
    
    println!("\n🧹 When done: tmux kill-session -t {}", session);
    
    Ok(())
}