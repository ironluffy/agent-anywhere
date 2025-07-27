use clap::Subcommand;
use anyhow::Result;
use std::process::Command;

#[derive(Subcommand)]
pub enum TmuxCommands {
    /// Create a new tmux session for an agent
    New {
        /// Session name
        name: String,
        
        /// Detach after creating
        #[arg(short, long)]
        detach: bool,
    },
    
    /// Attach to an agent's tmux session
    Attach {
        /// Session name
        name: String,
        
        /// Read-only mode
        #[arg(short, long)]
        readonly: bool,
    },
    
    /// Monitor an agent's session (read-only)
    Monitor {
        /// Session name
        name: String,
    },
    
    /// List all tmux sessions
    List,
    
    /// Kill a tmux session
    Kill {
        /// Session name
        name: String,
    },
    
    /// Show session health status
    Health {
        /// Session name
        name: String,
    },
    
    /// Clean up extra panes in a session
    Cleanup {
        /// Session name
        name: String,
    },
    
    /// Create a new modular agent with logging enabled
    Agent {
        /// Agent name/ID
        name: String,
        
        /// Hub URL (defaults to localhost:50052)
        #[arg(long, default_value = "localhost:50052")]
        hub_url: String,
        
        /// Disable logging to hub
        #[arg(long)]
        no_logging: bool,
        
        /// Additional modules to enable (comma-separated)
        #[arg(long)]
        modules: Option<String>,
    },
}

