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
            let warning = format!(
                r#"clear && echo -e '\033[1;33m
╔══════════════════════════════════════╗
║  ⚠️  AGENT CONTROLLED SESSION  ⚠️   ║
║                                      ║
║  Session: {}
║  This tmux session is managed by     ║
║  an automated agent.                 ║
║                                      ║
║  Manual changes may disrupt agent    ║
║  operations!                         ║
╚══════════════════════════════════════╝
\033[0m'"#, session_name
            );
            
            Command::new("tmux")
                .args(&["send-keys", "-t", &session_name, &warning, "Enter"])
                .output()?;
            
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
    }
}