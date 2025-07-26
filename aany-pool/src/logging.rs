use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use chrono::{DateTime, Utc, Local};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct AgentLogger {
    agent_name: String,
    log_dir: PathBuf,
    session_logger: Arc<Mutex<BufWriter<File>>>,
    event_logger: Arc<Mutex<BufWriter<File>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub direction: Option<LogDirection>,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LogDirection {
    Input,
    Output,
    System,
}

impl AgentLogger {
    /// Create a new logger for an agent
    pub fn new(agent_name: String, log_dir: PathBuf) -> Result<Self> {
        // Ensure log directory exists
        std::fs::create_dir_all(&log_dir)?;
        
        // Create log files with timestamps
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        
        // Session log - raw terminal I/O
        let session_path = log_dir.join(format!("session_{}.log", timestamp));
        let session_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&session_path)?;
        let session_logger = Arc::new(Mutex::new(BufWriter::new(session_file)));
        
        // Event log - structured JSON events
        let event_path = log_dir.join(format!("events_{}.jsonl", timestamp));
        let event_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&event_path)?;
        let event_logger = Arc::new(Mutex::new(BufWriter::new(event_file)));
        
        // Create symlinks to latest logs
        let session_latest = log_dir.join("session_latest.log");
        let event_latest = log_dir.join("events_latest.jsonl");
        
        // Remove old symlinks if they exist
        let _ = std::fs::remove_file(&session_latest);
        let _ = std::fs::remove_file(&event_latest);
        
        // Create new symlinks
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&session_path, &session_latest)?;
            std::os::unix::fs::symlink(&event_path, &event_latest)?;
        }
        
        Ok(Self {
            agent_name,
            log_dir,
            session_logger,
            event_logger,
        })
    }
    
    /// Log raw session data (stdin/stdout)
    pub fn log_session(&self, content: &str) -> Result<()> {
        let mut logger = self.session_logger.lock().unwrap();
        writeln!(logger, "{}", content)?;
        logger.flush()?;
        Ok(())
    }
    
    /// Log session data with timestamp
    pub fn log_session_with_timestamp(&self, content: &str) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let mut logger = self.session_logger.lock().unwrap();
        writeln!(logger, "[{}] {}", timestamp, content)?;
        logger.flush()?;
        Ok(())
    }
    
    /// Log input from user
    pub fn log_input(&self, input: &str) -> Result<()> {
        // Log to session file
        self.log_session_with_timestamp(&format!(">>> {}", input))?;
        
        // Log as event
        let event = LogEvent {
            timestamp: Utc::now(),
            event_type: "user_input".to_string(),
            direction: Some(LogDirection::Input),
            content: input.to_string(),
            metadata: None,
        };
        self.log_event(&event)?;
        
        Ok(())
    }
    
    /// Log output from agent
    pub fn log_output(&self, output: &str) -> Result<()> {
        // Log to session file
        self.log_session_with_timestamp(&format!("<<< {}", output))?;
        
        // Log as event
        let event = LogEvent {
            timestamp: Utc::now(),
            event_type: "agent_output".to_string(),
            direction: Some(LogDirection::Output),
            content: output.to_string(),
            metadata: None,
        };
        self.log_event(&event)?;
        
        Ok(())
    }
    
    /// Log system event
    pub fn log_system(&self, message: &str, metadata: Option<serde_json::Value>) -> Result<()> {
        // Log to session file
        self.log_session_with_timestamp(&format!("[SYSTEM] {}", message))?;
        
        // Log as event
        let event = LogEvent {
            timestamp: Utc::now(),
            event_type: "system".to_string(),
            direction: Some(LogDirection::System),
            content: message.to_string(),
            metadata,
        };
        self.log_event(&event)?;
        
        Ok(())
    }
    
    /// Log a structured event
    pub fn log_event(&self, event: &LogEvent) -> Result<()> {
        let mut logger = self.event_logger.lock().unwrap();
        let json = serde_json::to_string(event)?;
        writeln!(logger, "{}", json)?;
        logger.flush()?;
        Ok(())
    }
    
    /// Get the path to the current session log
    pub fn session_log_path(&self) -> PathBuf {
        self.log_dir.join("session_latest.log")
    }
    
    /// Get the path to the current event log
    pub fn event_log_path(&self) -> PathBuf {
        self.log_dir.join("events_latest.jsonl")
    }
    
    /// Rotate logs (call this periodically or when logs get too large)
    pub fn rotate_logs(&self) -> Result<()> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        
        // Close current loggers by dropping and recreating
        // This is a simplified version - in production you'd want more sophisticated rotation
        
        self.log_system(
            "Rotating logs",
            Some(serde_json::json!({
                "timestamp": timestamp.to_string(),
                "agent": self.agent_name,
            }))
        )?;
        
        Ok(())
    }
}

/// TMux pipe configuration for automatic logging
pub fn setup_tmux_logging(session_name: &str, log_path: &Path) -> Result<()> {
    use std::process::Command;
    
    // Set up tmux pipe-pane to capture all output
    let output = Command::new("tmux")
        .args(&[
            "pipe-pane",
            "-t", session_name,
            "-o",
            &format!("cat >> {}", log_path.display())
        ])
        .output()?;
        
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to setup tmux logging: {}", error);
    }
    
    Ok(())
}

/// Parse session log for replay
pub fn parse_session_log(log_path: &Path) -> Result<Vec<(DateTime<Local>, String, LogDirection)>> {
    use std::io::{BufRead, BufReader};
    
    let file = File::open(log_path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        
        // Parse timestamp and content
        if let Some(timestamp_end) = line.find(']') {
            if timestamp_end > 1 && line.starts_with('[') {
                let timestamp_str = &line[1..timestamp_end];
                let content = &line[timestamp_end + 2..];
                
                if let Ok(timestamp) = DateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S%.3f") {
                    let direction = if content.starts_with(">>> ") {
                        LogDirection::Input
                    } else if content.starts_with("<<< ") {
                        LogDirection::Output
                    } else if content.starts_with("[SYSTEM] ") {
                        LogDirection::System
                    } else {
                        LogDirection::Output
                    };
                    
                    let clean_content = content
                        .trim_start_matches(">>> ")
                        .trim_start_matches("<<< ")
                        .trim_start_matches("[SYSTEM] ")
                        .to_string();
                    
                    entries.push((timestamp.with_timezone(&Local), clean_content, direction));
                }
            }
        }
    }
    
    Ok(entries)
}