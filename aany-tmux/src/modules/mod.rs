// Module system for tmux-agent

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub session_name: String,
    pub window_id: Option<String>,
    pub pane_id: Option<String>,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub command: String,
    pub output: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

#[async_trait]
pub trait TmuxAgentModule: Send + Sync {
    /// Module name
    fn name(&self) -> &str;
    
    /// Initialize module
    async fn initialize(&mut self, config: &HashMap<String, serde_json::Value>) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Called when agent starts
    async fn on_agent_start(&mut self, _context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    /// Called when agent stops
    async fn on_agent_stop(&mut self, _context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    /// Called before command execution (can modify command)
    async fn pre_command(&mut self, command: &str, _context: &AgentContext) -> Result<String, Box<dyn std::error::Error>> {
        Ok(command.to_string())
    }
    
    /// Called after command execution
    async fn post_command(&mut self, _result: &CommandResult, _context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    /// Called when output is received
    async fn on_output(&mut self, _output: &str, _context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    /// Called on errors
    async fn on_error(&mut self, _error: &str, _context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

pub mod hub_connector;
pub mod autocorrect;
pub mod interaction_logger;
// pub mod terminal_broadcaster; // TODO: fix imports

/// Module registry
pub struct ModuleRegistry {
    modules: Vec<Box<dyn TmuxAgentModule>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }
    
    pub fn register(&mut self, module: Box<dyn TmuxAgentModule>) {
        println!("Registered module: {}", module.name());
        self.modules.push(module);
    }
    
    pub async fn on_agent_start(&mut self, context: &AgentContext) {
        for module in &mut self.modules {
            if let Err(e) = module.on_agent_start(context).await {
                eprintln!("Module {} error in on_agent_start: {}", module.name(), e);
            }
        }
    }
    
    pub async fn pre_command(&mut self, command: &str, context: &AgentContext) -> String {
        let mut cmd = command.to_string();
        for module in &mut self.modules {
            match module.pre_command(&cmd, context).await {
                Ok(modified_cmd) => cmd = modified_cmd,
                Err(e) => eprintln!("Module {} error in pre_command: {}", module.name(), e),
            }
        }
        cmd
    }
    
    pub async fn post_command(&mut self, result: &CommandResult, context: &AgentContext) {
        for module in &mut self.modules {
            if let Err(e) = module.post_command(result, context).await {
                eprintln!("Module {} error in post_command: {}", module.name(), e);
            }
        }
    }
    
    pub async fn on_error(&mut self, error: &str, context: &AgentContext) {
        for module in &mut self.modules {
            if let Err(e) = module.on_error(error, context).await {
                eprintln!("Module {} error in on_error: {}", module.name(), e);
            }
        }
    }
    
    pub async fn on_agent_stop(&mut self, context: &AgentContext) {
        for module in &mut self.modules {
            if let Err(e) = module.on_agent_stop(context).await {
                eprintln!("Module {} error in on_agent_stop: {}", module.name(), e);
            }
        }
    }
}