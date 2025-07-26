use crate::{metadata::*, error::{PoolResult, PoolError}, logging::{AgentLogger, setup_tmux_logging}};
use std::path::PathBuf;
use std::process::Command;
use tokio::fs;

pub struct Agent {
    pub name: String,
    pub root_path: PathBuf,
    pub metadata: AgentMetadata,
    logger: Option<AgentLogger>,
}

impl Agent {
    /// Create a new agent
    pub async fn new(name: String, root_path: PathBuf) -> PoolResult<Self> {
        let mut metadata = AgentMetadata::default();
        metadata.agent.name = name.clone();
        metadata.tmux.session_name = format!("agent-{}", name);
        
        Ok(Self {
            name,
            root_path,
            metadata,
            logger: None,
        })
    }
    
    /// Load an existing agent from disk
    pub async fn load(name: String, root_path: PathBuf) -> PoolResult<Self> {
        let agent_dir = root_path.join("agents").join(&name);
        let metadata_path = agent_dir.join(".agent.yaml");
        
        let metadata_content = fs::read_to_string(&metadata_path).await?;
        let metadata: AgentMetadata = serde_yaml::from_str(&metadata_content)?;
        
        // Initialize logger if agent is active
        let logger = if matches!(metadata.tmux.status, AgentStatus::Active) {
            let log_dir = agent_dir.join("logs");
            AgentLogger::new(name.clone(), log_dir).ok()
        } else {
            None
        };
        
        Ok(Self {
            name,
            root_path,
            metadata,
            logger,
        })
    }
    
    /// Save agent metadata to disk
    pub async fn save(&self) -> PoolResult<()> {
        let agent_dir = self.get_agent_dir();
        fs::create_dir_all(&agent_dir).await?;
        
        let metadata_path = agent_dir.join(".agent.yaml");
        let metadata_content = serde_yaml::to_string(&self.metadata)?;
        fs::write(&metadata_path, metadata_content).await?;
        
        Ok(())
    }
    
    /// Initialize agent directory structure
    pub async fn init_directories(&self) -> PoolResult<()> {
        let agent_dir = self.get_agent_dir();
        
        // Create directories
        fs::create_dir_all(&agent_dir).await?;
        fs::create_dir_all(agent_dir.join("workspace")).await?;
        fs::create_dir_all(agent_dir.join("logs")).await?;
        fs::create_dir_all(agent_dir.join(".claude")).await?;
        
        // Create Claude settings
        let claude_settings = serde_json::json!({
            "model": self.metadata.claude.model,
            "memory": self.metadata.claude.memory_enabled,
            "tools": self.metadata.claude.tools,
        });
        
        let settings_path = agent_dir.join(".claude").join("settings.json");
        fs::write(&settings_path, serde_json::to_string_pretty(&claude_settings)?).await?;
        
        // Create system prompt if specified
        if let Some(prompt_file) = &self.metadata.claude.system_prompt_file {
            let prompt_path = agent_dir.join(".claude").join(prompt_file);
            if !prompt_path.exists() {
                fs::write(&prompt_path, "You are a helpful AI assistant.").await?;
            }
        }
        
        Ok(())
    }
    
    /// Start the agent's tmux session
    pub async fn start(&mut self) -> PoolResult<()> {
        let session_name = &self.metadata.tmux.session_name;
        
        // Create tmux session
        let output = Command::new("tmux")
            .args(&["new-session", "-d", "-s", session_name])
            .output()
            .map_err(|e| PoolError::Io(e))?;
            
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(PoolError::Tmux(format!("Failed to create session: {}", error)));
        }
        
        // Change to agent workspace
        let workspace = self.get_agent_dir().join("workspace");
        Command::new("tmux")
            .args(&["send-keys", "-t", session_name, &format!("cd {}", workspace.display()), "Enter"])
            .output()
            .map_err(|e| PoolError::Io(e))?;
        
