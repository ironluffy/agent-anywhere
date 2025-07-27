use std::io::{self, Write};
use std::env;
use anyhow::Result;
use aany_pool::{PoolManager, metadata::AgentStatus};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor, SetBackgroundColor, Attribute, SetAttribute},
};
use unicode_width::UnicodeWidthStr;
use rand::Rng;

// Debug helper for line alignment - shows dots in debug builds, spaces in release
#[cfg(debug_assertions)]
const LINE_PREFIX: char = '.';
#[cfg(not(debug_assertions))]
const LINE_PREFIX: char = ' ';

// Global margin constant for consistent spacing
const MARGIN: usize = 2;

/// Check if we're already inside a tmux session
fn is_inside_tmux() -> bool {
    env::var("TMUX").is_ok()
}

/// Attach to a tmux session, handling nested tmux sessions
fn attach_to_tmux_session(session_name: &str) -> Result<()> {
    if is_inside_tmux() {
        // Get current session name to set as return session
        let current_session = std::process::Command::new("tmux")
            .args(&["display-message", "-p", "#S"])
            .output()?;
        let current_session_name = String::from_utf8_lossy(&current_session.stdout).trim().to_string();
        
        // Set the return session environment variable in the target session
        std::process::Command::new("tmux")
            .args(&["set-environment", "-t", session_name, "AANY_RETURN_SESSION", &current_session_name])
            .status()?;
        
        // Switch to the target session
        std::process::Command::new("tmux")
            .args(&["switch-client", "-t", session_name])
            .status()?;
    } else {
        // If we're not in tmux, attach normally
        std::process::Command::new("tmux")
            .args(&["attach-session", "-t", session_name])
            .status()?;
    }
    Ok(())
}

pub struct PoolUI {
    manager: PoolManager,
    selected_index: usize,
    message: Option<String>,
    mode: UIMode,
    create_state: CreateNewState,
}

#[derive(PartialEq)]
enum UIMode {
    List,
    CreateNew,
}

struct CreateNewState {
    generated_name: String,
    selected_type: usize,
    selected_field: CreateField,
    show_advanced: bool,
    selected_advanced_option: usize,
    selected_memo: usize,
    env_vars: Vec<(String, String)>,
    selected_env_var: usize,
}

#[derive(PartialEq, Clone, Copy)]
enum CreateField {
    Name,
    Type,
    Memo,
    EnvVars,
    Advanced,
    CreateButton,
}

const AGENT_ROLES: &[(&str, &str)] = &[
    ("claude", "Claude Code agent with Node.js setup"),
    ("general", "General purpose agent"),
    ("research", "Research and analysis agent"),
    ("custom", "Custom agent configuration"),
];

const MEMO_OPTIONS: &[&str] = &[
    "Development and testing",
    "Production workload",
    "Research and experimentation",
    "Personal project",
    "Team collaboration",
    "Custom purpose",
];

const ADVANCED_OPTIONS: &[(&str, &str)] = &[
    ("auto_start", "Auto-start agent on creation"),
    ("persist_logs", "Persist logs after stop"),
    ("hub_connect", "Connect to hub automatically"),
    ("memory_limit", "Set memory limit (2GB default)"),
];

// Predefined environment variable templates for each role
const ENV_VAR_TEMPLATES: &[(&str, &[(&str, &str)])] = &[
    ("claude", &[
        ("GIT_REPO", "https://github.com/username/project.git"),
        ("GIT_BRANCH", "main"),
        ("NODE_ENV", "development"),
    ]),
    ("general", &[
        ("GIT_REPO", "https://github.com/username/project.git"),
        ("GIT_BRANCH", "main"),
        ("WORKSPACE_TYPE", "general"),
    ]),
    ("research", &[
        ("RESEARCH_TOPIC", "AI Safety"),
        ("OUTPUT_FORMAT", "markdown"),
        ("MAX_SOURCES", "10"),
    ]),
    ("custom", &[
        ("CUSTOM_VAR_1", "value1"),
        ("CUSTOM_VAR_2", "value2"),
    ]),
];

// Common repo URLs for quick selection
const REPO_SUGGESTIONS: &[&str] = &[
    "https://github.com/username/project.git",
    "https://github.com/anthropics/claude-code.git",
    "https://github.com/microsoft/vscode.git",
    "https://github.com/rust-lang/rust.git",
    "Local repository (no clone)",
];

impl PoolUI {
    pub async fn new(pool_root: std::path::PathBuf) -> Result<Self> {
        let manager = PoolManager::new(pool_root).await?;
        Ok(Self {
            manager,
            selected_index: 0,
            message: None,
            mode: UIMode::List,
            create_state: CreateNewState {
                generated_name: String::new(),
                selected_type: 0, // Default to claude
                selected_field: CreateField::Name,
                show_advanced: false,
                selected_advanced_option: 0,
                selected_memo: 0,
                env_vars: Vec::new(),
                selected_env_var: 0,
            },
        })
    }
    
    /// Load environment variables for the selected agent type
    fn load_env_vars_for_type(&mut self) {
        let role_name = AGENT_ROLES[self.create_state.selected_type].0;
        
        // Find the env var template for this role
        if let Some((_, vars)) = ENV_VAR_TEMPLATES.iter().find(|(name, _)| *name == role_name) {
            self.create_state.env_vars = vars.iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect();
        } else {
            // Default env vars if no template found
            self.create_state.env_vars = vec![
                ("GIT_REPO".to_string(), "https://github.com/username/project.git".to_string()),
                ("GIT_BRANCH".to_string(), "main".to_string()),
            ];
        }
        self.create_state.selected_env_var = 0;
    }
    
