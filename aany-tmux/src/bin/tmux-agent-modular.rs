use aany_tmux::{ModularTmuxProxy, ModularProxyConfig};
use clap::Parser;
use std::fs;
use serde_json;

#[derive(Parser)]
#[command(name = "tmux-agent-modular")]
#[command(about = "Modular tmux agent with logging support", long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long)]
    config: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Load configuration
    let config_str = fs::read_to_string(&cli.config)?;
    let config: ModularProxyConfig = serde_json::from_str(&config_str)?;
    
    println!("Starting modular tmux agent...");
    println!("Agent ID: {}", config.agent_id);
    println!("Session: {}", config.session_name);
    
    // Create modular proxy
    let mut proxy = ModularTmuxProxy::new(config)?;
    
    println!("Agent initialized with modules:");
    println!("✅ Ready for operations");
    
    // Keep the agent running
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}