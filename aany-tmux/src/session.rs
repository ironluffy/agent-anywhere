use std::process::Command;
use std::io;

pub struct TmuxSession;

impl TmuxSession {
    // Create a new tmux session
    pub fn create(name: &str, detached: bool) -> io::Result<()> {
        let mut cmd = Command::new("tmux");
        cmd.arg("new-session");
        
        if detached {
            cmd.arg("-d");
        }
        
        cmd.arg("-s").arg(name);
        
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(())
    }
    
    // List all sessions
    pub fn list() -> io::Result<Vec<String>> {
        let output = Command::new("tmux")
            .args(["list-sessions", "-F", "#{session_name}"])
            .output()?;
            
        if !output.status.success() {
            // No sessions exist
            return Ok(Vec::new());
        }
        
        let sessions = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();
            
        Ok(sessions)
    }
    
    // Kill a session
    pub fn kill(name: &str) -> io::Result<()> {
        let output = Command::new("tmux")
            .args(["kill-session", "-t", name])
            .output()?;
            
        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(())
    }
    
    // Check if session exists
    pub fn exists(name: &str) -> bool {
        Command::new("tmux")
            .args(["has-session", "-t", name])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    // Create a new window in a session
    pub fn new_window(session: &str, window_name: Option<&str>) -> io::Result<u32> {
        let mut cmd = Command::new("tmux");
        cmd.args(["new-window", "-t", session, "-P", "-F", "#{window_index}"]);
        
        if let Some(name) = window_name {
            cmd.arg("-n").arg(name);
        }
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        let window_index = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<u32>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            
        Ok(window_index)
    }
    
    // Split a pane (returns new pane index)
    pub fn split_pane(session: &str, window: u32, pane: u32, vertical: bool) -> io::Result<u32> {
        let target = format!("{}:{}.{}", session, window, pane);
        let mut cmd = Command::new("tmux");
        
        cmd.arg("split-window");
        cmd.arg("-t").arg(&target);
        cmd.args(["-P", "-F", "#{pane_index}"]);
        
        if vertical {
            cmd.arg("-h");  // Horizontal split creates vertical panes
        }
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        let pane_index = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<u32>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            
        Ok(pane_index)
    }
}