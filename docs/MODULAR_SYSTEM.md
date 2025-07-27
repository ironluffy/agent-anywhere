# 🔌 Agent Anywhere Modular System

## Overview

The modular system allows tmux-agent to be extended with plugins that can hook into various lifecycle events. The first module is `hub-connector`, which enables agents to send logs to the centralized aany-hub platform.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      ModularTmuxProxy                       │
├─────────────────────────────────────────────────────────────┤
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────┐ │
│  │  TmuxProxy      │  │ ModuleRegistry │  │   Runtime    │ │
│  │  (Core tmux)    │  │  (Plugin mgr)  │  │  (Async RT)  │ │
│  └────────────────┘  └────────────────┘  └──────────────┘ │
│                              │                              │
│                              ▼                              │
│                    ┌─────────────────┐                      │
│                    │ TmuxAgentModule │                      │
│                    │   (trait/API)   │                      │
│                    └─────────────────┘                      │
│                              │                              │
│         ┌────────────────────┴──────────────────────┐      │
│         ▼                                           ▼      │
│  ┌──────────────┐                          ┌──────────────┐│
│  │HubConnector  │                          │Future Module ││
│  │   Module     │                          │  (e.g. MCP) ││
│  └──────────────┘                          └──────────────┘│
└─────────────────────────────────────────────────────────────┘
```

## Module API

Every module must implement the `TmuxAgentModule` trait:

```rust
#[async_trait]
pub trait TmuxAgentModule: Send + Sync {
    fn name(&self) -> &str;
    
    async fn initialize(&mut self, config: &HashMap<String, Value>) 
        -> Result<(), Box<dyn std::error::Error>>;
    
    // Lifecycle hooks
    async fn on_agent_start(&mut self, context: &AgentContext) -> Result<(), Box<dyn std::error::Error>>;
    async fn on_agent_stop(&mut self, context: &AgentContext) -> Result<(), Box<dyn std::error::Error>>;
    
    // Command hooks  
    async fn pre_command(&mut self, command: &str, context: &AgentContext) 
        -> Result<String, Box<dyn std::error::Error>>;
    async fn post_command(&mut self, result: &CommandResult, context: &AgentContext) 
        -> Result<(), Box<dyn std::error::Error>>;
    
    // Event hooks
    async fn on_output(&mut self, output: &str, context: &AgentContext) -> Result<(), Box<dyn std::error::Error>>;
    async fn on_error(&mut self, error: &str, context: &AgentContext) -> Result<(), Box<dyn std::error::Error>>;
}
```

## Hub Connector Module

The `hub-connector` module enables agents to:
- 📊 Send logs to the centralized hub
- 💓 Send heartbeats for liveness monitoring  
- 🔍 Track command execution and results
- ⚠️ Report errors centrally

### Configuration

```rust
let mut modules_config = HashMap::new();

// Hub connector configuration
let hub_config = serde_json::json!({
    "enabled": true,
    "hub_url": "localhost:50052",
});
modules_config.insert("hub_connector".to_string(), hub_config);

let config = ModularProxyConfig {
    session_name: "my-agent".to_string(),
    window: 0,
    pane: 0,
    agent_id: "agent-001".to_string(),
    modules_config,
};

let mut proxy = ModularTmuxProxy::new(config)?;
```

## Usage Example

```rust
// Commands are automatically logged to hub
proxy.send_command("echo 'Hello World'")?;

// Errors are reported
proxy.report_error("Something went wrong");

// Modules handle cleanup on shutdown
drop(proxy); // Calls on_agent_stop
```

## Future Modules

The modular system is designed to support:
- **MCP Module**: Model Context Protocol integration
- **Security Module**: Command filtering and sandboxing
- **Metrics Module**: Performance tracking
- **Storage Module**: Command history and replay
- **Automation Module**: Scripted workflows