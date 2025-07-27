// Modular tmux proxy that supports plugins/modules

use crate::modules::{ModuleRegistry, AgentContext, CommandResult, TmuxAgentModule};
use crate::TmuxProxy;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use tokio::runtime::Runtime;
use std::sync::{Arc, Mutex};

/// Configuration for modular proxy
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ModularProxyConfig {
    pub session_name: String,
    pub window: u32,
    pub pane: u32,
    pub agent_id: String,
    pub modules_config: HashMap<String, serde_json::Value>,
}

/// Tmux proxy with module support
pub struct ModularTmuxProxy {
    proxy: TmuxProxy,
    config: ModularProxyConfig,
    module_registry: ModuleRegistry,
    runtime: Runtime,
    interaction_logger: Arc<Mutex<Option<crate::modules::interaction_logger::InteractionLoggerModule>>>,
}

impl ModularTmuxProxy {
    /// Create new modular proxy
    pub fn new(config: ModularProxyConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let proxy = TmuxProxy::new(
            config.session_name.clone(),
            config.window,
            config.pane,
        );
        
        let runtime = Runtime::new()?;
        let mut module_registry = ModuleRegistry::new();
        let interaction_logger = Arc::new(Mutex::new(None));
        
        // Initialize modules based on config
        for (module_name, module_config) in &config.modules_config {
            match module_name.as_str() {
                "hub_connector" => {
                    let mut hub_module = crate::modules::hub_connector::HubConnectorModule::new();
                    let mut cfg = HashMap::new();
                    if let serde_json::Value::Object(map) = module_config {
                        for (k, v) in map {
                            cfg.insert(k.clone(), v.clone());
                        }
                    }
                    cfg.insert("agent_id".to_string(), serde_json::Value::String(config.agent_id.clone()));
                    runtime.block_on(hub_module.initialize(&cfg))?;
                    module_registry.register(Box::new(hub_module));
                }
                "autocorrect" => {
                    let mut autocorrect_module = crate::modules::autocorrect::AutoCorrectModule::new();
                    let mut cfg = HashMap::new();
                    if let serde_json::Value::Object(map) = module_config {
                        for (k, v) in map {
                            cfg.insert(k.clone(), v.clone());
                        }
                    }
                    runtime.block_on(autocorrect_module.initialize(&cfg))?;
                    module_registry.register(Box::new(autocorrect_module));
                }
                "interaction_logger" => {
                    let mut logger_module = crate::modules::interaction_logger::InteractionLoggerModule::new();
                    let mut cfg = HashMap::new();
                    if let serde_json::Value::Object(map) = module_config {
                        for (k, v) in map {
                            cfg.insert(k.clone(), v.clone());
                        }
                    }
                    cfg.insert("agent_id".to_string(), serde_json::Value::String(config.agent_id.clone()));
                    cfg.insert("session_name".to_string(), serde_json::Value::String(config.session_name.clone()));
                    runtime.block_on(logger_module.initialize(&cfg))?;
                    let logger_clone = logger_module.clone();
                    module_registry.register(Box::new(logger_module));
                    *interaction_logger.lock().unwrap() = Some(logger_clone);
                }
                // "terminal_broadcaster" => {
                //     let mut terminal_module = crate::modules::terminal_broadcaster::TerminalBroadcasterModule::new();
                //     let mut cfg = HashMap::new();
                //     if let serde_json::Value::Object(map) = module_config {
                //         for (k, v) in map {
                //             cfg.insert(k.clone(), v.clone());
                //         }
                //     }
                //     cfg.insert("agent_id".to_string(), serde_json::Value::String(config.agent_id.clone()));
                //     runtime.block_on(terminal_module.initialize(&cfg))?;
                //     module_registry.register(Box::new(terminal_module));
                // }
                _ => {
                    eprintln!("Unknown module: {}", module_name);
                }
            }
        }
        
        // Notify modules of agent start
        let context = Self::create_context(&config.session_name, None);
        runtime.block_on(module_registry.on_agent_start(&context));
        
        Ok(Self {
            proxy,
            config,
            module_registry,
            runtime,
            interaction_logger,
        })
    }
    
