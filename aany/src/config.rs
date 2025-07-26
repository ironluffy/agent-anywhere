use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;
use directories::BaseDirs;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    #[serde(default)]
    pub api: ApiConfig,
    
    #[serde(default)]
    pub defaults: DefaultsConfig,
    
    #[serde(default)]
    pub logging: LoggingConfig,
    
    #[serde(default)]
    pub resources: ResourceConfig,
    
    #[serde(default)]
    pub features: FeaturesConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ApiConfig {
    #[serde(default)]
    pub claude: ClaudeApiConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeApiConfig {
    pub api_key: Option<String>,
    pub default_model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl Default for ClaudeApiConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            default_model: "claude-3-opus-20240229".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DefaultsConfig {
    #[serde(default)]
    pub agent: AgentDefaults,
    
    #[serde(default)]
    pub tmux: TmuxDefaults,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentDefaults {
    #[serde(rename = "type")]
    pub agent_type: String,
    pub memory_enabled: bool,
    pub auto_save_interval: u64,
    pub max_workspace_size: String,
}

impl Default for AgentDefaults {
    fn default() -> Self {
        Self {
            agent_type: "general".to_string(),
            memory_enabled: true,
            auto_save_interval: 300,
            max_workspace_size: "1GB".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TmuxDefaults {
    pub default_shell: String,
    pub scrollback_lines: u32,
    pub mouse_support: bool,
}

impl Default for TmuxDefaults {
    fn default() -> Self {
        Self {
            default_shell: "/bin/bash".to_string(),
            scrollback_lines: 10000,
            mouse_support: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub max_file_size: String,
    pub retention_days: u32,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            max_file_size: "10MB".to_string(),
            retention_days: 30,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub max_agents: u32,
    pub cpu_limit_per_agent: String,
    pub memory_limit_per_agent: String,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            max_agents: 10,
            cpu_limit_per_agent: "2".to_string(),
            memory_limit_per_agent: "4GB".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeaturesConfig {
    pub auto_backup: bool,
    pub backup_interval: u64,
    pub health_check_interval: u64,
    pub auto_cleanup_crashed: bool,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            auto_backup: true,
            backup_interval: 3600,
            health_check_interval: 60,
            auto_cleanup_crashed: true,
        }
    }
}

impl GlobalConfig {
    /// Load config from default location (~/.aany/config.yaml)
    pub fn load() -> Result<Self> {
        let config_path = Self::default_path()?;
        
        if !config_path.exists() {
            // Create default config if it doesn't exist
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        
        let content = std::fs::read_to_string(&config_path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save config to default location
    pub fn save(&self) -> Result<()> {
        let config_path = Self::default_path()?;
        
        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = serde_yaml::to_string(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
    
    /// Get default config path
    fn default_path() -> Result<PathBuf> {
        if let Some(base_dirs) = BaseDirs::new() {
            Ok(base_dirs.home_dir().join(".aany").join("config.yaml"))
        } else {
            Ok(PathBuf::from("./.aany/config.yaml"))
        }
    }
    
    /// Get a value by key path (e.g., "api.claude.model")
    pub fn get(&self, key_path: &str) -> Option<String> {
        let parts: Vec<&str> = key_path.split('.').collect();
        
        match parts.as_slice() {
            ["api", "claude", "api_key"] => self.api.claude.api_key.clone(),
            ["api", "claude", "default_model"] => Some(self.api.claude.default_model.clone()),
            ["api", "claude", "max_tokens"] => Some(self.api.claude.max_tokens.to_string()),
            ["api", "claude", "temperature"] => Some(self.api.claude.temperature.to_string()),
            ["logging", "level"] => Some(self.logging.level.clone()),
            ["resources", "max_agents"] => Some(self.resources.max_agents.to_string()),
            _ => None,
        }
    }
    
    /// Set a value by key path
    pub fn set(&mut self, key_path: &str, value: String) -> Result<()> {
        let parts: Vec<&str> = key_path.split('.').collect();
        
        match parts.as_slice() {
            ["api", "claude", "api_key"] => self.api.claude.api_key = Some(value),
            ["api", "claude", "default_model"] => self.api.claude.default_model = value,
            ["api", "claude", "max_tokens"] => self.api.claude.max_tokens = value.parse()?,
            ["api", "claude", "temperature"] => self.api.claude.temperature = value.parse()?,
            ["logging", "level"] => self.logging.level = value,
            ["resources", "max_agents"] => self.resources.max_agents = value.parse()?,
            _ => anyhow::bail!("Unknown configuration key: {}", key_path),
        }
        
        Ok(())
    }
}