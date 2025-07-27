use clap::Subcommand;
use anyhow::Result;
use aany_pool::PoolManager;
use aany_pool::metadata::AgentStatus;
use std::path::PathBuf;
use directories::BaseDirs;
use std::env;

#[derive(Subcommand)]
pub enum PoolCommands {
    /// Launch interactive control panel UI (recommended)
    #[command(name = "ui", visible_alias = "UI")]
    Ui,
    
    /// List all agents in the pool
    List {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },
    
    /// Create a new agent
    Create {
        /// Agent name
        name: String,
        
        /// Template to use
        #[arg(short, long)]
        template: Option<String>,
        
        /// Start immediately after creation
        #[arg(short, long)]
        start: bool,
    },
    
    /// Start an agent
    Start {
        /// Agent name
        name: String,
        
        /// Attach after starting
        #[arg(short, long)]
        attach: bool,
    },
    
    /// Stop an agent
    Stop {
        /// Agent name
        name: String,
    },
    
    /// Attach to an agent's session
    Attach {
        /// Agent name
        name: String,
        
        /// Read-only mode
        #[arg(short, long)]
        readonly: bool,
    },
    
    /// Monitor an agent (read-only)
    Monitor {
        /// Agent name
        name: String,
    },
    
    /// Show agent status
    Status {
        /// Agent name (show all if not specified)
        name: Option<String>,
    },
    
    /// View agent logs
    Logs {
        /// Agent name
        name: String,
        
        /// Number of lines to show
        #[arg(short = 'n', long, default_value = "50")]
        lines: usize,
        
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },
    
