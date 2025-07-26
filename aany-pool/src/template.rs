use crate::metadata::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentTemplate {
    pub agent: AgentInfo,
    pub claude: ClaudeConfig,
    
    #[serde(default)]
    pub init_commands: Vec<String>,
    
    #[serde(default)]
    pub environment: HashMap<String, String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceConfig>,
}

impl AgentTemplate {
    /// Load a template from file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let template: Self = serde_yaml::from_str(&content)?;
        Ok(template)
    }
    
    /// Save template to file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Convert template to agent metadata
    pub fn to_metadata(&self, name: String) -> AgentMetadata {
        let mut metadata = AgentMetadata::default();
        
        // Apply template values
        metadata.agent = self.agent.clone();
        metadata.claude = self.claude.clone();
        
        // Apply resources if provided
        if let Some(resources) = &self.resources {
            metadata.resources = resources.clone();
        }
        
        // Override name
        metadata.agent.name = name.clone();
        metadata.tmux.session_name = format!("agent-{}", name);
        
        metadata
    }
}

pub struct TemplateManager {
    template_dir: PathBuf,
}

impl TemplateManager {
    pub fn new(template_dir: PathBuf) -> Self {
        Self { template_dir }
    }
    
    /// List available templates
    pub fn list_templates(&self) -> Result<Vec<String>> {
        let mut templates = Vec::new();
        
        if !self.template_dir.exists() {
            return Ok(templates);
        }
        
        for entry in std::fs::read_dir(&self.template_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension() == Some("yaml".as_ref()) || 
               path.extension() == Some("yml".as_ref()) {
                if let Some(stem) = path.file_stem() {
                    templates.push(stem.to_string_lossy().to_string());
                }
            }
        }
        
        Ok(templates)
    }
    
    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Result<AgentTemplate> {
        let mut path = self.template_dir.join(name);
        
        // Try with .yaml extension first
        if !path.exists() {
            path.set_extension("yaml");
        }
        
        // Try with .yml extension
        if !path.exists() {
            path.set_extension("yml");
        }
        
        if !path.exists() {
            anyhow::bail!("Template not found: {}", name);
        }
        
        AgentTemplate::load_from_file(&path)
    }
    
    /// Create default templates
    pub fn create_default_templates(&self) -> Result<()> {
        std::fs::create_dir_all(&self.template_dir)?;
        
        // General template
        let general = AgentTemplate {
            agent: AgentInfo {
                name: "template".to_string(),
                agent_type: "general".to_string(),
                description: "General purpose agent".to_string(),
                created_at: chrono::Utc::now(),
                created_by: crate::metadata::get_git_user_info(),
                last_active: chrono::Utc::now(),
            },
            claude: ClaudeConfig {
                model: "claude-3-opus-20240229".to_string(),
                memory_enabled: true,
                system_prompt_file: None,
                tools: vec![],
            },
            init_commands: vec![],
            environment: HashMap::new(),
            resources: None,
        };
        
        general.save_to_file(&self.template_dir.join("general.yaml"))?;
        
        // Research template
        let research = AgentTemplate {
            agent: AgentInfo {
                name: "template".to_string(),
                agent_type: "research".to_string(),
                description: "Autonomous research assistant".to_string(),
                created_at: chrono::Utc::now(),
                created_by: crate::metadata::get_git_user_info(),
                last_active: chrono::Utc::now(),
            },
            claude: ClaudeConfig {
                model: "claude-3-opus-20240229".to_string(),
                memory_enabled: true,
                system_prompt_file: Some("research_prompt.md".to_string()),
                tools: vec!["web_search".to_string(), "file_read".to_string()],
            },
            init_commands: vec![
                "mkdir -p research/{sources,notes,reports}".to_string(),
                "echo '# Research Log' > research/README.md".to_string(),
            ],
            environment: HashMap::from([
                ("RESEARCH_MODE".to_string(), "academic".to_string()),
                ("CITATION_STYLE".to_string(), "APA".to_string()),
            ]),
            resources: None,
        };
        
        research.save_to_file(&self.template_dir.join("research.yaml"))?;
        
        // Code template
        let code = AgentTemplate {
            agent: AgentInfo {
                name: "template".to_string(),
                agent_type: "code".to_string(),
                description: "Software development assistant".to_string(),
                created_at: chrono::Utc::now(),
                created_by: crate::metadata::get_git_user_info(),
                last_active: chrono::Utc::now(),
            },
            claude: ClaudeConfig {
                model: "claude-3-opus-20240229".to_string(),
                memory_enabled: true,
                system_prompt_file: Some("code_prompt.md".to_string()),
                tools: vec!["code_execution".to_string(), "git".to_string()],
            },
            init_commands: vec![
                "git init".to_string(),
                "echo '# Project' > README.md".to_string(),
            ],
            environment: HashMap::from([
                ("EDITOR".to_string(), "vim".to_string()),
                ("NODE_ENV".to_string(), "development".to_string()),
            ]),
            resources: None,
        };
        
        code.save_to_file(&self.template_dir.join("code.yaml"))?;
        
        Ok(())
    }
}