        // Display welcome message
        let welcome = format!(
            r#"clear && echo -e '\033[1;36m
╔══════════════════════════════════════╗
║     AGENT: {}
║══════════════════════════════════════║
║  Role: {}
║  Created: {}
║  Created By: {}
║  Workspace: {}
╚══════════════════════════════════════╝
\033[0m'"#,
            self.name,
            self.metadata.agent.agent_type,
            self.metadata.agent.created_at.format("%Y-%m-%d %H:%M:%S"),
            self.metadata.agent.created_by,
            workspace.display()
        );
        
        Command::new("tmux")
            .args(&["send-keys", "-t", session_name, &welcome, "Enter"])
            .output()
            .map_err(|e| PoolError::Io(e))?;
        
        // Set up logging
        let log_dir = self.get_agent_dir().join("logs");
        let logger = AgentLogger::new(self.name.clone(), log_dir.clone())
            .map_err(|e| PoolError::Logging(format!("Failed to create logger: {}", e)))?;
        
        // Configure tmux to pipe output to logs
        setup_tmux_logging(session_name, &logger.session_log_path())
            .map_err(|e| PoolError::Logging(format!("Failed to setup tmux logging: {}", e)))?;
        
        // Log session start
        let _ = logger.log_system(
            "Agent session started",
            Some(serde_json::json!({
                "agent": self.name,
                "type": self.metadata.agent.agent_type,
                "model": self.metadata.claude.model,
                "workspace": workspace.display().to_string(),
            }))
        );
        
        // Source environment variables if they exist
        let env_file = self.get_agent_dir().join(".env");
        if env_file.exists() {
            Command::new("tmux")
                .args(&["send-keys", "-t", session_name, &format!("source {}", env_file.display()), "Enter"])
                .output()
                .map_err(|e| PoolError::Io(e))?;
            let _ = logger.log_system("Sourced environment variables", None);
        }
        
        // Run init script if it exists
        let init_script = self.get_agent_dir().join(".init.sh");
        if init_script.exists() {
            Command::new("tmux")
                .args(&["send-keys", "-t", session_name, &format!("bash {}", init_script.display()), "Enter"])
                .output()
                .map_err(|e| PoolError::Io(e))?;
            let _ = logger.log_system("Executed init script", None);
        }
        
        self.logger = Some(logger);
        
        // Update status
        self.metadata.tmux.status = AgentStatus::Active;
        self.metadata.agent.last_active = chrono::Utc::now();
        self.save().await?;
        
        Ok(())
    }
    
    /// Stop the agent's tmux session
    pub async fn stop(&mut self) -> PoolResult<()> {
        let session_name = &self.metadata.tmux.session_name;
        
        // Log stop event
        if let Some(logger) = &self.logger {
            let _ = logger.log_system(
                "Agent session stopping",
                Some(serde_json::json!({
                    "agent": self.name,
                    "uptime_seconds": chrono::Utc::now().signed_duration_since(self.metadata.agent.last_active).num_seconds(),
                }))
            );
        }
        
        // Check if session exists
        let check_output = Command::new("tmux")
            .args(&["has-session", "-t", session_name])
            .output()
            .map_err(|e| PoolError::Io(e))?;
            
        if check_output.status.success() {
            // Kill the session
            Command::new("tmux")
                .args(&["kill-session", "-t", session_name])
                .output()
                .map_err(|e| PoolError::Io(e))?;
        }
        
        self.metadata.tmux.status = AgentStatus::Stopped;
        self.metadata.agent.last_active = chrono::Utc::now();
        self.save().await?;
        
        // Clear logger
        self.logger = None;
        
        Ok(())
    }
    
    /// Check if the agent is running
    pub fn is_running(&self) -> PoolResult<bool> {
        let session_name = &self.metadata.tmux.session_name;
        
        let output = Command::new("tmux")
            .args(&["has-session", "-t", session_name])
            .output()
            .map_err(|e| PoolError::Io(e))?;
            
        Ok(output.status.success())
    }
    
    /// Get agent directory path
    pub fn get_agent_dir(&self) -> PathBuf {
        self.root_path.join("agents").join(&self.name)
    }
    
    /// Update task information
    pub async fn update_task(&mut self, task: Option<String>) -> PoolResult<()> {
        if let Some(_current) = self.metadata.tasks.current.take() {
            // Move current task to completed
            self.metadata.tasks.completed += 1;
        }
        
        self.metadata.tasks.current = task;
        self.metadata.agent.last_active = chrono::Utc::now();
        self.save().await?;
        
        Ok(())
    }
    
    /// Add task to queue
    pub async fn queue_task(&mut self, task: String) -> PoolResult<()> {
        self.metadata.tasks.queue.push(task);
        self.save().await?;
        Ok(())
    }
}