    /// Delete an agent
    Delete {
        /// Agent name
        name: String,
        
        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Restart an agent (stop and start)
    Restart {
        /// Agent name
        name: String,
        
        /// Attach after restarting
        #[arg(short, long)]
        attach: bool,
    },
}

pub async fn handle_command(cmd: PoolCommands) -> Result<()> {
    // Get pool root directory
    let pool_root = get_pool_root()?;
    let mut manager = PoolManager::new(pool_root.clone()).await?;
    
    match cmd {
        PoolCommands::Ui => {
            use super::pool_ui::PoolUI;
            let mut ui = PoolUI::new(pool_root).await?;
            ui.run().await?;
        }
        
        PoolCommands::List { detailed } => {
            let agents = manager.list_agents();
            
            if agents.is_empty() {
                println!("No agents found in pool");
                return Ok(());
            }
            
            if detailed {
                for agent in agents {
                    println!("Agent: {}", agent.name);
                    println!("  Type: {}", agent.metadata.agent.agent_type);
                    println!("  Status: {:?}", agent.metadata.tmux.status);
                    println!("  Created: {}", agent.metadata.agent.created_at.format("%Y-%m-%d %H:%M"));
                    println!("  Last Active: {}", agent.metadata.agent.last_active.format("%Y-%m-%d %H:%M"));
                    println!("  Tasks Completed: {}", agent.metadata.tasks.completed);
                    if let Some(current) = &agent.metadata.tasks.current {
                        println!("  Current Task: {}", current);
                    }
                    println!();
                }
            } else {
                println!("Agent Pool ({} agents):", agents.len());
                for agent in agents {
                    let (status_text, status_color) = match agent.metadata.tmux.status {
                        AgentStatus::Active => (" RUNNING ", "\x1b[42m\x1b[30m"),
                        AgentStatus::Stopped => (" STOPPED ", "\x1b[100m\x1b[37m"),
                        AgentStatus::Crashed => (" CRASHED ", "\x1b[41m\x1b[37m"),
                        AgentStatus::Starting => (" STARTING ", "\x1b[43m\x1b[30m"),
                    };
                    println!("  {}{}\x1b[0m {:20} - {}", 
                        status_color,
                        status_text,
                        agent.name,
                        agent.metadata.agent.description
                    );
                }
            }
        }
        
        PoolCommands::Create { name, template, start } => {
            println!("Creating agent: {}", name);
            
            let _agent = manager.create_agent(name.clone(), template).await?;
            println!("Agent created successfully");
            
            if start {
                println!("Starting agent...");
                manager.get_agent_mut(&name)?.start().await?;
                println!("Agent started");
            }
        }
        
        PoolCommands::Start { name, attach } => {
            println!("Starting agent: {}", name);
            
            let agent = manager.get_agent_mut(&name)?;
            agent.start().await?;
            println!("✅ Agent started");
            
            if attach {
                let session_name = format!("agent-{}", name);
                attach_to_tmux_session(&session_name, false)?;
            }
        }
        
        PoolCommands::Stop { name } => {
            println!("Stopping agent: {}", name);
            
            let agent = manager.get_agent_mut(&name)?;
            agent.stop().await?;
            println!("Agent stopped");
        }
        
        PoolCommands::Attach { name, readonly } => {
            let agent = manager.get_agent(&name)?;
            let session_name = &agent.metadata.tmux.session_name;
            
            attach_to_tmux_session(session_name, readonly)?;
        }
        
        PoolCommands::Monitor { name } => {
            let agent = manager.get_agent(&name)?;
            let session_name = &agent.metadata.tmux.session_name;
            
            attach_to_tmux_session(session_name, true)?;
        }
        
        PoolCommands::Status { name } => {
            if let Some(name) = name {
                // Show specific agent status
                let agent = manager.get_agent(&name)?;
                println!("Agent: {}", agent.name);
                println!("Status: {:?}", agent.metadata.tmux.status);
                println!("Type: {}", agent.metadata.agent.agent_type);
                println!("Model: {}", agent.metadata.claude.model);
                println!("Created: {}", agent.metadata.agent.created_at.format("%Y-%m-%d %H:%M"));
                println!("Last Active: {}", agent.metadata.agent.last_active.format("%Y-%m-%d %H:%M"));
                println!("Tasks Completed: {}", agent.metadata.tasks.completed);
                
                if let Some(current) = &agent.metadata.tasks.current {
                    println!("Current Task: {}", current);
                }
                
                if !agent.metadata.tasks.queue.is_empty() {
                    println!("Queued Tasks:");
                    for task in &agent.metadata.tasks.queue {
                        println!("  - {}", task);
                    }
                }
            } else {
                // Show pool statistics
                let stats = manager.get_stats();
                println!("Agent Pool Statistics:");
                println!("  Total Agents: {}", stats.total_agents);
                println!("  Active: {}", stats.active_agents);
                println!("  Stopped: {}", stats.stopped_agents);
                println!("  Crashed: {}", stats.crashed_agents);
                println!("  Starting: {}", stats.starting_agents);
                println!("  Total Tasks Completed: {}", stats.total_tasks_completed);
            }
        }
        
        PoolCommands::Logs { name, lines, follow } => {
            let agent = manager.get_agent(&name)?;
            let log_dir = agent.get_agent_dir().join("logs");
            
            // Find latest log file - check session log first
            let session_log = log_dir.join("session_latest.log");
            let event_log = log_dir.join("events_latest.jsonl");
            
            let log_file = if session_log.exists() {
                session_log
            } else if event_log.exists() {
                event_log
            } else {
                println!("No logs found for agent: {}", name);
                println!("The agent may not have been started yet.");
                return Ok(());
            };
            
            println!("Viewing logs from: {}", log_file.display());
            println!();
            
            if follow {
                std::process::Command::new("tail")
                    .args(&["-f", "-n", &lines.to_string()])
                    .arg(&log_file)
                    .status()?;
            } else {
                std::process::Command::new("tail")
                    .args(&["-n", &lines.to_string()])
                    .arg(&log_file)
                    .status()?;
            }
        }
        
        PoolCommands::Delete { name, force } => {
            if !force {
                println!("Are you sure you want to delete agent '{}'? This action cannot be undone.", name);
                println!("Type 'yes' to confirm:");
                
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                
                if input.trim() != "yes" {
                    println!("Deletion cancelled");
                    return Ok(());
                }
            }
            
            println!("Deleting agent: {}", name);
            manager.delete_agent(&name).await?;
            println!("Agent deleted");
        }
        
        PoolCommands::Restart { name, attach } => {
            println!("Restarting agent: {}", name);
            
            // Stop the agent if it's running
            if let Ok(agent) = manager.get_agent_mut(&name) {
                if agent.is_running()? {
                    println!("Stopping agent...");
                    agent.stop().await?;
                    
                    // Small delay to ensure clean shutdown
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            }
            
            // Start the agent
            println!("Starting agent...");
            let agent = manager.get_agent_mut(&name)?;
            agent.start().await?;
            println!("Agent restarted");
            
            if attach {
                let session_name = format!("agent-{}", name);
                attach_to_tmux_session(&session_name, false)?;
            }
        }
    }
    
    Ok(())
}

fn get_pool_root() -> Result<PathBuf> {
    // Check environment variable first
    if let Ok(root) = std::env::var("AGENT_POOL_ROOT") {
        return Ok(PathBuf::from(root));
    }
    
    // Use default in user's home directory
    if let Some(base_dirs) = BaseDirs::new() {
        Ok(base_dirs.home_dir().join(".aany"))
    } else {
        // Fallback to home directory
        Ok(std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".aany"))
    }
}

/// Check if we're already inside a tmux session
fn is_inside_tmux() -> bool {
    env::var("TMUX").is_ok()
}

/// Attach to a tmux session, handling nested tmux sessions
fn attach_to_tmux_session(session_name: &str, readonly: bool) -> Result<()> {
    if is_inside_tmux() {
        // If we're already in tmux, switch to the target session instead of attaching
        println!("Already in tmux, switching to session: {}", session_name);
        
        // Get current session name to set as return session
        let current_session = std::process::Command::new("tmux")
            .args(&["display-message", "-p", "#S"])
            .output()?;
        let current_session_name = String::from_utf8_lossy(&current_session.stdout).trim().to_string();
        
        // Set the return session environment variable in the target session
        std::process::Command::new("tmux")
            .args(&["set-environment", "-t", session_name, "AANY_RETURN_SESSION", &current_session_name])
            .status()?;
        
        // Switch to the target session
        std::process::Command::new("tmux")
            .args(&["switch-client", "-t", session_name])
            .status()?;
    } else {
        // If we're not in tmux, attach normally
        let mut cmd = std::process::Command::new("tmux");
        cmd.args(&["attach-session", "-t", session_name]);
        
        if readonly {
            cmd.arg("-r");
        }
        
        cmd.status()?;
    }
    Ok(())
}