    /// Create context for module calls
    fn create_context(session_name: &str, metadata: Option<HashMap<String, String>>) -> AgentContext {
        AgentContext {
            session_name: session_name.to_string(),
            window_id: None,
            pane_id: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            metadata: metadata.unwrap_or_default(),
        }
    }
    
    /// Send command with module hooks
    pub fn send_command(&mut self, command: &str) -> Result<String, Box<dyn std::error::Error>> {
        let context = Self::create_context(&self.config.session_name, None);
        
        // Log user input before processing
        if let Ok(logger_guard) = self.interaction_logger.lock() {
            if let Some(logger) = logger_guard.as_ref() {
                self.runtime.block_on(logger.log_user_input(command)).ok();
            }
        }
        
        // Pre-command hook - allow modules to modify command
        let modified_command = self.runtime.block_on(
            self.module_registry.pre_command(command, &context)
        );
        
        // Execute command
        let start = Instant::now();
        let result = self.proxy.send_line(&modified_command)?;
        let duration_ms = start.elapsed().as_millis() as u64;
        
        // Wait for output
        std::thread::sleep(std::time::Duration::from_millis(100));
        let output = self.proxy.capture_pane()?;
        
        // Log screen content after command
        if let Ok(logger_guard) = self.interaction_logger.lock() {
            if let Some(logger) = logger_guard.as_ref() {
                self.runtime.block_on(logger.capture_screen_content(output.clone())).ok();
            }
        }
        
        // Create command result
        let cmd_result = CommandResult {
            command: modified_command.clone(),
            output: output.clone(),
            exit_code: if result.status.success() { 0 } else { 1 },
            duration_ms,
        };
        
        // Post-command hook
        self.runtime.block_on(
            self.module_registry.post_command(&cmd_result, &context)
        );
        
        Ok(output)
    }
    
    /// Report error to modules
    pub fn report_error(&mut self, error: &str) {
        let context = Self::create_context(&self.config.session_name, None);
        self.runtime.block_on(
            self.module_registry.on_error(error, &context)
        );
    }
    
    /// Get underlying proxy for direct access
    pub fn proxy(&self) -> &TmuxProxy {
        &self.proxy
    }
    
    /// Send keys (raw input) with logging
    pub fn send_keys(&mut self, keys: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Log user input
        if let Ok(logger_guard) = self.interaction_logger.lock() {
            if let Some(logger) = logger_guard.as_ref() {
                self.runtime.block_on(logger.log_user_input(keys)).ok();
            }
        }
        
        // Send the keys
        self.proxy.send_keys(keys)?;
        
        // Capture screen after a short delay
        std::thread::sleep(std::time::Duration::from_millis(50));
        if let Ok(content) = self.proxy.capture_pane() {
            if let Ok(logger_guard) = self.interaction_logger.lock() {
                if let Some(logger) = logger_guard.as_ref() {
                    self.runtime.block_on(logger.capture_screen_content(content)).ok();
                }
            }
        }
        
        Ok(())
    }
    
    /// Capture current pane content with logging
    pub fn capture_pane(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let content = self.proxy.capture_pane()?;
        
        // Log the captured content
        if let Ok(logger_guard) = self.interaction_logger.lock() {
            if let Some(logger) = logger_guard.as_ref() {
                self.runtime.block_on(logger.capture_screen_content(content.clone())).ok();
            }
        }
        
        Ok(content)
    }
    
    /// Cleanup on drop
    pub fn shutdown(&mut self) {
        let context = Self::create_context(&self.config.session_name, None);
        self.runtime.block_on(
            self.module_registry.on_agent_stop(&context)
        );
    }
}

impl Drop for ModularTmuxProxy {
    fn drop(&mut self) {
        self.shutdown();
    }
}