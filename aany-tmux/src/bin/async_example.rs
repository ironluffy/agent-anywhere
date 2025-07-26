use aany_tmux::{AsyncTmuxProxy, PaneMonitor, TmuxSession};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Async TMux Agent Proxy Example");
    println!("==============================\n");
    
    // Ensure session exists
    let session = "async-test";
    if !TmuxSession::exists(session) {
        TmuxSession::create(session, true)?;
    }
    
    // Example 1: Basic async operations
    println!("1. Basic async operations:");
    let proxy = AsyncTmuxProxy::new(session.to_string(), 0, 0);
    
    proxy.send_line("echo 'Starting async operations...'").await?;
    sleep(Duration::from_millis(100)).await;
    
    let content = proxy.capture_pane().await?;
    println!("   Captured: {}", content.lines().rev().nth(1).unwrap_or(""));
    
    // Example 2: Wait for text with timeout
    println!("\n2. Waiting for text (async):");
    proxy.send_line("sleep 1 && echo 'ASYNC_READY'").await?;
    
    let found = proxy.wait_for_text("ASYNC_READY", 2000).await?;
    println!("   Found text: {}", found);
    
    // Example 3: Run command with streaming callback
    println!("\n3. Command with streaming output:");
    let proxy_clone = AsyncTmuxProxy::new(session.to_string(), 0, 0);
    
    proxy_clone.run_with_callback("for i in 1 2 3; do echo \"Count: $i\"; sleep 0.5; done", |output| {
        print!("   [STREAM] {}", output);
    }).await?;
    
    // Example 4: Concurrent commands in split panes
    println!("\n4. Running concurrent commands:");
    
    // Create split panes
    let pane1 = TmuxSession::split_pane(session, 0, 0, true)?;
    let pane2 = TmuxSession::split_pane(session, 0, 0, true)?;
    
    let proxy0 = Arc::new(AsyncTmuxProxy::new(session.to_string(), 0, 0));
    let proxy1 = Arc::new(AsyncTmuxProxy::new(session.to_string(), 0, pane1));
    let proxy2 = Arc::new(AsyncTmuxProxy::new(session.to_string(), 0, pane2));
    
    let commands = vec![
        "echo 'Pane 0 working...' && sleep 1 && echo 'Pane 0 done!'".to_string(),
        "echo 'Pane 1 working...' && sleep 1 && echo 'Pane 1 done!'".to_string(),
        "echo 'Pane 2 working...' && sleep 1 && echo 'Pane 2 done!'".to_string(),
    ];
    
    let results = AsyncTmuxProxy::run_concurrent(
        vec![proxy0, proxy1, proxy2],
        commands
    ).await;
    
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(output) => {
                let last_line = output.lines().rev().find(|l| l.contains("done")).unwrap_or("");
                println!("   Pane {}: {}", i, last_line);
            }
            Err(e) => println!("   Pane {} error: {}", i, e),
        }
    }
    
    // Example 5: Pane monitoring
    println!("\n5. Starting pane monitor (will capture next 3 events):");
    let monitor_proxy = AsyncTmuxProxy::new(session.to_string(), 0, 0);
    let mut monitor = PaneMonitor::new(monitor_proxy);
    
    monitor.start();
    
    // Send some commands to trigger events
    proxy.send_line("echo 'Event 1'").await?;
    sleep(Duration::from_millis(200)).await;
    proxy.send_line("echo 'Event 2'").await?;
    sleep(Duration::from_millis(200)).await;
    proxy.send_line("echo 'Event 3'").await?;
    
    // Collect events
    let mut events_received = 0;
    while let Some(event) = monitor.receiver().recv().await {
        println!("   [MONITOR] {}", event.trim());
        events_received += 1;
        if events_received >= 3 {
            break;
        }
    }
    
    println!("\n6. Cleanup:");
    println!("   Run 'tmux kill-session -t {}' to clean up", session);
    
    Ok(())
}