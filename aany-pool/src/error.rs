use thiserror::Error;

#[derive(Error, Debug)]
pub enum PoolError {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Agent already exists: {0}")]
    AgentAlreadyExists(String),
    
    #[error("Pool is full (max agents: {0})")]
    PoolFull(usize),
    
    #[error("Invalid agent name: {0}")]
    InvalidAgentName(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("TMux error: {0}")]
    Tmux(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Logging error: {0}")]
    Logging(String),
}

pub type PoolResult<T> = Result<T, PoolError>;