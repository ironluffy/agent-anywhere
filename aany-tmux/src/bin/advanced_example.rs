use aany_tmux::{TmuxProxy, TmuxSession};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Advanced TMux Agent Proxy Example");
    println!("=================================\n");
    
    // List existing sessions
    println!("Current tmux sessions:");
    let sessions = TmuxSession::list()?;
    for session in &sessions {
        println!("  - {}", session);
    }
    
    // Create a test session if it doesn't exist
    let session_name = "agent-test";
    if !TmuxSession::exists(session_name) {
        println!("\nCreating new session: {}", session_name);
        TmuxSession::create(session_name, true)?;
    }
    
    // Create a proxy for the main pane
    let proxy = TmuxProxy::new(session_name.to_string(), 0, 0);
    
    // Example 1: Execute command and wait for result
    println!("\n1. Executing command with wait:");
    match proxy.execute_and_wait("echo 'Hello from advanced example!'", "$", 5000) {
        Ok(output) => println!("   Output: {}", output.trim()),
        Err(e) => println!("   Error: {}", e),
    }
    
    // Example 2: Get and display pane size
    println!("\n2. Pane dimensions:");
    match proxy.get_size() {
        Ok((width, height)) => println!("   Width: {}, Height: {}", width, height),
        Err(e) => println!("   Error: {}", e),
    }
    
    // Example 3: Create a new window
    println!("\n3. Creating new window:");
    let window_index = TmuxSession::new_window(session_name, Some("worker"))?;
    println!("   Created window {} named 'worker'", window_index);
    
    // Example 4: Split pane
    println!("\n4. Splitting pane vertically:");
    let new_pane = TmuxSession::split_pane(session_name, 0, 0, true)?;
    println!("   Created pane {}", new_pane);
    
    // Create proxy for the new pane
    let split_proxy = TmuxProxy::new(session_name.to_string(), 0, new_pane);
    split_proxy.send_line("echo 'This is the split pane!'")?;
    
    // Example 5: Wait for specific text
    println!("\n5. Waiting for text in original pane:");
    proxy.send_line("sleep 1 && echo 'READY'")?;
    
    if proxy.wait_for_text("READY", 3000)? {
        println!("   Found 'READY' in output!");
    } else {
        println!("   Timeout waiting for 'READY'");
    }
    
    // Example 6: Capture last N lines
    println!("\n6. Last 5 lines from pane:");
    let last_lines = proxy.capture_last_lines(5)?;
    for line in last_lines.lines() {
        println!("   > {}", line);
    }
    
    // Cleanup option
    println!("\n7. Session cleanup:");
    println!("   Run 'tmux kill-session -t {}' to clean up", session_name);
    
    Ok(())
}