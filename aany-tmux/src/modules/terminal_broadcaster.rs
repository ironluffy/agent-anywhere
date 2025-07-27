// Terminal broadcaster module for tmux-agent
// Streams terminal I/O to aany-hub via gRPC

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::Value;
use log::{info, error, debug};
use std::process::Command;

use crate::modules::{TmuxAgentModule, AgentContext, CommandResult};

pub struct TerminalBroadcasterModule {
    hub_url: Option<String>,
    agent_id: String,
    tmux_session: Option<String>,
    monitoring_active: Arc<Mutex<bool>>,
}

impl TerminalBroadcasterModule {
    pub fn new() -> Self {
        Self {
            hub_url: None,
            agent_id: String::new(),
            tmux_session: None,
            monitoring_active: Arc::new(Mutex::new(false)),
        }
    }
    
    fn name(&self) -> &str {
        "terminal_broadcaster"
    }
    
    async fn start_monitoring(&self, tmux_session: &str) {
        let monitoring = self.monitoring_active.clone();
        let session = tmux_session.to_string();
        
        // Start monitoring in background
        tokio::spawn(async move {
            let mut active = monitoring.lock().await;
            *active = true;
            drop(active);
            
            info!("Starting terminal monitoring for session: {}", session);
            
            loop {
                // Check if still active
                let active = monitoring.lock().await;
                if !*active {
                    break;
                }
                drop(active);
                
                // Capture pane output
                match Command::new("tmux")
                    .args(&["capture-pane", "-t", &session, "-p"])
                    .output()
                {
                    Ok(output) => {
                        if output.status.success() {
                            let content = String::from_utf8_lossy(&output.stdout);
                            // TODO: Send to hub via gRPC when proper proto is implemented
                            debug!("Captured {} bytes from tmux", content.len());
                        }
                    }
                    Err(e) => {
                        error!("Failed to capture tmux pane: {}", e);
                    }
                }
                
                // Wait before next capture
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
            
            info!("Stopped terminal monitoring for session: {}", session);
        });
    }
}

#[async_trait]
impl TmuxAgentModule for TerminalBroadcasterModule {
    fn name(&self) -> &str {
        "terminal_broadcaster"
    }
    
    async fn initialize(&mut self, config: &HashMap<String, Value>) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(Value::String(url)) = config.get("hub_url") {
            self.hub_url = Some(url.clone());
        }
        
        if let Some(Value::String(id)) = config.get("agent_id") {
            self.agent_id = id.clone();
        }
        
        info!("Terminal broadcaster module initialized");
        Ok(())
    }
    
    async fn on_agent_start(&mut self, context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        info!("Terminal broadcaster started for session: {}", context.session_name);
        
        self.tmux_session = Some(context.session_name.clone());
        
        // Start monitoring tmux session
        self.start_monitoring(&context.session_name).await;
        
        Ok(())
    }
    
    async fn post_command(&mut self, result: &CommandResult, _context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        // Log command result
        info!("Command executed: {} (exit: {})", result.command, result.exit_code);
        
        // TODO: Send to hub when gRPC is properly implemented
        
        Ok(())
    }
    
    async fn on_output(&mut self, output: &str, _context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Output received: {} bytes", output.len());
        
        // TODO: Send to hub when gRPC is properly implemented
        
        Ok(())
    }
    
    async fn on_agent_stop(&mut self, context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        info!("Terminal broadcaster stopping for session: {}", context.session_name);
        
        // Stop monitoring
        let mut active = self.monitoring_active.lock().await;
        *active = false;
        
        Ok(())
    }
}