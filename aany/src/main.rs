use clap::{Parser, Subcommand};
use anyhow::Result;

mod commands;
mod config;
use commands::{tmux, pool, auth, template};

#[derive(Parser)]
#[command(name = "aany")]
#[command(about = "Agent Anywhere - Unified CLI for AI agent management", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Enable debug logging
    #[arg(short, long, global = true)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new tmux agent session (alias for 'tmux new')
    New {
        /// Name for the tmux session
        #[arg(value_name = "SESSION_NAME")]
        name: String,
        
        /// Detach after creating
        #[arg(short, long)]
        detach: bool,
    },
    
    /// Manage tmux sessions for agents
    #[command(subcommand)]
    Tmux(tmux::TmuxCommands),
    
    /// Manage agent pool
    #[command(subcommand)]
    Pool(pool::PoolCommands),
    
    /// Authentication and API management
    #[command(subcommand)]
    Auth(auth::AuthCommands),
    
    /// Initialize agent workspace
    Init {
        /// Name of the agent
        name: String,
        
        /// Template to use
        #[arg(short, long)]
        template: Option<String>,
    },
    
    /// Show global configuration
    Config {
        /// Configuration key to get/set
        key: Option<String>,
        
        /// Value to set
        value: Option<String>,
    },
    
    /// Manage agent templates
    #[command(subcommand)]
    Template(template::TemplateCommands),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    if cli.debug {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter("info")
            .init();
    }
    
    match cli.command {
        None => {
            // No subcommand - launch pool UI
            pool::handle_command(pool::PoolCommands::Ui).await
        }
        Some(Commands::New { name, detach }) => {
            // Alias for tmux new
            tmux::handle_command(tmux::TmuxCommands::New {
                name,
                detach,
            }).await
        }
        Some(Commands::Tmux(cmd)) => tmux::handle_command(cmd).await,
        Some(Commands::Pool(cmd)) => pool::handle_command(cmd).await,
        Some(Commands::Auth(cmd)) => auth::handle_command(cmd).await,
        Some(Commands::Init { name, template }) => {
            println!("Initializing agent workspace: {}", name);
            if let Some(tpl) = template {
                println!("Using template: {}", tpl);
            }
            Ok(())
        }
        Some(Commands::Config { key, value }) => {
            handle_config(key, value).await
        }
        Some(Commands::Template(cmd)) => template::handle_command(cmd).await,
    }
}

async fn handle_config(key: Option<String>, value: Option<String>) -> Result<()> {
    use config::GlobalConfig;
    
    match (key, value) {
        (Some(k), Some(v)) => {
            // Set a configuration value
            let mut config = GlobalConfig::load()?;
            config.set(&k, v)?;
            config.save()?;
            println!("✅ Set {} = {}", k, config.get(&k).unwrap_or_default());
        }
        (Some(k), None) => {
            // Get a configuration value
            let config = GlobalConfig::load()?;
            if let Some(value) = config.get(&k) {
                println!("{}", value);
            } else {
                println!("Configuration key not found: {}", k);
            }
        }
        (None, _) => {
            // Show all configuration
            let config = GlobalConfig::load()?;
            let yaml = serde_yaml::to_string(&config)?;
            println!("Global Configuration (~/.aany/config.yaml):");
            println!("{}", yaml);
        }
    }
    
    Ok(())
}