use std::io::{self, Write};
use anyhow::Result;
use aany_pool::{PoolManager, metadata::AgentStatus};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor, SetBackgroundColor, Attribute, SetAttribute},
};

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
    name: String,
    cursor_pos: usize,
    selected_type: usize,
    show_type_selection: bool,
}

const AGENT_ROLES: &[(&str, &str)] = &[
    ("claude", "Claude Code agent with Node.js setup"),
    ("general", "General purpose agent"),
    ("research", "Research and analysis agent"),
    ("custom", "Custom agent configuration"),
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
                name: String::new(),
                cursor_pos: 0,
                selected_type: 0, // Default to claude
                show_type_selection: false,
            },
        })
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
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Up | KeyCode::Char('k') => self.move_selection_up(),
                            KeyCode::Down | KeyCode::Char('j') => self.move_selection_down(),
                            KeyCode::Enter => self.handle_attach().await?,
                            KeyCode::Char(' ') => self.handle_toggle().await?,
                            KeyCode::Char('a') => self.handle_attach().await?,
                            KeyCode::Char('n') => self.mode = UIMode::CreateNew,
                            KeyCode::Char('r') => self.refresh().await?,
                            KeyCode::Tab => self.move_selection_down(),
                            KeyCode::BackTab => self.move_selection_up(),
                            _ => {}
                        }
                    }
                    UIMode::CreateNew => {
                        if self.create_state.show_type_selection {
                            // Role selection mode
                            match key_event.code {
                                KeyCode::Esc => {
                                    self.create_state.show_type_selection = false;
                                }
                                KeyCode::Up => {
                                    if self.create_state.selected_type > 0 {
                                        self.create_state.selected_type -= 1;
                                    }
                                }
                                KeyCode::Down => {
                                    if self.create_state.selected_type < AGENT_ROLES.len() - 1 {
                                        self.create_state.selected_type += 1;
                                    }
                                }
                                KeyCode::Enter => {
                                    self.create_agent().await?;
                                }
                                _ => {}
                            }
                        } else {
                            // Name input mode
                            match key_event.code {
                                KeyCode::Esc => {
                                    self.mode = UIMode::List;
                                    self.create_state.name.clear();
                                    self.create_state.cursor_pos = 0;
                                    self.create_state.selected_type = 0;
                                    self.create_state.show_type_selection = false;
                                }
                                KeyCode::Enter => {
                                    if !self.create_state.name.is_empty() {
                                        self.create_state.show_type_selection = true;
                                    } else {
                                        self.message = Some("Agent name cannot be empty".to_string());
                                    }
                                }
                                KeyCode::Char(c) => {
                                    self.create_state.name.insert(self.create_state.cursor_pos, c);
                                    self.create_state.cursor_pos += 1;
                                }
                                KeyCode::Backspace => {
                                    if self.create_state.cursor_pos > 0 {
                                        self.create_state.cursor_pos -= 1;
                                        self.create_state.name.remove(self.create_state.cursor_pos);
                                    }
                                }
                                KeyCode::Left => {
                                    if self.create_state.cursor_pos > 0 {
                                        self.create_state.cursor_pos -= 1;
                                    }
                                }
                                KeyCode::Right => {
                                    if self.create_state.cursor_pos < self.create_state.name.len() {
                                        self.create_state.cursor_pos += 1;
                                    }
                                }
                                _ => {}
                            }
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

        let (width, height) = terminal::size()?;
        
        // Add top margin
        execute!(stdout, Print("\n"))?;
        
        // Header with colored background
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkBlue),
            SetForegroundColor(Color::White),
            SetAttribute(Attribute::Bold)
        )?;
        
        // Center the title with margin
        let title = " Agent Pool Manager ";
        let margin = 2;
        let inner_width = (width as usize).saturating_sub(margin * 2);
        let padding = " ".repeat((inner_width.saturating_sub(title.len())) / 2);
        execute!(
            stdout,
            Print(" ".repeat(margin)),
            Print(format!("{}{}{}", padding, title, padding)),
            Print(" ".repeat(margin)),
            Print("\n"),
            ResetColor
        )?;

        // Main content area with border
        self.draw_border(stdout, width)?;
        
        match self.mode {
            UIMode::List => self.draw_agent_list(stdout, width, height)?,
            UIMode::CreateNew => self.draw_create_form(stdout, width, height)?,
        }

        // Footer
        self.draw_footer(stdout, width, height)?;

        stdout.flush()?;
        Ok(())
    }

    fn draw_border(&self, stdout: &mut io::Stdout, width: u16) -> Result<()> {
        let margin = 2;
        let inner_width = (width as usize).saturating_sub(margin * 2);
        
        execute!(
            stdout,
            Print(" ".repeat(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("┌"),
            Print("─".repeat(inner_width - 2)),
            Print("┐\n"),
            ResetColor
        )?;
        Ok(())
    }

    fn draw_agent_list(&self, stdout: &mut io::Stdout, width: u16, _height: u16) -> Result<()> {
        let agents = self.manager.list_agents();
        let margin = 2;
        let inner_width = (width as usize).saturating_sub(margin * 2);
        
        // Option: Create New Agent
        self.draw_menu_item(stdout, 0, "Create New Agent", None, None, width)?;
        
        // Separator  
        execute!(
            stdout,
            Print(" ".repeat(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("├"),
            Print("─".repeat(inner_width.saturating_sub(2))),
            Print("┤\n"),
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
            
            self.draw_menu_item(
                stdout,
                real_index,
                &agent.name,
                Some(status),
                Some(&memo),
                width
            )?;
        }

        // Fill remaining space
        let total_items = agents.len() + 1;
        let margin = 2;
        let inner_width = (width as usize).saturating_sub(margin * 2);
        
        for _ in total_items..15 {
            execute!(
                stdout,
                Print(" ".repeat(margin)),
                SetForegroundColor(Color::DarkGrey),
                Print("│ "),
                Print(" ".repeat(inner_width.saturating_sub(4))),  // -4 for "│ " and " │"
                Print(" │\n"),
                ResetColor
            )?;
        }

        Ok(())
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
        let margin = 2;
        
        execute!(
            stdout,
            Print(" ".repeat(margin)),
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
        let status_width = status.map(|(s, _)| s.len() + 3).unwrap_or(0);
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
        actual_width += 2; // "◆ " or "  " indicator
        actual_width += truncated_name.len();
        
        if status.is_some() {
            actual_width += 1; // space before status
            actual_width += status.map(|(s, _)| s.len() + 2).unwrap_or(0); // status text + padding
        }
        
        if let Some(memo_text) = memo {
            if memo_width > 10 {
                actual_width += 1; // space before memo
                let truncated_memo = truncate_string(memo_text, memo_width.saturating_sub(1));
                actual_width += truncated_memo.len();
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
            Print(" │\n"),
            ResetColor
        )?;

        Ok(())
    }

    fn draw_create_form(&self, stdout: &mut io::Stdout, width: u16, _height: u16) -> Result<()> {
        let margin = 2;
        let inner_width = (width as usize).saturating_sub(margin * 2);
        
        execute!(
            stdout,
            Print(" ".repeat(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("│ "),
            SetForegroundColor(Color::Cyan),
            SetAttribute(Attribute::Bold),
            Print("Create New Agent"),
            ResetColor
        )?;
        
        // Fill to align with border
        let text_len = "Create New Agent".len();
        let padding = inner_width.saturating_sub(4 + text_len); // -4 for "│ " and " │"
        execute!(
            stdout,
            Print(" ".repeat(padding)),
            SetForegroundColor(Color::DarkGrey),
            Print(" │\n"),
            ResetColor
        )?;
        
        execute!(
            stdout,
            Print(" ".repeat(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("├"),
            Print("─".repeat(inner_width.saturating_sub(2))),
            Print("┤\n"),
            ResetColor
        )?;

        // Instructions
        execute!(
            stdout,
            Print(" ".repeat(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("│ "),
            SetForegroundColor(Color::White),
            Print("Enter agent name:"),
            Print(" ".repeat(inner_width.saturating_sub(20))),
            SetForegroundColor(Color::DarkGrey),
            Print("│\n"),
            ResetColor
        )?;
        
        // Input field
        execute!(
            stdout,
            Print(" ".repeat(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("│ "),
            Print("  "),
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::White),
            Print(" ")
        )?;
        
        let input_width = 40.min(inner_width.saturating_sub(10));
        let display_name = if self.create_state.name.len() > input_width {
            &self.create_state.name[self.create_state.name.len() - input_width..]
        } else {
            &self.create_state.name
        };
        
        execute!(
            stdout,
            Print(format!("{:<width$}", display_name, width = input_width)),
            Print(" "),
            ResetColor
        )?;
        
        let remaining = inner_width.saturating_sub(7 + input_width);
        execute!(
            stdout,
            Print(" ".repeat(remaining)),
            SetForegroundColor(Color::DarkGrey),
            Print("│\n"),
            ResetColor
        )?;
        
        // Empty line
        execute!(
            stdout,
            Print(" ".repeat(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("│"),
            Print(" ".repeat(inner_width.saturating_sub(2))),
            Print("│\n"),
            ResetColor
        )?;
        
        // Help text
        execute!(
            stdout,
            Print(" ".repeat(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("│ "),
            SetForegroundColor(Color::DarkGrey),
            Print(if self.create_state.show_type_selection {
                "Select agent role with ↑↓, Enter to create"
            } else {
                "Press Enter to continue, ESC to cancel"
            }),
            Print(" ".repeat(inner_width.saturating_sub(if self.create_state.show_type_selection { 44 } else { 39 }))),
            Print("│\n"),
            ResetColor
        )?;
        
        // Show agent type selection if name is entered
        if self.create_state.show_type_selection {
            // Separator
            execute!(
                stdout,
                Print(" ".repeat(margin)),
                SetForegroundColor(Color::DarkGrey),
                Print("├"),
                Print("─".repeat(inner_width.saturating_sub(2))),
                Print("┤\n"),
                ResetColor
            )?;
            
            // Agent type selection
            execute!(
                stdout,
                Print(" ".repeat(margin)),
                SetForegroundColor(Color::DarkGrey),
                Print("│ "),
                SetForegroundColor(Color::White),
                Print("Select agent role:"),
                Print(" ".repeat(inner_width.saturating_sub(21))),
                SetForegroundColor(Color::DarkGrey),
                Print("│\n"),
                ResetColor
            )?;
            
            // Role options
            for (i, (role_name, description)) in AGENT_ROLES.iter().enumerate() {
                execute!(
                    stdout,
                    Print(" ".repeat(margin)),
                    SetForegroundColor(Color::DarkGrey),
                    Print("│ "),
                    Print("  ")
                )?;
                
                if i == self.create_state.selected_type {
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
                
                execute!(
                    stdout,
                    SetForegroundColor(if i == self.create_state.selected_type { Color::White } else { Color::DarkGrey }),
                    Print(format!("{:<10} - {}", role_name, description))
                )?;
                
                let content_len = 14 + role_name.len() + description.len();
                execute!(
                    stdout,
                    Print(" ".repeat(inner_width.saturating_sub(content_len + 4))),
                    SetForegroundColor(Color::DarkGrey),
                    Print("│\n"),
                    ResetColor
                )?;
            }
        }

        Ok(())
    }

    fn draw_footer(&self, stdout: &mut io::Stdout, width: u16, height: u16) -> Result<()> {
        let margin = 2;
        let inner_width = (width as usize).saturating_sub(margin * 2);
        
        // Calculate footer position (leave space for margins and controls)
        let footer_pos = height.saturating_sub(5);
        
        // Bottom border
        execute!(
            stdout,
            cursor::MoveTo(0, footer_pos),
            Print(" ".repeat(margin)),
            SetForegroundColor(Color::DarkGrey),
            Print("└"),
            Print("─".repeat(inner_width - 2)),
            Print("┘\n"),
            ResetColor
        )?;

        // Status message
        if let Some(msg) = &self.message {
            execute!(
                stdout,
                Print(" ".repeat(margin)),
                SetForegroundColor(Color::Yellow),
                Print(format!(" ▶ {}\n", msg)),
                ResetColor
            )?;
        } else {
            execute!(stdout, Print("\n"))?;
        }

        // Controls
        let controls = match self.mode {
            UIMode::List => vec![
                ("↑↓/jk", "Navigate"),
                ("Enter/a", "Attach"),
                ("Space", "Start/Stop"),
                ("n", "New"),
                ("r", "Refresh"),
                ("q", "Quit"),
            ],
            UIMode::CreateNew => vec![
                ("ESC", "Cancel"),
                ("Enter", "Create"),
            ],
        };

        execute!(stdout, Print(" ".repeat(margin)), SetForegroundColor(Color::DarkGrey))?;
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
        execute!(stdout, Print("\n"), ResetColor)?;

        Ok(())
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
            return Ok(());
        }
        
        // Attach to agent session
        let agents = self.manager.list_agents();
        if let Some(agent) = agents.get(self.selected_index - 1) {
            // Check if agent is running
            match agent.metadata.tmux.status {
                AgentStatus::Active => {
                    // Exit UI and attach to tmux session
                    execute!(io::stdout(), cursor::Show, LeaveAlternateScreen)?;
                    terminal::disable_raw_mode()?;
                    
                    // Attach to the tmux session
                    let session_name = &agent.metadata.tmux.session_name;
                    std::process::Command::new("tmux")
                        .args(&["attach-session", "-t", session_name])
                        .status()?;
                    
                    // Re-enter UI after detaching
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
    
    async fn create_agent(&mut self) -> Result<()> {
        let name = self.create_state.name.clone();
        let agent_role = AGENT_ROLES[self.create_state.selected_type].0;
        
        // Create the agent with the selected role as template
        match self.manager.create_agent(name.clone(), Some(agent_role.to_string())).await {
            Ok(_) => {
                self.message = Some(format!("Created {} agent: {}", agent_role, name));
                self.mode = UIMode::List;
                self.create_state.name.clear();
                self.create_state.cursor_pos = 0;
                self.create_state.selected_type = 0;
                self.create_state.show_type_selection = false;
                
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