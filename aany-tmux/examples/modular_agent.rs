// Example of using modular tmux proxy with hub connector

use aany_tmux::{ModularTmuxProxy, ModularProxyConfig};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the modular proxy
    let mut modules_config = HashMap::new();
    
    // Hub connector configuration
    let hub_config = serde_json::json!({
        "enabled": true,
        "hub_url": "localhost:50052",
    });
    modules_config.insert("hub_connector".to_string(), hub_config);
    
    let config = ModularProxyConfig {
        session_name: "test-agent".to_string(),
        window: 0,
        pane: 0,
        agent_id: "test-agent-001".to_string(),
        modules_config,
    };
    
    // Create modular proxy (this will connect to hub if configured)
    println!("Creating modular tmux proxy with hub connection...");
    let mut proxy = ModularTmuxProxy::new(config)?;
    
    // Execute some commands - they will be logged to hub automatically
    println!("Executing commands (will be logged to hub)...");
    
    proxy.send_command("echo 'Hello from modular agent!'")?;
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    proxy.send_command("ls -la")?;
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    proxy.send_command("date")?;
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    // Simulate an error
    proxy.report_error("Simulated error for testing");
    
    // The hub connector will log all these events
    println!("Check the aany-hub dashboard to see the logs!");
    
    Ok(())
}