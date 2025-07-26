// Example usage of the tmux-agent-proxy library
use aany_tmux::TmuxProxy;
use std::thread;
use std::time::Duration;

fn main() {
    println!("TMux Agent Proxy Example");
    println!("========================");
    
    // Create a proxy for session "test", window 0, pane 0
    let proxy = TmuxProxy::new("test".to_string(), 0, 0);
    
    // Check if the session exists
    if !proxy.exists() {
        println!("❌ Session 'test:0.0' doesn't exist!");
        println!("Create it with: tmux new-session -d -s test");
        return;
    }
    
    println!("✅ Connected to tmux session 'test:0.0'");
    
    // Example 1: Send a simple command
    println!("\n1. Sending 'echo Hello from Rust!'");
    match proxy.send_line("echo Hello from Rust!") {
        Ok(_) => println!("   Command sent successfully"),
        Err(e) => println!("   Error: {}", e),
    }
    
    // Wait a bit for the command to execute
    thread::sleep(Duration::from_millis(100));
    
    // Example 2: Capture the pane content
    println!("\n2. Capturing pane content:");
    match proxy.capture_pane() {
        Ok(content) => {
            println!("   --- Pane Content ---");
            for line in content.lines().take(10) {
                println!("   {}", line);
            }
            if content.lines().count() > 10 {
                println!("   ... (truncated)");
            }
        }
        Err(e) => println!("   Error capturing pane: {}", e),
    }
    
    // Example 3: Send multiple commands
    println!("\n3. Sending multiple commands:");
    let commands = vec![
        "pwd",
        "date",
        "echo 'Agent proxy is working!'",
    ];
    
    for cmd in commands {
        println!("   Sending: {}", cmd);
        if let Err(e) = proxy.send_line(cmd) {
            println!("   Error: {}", e);
        }
        thread::sleep(Duration::from_millis(50));
    }
    
    // // Example 4: Clear the pane
    // println!("\n4. Clearing the pane (Ctrl+L)");
    // match proxy.clear() {
    //     Ok(_) => println!("   Pane cleared"),
    //     Err(e) => println!("   Error: {}", e),
    // }
}