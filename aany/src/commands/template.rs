use clap::Subcommand;
use anyhow::Result;
use aany_pool::TemplateManager;
use std::path::PathBuf;
use directories::BaseDirs;

#[derive(Subcommand)]
pub enum TemplateCommands {
    /// List available templates
    List,
    
    /// Show template details
    Show {
        /// Template name
        name: String,
    },
    
    /// Create a new template
    Create {
        /// Template name
        name: String,
        
        /// Base template to copy from
        #[arg(short, long)]
        from: Option<String>,
    },
    
    /// Edit a template
    Edit {
        /// Template name
        name: String,
    },
    
    /// Create default templates
    Init,
}

pub async fn handle_command(cmd: TemplateCommands) -> Result<()> {
    let template_dir = get_template_dir()?;
    let manager = TemplateManager::new(template_dir.clone());
    
    match cmd {
        TemplateCommands::List => {
            let templates = manager.list_templates()?;
            
            if templates.is_empty() {
                println!("No templates found.");
                println!("\nRun 'aany template init' to create default templates.");
            } else {
                println!("Available templates:");
                for template in templates {
                    println!("  - {}", template);
                }
                println!("\nTemplate directory: {}", template_dir.display());
            }
        }
        
        TemplateCommands::Show { name } => {
            match manager.get_template(&name) {
                Ok(template) => {
                    let yaml = serde_yaml::to_string(&template)?;
                    println!("Template: {}\n", name);
                    println!("{}", yaml);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    eprintln!("\nAvailable templates:");
                    for t in manager.list_templates()? {
                        eprintln!("  - {}", t);
                    }
                }
            }
        }
        
        TemplateCommands::Create { name, from } => {
            let template_path = template_dir.join(format!("{}.yaml", name));
            
            if template_path.exists() {
                anyhow::bail!("Template '{}' already exists", name);
            }
            
            if let Some(base) = from {
                // Copy from existing template
                let base_template = manager.get_template(&base)?;
                base_template.save_to_file(&template_path)?;
                println!("✅ Created template '{}' from '{}'", name, base);
            } else {
                // Create new empty template
                use aany_pool::template::AgentTemplate;
                use aany_pool::metadata::*;
                use std::collections::HashMap;
                
                let template = AgentTemplate {
                    agent: AgentInfo {
                        name: "template".to_string(),
                        agent_type: "custom".to_string(),
                        description: format!("Custom {} agent", name),
                        created_at: chrono::Utc::now(),
                        created_by: aany_pool::metadata::get_git_user_info(),
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
                
                template.save_to_file(&template_path)?;
                println!("✅ Created new template: {}", name);
            }
            
            println!("\nEdit with: aany template edit {}", name);
        }
        
        TemplateCommands::Edit { name } => {
            let template_path = template_dir.join(format!("{}.yaml", name));
            
            if !template_path.exists() {
                anyhow::bail!("Template '{}' not found", name);
            }
            
            // Use $EDITOR or default to vi
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            
            let status = std::process::Command::new(&editor)
                .arg(&template_path)
                .status()?;
                
            if status.success() {
                println!("✅ Template edited successfully");
            } else {
                anyhow::bail!("Editor exited with error");
            }
        }
        
        TemplateCommands::Init => {
            println!("Creating default templates...");
            
            std::fs::create_dir_all(&template_dir)?;
            manager.create_default_templates()?;
            
            println!("✅ Created default templates:");
            for template in manager.list_templates()? {
                println!("  - {}", template);
            }
            
            println!("\nTemplate directory: {}", template_dir.display());
        }
    }
    
    Ok(())
}

fn get_template_dir() -> Result<PathBuf> {
    // Check environment variable first
    if let Ok(root) = std::env::var("AANY_TEMPLATE_DIR") {
        return Ok(PathBuf::from(root));
    }
    
    // Use default in user's .aany directory
    if let Some(base_dirs) = BaseDirs::new() {
        Ok(base_dirs.home_dir().join(".aany").join("templates"))
    } else {
        Ok(PathBuf::from("./.aany/templates"))
    }
}