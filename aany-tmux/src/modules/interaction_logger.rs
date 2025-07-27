// Interaction logger module for tmux-agent
// Logs all user input, tmux screen history, and agent interactions

use super::{AgentContext, CommandResult, TmuxAgentModule};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use chrono::{Utc, Local};
use std::path::PathBuf;
use crate::proto::{AgentHubClient, LogEntry, AgentInfo, RegisterRequest};

#[derive(Debug, Clone)]
struct InteractionLog {
    timestamp: String,
    agent_id: String,
    session_name: String,
    interaction_type: String, // "user_input", "command", "output", "screen_history"
    content: String,
    metadata: HashMap<String, String>,
}

#[derive(Clone)]
pub struct InteractionLoggerModule {
    name: String,
    agent_id: String,
    session_name: String,
    enabled: bool,
    log_dir: PathBuf,
    log_file: Arc<Mutex<Option<tokio::fs::File>>>,
    hub_client: Arc<Mutex<Option<AgentHubClient>>>,
    hub_url: Option<String>,
    buffer: Arc<Mutex<Vec<InteractionLog>>>,
    last_screen_content: Arc<Mutex<String>>,
}

impl InteractionLoggerModule {
    pub fn new() -> Self {
        // Default log dir will be set during initialization
        let log_dir = PathBuf::from("");
        
        Self {
            name: "interaction_logger".to_string(),
            agent_id: "unknown".to_string(),
            session_name: "unknown".to_string(),
            enabled: false,
            log_dir,
            log_file: Arc::new(Mutex::new(None)),
            hub_client: Arc::new(Mutex::new(None)),
            hub_url: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            last_screen_content: Arc::new(Mutex::new(String::new())),
        }
    }
    
    async fn ensure_log_dir(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.log_dir).await?;
        Ok(())
    }
    
    async fn open_log_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.ensure_log_dir().await?;
        
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("tmux_agent_{}_{}.log", self.agent_id, timestamp);
        let log_path = self.log_dir.join(filename);
        
        println!("Opening interaction log file: {}", log_path.display());
        
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path)
            .await?;
            
        *self.log_file.lock().await = Some(file);
        
        // Write header
        self.write_log_header().await?;
        
        Ok(())
    }
    
    async fn write_log_header(&self) -> Result<(), Box<dyn std::error::Error>> {
        let header = format!(
            "=== TMUX AGENT INTERACTION LOG ===\n\
             Agent ID: {}\n\
             Session: {}\n\
             Started: {}\n\
             ===================================\n\n",
            self.agent_id,
            self.session_name,
            Local::now().format("%Y-%m-%d %H:%M:%S")
        );
        
        if let Some(file) = self.log_file.lock().await.as_mut() {
            file.write_all(header.as_bytes()).await?;
            file.flush().await?;
        }
        
        Ok(())
    }
    
    async fn log_interaction(&self, interaction: InteractionLog) -> Result<(), Box<dyn std::error::Error>> {
        // Log to file
        if let Some(file) = self.log_file.lock().await.as_mut() {
            let log_entry = format!(
                "[{}] [{}] {}: {}\n",
                interaction.timestamp,
                interaction.interaction_type,
                interaction.agent_id,
                interaction.content
            );
            file.write_all(log_entry.as_bytes()).await?;
            
            // Log metadata if present
            if !interaction.metadata.is_empty() {
                let metadata_str = format!("  Metadata: {:?}\n", interaction.metadata);
                file.write_all(metadata_str.as_bytes()).await?;
            }
            
            file.flush().await?;
        }
        
        // Send to hub if connected
        if let Some(client) = self.hub_client.lock().await.as_ref() {
            let mut metadata = interaction.metadata.clone();
            metadata.insert("interaction_type".to_string(), interaction.interaction_type.clone());
            metadata.insert("session_name".to_string(), interaction.session_name.clone());
            
            let log_entry = LogEntry {
                agent_id: interaction.agent_id,
                timestamp: interaction.timestamp,
                level: "INFO".to_string(),
                message: interaction.content,
                component: "interaction".to_string(),
                metadata,
            };
            
            client.send_logs(vec![log_entry]).await.ok();
        }
        
        Ok(())
    }
    
    async fn connect_to_hub(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(hub_url) = &self.hub_url {
            println!("Interaction logger connecting to hub at {}", hub_url);
            
            let client = AgentHubClient::connect(hub_url).await?;
            
            // Register agent with interaction logging capability
            let agent_info = AgentInfo {
                agent_id: self.agent_id.clone(),
                session_name: self.session_name.clone(),
                version: "0.1.0".to_string(),
                capabilities: vec![
                    "interaction_logging".to_string(),
                    "screen_capture".to_string(),
                    "user_input_tracking".to_string(),
                ],
            };
            
            client.register(RegisterRequest {
                agent_info: Some(agent_info),
            }).await?;
            
            *self.hub_client.lock().await = Some(client);
            
            println!("Interaction logger connected to hub");
        }
        Ok(())
    }
    
    pub async fn capture_screen_content(&self, content: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut last_content = self.last_screen_content.lock().await;
        
        // Only log if content has changed
        if *last_content != content {
            let interaction = InteractionLog {
                timestamp: Utc::now().to_rfc3339(),
                agent_id: self.agent_id.clone(),
                session_name: self.session_name.clone(),
                interaction_type: "screen_history".to_string(),
                content: content.clone(),
                metadata: HashMap::new(),
            };
            
            self.log_interaction(interaction).await?;
            *last_content = content;
        }
        
        Ok(())
    }
    
    pub async fn log_user_input(&self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
        let interaction = InteractionLog {
            timestamp: Utc::now().to_rfc3339(),
            agent_id: self.agent_id.clone(),
            session_name: self.session_name.clone(),
            interaction_type: "user_input".to_string(),
            content: input.to_string(),
            metadata: HashMap::new(),
        };
        
        self.log_interaction(interaction).await?;
        Ok(())
    }
}

