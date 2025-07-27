use aany_tmux::TmuxProxy;
use std::thread;
use std::time::{Duration, Instant};

fn type_slowly(proxy: &TmuxProxy, text: &str, delay_ms: u64) -> std::io::Result<()> {
    for ch in text.chars() {
        proxy.send_keys(&ch.to_string())?;
        thread::sleep(Duration::from_millis(delay_ms));
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Starting typing simulation...");
    
    // Connect to tmux session
    let proxy = TmuxProxy::new("typing-test".to_string(), 0, 0);
    
    // Clear the pane
    proxy.clear()?;
    thread::sleep(Duration::from_millis(500));
    
    // Simulate typing with different speeds
    println!("📝 Phase 1: Slow typing (like thinking)");
    type_slowly(&proxy, "# Agent Anywhere Typing Demo", 100)?;
    proxy.send_keys("Enter")?;
    thread::sleep(Duration::from_millis(1000));
    
    println!("📝 Phase 2: Normal typing");
    type_slowly(&proxy, "echo 'Hello from Agent Anywhere!'", 50)?;
    proxy.send_keys("Enter")?;
    thread::sleep(Duration::from_millis(1000));
    
    println!("📝 Phase 3: Fast typing (like copy-paste)");
    type_slowly(&proxy, "date && uname -a", 20)?;
    proxy.send_keys("Enter")?;
    thread::sleep(Duration::from_millis(1000));
    
    // Capture what happened
    let start = Instant::now();
    
    println!("\n🚀 Running some commands...");
    
    // Type and execute commands
    let commands = vec![
        ("ls -la | head -5", "List files"),
        ("ps aux | grep tmux | head -3", "Show tmux processes"),
        ("echo $SHELL", "Show current shell"),
        ("echo 'Time: '$(date +%H:%M:%S)", "Show current time"),
    ];
    
    for (cmd, desc) in commands {
        println!("   {} - {}", desc, cmd);
        
        // Simulate realistic typing
        type_slowly(&proxy, cmd, 30)?;
        proxy.send_keys("Enter")?;
        
        // Wait for command to complete
        thread::sleep(Duration::from_millis(500));
        
        // Capture output
        let output = proxy.capture_last_lines(10)?;
        let lines: Vec<&str> = output.lines()
            .filter(|line| !line.is_empty())
            .collect();
        
        if lines.len() > 2 {
            println!("   Output preview: {}", lines[lines.len() - 2]);
        }
    }
    
    // Simulate making a typo and correcting it
    println!("\n✏️  Simulating typo correction...");
    type_slowly(&proxy, "ehco 'Oops, typo!'", 50)?;
    thread::sleep(Duration::from_millis(500));
    
    // Delete back to fix typo (simulate Ctrl+W to delete word)
    proxy.send_keys("C-w")?;
    thread::sleep(Duration::from_millis(300));
    proxy.send_keys("C-w")?;
    thread::sleep(Duration::from_millis(300));
    
    type_slowly(&proxy, "echo 'Fixed the typo!'", 50)?;
    proxy.send_keys("Enter")?;
    thread::sleep(Duration::from_millis(1000));
    
    // Final capture
    println!("\n📸 Capturing final pane content...");
    let final_content = proxy.capture_pane()?;
    
    let elapsed = start.elapsed();
    println!("\n✅ Simulation complete!");
    println!("   Total time: {:.2} seconds", elapsed.as_secs_f64());
    println!("   Characters typed: ~{}", 
        commands.iter().map(|(cmd, _)| cmd.len()).sum::<usize>() + 100);
    
    // Show last few lines of output
    println!("\n📄 Last few lines from tmux session:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let last_lines: Vec<&str> = final_content.lines()
        .rev()
        .take(15)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    
    for line in last_lines {
        if !line.trim().is_empty() {
            println!("{}", line);
        }
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    Ok(())
}