pub async fn handle_command(cmd: TmuxCommands) -> Result<()> {
    
    match cmd {
        TmuxCommands::New { name, detach } => {
            println!("Creating new tmux session: {}", name);
            let session_name = format!("agent-{}", name);
            
            // Create new tmux session
            let output = Command::new("tmux")
                .args(&["new-session", "-d", "-s", &session_name])
                .output()?;
                
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to create session: {}", error);
            }
            
            // Send warning message to the session
            // Remove the warning variable since we're not using it anymore
            
            // Don't send any commands to the agent - keep it clean
            
//             // ENABLE LOGGING BY DEFAULT!
//             let hub_url = std::env::var("AANY_HUB_URL").unwrap_or_else(|_| "localhost:50052".to_string());
            
//             // Set up environment for logging
//             let setup_logging = format!(
//                 r#"
// export AANY_AGENT_ID='{}'
// export AANY_HUB_URL='{}'
// export AANY_LOGGING_ENABLED=true
// echo '🔌 Agent {} connected to hub at {}'
// echo '📊 Logging is ENABLED by default'
// echo '💡 To disable logging: export AANY_LOGGING_ENABLED=false'
// "#, name, hub_url, name, hub_url
//             );
            
//             Command::new("tmux")
//                 .args(&["send-keys", "-t", &session_name, &setup_logging, "C-m"])
//                 .output()?;
            
//             // Start BOTH loggers for comprehensive logging
//             let logger_script = "/Users/hsuh/Gitrepo/agent-anywhere/aany-tmux/tmux-logger.sh";
//             let interaction_logger_script = "/Users/hsuh/Gitrepo/agent-anywhere/aany-tmux/tmux-interaction-logger.sh";
            
//             // Start the original tmux logger
//             if std::path::Path::new(logger_script).exists() {
//                 Command::new("bash")
//                     .args(&[logger_script, &session_name, &name])
//                     .env("AANY_HUB_URL", &hub_url)
//                     .env("GRPC_ENABLE_FORK_SUPPORT", "1")
//                     .env("GRPC_POLL_STRATEGY", "poll")
//                     .stdout(std::process::Stdio::null())
//                     .stderr(std::process::Stdio::null())
//                     .spawn()
//                     .ok();
//             }
            
//             // Start the interaction logger
//             if std::path::Path::new(interaction_logger_script).exists() {
//                 Command::new("bash")
//                     .args(&[interaction_logger_script, &session_name, &name])
//                     .env("AANY_HUB_URL", &hub_url)
//                     .env("AANY_LOGGING_ENABLED", "true")
//                     .env("GRPC_ENABLE_FORK_SUPPORT", "1")
//                     .env("GRPC_POLL_STRATEGY", "poll")
//                     .stdout(std::process::Stdio::null())
//                     .stderr(std::process::Stdio::null())
//                     .spawn()
//                     .ok();
//                 println!("✅ Agent created with comprehensive logging enabled to {}", hub_url);
//                 println!("📊 View logs at: http://localhost:8090");
//                 println!("📝 Interaction logs saved to: ~/.aany/agents/{}/logs/", name);
//             } else {
//                 println!("✅ Agent created with basic logging enabled");
//                 println!("📊 View logs at: http://localhost:8090");
            // }
            
            if !detach {
                // Attach to the session
                Command::new("tmux")
                    .args(&["attach-session", "-t", &session_name])
                    .status()?;
            }
            
            Ok(())
        }
        
        TmuxCommands::Attach { name, readonly } => {
            let session_name = format!("agent-{}", name);
            println!("Attaching to session: {}", session_name);
            
            let mut cmd = std::process::Command::new("tmux");
            cmd.args(&["attach-session", "-t", &session_name]);
            
            if readonly {
                cmd.arg("-r");
            }
            
            cmd.status()?;
            Ok(())
        }
        
        TmuxCommands::Monitor { name } => {
            let session_name = format!("agent-{}", name);
            println!("Monitoring session (read-only): {}", session_name);
            
            Command::new("tmux")
                .args(&["attach-session", "-r", "-t", &session_name])
                .status()?;
            
            Ok(())
        }
        
        TmuxCommands::List => {
            println!("Agent tmux sessions:");
            let output = std::process::Command::new("tmux")
                .args(&["list-sessions"])
                .output()?;
            
            let sessions = String::from_utf8_lossy(&output.stdout);
            for line in sessions.lines() {
                if line.contains("agent-") {
                    println!("  {}", line);
                }
            }
            
            if !sessions.contains("agent-") {
                println!("  No agent sessions found");
            }
            
            Ok(())
        }
        
        TmuxCommands::Kill { name } => {
            let session_name = format!("agent-{}", name);
            println!("Killing session: {}", session_name);
            
            let output = Command::new("tmux")
                .args(&["kill-session", "-t", &session_name])
                .output()?;
                
            if output.status.success() {
                println!("Session killed successfully");
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to kill session: {}", error);
            }
            
            Ok(())
        }
        
        TmuxCommands::Health { name } => {
            let session_name = format!("agent-{}", name);
            println!("Checking health of session: {}", session_name);
            
            // Check if session exists
            let output = Command::new("tmux")
                .args(&["has-session", "-t", &session_name])
                .output()?;
                
            if output.status.success() {
                println!("✅ Session is active");
                
                // Count panes
                let pane_output = Command::new("tmux")
                    .args(&["list-panes", "-t", &session_name])
                    .output()?;
                    
                if pane_output.status.success() {
                    let pane_count = String::from_utf8_lossy(&pane_output.stdout)
                        .lines()
                        .count();
                    
                    if pane_count > 1 {
                        println!("⚠️  Multiple panes detected ({} panes)", pane_count);
                    } else {
                        println!("✅ Single pane (no interference detected)");
                    }
                }
            } else {
                println!("❌ Session not found");
            }
            
            Ok(())
        }
        
        TmuxCommands::Cleanup { name } => {
            let session_name = format!("agent-{}", name);
            println!("Cleaning up session: {}", session_name);
            
            // Get list of panes
            let output = Command::new("tmux")
                .args(&["list-panes", "-t", &session_name, "-F", "#{pane_id}"])
                .output()?;
                
            if !output.status.success() {
                anyhow::bail!("Failed to list panes");
            }
            
            let panes: Vec<String> = String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect();
                
            // Keep only the first pane, kill the rest
            for (i, pane) in panes.iter().enumerate() {
                if i > 0 {
                    Command::new("tmux")
                        .args(&["kill-pane", "-t", &format!("{}.{}", session_name, pane)])
                        .output()?;
                }
            }
            
            println!("✅ Cleanup complete (removed {} extra panes)", panes.len().saturating_sub(1));
            
            Ok(())
        }
        
        TmuxCommands::Agent { name, hub_url, no_logging, modules } => {
            println!("Creating modular agent: {}", name);
            let session_name = format!("agent-{}", name);
            
            // Create new tmux session
            let output = Command::new("tmux")
                .args(&["new-session", "-d", "-s", &session_name])
                .output()?;
                
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to create session: {}", error);
            }
            
            // Build the modular agent command
            let mut agent_cmd = format!("aany-tmux-modular --session {} --agent-id {}", session_name, name);
            
            if !no_logging {
                agent_cmd.push_str(&format!(" --enable-hub --hub-url {}", hub_url));
                println!("✅ Logging enabled to hub: {}", hub_url);
            }
            
            if let Some(module_list) = modules {
                agent_cmd.push_str(&format!(" --modules {}", module_list));
            }
            
            // Send the command to start the modular agent
            Command::new("tmux")
                .args(&["send-keys", "-t", &session_name, &agent_cmd, "C-m"])
                .output()?;
            
            println!("✅ Modular agent created: {}", session_name);
            if !no_logging {
                println!("📊 Logs will be sent to: {}", hub_url);
            }
            println!("👁️  Monitor with: aany tmux monitor {}", name);
            
            Ok(())
        }
    }
}