#[async_trait]
impl TmuxAgentModule for InteractionLoggerModule {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn initialize(&mut self, config: &HashMap<String, Value>) -> Result<(), Box<dyn std::error::Error>> {
        // Parse configuration
        if let Some(Value::Bool(enabled)) = config.get("enabled") {
            self.enabled = *enabled;
        }
        
        if let Some(Value::String(agent_id)) = config.get("agent_id") {
            self.agent_id = agent_id.clone();
        }
        
        if let Some(Value::String(session_name)) = config.get("session_name") {
            self.session_name = session_name.clone();
        }
        
        if let Some(Value::String(hub_url)) = config.get("hub_url") {
            self.hub_url = Some(format!("http://{}", hub_url));
        }
        
        if let Some(Value::String(log_dir)) = config.get("log_dir") {
            self.log_dir = PathBuf::from(log_dir);
        } else {
            // Default to agent-specific directory
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            self.log_dir = PathBuf::from(format!("{home}/.aany/agents/{}/logs", self.agent_id));
        }
        
        if self.enabled {
            println!("Interaction logger enabled for agent: {}", self.agent_id);
            self.open_log_file().await?;
            
            if self.hub_url.is_some() {
                self.connect_to_hub().await?;
            }
        }
        
        Ok(())
    }
    
    async fn on_agent_start(&mut self, context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }
        
        let interaction = InteractionLog {
            timestamp: Utc::now().to_rfc3339(),
            agent_id: self.agent_id.clone(),
            session_name: context.session_name.clone(),
            interaction_type: "lifecycle".to_string(),
            content: format!("Agent {} started in session {}", self.agent_id, context.session_name),
            metadata: context.metadata.clone(),
        };
        
        self.log_interaction(interaction).await?;
        Ok(())
    }
    
    async fn on_agent_stop(&mut self, context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }
        
        let interaction = InteractionLog {
            timestamp: Utc::now().to_rfc3339(),
            agent_id: self.agent_id.clone(),
            session_name: context.session_name.clone(),
            interaction_type: "lifecycle".to_string(),
            content: format!("Agent {} stopped", self.agent_id),
            metadata: context.metadata.clone(),
        };
        
        self.log_interaction(interaction).await?;
        
        // Close log file
        if let Some(file) = self.log_file.lock().await.as_mut() {
            file.write_all(b"\n=== SESSION ENDED ===\n").await?;
            file.flush().await?;
        }
        
        Ok(())
    }
    
    async fn pre_command(&mut self, command: &str, _context: &AgentContext) -> Result<String, Box<dyn std::error::Error>> {
        if self.enabled {
            // Log the command being sent
            let interaction = InteractionLog {
                timestamp: Utc::now().to_rfc3339(),
                agent_id: self.agent_id.clone(),
                session_name: self.session_name.clone(),
                interaction_type: "command".to_string(),
                content: command.to_string(),
                metadata: HashMap::new(),
            };
            
            self.log_interaction(interaction).await?;
        }
        
        Ok(command.to_string())
    }
    
    async fn post_command(&mut self, result: &CommandResult, _context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }
        
        // Log command output
        let mut metadata = HashMap::new();
        metadata.insert("exit_code".to_string(), result.exit_code.to_string());
        metadata.insert("duration_ms".to_string(), result.duration_ms.to_string());
        
        let interaction = InteractionLog {
            timestamp: Utc::now().to_rfc3339(),
            agent_id: self.agent_id.clone(),
            session_name: self.session_name.clone(),
            interaction_type: "output".to_string(),
            content: result.output.clone(),
            metadata,
        };
        
        self.log_interaction(interaction).await?;
        Ok(())
    }
    
    async fn on_output(&mut self, output: &str, _context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }
        
        // Capture any screen output
        self.capture_screen_content(output.to_string()).await?;
        Ok(())
    }
    
    async fn on_error(&mut self, error: &str, context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }
        
        let interaction = InteractionLog {
            timestamp: Utc::now().to_rfc3339(),
            agent_id: self.agent_id.clone(),
            session_name: context.session_name.clone(),
            interaction_type: "error".to_string(),
            content: error.to_string(),
            metadata: context.metadata.clone(),
        };
        
        self.log_interaction(interaction).await?;
        Ok(())
    }
}