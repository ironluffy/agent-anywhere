// Demo of auto-correction module

use aany_tmux::{ModularTmuxProxy, ModularProxyConfig};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Auto-correction Demo");
    println!("======================");
    
    // Configure modules
    let mut modules_config = HashMap::new();
    
    // Enable auto-correct
    let autocorrect_config = serde_json::json!({
        "enabled": true,
        "log_corrections": true,
    });
    modules_config.insert("autocorrect".to_string(), autocorrect_config);
    
    // Also enable hub connector
    let hub_config = serde_json::json!({
        "enabled": true,
        "hub_url": "localhost:50052",
    });
    modules_config.insert("hub_connector".to_string(), hub_config);
    
    let config = ModularProxyConfig {
        session_name: "autocorrect-demo".to_string(),
        window: 0,
        pane: 0,
        agent_id: "autocorrect-agent-001".to_string(),
        modules_config,
    };
    
    // Create proxy with auto-correct
    let mut proxy = ModularTmuxProxy::new(config)?;
    
    println!("\n📝 Testing auto-correction...\n");
    
    // Test typos that should be corrected
    let typo_commands = vec![
        ("ehco 'Hello World!'", "Common typo: ehco → echo"),
        ("sl -la", "Common typo: sl → ls"),
        ("gti status", "Common typo: gti → git"),
        ("pyhton --version", "Common typo: pyhton → python"),
        ("gerp 'test' file.txt", "Common typo: gerp → grep"),
        ("celar", "Common typo: celar → clear"),
    ];
    
    for (typo_cmd, description) in typo_commands {
        println!("Testing: {}", description);
        println!("  Input:  {}", typo_cmd);
        
        // This will be auto-corrected by the module
        let result = proxy.send_command(typo_cmd)?;
        
        println!("  Result: Command executed with auto-correction\n");
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    
    // Test correct commands (no correction needed)
    println!("Testing correctly typed commands...");
    proxy.send_command("echo 'This is typed correctly'")?;
    proxy.send_command("ls -la")?;
    
    println!("\n✅ Auto-correction demo complete!");
    
    Ok(())
}