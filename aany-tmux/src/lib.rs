// Standard library imports
use std::process::{Command, Output};
use std::io;
use std::time::Duration;
use std::thread;

// Module declarations
pub mod session;
pub mod async_proxy;
pub mod safe_proxy;
pub mod interference;

// Re-export for convenience
pub use session::TmuxSession;
pub use async_proxy::{AsyncTmuxProxy, PaneMonitor};
pub use safe_proxy::{SafeTmuxProxy, SafetyConfig, PaneHealth, HealthStatus};
pub use interference::{InterferenceDetector, InterferenceEvent, RecoveryReport};

// Our TmuxProxy struct - this will hold the state for a single tmux pane
// In Rust, we define data structures with 'struct'
pub struct TmuxProxy {
    pub(crate) session: String,
    pub(crate) window: u32,
    pub(crate) pane: u32,
}

// Implementation block - where we define methods for our struct
impl TmuxProxy {
    // Constructor - called with TmuxProxy::new()
    // 'pub' means public (accessible from outside this module)
    pub fn new(session: String, window: u32, pane: u32) -> Self {
        TmuxProxy {
            session,
            window,
            pane,
        }
    }
    
    // Get the target string for tmux commands (e.g., "my-session:0.1")
    fn target(&self) -> String {
        format!("{}:{}.{}", self.session, self.window, self.pane)
    }
    
    // Send keys to the tmux pane
    // Result<T, E> is Rust's error handling - either Ok(value) or Err(error)
    pub fn send_keys(&self, keys: &str) -> io::Result<Output> {
        Command::new("tmux")
            .args(["send-keys", "-t", &self.target(), keys])
            .output()
    }
    
    // Send keys followed by Enter
    pub fn send_line(&self, line: &str) -> io::Result<Output> {
        Command::new("tmux")
            .args(["send-keys", "-t", &self.target(), line, "Enter"])
            .output()
    }
    
    // Capture the current pane content
    pub fn capture_pane(&self) -> io::Result<String> {
        let output = Command::new("tmux")
            .args(["capture-pane", "-t", &self.target(), "-p"])
            .output()?;
        
        // Convert bytes to String, handling potential UTF-8 errors
        String::from_utf8(output.stdout)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
    
    // Clear the pane
    pub fn clear(&self) -> io::Result<Output> {
        self.send_keys("C-l")
    }
    
    // Check if the session:window:pane exists
    pub fn exists(&self) -> bool {
        Command::new("tmux")
            .args(["has-session", "-t", &self.target()])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    // Wait for a specific string to appear in the pane
    pub fn wait_for_text(&self, text: &str, timeout_ms: u64) -> io::Result<bool> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);
        
        while start.elapsed() < timeout {
            let content = self.capture_pane()?;
            if content.contains(text) {
                return Ok(true);
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        Ok(false)
    }
    
    // Get only the last N lines from the pane
    pub fn capture_last_lines(&self, n: usize) -> io::Result<String> {
        let content = self.capture_pane()?;
        let lines: Vec<&str> = content.lines().collect();
        let start = lines.len().saturating_sub(n);
        Ok(lines[start..].join("\n"))
    }
    
    // Send Ctrl+C to interrupt current process
    pub fn interrupt(&self) -> io::Result<Output> {
        self.send_keys("C-c")
    }
    
    // Execute a command and wait for prompt to return
    pub fn execute_and_wait(&self, command: &str, prompt: &str, timeout_ms: u64) -> io::Result<String> {
        // Capture initial state
        let before = self.capture_pane()?;
        
        // Send the command
        self.send_line(command)?;
        
        // Wait for the prompt to appear
        let found = self.wait_for_text(prompt, timeout_ms)?;
        if !found {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("Timeout waiting for prompt: {}", prompt)
            ));
        }
        
        // Capture the result
        let after = self.capture_pane()?;
        
        // Extract just the new output
        Ok(after.trim_start_matches(&before).to_string())
    }
    
    // Get pane dimensions
    pub fn get_size(&self) -> io::Result<(u32, u32)> {
        let width_output = Command::new("tmux")
            .args(["display", "-t", &self.target(), "-p", "#{pane_width}"])
            .output()?;
        
        let height_output = Command::new("tmux")
            .args(["display", "-t", &self.target(), "-p", "#{pane_height}"])
            .output()?;
        
        let width = String::from_utf8_lossy(&width_output.stdout)
            .trim()
            .parse::<u32>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            
        let height = String::from_utf8_lossy(&height_output.stdout)
            .trim()
            .parse::<u32>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            
        Ok((width, height))
    }
    
    // Resize the pane
    pub fn resize(&self, width: u32, height: u32) -> io::Result<Output> {
        Command::new("tmux")
            .args([
                "resize-pane",
                "-t", &self.target(),
                "-x", &width.to_string(),
                "-y", &height.to_string()
            ])
            .output()
    }
    
    // Scroll up/down in copy mode
    pub fn scroll(&self, lines: i32) -> io::Result<()> {
        // Enter copy mode
        self.send_keys("[")?;
        thread::sleep(Duration::from_millis(50));
        
        // Scroll
        if lines > 0 {
            for _ in 0..lines {
                self.send_keys("C-u")?;
            }
        } else {
            for _ in 0..lines.abs() {
                self.send_keys("C-d")?;
            }
        }
        
        // Exit copy mode
        thread::sleep(Duration::from_millis(50));
        self.send_keys("q")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_formatting() {
        let proxy = TmuxProxy::new("test-session".to_string(), 0, 1);
        assert_eq!(proxy.target(), "test-session:0.1");
    }
}