    /// Generate a random agent name
    fn generate_agent_name() -> String {
        let adjectives = [
            "swift", "bright", "clever", "nimble", "keen", "sharp", "quick", "agile",
            "smart", "wise", "bold", "brave", "cool", "calm", "noble", "prime",
            "stellar", "cosmic", "quantum", "neural", "cyber", "digital", "virtual",
            "alpha", "beta", "gamma", "delta", "sigma", "omega", "epsilon", "zeta"
        ];
        
        let nouns = [
            "eagle", "falcon", "hawk", "raven", "phoenix", "dragon", "wolf", "fox",
            "panther", "tiger", "lion", "jaguar", "lynx", "bear", "shark", "ray",
            "comet", "nova", "pulsar", "quasar", "nebula", "cosmos", "vertex", "nexus",
            "matrix", "vector", "tensor", "cipher", "enigma", "oracle", "sage", "pilot"
        ];
        
        let mut rng = rand::thread_rng();
        let adj = adjectives[rng.gen_range(0..adjectives.len())];
        let noun = nouns[rng.gen_range(0..nouns.len())];
        let num = rng.gen_range(100..999);
        
        format!("{}-{}-{}", adj, noun, num)
    }
    
    // Helper method to generate margin string with debug visualization
    fn margin_str(count: usize) -> String {
        LINE_PREFIX.to_string().repeat(count)
    }
    
    // Debug helper to show line and char position in debug builds
    #[cfg(debug_assertions)]
    fn debug_position(_stdout: &mut io::Stdout, _line: usize, _char: usize) -> Result<()> {
        // Uncomment to enable position display
        // execute!(
        //     _stdout,
        //     cursor::SavePosition,
        //     cursor::MoveTo(0, 0),
        //     SetForegroundColor(Color::Magenta),
        //     Print(format!("L{:02}:C{:03}", _line, _char)),
        //     ResetColor,
        //     cursor::RestorePosition
        // )?;
        Ok(())
    }
    
    #[cfg(not(debug_assertions))]
    fn debug_position(_stdout: &mut io::Stdout, _line: usize, _char: usize) -> Result<()> {
        Ok(())
    }
    
    // Print wrapper that tracks character position
    fn print_with_tracking(
        stdout: &mut io::Stdout, 
        content: &str,
        char_num: &mut usize
    ) -> Result<()> {
        let width = UnicodeWidthStr::width(content);
        execute!(stdout, Print(content))?;
        *char_num += width;
        Ok(())
    }
    
    // Debug helper to show line numbers in debug builds
    #[cfg(debug_assertions)]
    fn debug_line_number(_stdout: &mut io::Stdout, _line_num: usize) -> Result<()> {
        // Uncomment to show line numbers
        // execute!(
        //     stdout,
        //     cursor::SavePosition,
        //     cursor::MoveTo(0, line_num.saturating_sub(1) as u16),
        //     SetForegroundColor(Color::Magenta),
        //     Print(format!("{:02}│", line_num)),
        //     ResetColor,
        //     cursor::RestorePosition
        // )?;
        Ok(())
    }
    
    #[cfg(not(debug_assertions))]
    fn debug_line_number(_stdout: &mut io::Stdout, _line_num: usize) -> Result<()> {
        Ok(())
    }
    
    // Shared function to fill empty rows and ensure consistent footer positioning
    // This ensures that the footer always appears at the same position regardless of content
    fn fill_empty_rows(
        stdout: &mut io::Stdout,
        current_line: usize,
        target_line: usize,
        inner_width: usize,
        margin: usize,
    ) -> Result<usize> {
        let mut line_num = current_line;
        let empty_rows_needed = target_line.saturating_sub(current_line);
        
        for _ in 0..empty_rows_needed {
            line_num += 1;
            Self::debug_line_number(stdout, line_num)?;
            execute!(
                stdout,
                Print(Self::margin_str(margin)),
                SetForegroundColor(Color::DarkGrey),
                Print("│"),
                Print(" ".repeat(inner_width.saturating_sub(2))),  // -2 for "│" at both ends
                Print("│"),
                Print(Self::margin_str(margin)),
                ResetColor
            )?;
        }
        
        Ok(line_num)
    }

