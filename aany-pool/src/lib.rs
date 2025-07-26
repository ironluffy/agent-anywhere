pub mod agent;
pub mod manager;
pub mod metadata;
pub mod error;
pub mod template;
pub mod logging;

pub use agent::Agent;
pub use manager::PoolManager;
pub use metadata::{AgentMetadata, PoolConfig};
pub use error::{PoolError, PoolResult};
pub use template::{AgentTemplate, TemplateManager};
pub use logging::{AgentLogger, LogEvent, LogDirection};