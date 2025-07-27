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
        let metadata_path = agent_dir.join("metadata").join("agent.yaml");
        
        // Try new location first, fall back to old location
        let metadata_content = if metadata_path.exists() {
            fs::read_to_string(&metadata_path).await?
        } else {
            // Fallback to old location for backward compatibility
            let old_path = agent_dir.join(".agent.yaml");
            fs::read_to_string(&old_path).await?
        };
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
        let metadata_dir = agent_dir.join("metadata");
        fs::create_dir_all(&metadata_dir).await?;
        
        let metadata_path = metadata_dir.join("agent.yaml");
        let metadata_content = serde_yaml::to_string(&self.metadata)?;
        fs::write(&metadata_path, metadata_content).await?;
        
        Ok(())
    }
    
    /// Initialize agent directory structure
    pub async fn init_directories(&self) -> PoolResult<()> {
        let agent_dir = self.get_agent_dir();
        
        // Create directories with new flat structure
        fs::create_dir_all(&agent_dir).await?;
        fs::create_dir_all(agent_dir.join("workspace")).await?;
        fs::create_dir_all(agent_dir.join("logs")).await?;
        fs::create_dir_all(agent_dir.join("metadata")).await?;
        fs::create_dir_all(agent_dir.join("config")).await?;
        
        // Create Claude settings in metadata directory
        let claude_settings = serde_json::json!({
            "model": self.metadata.claude.model,
            "memory": self.metadata.claude.memory_enabled,
            "tools": self.metadata.claude.tools,
        });
        
        let settings_path = agent_dir.join("metadata").join("claude.json");
        fs::write(&settings_path, serde_json::to_string_pretty(&claude_settings)?).await?;
        
        // Create system prompt if specified
        if let Some(prompt_file) = &self.metadata.claude.system_prompt_file {
            let prompt_path = agent_dir.join("config").join(prompt_file);
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
        
        // Add a small delay to ensure tmux session is ready
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Change to workspace and launch claude with nvm
        let workspace = self.get_workspace_dir();
        
        // Load environment variables from .env file if it exists
        let env_file = self.get_agent_dir().join(".env");
        let git_repo = if env_file.exists() {
            // Read .env file and look for GIT_REPO
            let env_content = std::fs::read_to_string(&env_file).unwrap_or_default();
            env_content.lines()
                .find(|line| line.starts_with("GIT_REPO="))
                .and_then(|line| line.strip_prefix("GIT_REPO="))
                .filter(|s| !s.is_empty() && !s.contains("Local repository"))
                .map(|s| s.to_string())
        } else {
            None
        };
        
        // Try to find claude wrapper script
        let claude_wrapper = self.find_script("claude-with-return.sh", "AANY_CLAUDE_WRAPPER");
        
        let startup_command = if let Some(git_repo) = git_repo {
            // If GIT_REPO is set, clone that repository
            if let Some(wrapper) = &claude_wrapper {
                format!("cd {} && git clone {} . && nvm use 22 && {}", workspace.display(), git_repo, wrapper)
            } else {
                format!("cd {} && git clone {} . && nvm use 22 && claude", workspace.display(), git_repo)
            }
        } else {
            // Default behavior - just change to workspace and start claude
            if let Some(wrapper) = &claude_wrapper {
                format!("cd {} && nvm use 22 && {}", workspace.display(), wrapper)
            } else {
                format!("cd {} && nvm use 22 && claude", workspace.display())
            }
        };
        
        // Log the command for debugging
        tracing::info!("Executing startup command: {}", startup_command);
        
        let output = Command::new("tmux")
            .args(&["send-keys", "-t", session_name, &startup_command, "Enter"])
            .output()
            .map_err(|e| PoolError::Io(e))?;
            
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            tracing::error!("Failed to send startup command: {}", error);
        }
        
        // Check if logging is disabled
        let logging_disabled = std::env::var("AANY_LOGGING_DISABLED")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);
        
        if !logging_disabled {
            let hub_url = std::env::var("AANY_HUB_URL").unwrap_or_else(|_| "localhost:50052".to_string());
            
            // Start all loggers for comprehensive logging
            // Try to find logger scripts
            let logger_script = self.find_script("tmux-logger.sh", "AANY_TMUX_LOGGER");
            let interaction_logger_script = self.find_script("tmux-interaction-logger.sh", "AANY_INTERACTION_LOGGER");
            let screenshot_logger_script = self.find_script("tmux-screenshot-rotating-logger.sh", "AANY_SCREENSHOT_LOGGER");
            
            // Start the original tmux logger
            if let Some(script) = &logger_script {
                Command::new("bash")
                    .args(&[script, session_name, &self.name])
                    .env("AANY_HUB_URL", &hub_url)
                    .env("GRPC_ENABLE_FORK_SUPPORT", "1")
                    .env("GRPC_POLL_STRATEGY", "poll")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .ok();
            } else {
                tracing::warn!("tmux-logger.sh not found, logging disabled");
            }
            
            // Start the interaction logger to capture all screen and user input
            if let Some(script) = &interaction_logger_script {
                Command::new("bash")
                    .args(&[script, session_name, &self.name])
                    .env("AANY_HUB_URL", &hub_url)
                    .env("GRPC_ENABLE_FORK_SUPPORT", "1")
                    .env("GRPC_POLL_STRATEGY", "poll")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .ok();
            } else {
                tracing::warn!("tmux-interaction-logger.sh not found, interaction logging disabled");
            }
            
            // Start the screenshot logger with 5s interval and 5 minute file rotation
            if let Some(script) = &screenshot_logger_script {
                Command::new("bash")
                    .args(&[script, session_name, &self.name, "5", "300"])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .ok();
            } else {
                tracing::warn!("tmux-screenshot-rotating-logger.sh not found, screenshot logging disabled");
            }
        }
        
        // Set up logging
        let log_dir = self.get_agent_dir().join("logs");
        let logger = if !logging_disabled {
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
            
            Some(logger)
        } else {
            None
        };
        
        // // Source environment variables if they exist
        // let env_file = self.get_agent_dir().join(".env");
        // if env_file.exists() {
        //     Command::new("tmux")
        //         .args(&["send-keys", "-t", session_name, &format!("source {}", env_file.display()), "Enter"])
        //         .output()
        //         .map_err(|e| PoolError::Io(e))?;
        //     let _ = logger.log_system("Sourced environment variables", None);
        // }
        
        // // Run init script if it exists
        // let init_script = self.get_agent_dir().join(".init.sh");
        // if init_script.exists() {
        //     Command::new("tmux")
        //         .args(&["send-keys", "-t", session_name, &format!("bash {}", init_script.display()), "Enter"])
        //         .output()
        //         .map_err(|e| PoolError::Io(e))?;
        //     let _ = logger.log_system("Executed init script", None);
        // }
        
        self.logger = logger;
        
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
    
    /// Get workspace directory path
    pub fn get_workspace_dir(&self) -> PathBuf {
        self.get_agent_dir().join("workspace")
    }
    
    /// Find a script file using multiple strategies
    fn find_script(&self, script_name: &str, env_var: &str) -> Option<String> {
        // 1. Check environment variable override
        if let Ok(path) = std::env::var(env_var) {
            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }
        
        // 2. Check AANY_REPO_PATH if available
        if let Ok(repo_path) = std::env::var("AANY_REPO_PATH") {
            let script_path = format!("{}/aany-tmux/{}", repo_path, script_name);
            if std::path::Path::new(&script_path).exists() {
                return Some(script_path);
            }
        }
        
        // 3. Try to find in PATH using which command
        if let Ok(output) = Command::new("which").arg(script_name).output() {
            if output.status.success() {
                if let Ok(path) = String::from_utf8(output.stdout) {
                    let path = path.trim();
                    if !path.is_empty() {
                        return Some(path.to_string());
                    }
                }
            }
        }
        
        // 4. Check common installation locations
        let common_paths = [
            format!("/usr/local/bin/{}", script_name),
            format!("/opt/homebrew/bin/{}", script_name),
            format!("{}/.local/bin/{}", std::env::var("HOME").unwrap_or_default(), script_name),
            format!("{}/bin/{}", std::env::var("HOME").unwrap_or_default(), script_name),
        ];
        
        for path in &common_paths {
            if std::path::Path::new(path).exists() {
                return Some(path.clone());
            }
        }
        
        // 5. Try relative to current executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // Check in same directory
                let same_dir = exe_dir.join(script_name);
                if same_dir.exists() {
                    return Some(same_dir.to_string_lossy().to_string());
                }
                
                // Check in ../aany-tmux/
                if let Some(parent) = exe_dir.parent() {
                    let tmux_dir = parent.join("aany-tmux").join(script_name);
                    if tmux_dir.exists() {
                        return Some(tmux_dir.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        tracing::warn!("Could not find script: {}", script_name);
        None
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