    pub async fn run(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

        loop {
            self.draw_ui(&mut stdout)?;
            
            if let Event::Key(key_event) = event::read()? {
                match self.mode {
                    UIMode::List => {
                        match key_event.code {
                            KeyCode::Esc => break,
                            KeyCode::Char('q') => break,
                            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => self.move_selection_up(),
                            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => self.move_selection_down(),
                            KeyCode::Enter => self.handle_attach().await?,
                            KeyCode::Char(' ') => self.handle_toggle().await?,
                            KeyCode::Char('a') => self.handle_attach().await?,
                            KeyCode::Char('n') => {
                                self.mode = UIMode::CreateNew;
                                self.create_state.generated_name = Self::generate_agent_name();
                                self.create_state.selected_type = 0;
                                self.create_state.selected_field = CreateField::Name;
                                self.create_state.show_advanced = false;
                                self.create_state.selected_advanced_option = 0;
                                self.create_state.selected_memo = 0;
                                self.load_env_vars_for_type();
                            }
                            KeyCode::Char('r') => self.refresh().await?,
                            KeyCode::Char('d') => self.handle_delete().await?,
                            KeyCode::Tab => self.move_selection_down(),
                            KeyCode::BackTab => self.move_selection_up(),
                            _ => {}
                        }
                    }
                    UIMode::CreateNew => {
                        match key_event.code {
                            KeyCode::Esc => break,
                            KeyCode::Char('q') => {
                                // Go back to list mode
                                self.mode = UIMode::List;
                            }
                            KeyCode::Tab => {
                                // Move to next field
                                self.create_state.selected_field = match self.create_state.selected_field {
                                    CreateField::Name => CreateField::Type,
                                    CreateField::Type => CreateField::Memo,
                                    CreateField::Memo => CreateField::EnvVars,
                                    CreateField::EnvVars => CreateField::Advanced,
                                    CreateField::Advanced => CreateField::CreateButton,
                                    CreateField::CreateButton => CreateField::Name,
                                };
                            }
                            KeyCode::BackTab => {
                                // Move to previous field
                                self.create_state.selected_field = match self.create_state.selected_field {
                                    CreateField::Name => CreateField::CreateButton,
                                    CreateField::Type => CreateField::Name,
                                    CreateField::Memo => CreateField::Type,
                                    CreateField::EnvVars => CreateField::Memo,
                                    CreateField::Advanced => CreateField::EnvVars,
                                    CreateField::CreateButton => CreateField::Advanced,
                                };
                            }
                            KeyCode::Enter => {
                                match self.create_state.selected_field {
                                    CreateField::Name => {
                                        // Regenerate name
                                        self.create_state.generated_name = Self::generate_agent_name();
                                    }
                                    CreateField::EnvVars => {
                                        // Cycle through predefined values for the selected env var
                                        if let Some((key, value)) = self.create_state.env_vars.get_mut(self.create_state.selected_env_var) {
                                            if key == "GIT_REPO" {
                                                // Cycle through repo suggestions
                                                let current_index = REPO_SUGGESTIONS.iter().position(|&s| s == value).unwrap_or(0);
                                                let next_index = (current_index + 1) % REPO_SUGGESTIONS.len();
                                                *value = REPO_SUGGESTIONS[next_index].to_string();
                                            } else if key == "GIT_BRANCH" {
                                                // Cycle through common branch names
                                                *value = match value.as_str() {
                                                    "main" => "master".to_string(),
                                                    "master" => "develop".to_string(),
                                                    "develop" => "feature/dev".to_string(),
                                                    _ => "main".to_string(),
                                                };
                                            }
                                        }
                                    }
                                    CreateField::Advanced => {
                                        // Toggle advanced options
                                        self.create_state.show_advanced = !self.create_state.show_advanced;
                                    }
                                    CreateField::CreateButton => {
                                        // Create the agent
                                        self.create_agent().await?;
                                    }
                                    _ => {}
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                                match self.create_state.selected_field {
                                    CreateField::EnvVars => {
                                        // Navigate up in env vars list
                                        if self.create_state.selected_env_var > 0 {
                                            self.create_state.selected_env_var -= 1;
                                        }
                                    }
                                    CreateField::Advanced if self.create_state.show_advanced => {
                                        // Navigate up in advanced options
                                        if self.create_state.selected_advanced_option > 0 {
                                            self.create_state.selected_advanced_option -= 1;
                                        }
                                    }
                                    _ => {
                                        // Move to previous field (same as Shift+Tab)
                                        self.create_state.selected_field = match self.create_state.selected_field {
                                            CreateField::Name => CreateField::CreateButton,
                                            CreateField::Type => CreateField::Name,
                                            CreateField::Memo => CreateField::Type,
                                            CreateField::EnvVars => CreateField::Memo,
                                            CreateField::Advanced => CreateField::EnvVars,
                                            CreateField::CreateButton => CreateField::Advanced,
                                        };
                                    }
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                                match self.create_state.selected_field {
                                    CreateField::EnvVars => {
                                        // Navigate down in env vars list
                                        if self.create_state.selected_env_var < self.create_state.env_vars.len().saturating_sub(1) {
                                            self.create_state.selected_env_var += 1;
                                        }
                                    }
                                    CreateField::Advanced if self.create_state.show_advanced => {
                                        // Navigate down in advanced options
                                        if self.create_state.selected_advanced_option < ADVANCED_OPTIONS.len() - 1 {
                                            self.create_state.selected_advanced_option += 1;
                                        }
                                    }
                                    _ => {
                                        // Move to next field (same as Tab)
                                        self.create_state.selected_field = match self.create_state.selected_field {
                                            CreateField::Name => CreateField::Type,
                                            CreateField::Type => CreateField::Memo,
                                            CreateField::Memo => CreateField::EnvVars,
                                            CreateField::EnvVars => CreateField::Advanced,
                                            CreateField::Advanced => CreateField::CreateButton,
                                            CreateField::CreateButton => CreateField::Name,
                                        };
                                    }
                                }
                            }
                            KeyCode::Left | KeyCode::Char('a') => {
                                match self.create_state.selected_field {
                                    CreateField::Type => {
                                        if self.create_state.selected_type > 0 {
                                            self.create_state.selected_type -= 1;
                                            self.load_env_vars_for_type();
                                        }
                                    }
                                    CreateField::Memo => {
                                        if self.create_state.selected_memo > 0 {
                                            self.create_state.selected_memo -= 1;
                                        }
                                    }
                                    CreateField::EnvVars => {
                                        if self.create_state.selected_env_var > 0 {
                                            self.create_state.selected_env_var -= 1;
                                        }
                                    }
                                    CreateField::Advanced if self.create_state.show_advanced => {
                                        if self.create_state.selected_advanced_option > 0 {
                                            self.create_state.selected_advanced_option -= 1;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            KeyCode::Right | KeyCode::Char('d') => {
                                match self.create_state.selected_field {
                                    CreateField::Type => {
                                        if self.create_state.selected_type < AGENT_ROLES.len() - 1 {
                                            self.create_state.selected_type += 1;
                                            self.load_env_vars_for_type();
                                        }
                                    }
                                    CreateField::Memo => {
                                        if self.create_state.selected_memo < MEMO_OPTIONS.len() - 1 {
                                            self.create_state.selected_memo += 1;
                                        }
                                    }
                                    CreateField::EnvVars => {
                                        if self.create_state.selected_env_var < self.create_state.env_vars.len().saturating_sub(1) {
                                            self.create_state.selected_env_var += 1;
                                        }
                                    }
                                    CreateField::Advanced if self.create_state.show_advanced => {
                                        if self.create_state.selected_advanced_option < ADVANCED_OPTIONS.len() - 1 {
                                            self.create_state.selected_advanced_option += 1;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn draw_ui(&self, stdout: &mut io::Stdout) -> Result<()> {
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        
        // Always hide cursor - no text input in this UI
        execute!(stdout, cursor::Hide)?;

        let (width, height) = terminal::size()?;
        let mut line_num = 0;
        let mut char_num = 0;
        
        // Add top margin
        line_num += 1;
        char_num = 0;
        execute!(stdout, Print("\n"))?;
        
        // Header with colored background
        line_num += 1;
        char_num = 0;
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkBlue),
            SetForegroundColor(Color::White),
            SetAttribute(Attribute::Bold)
        )?;
        
        // Center the title with margin
        let title = " Agent Pool Manager ";
        let margin = MARGIN;
        let inner_width = (width as usize).saturating_sub(margin * 2);
        let title_len = title.len();
        let left_padding = (inner_width.saturating_sub(title_len)) / 2;
        let right_padding = inner_width.saturating_sub(title_len + left_padding);
        
        // Track character positions as we print the title
        Self::print_with_tracking(stdout, &Self::margin_str(margin), &mut char_num)?;
        Self::print_with_tracking(stdout, &" ".repeat(left_padding), &mut char_num)?;
        Self::print_with_tracking(stdout, title, &mut char_num)?;
        Self::print_with_tracking(stdout, &" ".repeat(right_padding), &mut char_num)?;
        Self::print_with_tracking(stdout, &Self::margin_str(margin), &mut char_num)?;
        execute!(stdout, ResetColor)?;
        
        // Debug: show position at end of title line
        Self::debug_position(stdout, line_num, char_num)?;

        // Main content area with border
        line_num += 1;
        Self::debug_line_number(stdout, line_num)?;
        line_num = self.draw_border(stdout, width, line_num)?;
        
        // Calculate where footer should start
        // Header takes 3 lines (margin + title + border)
        // Footer takes 4 lines (border + status + controls + margin)
        let footer_lines = 4;
        let target_footer_line = (height as usize).saturating_sub(footer_lines);
        
        // Lines 4+: Content (varies based on mode)
        line_num = match self.mode {
            UIMode::List => self.draw_agent_list(stdout, width, height, line_num, target_footer_line)?,
            UIMode::CreateNew => self.draw_create_form(stdout, width, height, line_num, target_footer_line)?,
        };

        // Footer (starts after content)
        self.draw_footer(stdout, width, height, line_num)?;

        stdout.flush()?;
        Ok(())
    }

    fn draw_border(&self, stdout: &mut io::Stdout, width: u16, line_num: usize) -> Result<usize> {
        let margin = MARGIN;
        let inner_width = (width as usize).saturating_sub(margin * 2);
        let mut char_num = 0;
        
        // Track character position as we print
        Self::print_with_tracking(stdout, &Self::margin_str(margin), &mut char_num)?;
        execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
        Self::print_with_tracking(stdout, "┌", &mut char_num)?;
        Self::print_with_tracking(stdout, &"─".repeat(inner_width - 2), &mut char_num)?;
        Self::print_with_tracking(stdout, "┐", &mut char_num)?;
        Self::print_with_tracking(stdout, &Self::margin_str(margin), &mut char_num)?;
        execute!(stdout, ResetColor)?;
        
        // Debug: show position at end of line
        Self::debug_position(stdout, line_num, char_num)?;
        
        Ok(line_num)
    }

    fn draw_agent_list(&self, stdout: &mut io::Stdout, width: u16, _height: u16, mut line_num: usize, target_footer_line: usize) -> Result<usize> {
        let agents = self.manager.list_agents();
        let margin = MARGIN;
        let inner_width = (width as usize).saturating_sub(margin * 2);
        
        // Option: Create New Agent
        line_num += 1;
        Self::debug_line_number(stdout, line_num)?;
        self.draw_menu_item(stdout, 0, "Create New Agent", None, None, width)?;
        
        // Separator  
        line_num += 1;
        Self::debug_line_number(stdout, line_num)?;
        execute!(
            stdout,
            Print(Self::margin_str(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("├"),
            Print("─".repeat(inner_width.saturating_sub(2))),
            Print("┤"),
            Print(Self::margin_str(margin)),
            ResetColor
        )?;

        // Agent list
        for (index, agent) in agents.iter().enumerate() {
            let real_index = index + 1; // Account for "Create New" option
            
            let status = match agent.metadata.tmux.status {
                AgentStatus::Active => ("RUNNING", Color::Green),
                AgentStatus::Stopped => ("STOPPED", Color::DarkGrey),
                AgentStatus::Crashed => ("CRASHED", Color::Red),
                AgentStatus::Starting => ("STARTING", Color::Yellow),
            };
            
            let memo = if let Some(current_task) = &agent.metadata.tasks.current {
                format!("working on: {}", truncate_string(current_task, 40))
            } else {
                "idle".to_string()
            };
            
            line_num += 1;
            Self::debug_line_number(stdout, line_num)?;
            self.draw_menu_item(
                stdout,
                real_index,
                &agent.name,
                Some(status),
                Some(&memo),
                width
            )?;
        }

        // Fill remaining space with empty rows to reach footer
        line_num = Self::fill_empty_rows(stdout, line_num, target_footer_line, inner_width, margin)?;

        Ok(line_num)
    }

    fn draw_menu_item(
        &self,
        stdout: &mut io::Stdout,
        index: usize,
        name: &str,
        status: Option<(&str, Color)>,
        memo: Option<&str>,
        width: u16,
    ) -> Result<()> {
        let is_selected = index == self.selected_index;
        let margin = MARGIN;
        
        execute!(
            stdout,
            Print(Self::margin_str(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("│ ")
        )?;

        // Selection indicator
        if is_selected {
            execute!(
                stdout,
                SetForegroundColor(Color::Cyan),
                SetAttribute(Attribute::Bold),
                Print("◆ "),
                ResetColor
            )?;
        } else {
            execute!(stdout, Print("  "))?;
        }

        // Calculate available width for dynamic content
        let inner_width = (width as usize).saturating_sub(margin * 2);
        let base_width = 4; // "│ " + indicator
        let status_width = status
            .map(|(s, _)| UnicodeWidthStr::width(s) + 3)
            .unwrap_or(0);
        let border_width = 2; // " │"
        let min_name_width = 15;
        let available_width = inner_width.saturating_sub(base_width + status_width + border_width);
        
        // Allocate space for name and memo
        let name_width = if available_width > 60 {
            30 // More space available, use wider name column
        } else if available_width > 40 {
            20 // Standard width
        } else {
            min_name_width // Minimum width
        };
        
        let memo_width = available_width.saturating_sub(name_width + 2);

        // Name with highlight if selected
        if is_selected {
            execute!(
                stdout,
                SetBackgroundColor(Color::DarkGrey),
                SetForegroundColor(Color::White),
                SetAttribute(Attribute::Bold)
            )?;
        }
        
        let truncated_name = if name.len() > name_width {
            format!("{}...", &name[..name_width.saturating_sub(3)])
        } else {
            format!("{:<width$}", name, width = name_width)
        };
        
        execute!(stdout, Print(&truncated_name))?;
        
        if is_selected {
            execute!(stdout, ResetColor)?;
        }

        // Status badge
        if let Some((status_text, status_color)) = status {
            execute!(
                stdout,
                Print(" "),
                SetBackgroundColor(status_color),
                SetForegroundColor(if matches!(status_color, Color::Yellow | Color::Green) { 
                    Color::Black 
                } else { 
                    Color::White 
                }),
                Print(format!(" {} ", status_text)),
                ResetColor
            )?;
        }

        // Memo (if there's space)
        if let Some(memo_text) = memo {
            if memo_width > 10 {
                let truncated_memo = truncate_string(memo_text, memo_width.saturating_sub(1));
                execute!(
                    stdout,
                    Print(" "),
                    SetForegroundColor(Color::DarkGrey),
                    Print(&truncated_memo)
                )?;
            }
        }

        // Fill remaining space and close border
        // Calculate actual printed width
        let mut actual_width = 0;
        actual_width += UnicodeWidthStr::width(if is_selected { "◆ " } else { "  " });
        actual_width += UnicodeWidthStr::width(truncated_name.as_str());
        
        if status.is_some() {
            actual_width += 1; // space before status
            actual_width += status.map(|(s, _)| UnicodeWidthStr::width(s) + 2).unwrap_or(0); // status text + padding
        }
        
        if let Some(memo_text) = memo {
            if memo_width > 10 {
                actual_width += 1; // space before memo
                let truncated_memo = truncate_string(memo_text, memo_width.saturating_sub(1));
                actual_width += UnicodeWidthStr::width(truncated_memo.as_str());
            }
        }
        
        // Calculate padding to reach the right border
        let inner_width = (width as usize).saturating_sub(margin * 2);
        let content_area = inner_width.saturating_sub(4); // "│ " at start and " │" at end
        let padding = content_area.saturating_sub(actual_width);
        
        execute!(
            stdout,
            ResetColor,
            Print(" ".repeat(padding)),
            SetForegroundColor(Color::DarkGrey),
            Print(" │"),
            Print(Self::margin_str(margin)),
            ResetColor
        )?;

        Ok(())
    }

    fn draw_form_field(
        &self,
        stdout: &mut io::Stdout,
        label: &str,
        value: &str,
        is_selected: bool,
        hint: &str,
        inner_width: usize,
        margin: usize,
        mut line_num: usize,
    ) -> Result<usize> {
        line_num += 1;
        Self::debug_line_number(stdout, line_num)?;
        execute!(
            stdout,
            Print(Self::margin_str(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("│ "),
            SetForegroundColor(if is_selected { Color::Cyan } else { Color::DarkGrey }),
            Print(format!("{:>8}: ", label)),
            ResetColor
        )?;
        
        if is_selected {
            execute!(
                stdout,
                SetBackgroundColor(Color::DarkGrey),
                SetForegroundColor(Color::White),
                SetAttribute(Attribute::Bold),
                Print(format!(" {} ", value)),
                ResetColor,
                Print("  "),
                SetForegroundColor(Color::DarkGrey),
                Print(hint)
            )?;
        } else {
            execute!(
                stdout,
                SetForegroundColor(Color::White),
                Print(value)
            )?;
        }
        
        // Calculate actual display width: "│ " (3) + label (padded to 8) + ": " (2) + value + optional(hint+4)
        let label_display_width = 8; // Labels are padded to 8 chars
        let used_len = 3 + label_display_width + 2 + UnicodeWidthStr::width(value) + if is_selected { UnicodeWidthStr::width(hint) + 4 } else { 0 };
        execute!(
            stdout,
            Print(" ".repeat(inner_width.saturating_sub(used_len))),
            SetForegroundColor(Color::DarkGrey),
            Print("│"),
            Print(Self::margin_str(margin)),
            ResetColor
        )?;
        
        // Add empty line after field
        line_num += 1;
        Self::debug_line_number(stdout, line_num)?;
        execute!(
            stdout,
            Print(Self::margin_str(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("│"),
            Print(" ".repeat(inner_width.saturating_sub(2))),
            Print("│"),
            Print(Self::margin_str(margin)),
            ResetColor
        )?;
        
        Ok(line_num)
    }
    
    fn draw_create_form(&self, stdout: &mut io::Stdout, width: u16, _height: u16, mut line_num: usize, target_footer_line: usize) -> Result<usize> {
        let margin = MARGIN;
        let inner_width = (width as usize).saturating_sub(margin * 2);
        
        // Title
        line_num += 1;
        Self::debug_line_number(stdout, line_num)?;
        execute!(
            stdout,
            Print(Self::margin_str(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("│ "),
            SetForegroundColor(Color::Cyan),
            SetAttribute(Attribute::Bold),
            Print("Create New Agent"),
            ResetColor
        )?;
        
        let text_len = "Create New Agent".len();
        let padding = inner_width.saturating_sub(4 + text_len);
        execute!(
            stdout,
            Print(" ".repeat(padding)),
            SetForegroundColor(Color::DarkGrey),
            Print(" │"),
            Print(Self::margin_str(margin)),
            ResetColor
        )?;
        
        // Separator
        line_num += 1;
        Self::debug_line_number(stdout, line_num)?;
        execute!(
            stdout,
            Print(Self::margin_str(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("├"),
            Print("─".repeat(inner_width.saturating_sub(2))),
            Print("┤"),
            Print(Self::margin_str(margin)),
            ResetColor
        )?;
        
        // Name field
        line_num = self.draw_form_field(
            stdout,
            "Name",
            &self.create_state.generated_name,
            self.create_state.selected_field == CreateField::Name,
            "[Enter to regenerate]",
            inner_width,
            margin,
            line_num
        )?;
        
        // Type field
        let role_display = format!(
            "{} - {}",
            AGENT_ROLES[self.create_state.selected_type].0,
            AGENT_ROLES[self.create_state.selected_type].1
        );
        line_num = self.draw_form_field(
            stdout,
            "Type",
            &role_display,
            self.create_state.selected_field == CreateField::Type,
            "[←→ to change]",
            inner_width,
            margin,
            line_num
        )?;
        
        // Memo field
        line_num = self.draw_form_field(
            stdout,
            "Purpose",
            MEMO_OPTIONS[self.create_state.selected_memo],
            self.create_state.selected_field == CreateField::Memo,
            "[←→ to change]",
            inner_width,
            margin,
            line_num
        )?;
        
        // Environment Variables field
        let env_display = if self.create_state.env_vars.is_empty() {
            "No environment variables".to_string()
        } else if self.create_state.selected_env_var < self.create_state.env_vars.len() {
            let (key, value) = &self.create_state.env_vars[self.create_state.selected_env_var];
            format!("{} = {}", key, value)
        } else {
            "No environment variables".to_string()
        };
        
        line_num = self.draw_form_field(
            stdout,
            "Env Vars",
            &env_display,
            self.create_state.selected_field == CreateField::EnvVars,
            "[↑↓ to select, Enter to cycle values]",
            inner_width,
            margin,
            line_num
        )?;
        
        // Show all env vars if this field is selected
        if self.create_state.selected_field == CreateField::EnvVars && !self.create_state.env_vars.is_empty() {
            for (i, (key, value)) in self.create_state.env_vars.iter().enumerate() {
                line_num += 1;
                Self::debug_line_number(stdout, line_num)?;
                execute!(
                    stdout,
                    Print(Self::margin_str(margin)),
                    SetForegroundColor(Color::DarkGrey),
                    Print("│ "),
                    Print("          ")
                )?;
                
                if i == self.create_state.selected_env_var {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Cyan),
                        Print("▶ "),
                        SetForegroundColor(Color::White)
                    )?;
                } else {
                    execute!(
                        stdout,
                        Print("  "),
                        SetForegroundColor(Color::DarkGrey)
                    )?;
                }
                
                execute!(
                    stdout,
                    Print(format!("{:<15} = {}", key, value))
                )?;
                
                let var_len = 18 + UnicodeWidthStr::width(key.as_str()) + UnicodeWidthStr::width(value.as_str()) + 10;
                execute!(
                    stdout,
                    Print(" ".repeat(inner_width.saturating_sub(var_len))),
                    SetForegroundColor(Color::DarkGrey),
                    Print("│"),
                    Print(Self::margin_str(margin)),
                    ResetColor
                )?;
            }
            
            // Add empty line after env vars list
            line_num += 1;
            Self::debug_line_number(stdout, line_num)?;
            execute!(
                stdout,
                Print(Self::margin_str(margin)),
                SetForegroundColor(Color::DarkGrey),
                Print("│"),
                Print(" ".repeat(inner_width.saturating_sub(2))),
                Print("│"),
                Print(Self::margin_str(margin)),
                ResetColor
            )?;
        }
        
        // Advanced options field
        let advanced_text = if self.create_state.show_advanced {
            format!("▼ Advanced Options ({})", ADVANCED_OPTIONS[self.create_state.selected_advanced_option].0)
        } else {
            "▶ Advanced Options".to_string()
        };
        line_num = self.draw_form_field(
            stdout,
            "Config",
            &advanced_text,
            self.create_state.selected_field == CreateField::Advanced,
            "[Enter to toggle]",
            inner_width,
            margin,
            line_num
        )?;
        
        // Show expanded advanced options if open
        if self.create_state.show_advanced {
            for (i, (option_key, option_desc)) in ADVANCED_OPTIONS.iter().enumerate() {
                line_num += 1;
                Self::debug_line_number(stdout, line_num)?;
                execute!(
                    stdout,
                    Print(Self::margin_str(margin)),
                    SetForegroundColor(Color::DarkGrey),
                    Print("│ "),
                    Print("      ")
                )?;
                
                if i == self.create_state.selected_advanced_option {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Cyan),
                        Print("▶ "),
                        SetForegroundColor(Color::White)
                    )?;
                } else {
                    execute!(
                        stdout,
                        Print("  "),
                        SetForegroundColor(Color::DarkGrey)
                    )?;
                }
                
                execute!(
                    stdout,
                    Print(format!("{:<20} {}", option_key, option_desc))
                )?;
                
                let option_len = 28 + UnicodeWidthStr::width(*option_key) + UnicodeWidthStr::width(*option_desc);
                execute!(
                    stdout,
                    Print(" ".repeat(inner_width.saturating_sub(option_len))),
                    SetForegroundColor(Color::DarkGrey),
                    Print("│"),
                    Print(Self::margin_str(margin)),
                    ResetColor
                )?;
            }
        }
        
        // Separator before button
        line_num += 1;
        Self::debug_line_number(stdout, line_num)?;
        execute!(
            stdout,
            Print(Self::margin_str(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("├"),
            Print("─".repeat(inner_width.saturating_sub(2))),
            Print("┤"),
            Print(Self::margin_str(margin)),
            ResetColor
        )?;
        
        // Create button - properly centered
        let button_text = "  [ CREATE AGENT ]  ";
        let button_width = UnicodeWidthStr::width(button_text);
        let left_padding = (inner_width.saturating_sub(button_width + 4)) / 2; // +4 for "│ " and " │"
        let right_padding = inner_width.saturating_sub(button_width + 4 + left_padding);
        
        line_num += 1;
        Self::debug_line_number(stdout, line_num)?;
        execute!(
            stdout,
            Print(Self::margin_str(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("│ "),
            Print(" ".repeat(left_padding))
        )?;
        
        if self.create_state.selected_field == CreateField::CreateButton {
            execute!(
                stdout,
                SetBackgroundColor(Color::Green),
                SetForegroundColor(Color::Black),
                SetAttribute(Attribute::Bold),
                Print(button_text),
                ResetColor
            )?;
        } else {
            execute!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print(button_text)
            )?;
        }
        
        execute!(
            stdout,
            Print(" ".repeat(right_padding)),
            SetForegroundColor(Color::DarkGrey),
            Print(" │"),
            Print(Self::margin_str(margin)),
            ResetColor
        )?;
        
        // Help text - properly centered
        let help_text = "Navigate: ↑↓/jk/ws or Tab • Within fields: ←→/ad • Enter to select • q to cancel";
        let help_width = UnicodeWidthStr::width(help_text);
        let help_left_padding = (inner_width.saturating_sub(help_width + 4)) / 2; // +4 for "│ " and " │"
        let help_right_padding = inner_width.saturating_sub(help_width + 4 + help_left_padding);
        
        line_num += 1;
        Self::debug_line_number(stdout, line_num)?;
        execute!(
            stdout,
            Print(Self::margin_str(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("│ "),
            Print(" ".repeat(help_left_padding)),
            SetForegroundColor(Color::DarkGrey),
            Print(help_text),
            Print(" ".repeat(help_right_padding)),
            Print(" │"),
            Print(Self::margin_str(margin)),
            ResetColor
        )?;
        
        // Fill remaining space with empty rows to reach footer
        line_num = Self::fill_empty_rows(stdout, line_num, target_footer_line, inner_width, margin)?;

        Ok(line_num)
    }

    fn draw_footer(&self, stdout: &mut io::Stdout, width: u16, _height: u16, mut line_num: usize) -> Result<usize> {
        let margin = MARGIN;
        let inner_width = (width as usize).saturating_sub(margin * 2);
        
        // Bottom border
        line_num += 1;
        Self::debug_line_number(stdout, line_num)?;
        execute!(
            stdout,
            Print(Self::margin_str(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("└"),
            Print("─".repeat(inner_width - 2)),
            Print("┘"),
            Print(Self::margin_str(margin)),
            ResetColor
        )?;

        // Status message
        line_num += 1;
        Self::debug_line_number(stdout, line_num)?;
        if let Some(msg) = &self.message {
            // Choose appropriate icon and color based on the message content
            let (icon, color) = if msg.starts_with("Started") {
                ("▶", Color::Green)  // Play button for started
            } else if msg.starts_with("Stopped") {
                ("■", Color::DarkGrey)  // Stop square for stopped
            } else if msg.starts_with("Created") {
                ("✓", Color::Green)  // Checkmark for created
            } else if msg.starts_with("Deleted") {
                ("✗", Color::Red)  // X for deleted
            } else if msg.starts_with("Failed") {
                ("⚠", Color::Red)  // Warning for errors
            } else if msg.starts_with("Refreshed") {
                ("↻", Color::Cyan)  // Refresh symbol
            } else {
                ("•", Color::Yellow)  // Default bullet for other messages
            };
            
            let status_text = format!(" {} {}", icon, msg);
            // Use unicode width for proper padding calculation
            let status_width = UnicodeWidthStr::width(status_text.as_str());
            let padding = inner_width.saturating_sub(status_width);
            execute!(
                stdout,
                Print(Self::margin_str(margin)),
                SetForegroundColor(color),
                Print(&status_text),
                Print(" ".repeat(padding)),
                Print(Self::margin_str(margin)),
                ResetColor,
            )?;
        } else {
            execute!(
                stdout, 
                Print(Self::margin_str(margin)),
                Print(" ".repeat(inner_width)),
                Print(Self::margin_str(margin)),
            )?;
        }

        // Controls
        let controls = match self.mode {
            UIMode::List => vec![
                ("↑↓/jk/ws", "Navigate"),
                ("Enter/a", "Attach"),
                ("Space", "Start/Stop"),
                ("n", "New"),
                ("d", "Delete"),
                ("r", "Refresh"),
                ("q", "Quit"),
            ],
            UIMode::CreateNew => vec![
                ("q", "Go Back"),
                ("Enter", "Continue/Create"),
                ("ESC", "Exit"),
            ],
        };

        line_num += 1;
        Self::debug_line_number(stdout, line_num)?;
        execute!(stdout, Print(Self::margin_str(margin)), SetForegroundColor(Color::DarkGrey))?;
        for (i, (key, action)) in controls.iter().enumerate() {
            if i > 0 {
                execute!(stdout, Print(" │ "))?;
            }
            execute!(
                stdout,
                SetForegroundColor(Color::White),
                SetAttribute(Attribute::Bold),
                Print(key),
                ResetColor,
                SetForegroundColor(Color::DarkGrey),
                Print(format!(" {}", action))
            )?;
        }
        execute!(stdout, ResetColor)?;

        Ok(line_num)
    }

    fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    fn move_selection_down(&mut self) {
        let max_index = self.manager.list_agents().len(); // +1 for "Create New" option
        if self.selected_index < max_index {
            self.selected_index += 1;
        }
    }

    async fn handle_toggle(&mut self) -> Result<()> {
        if self.selected_index == 0 {
            // Create new agent option - do nothing
            return Ok(());
        }
        
        // Toggle agent state
        let agents = self.manager.list_agents();
        if let Some(agent_name) = agents.get(self.selected_index - 1).map(|a| a.name.clone()) {
            if let Ok(agent) = self.manager.get_agent_mut(&agent_name) {
                match agent.metadata.tmux.status {
                    AgentStatus::Active => {
                        agent.stop().await?;
                        self.message = Some(format!("Stopped: {}", agent_name));
                    }
                    _ => {
                        agent.start().await?;
                        self.message = Some(format!("Started: {}", agent_name));
                    }
                }
            }
        }
        Ok(())
    }
    
    async fn handle_attach(&mut self) -> Result<()> {
        if self.selected_index == 0 {
            // Create new agent
            self.mode = UIMode::CreateNew;
            self.create_state.generated_name = Self::generate_agent_name();
            self.create_state.selected_type = 0;
            self.create_state.selected_field = CreateField::Name;
            self.create_state.show_advanced = false;
            self.create_state.selected_advanced_option = 0;
            self.create_state.selected_memo = 0;
            self.load_env_vars_for_type();
            return Ok(());
        }
        
        // Attach to agent session
        let agents = self.manager.list_agents();
        if let Some(agent) = agents.get(self.selected_index - 1) {
            // Check if agent is running
            match agent.metadata.tmux.status {
                AgentStatus::Active => {
                    // Exit UI and attach to tmux session
                    let session_name = &agent.metadata.tmux.session_name;
                    
                    // Exit UI temporarily to attach to tmux session
                    execute!(io::stdout(), cursor::Show, LeaveAlternateScreen)?;
                    terminal::disable_raw_mode()?;
                    
                    if is_inside_tmux() {
                        // If we're already in tmux, switch to the agent session
                        attach_to_tmux_session(session_name)?;
                    } else {
                        // If we're not in tmux, attach normally
                        attach_to_tmux_session(session_name)?;
                    }
                    
                    // Re-enter UI after detaching/switching back
                    terminal::enable_raw_mode()?;
                    execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
                }
                _ => {
                    self.message = Some(format!("Agent '{}' is not running. Press Space to start it.", agent.name));
                }
            }
        }
        Ok(())
    }

    async fn refresh(&mut self) -> Result<()> {
        self.manager = PoolManager::new(self.manager.root_path.clone()).await?;
        self.message = Some("Refreshed".to_string());
        Ok(())
    }
    
    async fn handle_delete(&mut self) -> Result<()> {
        if self.selected_index == 0 {
            // Can't delete "Create New Agent" option
            return Ok(());
        }
        
        let agents = self.manager.list_agents();
        if let Some(agent) = agents.get(self.selected_index - 1) {
            let agent_name = agent.name.clone();
            
            // Stop the agent first if it's running
            if matches!(agent.metadata.tmux.status, AgentStatus::Active) {
                if let Ok(agent) = self.manager.get_agent_mut(&agent_name) {
                    let _ = agent.stop().await;
                }
            }
            
            // Delete the agent
            match self.manager.delete_agent(&agent_name).await {
                Ok(_) => {
                    self.message = Some(format!("Deleted agent: {}", agent_name));
                    // Adjust selection if needed
                    if self.selected_index > 0 && self.selected_index > self.manager.list_agents().len() {
                        self.selected_index -= 1;
                    }
                }
                Err(e) => {
                    self.message = Some(format!("Failed to delete agent: {}", e));
                }
            }
        }
        Ok(())
    }
    
    async fn create_agent(&mut self) -> Result<()> {
        let name = self.create_state.generated_name.clone();
        let agent_role = AGENT_ROLES[self.create_state.selected_type].0;
        
        // Create the agent with the selected role as template
        match self.manager.create_agent(name.clone(), Some(agent_role.to_string())).await {
            Ok(_) => {
                // Save environment variables to .env file
                if !self.create_state.env_vars.is_empty() {
                    if let Ok(agent) = self.manager.get_agent_mut(&name) {
                        let env_file_path = agent.get_agent_dir().join(".env");
                        let env_content = self.create_state.env_vars.iter()
                            .map(|(k, v)| format!("{}={}", k, v))
                            .collect::<Vec<_>>()
                            .join("\n");
                        
                        if let Err(e) = tokio::fs::write(&env_file_path, env_content).await {
                            self.message = Some(format!("Agent created but failed to save env vars: {}", e));
                        }
                    }
                }
                
                self.message = Some(format!("Created {} agent: {}", agent_role, name));
                self.mode = UIMode::List;
                self.create_state.generated_name.clear();
                self.create_state.selected_type = 0;
                self.create_state.selected_field = CreateField::Name;
                self.create_state.show_advanced = false;
                self.create_state.selected_memo = 0;
                
                // Refresh to show the new agent
                self.refresh().await?;
                
                // Auto-start Claude agents
                if agent_role == "claude" {
                    if let Ok(agent) = self.manager.get_agent_mut(&name) {
                        agent.start().await?;
                        self.message = Some(format!("Started Claude agent: {}", name));
                    }
                }
            }
            Err(e) => {
                self.message = Some(format!("Failed to create agent: {}", e));
            }
        }
        
        Ok(())
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len-3])
    }
}