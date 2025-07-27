use crate::{agent::Agent, metadata::*, error::*, template::TemplateManager};
use std::path::PathBuf;
use std::collections::HashMap;
use tokio::fs;
use directories::BaseDirs;

pub struct PoolManager {
    pub root_path: PathBuf,
    pub config: PoolConfig,
    agents: HashMap<String, Agent>,
}

impl PoolManager {
    /// Create a new pool manager
    pub async fn new(root_path: PathBuf) -> PoolResult<Self> {
        // Ensure root directory exists
        fs::create_dir_all(&root_path).await?;
        
        // Load or create pool config
        let config_path = root_path.join(".pool.yaml");
        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path).await?;
            serde_yaml::from_str(&content)?
        } else {
            let default_config = PoolConfig::default();
            let content = serde_yaml::to_string(&default_config)?;
            fs::write(&config_path, content).await?;
            default_config
        };
        
        let mut manager = Self {
            root_path,
            config,
            agents: HashMap::new(),
        };
        
        // Load existing agents
        manager.load_agents().await?;
        
        Ok(manager)
    }
    
    /// Load all agents from disk
    async fn load_agents(&mut self) -> PoolResult<()> {
        let agents_dir = self.root_path.join("agents");
        if !agents_dir.exists() {
            fs::create_dir_all(&agents_dir).await?;
            return Ok(());
        }
        
        let mut entries = fs::read_dir(&agents_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                
                // Check if metadata exists before trying to load
                let agent_dir = self.root_path.join("agents").join(&name);
                let new_metadata_path = agent_dir.join("metadata").join("agent.yaml");
                let old_metadata_path = agent_dir.join(".agent.yaml");
                
                if new_metadata_path.exists() || old_metadata_path.exists() {
                    match Agent::load(name.clone(), self.root_path.clone()).await {
                        Ok(agent) => {
                            self.agents.insert(name, agent);
                        }
                        Err(e) => {
                            eprintln!("Failed to load agent {}: {}", name, e);
                        }
                    }
                }
                // Silently skip agents without metadata files
            }
        }
        
        Ok(())
    }
    
    /// Create a new agent
    pub async fn create_agent(&mut self, name: String, template: Option<String>) -> PoolResult<&Agent> {
        // Validate name
        if self.agents.contains_key(&name) {
            return Err(PoolError::AgentAlreadyExists(name));
        }
        
        if self.agents.len() >= self.config.pool.max_agents {
            return Err(PoolError::PoolFull(self.config.pool.max_agents));
        }
        
        // Create agent
        let mut agent = Agent::new(name.clone(), self.root_path.clone()).await?;
        
        // Apply template if specified
        if let Some(tpl) = template {
            self.apply_template(&mut agent, &tpl).await?;
        }
        
        // Initialize directories and save
        agent.init_directories().await?;
        agent.save().await?;
        
        // Add to manager
        self.agents.insert(name.clone(), agent);
        
        Ok(self.agents.get(&name).unwrap())
    }
    
    /// Apply a template to an agent
    async fn apply_template(&self, agent: &mut Agent, template_name: &str) -> PoolResult<()> {
        // First check pool-specific templates
        let pool_template_dir = self.root_path.join("templates");
        let mut template_manager = TemplateManager::new(pool_template_dir.clone());
        
        // If pool templates don't exist, check global templates
        if !pool_template_dir.exists() {
            if let Some(base_dirs) = BaseDirs::new() {
                let global_template_dir = base_dirs.home_dir().join(".aany").join("templates");
                template_manager = TemplateManager::new(global_template_dir);
            }
        }
        
        match template_manager.get_template(template_name) {
            Ok(template) => {
                let name = agent.metadata.agent.name.clone();
                agent.metadata = template.to_metadata(name);
                
                // Run init commands
                if !template.init_commands.is_empty() {
                    let workspace = agent.get_agent_dir().join("workspace");
                    fs::create_dir_all(&workspace).await?;
                    
                    // Store init commands for later execution when session starts
                    let init_script = template.init_commands.join("\n");
                    let script_path = agent.get_agent_dir().join(".init.sh");
                    fs::write(&script_path, init_script).await?;
                }
                
                // Set environment variables in agent metadata
                if !template.environment.is_empty() {
                    // Store environment in a file for later sourcing
                    let mut env_content = String::new();
                    for (key, value) in &template.environment {
                        env_content.push_str(&format!("export {}=\"{}\"\n", key, value));
                    }
                    let env_path = agent.get_agent_dir().join(".env");
                    fs::write(&env_path, env_content).await?;
                }
            }
            Err(_) => {
                // Template not found, use default
                eprintln!("Warning: Template '{}' not found, using defaults", template_name);
            }
        }
        
        Ok(())
    }
    
    /// Get an agent by name
    pub fn get_agent(&self, name: &str) -> PoolResult<&Agent> {
        self.agents.get(name)
            .ok_or_else(|| PoolError::AgentNotFound(name.to_string()))
    }
    
    /// Get a mutable agent by name
    pub fn get_agent_mut(&mut self, name: &str) -> PoolResult<&mut Agent> {
        self.agents.get_mut(name)
            .ok_or_else(|| PoolError::AgentNotFound(name.to_string()))
    }
    
    /// Save the pool configuration
    pub async fn save_config(&self) -> PoolResult<()> {
        let config_path = self.root_path.join(".pool.yaml");
        let content = serde_yaml::to_string(&self.config)?;
        fs::write(&config_path, content).await?;
        Ok(())
    }
    
    /// Update last used environment variables
    pub async fn update_last_env_vars(&mut self, env_vars: Vec<(String, String)>) -> PoolResult<()> {
        self.config.defaults.last_env_vars = env_vars.into_iter().collect();
        self.save_config().await?;
        Ok(())
    }
    
    /// Get last used environment variables
    pub fn get_last_env_vars(&self) -> Vec<(String, String)> {
        self.config.defaults.last_env_vars.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
    
    /// List all agents sorted by created_at (newest first)
    pub fn list_agents(&self) -> Vec<&Agent> {
        let mut agents: Vec<&Agent> = self.agents.values().collect();
        agents.sort_by(|a, b| b.metadata.agent.created_at.cmp(&a.metadata.agent.created_at));
        agents
    }
    
    /// Delete an agent
    pub async fn delete_agent(&mut self, name: &str) -> PoolResult<()> {
        // Stop agent if running
        if let Ok(agent) = self.get_agent_mut(name) {
            agent.stop().await?;
        }
        
        // Remove from manager
        self.agents.remove(name);
        
        // Delete from disk
        let agent_dir = self.root_path.join("agents").join(name);
        if agent_dir.exists() {
            fs::remove_dir_all(&agent_dir).await?;
        }
        
        Ok(())
    }
    
    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        let mut stats = PoolStats::default();
        stats.total_agents = self.agents.len();
        
        for agent in self.agents.values() {
            match agent.metadata.tmux.status {
                AgentStatus::Active => stats.active_agents += 1,
                AgentStatus::Stopped => stats.stopped_agents += 1,
                AgentStatus::Crashed => stats.crashed_agents += 1,
                AgentStatus::Starting => stats.starting_agents += 1,
            }
            stats.total_tasks_completed += agent.metadata.tasks.completed;
        }
        
        stats
    }
}

#[derive(Debug, Default)]
pub struct PoolStats {
    pub total_agents: usize,
    pub active_agents: usize,
    pub stopped_agents: usize,
    pub crashed_agents: usize,
    pub starting_agents: usize,
    pub total_tasks_completed: usize,
}