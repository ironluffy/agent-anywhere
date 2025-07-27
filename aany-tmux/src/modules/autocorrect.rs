// Auto-correction module for tmux-agent

use super::{AgentContext, TmuxAgentModule};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// Common typos and their corrections
struct TypoDatabase {
    corrections: HashMap<String, String>,
}

impl TypoDatabase {
    fn new() -> Self {
        let mut corrections = HashMap::new();
        
        // Common command typos
        corrections.insert("ehco".to_string(), "echo".to_string());
        corrections.insert("ech".to_string(), "echo".to_string());
        corrections.insert("ecoh".to_string(), "echo".to_string());
        corrections.insert("sl".to_string(), "ls".to_string());
        corrections.insert("lls".to_string(), "ls".to_string());
        corrections.insert("cd..".to_string(), "cd ..".to_string());
        corrections.insert("gti".to_string(), "git".to_string());
        corrections.insert("gi".to_string(), "git".to_string());
        corrections.insert("pyhton".to_string(), "python".to_string());
        corrections.insert("pytohn".to_string(), "python".to_string());
        corrections.insert("suod".to_string(), "sudo".to_string());
        corrections.insert("sduo".to_string(), "sudo".to_string());
        corrections.insert("gerp".to_string(), "grep".to_string());
        corrections.insert("grpe".to_string(), "grep".to_string());
        corrections.insert("mkae".to_string(), "make".to_string());
        corrections.insert("amke".to_string(), "make".to_string());
        corrections.insert("celar".to_string(), "clear".to_string());
        corrections.insert("claer".to_string(), "clear".to_string());
        
        Self { corrections }
    }
    
    fn check_and_correct(&self, word: &str) -> Option<String> {
        self.corrections.get(word).cloned()
    }
}

/// Auto-correction module
pub struct AutoCorrectModule {
    name: String,
    enabled: bool,
    typo_db: TypoDatabase,
    corrections_made: u32,
    log_corrections: bool,
}

impl AutoCorrectModule {
    pub fn new() -> Self {
        Self {
            name: "autocorrect".to_string(),
            enabled: true,
            typo_db: TypoDatabase::new(),
            corrections_made: 0,
            log_corrections: true,
        }
    }
    
    fn correct_command(&mut self, command: &str) -> (String, Vec<(String, String)>) {
        let mut corrected = command.to_string();
        let mut corrections = Vec::new();
        
        // Split command into words
        let words: Vec<&str> = command.split_whitespace().collect();
        
        // Check each word for typos
        for (i, word) in words.iter().enumerate() {
            if let Some(correction) = self.typo_db.check_and_correct(word) {
                if i == 0 {
                    // First word is likely a command
                    corrected = corrected.replace(word, &correction);
                    corrections.push((word.to_string(), correction));
                    self.corrections_made += 1;
                }
            }
        }
        
        (corrected, corrections)
    }
}

#[async_trait]
impl TmuxAgentModule for AutoCorrectModule {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn initialize(&mut self, config: &HashMap<String, Value>) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(Value::Bool(enabled)) = config.get("enabled") {
            self.enabled = *enabled;
        }
        
        if let Some(Value::Bool(log)) = config.get("log_corrections") {
            self.log_corrections = *log;
        }
        
        println!("Auto-correct module initialized (enabled: {})", self.enabled);
        Ok(())
    }
    
    async fn pre_command(&mut self, command: &str, _context: &AgentContext) -> Result<String, Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(command.to_string());
        }
        
        let (corrected, corrections) = self.correct_command(command);
        
        if !corrections.is_empty() && self.log_corrections {
            for (typo, correction) in corrections {
                println!("🔧 Auto-corrected: '{}' → '{}'", typo, correction);
            }
        }
        
        Ok(corrected)
    }
    
    async fn on_agent_stop(&mut self, _context: &AgentContext) -> Result<(), Box<dyn std::error::Error>> {
        if self.corrections_made > 0 {
            println!("📊 Auto-correct stats: {} corrections made", self.corrections_made);
        }
        Ok(())
    }
}