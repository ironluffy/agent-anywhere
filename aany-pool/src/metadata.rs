use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Get git user info in the format "Name <email>"
pub fn get_git_user_info() -> String {
    "Anonymous <anonymous@localhost>".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub agent: AgentInfo,
    pub tmux: TmuxInfo,
    pub claude: ClaudeConfig,
    pub resources: ResourceConfig,
    pub tasks: TaskInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    #[serde(default = "get_git_user_info")]
    pub created_by: String,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxInfo {
    pub session_name: String,
    pub status: AgentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Active,
    Stopped,
    Crashed,
    Starting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    pub model: String,
    pub system_prompt_file: Option<String>,
    pub memory_enabled: bool,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub working_directory: PathBuf,
    pub log_directory: PathBuf,
    pub max_context_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub current: Option<String>,
    pub completed: usize,
    pub queue: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub pool: PoolSettings,
    pub defaults: DefaultSettings,
    pub monitoring: MonitoringSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolSettings {
    pub name: String,
    pub root: PathBuf,
    pub max_agents: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultSettings {
    pub claude_model: String,
    pub max_context_size: usize,
    pub auto_restart: bool,
    pub log_retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSettings {
    pub health_check_interval: u64,
    pub alert_on_crash: bool,
    pub webhook_url: Option<String>,
}

impl Default for AgentMetadata {
    fn default() -> Self {
        Self {
            agent: AgentInfo {
                name: "unnamed".to_string(),
                agent_type: "general".to_string(),
                description: "General purpose agent".to_string(),
                created_at: Utc::now(),
                created_by: get_git_user_info(),
                last_active: Utc::now(),
            },
            tmux: TmuxInfo {
                session_name: "agent-unnamed".to_string(),
                status: AgentStatus::Stopped,
            },
            claude: ClaudeConfig {
                model: "claude-3-opus-20240229".to_string(),
                system_prompt_file: None,
                memory_enabled: true,
                tools: vec!["file_operations".to_string()],
            },
            resources: ResourceConfig {
                working_directory: PathBuf::from("./workspace"),
                log_directory: PathBuf::from("./logs"),
                max_context_size: 200_000,
            },
            tasks: TaskInfo {
                current: None,
                completed: 0,
                queue: Vec::new(),
            },
        }
    }
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            pool: PoolSettings {
                name: "AI Agent Pool".to_string(),
                root: PathBuf::from("./agent-pool"),
                max_agents: 10,
            },
            defaults: DefaultSettings {
                claude_model: "claude-3-opus-20240229".to_string(),
                max_context_size: 200_000,
                auto_restart: true,
                log_retention_days: 30,
            },
            monitoring: MonitoringSettings {
                health_check_interval: 60,
                alert_on_crash: true,
                webhook_url: None,
            },
        }
    }
}