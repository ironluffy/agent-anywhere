// Hub connector module for tmux-agent

use super::{AgentContext, CommandResult, TmuxAgentModule};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::Utc;
use crate::proto::{AgentHubClient, LogEntry, AgentInfo, RegisterRequest, HeartbeatRequest};

/// Hub connector module for sending logs to aany-hub
pub struct HubConnectorModule {
    name: String,
    hub_url: Option<String>,
    agent_id: String,
    enabled: bool,
    client: Arc<Mutex<Option<AgentHubClient>>>,
}

impl HubConnectorModule {
    pub fn new() -> Self {
        Self {
            name: "hub_connector".to_string(),
            hub_url: None,
            agent_id: "unknown".to_string(),
            enabled: false,
            client: Arc::new(Mutex::new(None)),
        }
    }
    
    async fn connect_to_hub(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(hub_url) = &self.hub_url {
            println!("Connecting to hub at {}", hub_url);
            
            let client = AgentHubClient::connect(hub_url).await?;
            
            // Register agent
            let agent_info = AgentInfo {
                agent_id: self.agent_id.clone(),
                session_name: "tmux-agent".to_string(),
                version: "0.1.0".to_string(),
                capabilities: vec!["logging".to_string(), "monitoring".to_string()],
            };
            
            client.register(RegisterRequest {
                agent_info: Some(agent_info),
            }).await?;
            
            *self.client.lock().await = Some(client);
            
            println!("Successfully connected to hub");
        }
        Ok(())
    }
    
    async fn log_to_hub(
        &self, 
        level: &str, 
        message: &str, 
        component: &str,
        metadata: HashMap<String, String>
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled || self.hub_url.is_none() {
            return Ok(());
        }
        
        let log_entry = LogEntry {
            agent_id: self.agent_id.clone(),
            timestamp: Utc::now().to_rfc3339(),
            level: level.to_string(),
            message: message.to_string(),
            component: component.to_string(),
            metadata,
        };
        
        if let Some(client) = self.client.lock().await.as_ref() {
            client.send_logs(vec![log_entry]).await?;
        }
        
        Ok(())
    }
    
    async fn send_heartbeat(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }
        
        if let Some(client) = self.client.lock().await.as_ref() {
            client.heartbeat(HeartbeatRequest {
                agent_id: self.agent_id.clone(),
                timestamp: Utc::now().to_rfc3339(),
            }).await?;
        }
        
        Ok(())
    }
}

#[async_trait]
impl TmuxAgentModule for HubConnectorModule {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn initialize(&mut self, config: &HashMap<String, Value>) -> Result<(), Box<dyn std::error::Error>> {
        // Parse configuration
        if let Some(Value::Bool(enabled)) = config.get("enabled") {
            self.enabled = *enabled;
        }
        
        if let Some(Value::String(hub_url)) = config.get("hub_url") {
            self.hub_url = Some(format!("http://{}", hub_url));
        }
        
        if let Some(Value::String(agent_id)) = config.get("agent_id") {
            self.agent_id = agent_id.clone();
        }
        
        if self.enabled {
            println!("Hub connector module enabled for agent: {}", self.agent_id);
            self.connect_to_hub().await?;
        }
        
        Ok(())
    }
    
    async fn on_agent_start(&mut self, context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = context.metadata.clone();
        metadata.insert("session".to_string(), context.session_name.clone());
        
        self.log_to_hub(
            "INFO", 
            &format!("Agent {} started", self.agent_id),
            "lifecycle",
            metadata
        ).await?;
        
        Ok(())
    }
    
    async fn on_agent_stop(&mut self, context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = context.metadata.clone();
        metadata.insert("session".to_string(), context.session_name.clone());
        
        self.log_to_hub(
            "INFO", 
            &format!("Agent {} stopped", self.agent_id),
            "lifecycle",
            metadata
        ).await?;
        
        Ok(())
    }
    
    async fn post_command(&mut self, result: &CommandResult, context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = context.metadata.clone();
        metadata.insert("session".to_string(), context.session_name.clone());
        metadata.insert("exit_code".to_string(), result.exit_code.to_string());
        metadata.insert("duration_ms".to_string(), result.duration_ms.to_string());
        
        // Log command execution
        self.log_to_hub(
            if result.exit_code == 0 { "INFO" } else { "WARN" },
            &format!("Command executed: {}", result.command),
            "command",
            metadata
        ).await?;
        
        // Log output preview if there's output
        if !result.output.is_empty() {
            let preview = if result.output.len() > 100 {
                format!("{}...", &result.output[..100])
            } else {
                result.output.clone()
            };
            
            let mut output_metadata = context.metadata.clone();
            output_metadata.insert("preview".to_string(), preview);
            
            self.log_to_hub(
                "DEBUG",
                "Command output",
                "output",
                output_metadata
            ).await?;
        }
        
        // Send periodic heartbeat
        self.send_heartbeat().await.ok();
        
        Ok(())
    }
    
    async fn on_error(&mut self, error: &str, context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = context.metadata.clone();
        metadata.insert("session".to_string(), context.session_name.clone());
        
        self.log_to_hub(
            "ERROR",
            error,
            "error",
            metadata
        ).await?;
        
        Ok(())
    }
}