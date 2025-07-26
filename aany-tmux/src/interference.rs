use crate::SafeTmuxProxy;
use std::process::Command;
use std::io;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Detects and handles human interference in agent-controlled tmux sessions
pub struct InterferenceDetector {
    pub proxy: SafeTmuxProxy,
    last_known_state: SessionState,
    interference_callbacks: Vec<Box<dyn Fn(&InterferenceEvent) + Send>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SessionState {
    pub pane_count: u32,
    pub active_pane: u32,
    pub pane_dimensions: HashMap<u32, (u32, u32)>,
    pub window_name: String,
    pub last_content_hash: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum InterferenceEvent {
    /// Human manually split/created new panes
    PaneCountChanged { before: u32, after: u32 },
    
    /// Human switched to different pane
    ActivePaneChanged { before: u32, after: u32 },
    
    /// Human resized panes
    PaneDimensionsChanged { pane: u32, before: (u32, u32), after: (u32, u32) },
    
    /// Human typed commands (content changed unexpectedly)
    UnexpectedInput { content_snippet: String },
    
    /// Human killed a pane the agent was using
    PaneKilled { pane: u32 },
    
    /// Human renamed the window
    WindowRenamed { before: String, after: String },
    
    /// Human attached to the session
    HumanAttached { client_info: String, read_only: bool },
}

impl InterferenceDetector {
    pub fn new(proxy: SafeTmuxProxy) -> io::Result<Self> {
        let state = Self::capture_state(&proxy)?;
        Ok(InterferenceDetector {
            proxy,
            last_known_state: state,
            interference_callbacks: Vec::new(),
        })
    }
    
    /// Capture current session state
    fn capture_state(proxy: &SafeTmuxProxy) -> io::Result<SessionState> {
        let session = &proxy.inner.session;
        let window = proxy.inner.window;
        
        // Get pane count and dimensions
        let mut pane_dimensions = HashMap::new();
        let output = Command::new("tmux")
            .args([
                "list-panes",
                "-t", &format!("{}:{}", session, window),
                "-F", "#{pane_index}:#{pane_width}:#{pane_height}:#{pane_active}"
            ])
            .output()?;
            
        let mut pane_count = 0;
        let mut active_pane = 0;
        
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 4 {
                let idx = parts[0].parse::<u32>().unwrap_or(0);
                let width = parts[1].parse::<u32>().unwrap_or(0);
                let height = parts[2].parse::<u32>().unwrap_or(0);
                let is_active = parts[3] == "1";
                
                pane_dimensions.insert(idx, (width, height));
                pane_count += 1;
                
                if is_active {
                    active_pane = idx;
                }
            }
        }
        
        // Get window name
        let window_name = Command::new("tmux")
            .args([
                "display-message",
                "-t", &format!("{}:{}", session, window),
                "-p", "#{window_name}"
            ])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        
        // Get content hash (simplified - just length for now)
        let content = proxy.capture_pane().unwrap_or_default();
        let content_hash = content.len() as u64;
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Ok(SessionState {
            pane_count,
            active_pane,
            pane_dimensions,
            window_name,
            last_content_hash: content_hash,
            timestamp,
        })
    }
    
    /// Check for interference and return detected events
    pub fn check_interference(&mut self) -> io::Result<Vec<InterferenceEvent>> {
        let current_state = Self::capture_state(&self.proxy)?;
        let mut events = Vec::new();
        
        // Check pane count changes
        if current_state.pane_count != self.last_known_state.pane_count {
            events.push(InterferenceEvent::PaneCountChanged {
                before: self.last_known_state.pane_count,
                after: current_state.pane_count,
            });
            
            // Check if specific panes were killed
            for (pane, _) in &self.last_known_state.pane_dimensions {
                if !current_state.pane_dimensions.contains_key(pane) {
                    events.push(InterferenceEvent::PaneKilled { pane: *pane });
                }
            }
        }
        
        // Check active pane change
        if current_state.active_pane != self.last_known_state.active_pane {
            events.push(InterferenceEvent::ActivePaneChanged {
                before: self.last_known_state.active_pane,
                after: current_state.active_pane,
            });
        }
        
        // Check dimension changes
        for (pane, new_dims) in &current_state.pane_dimensions {
            if let Some(old_dims) = self.last_known_state.pane_dimensions.get(pane) {
                if old_dims != new_dims {
                    events.push(InterferenceEvent::PaneDimensionsChanged {
                        pane: *pane,
                        before: *old_dims,
                        after: *new_dims,
                    });
                }
            }
        }
        
        // Check window rename
        if current_state.window_name != self.last_known_state.window_name {
            events.push(InterferenceEvent::WindowRenamed {
                before: self.last_known_state.window_name.clone(),
                after: current_state.window_name.clone(),
            });
        }
        
        // Check for unexpected input (simplified check)
        if current_state.last_content_hash != self.last_known_state.last_content_hash {
            // In a real implementation, we'd do more sophisticated content diffing
            let content = self.proxy.capture_pane().unwrap_or_default();
            let last_line = content.lines().last().unwrap_or("");
            
            // Only flag as interference if we haven't sent commands recently
            let time_diff = current_state.timestamp - self.last_known_state.timestamp;
            if time_diff > 2 {  // More than 2 seconds since last check
                events.push(InterferenceEvent::UnexpectedInput {
                    content_snippet: last_line.to_string(),
                });
            }
        }
        
        // Check for human attachment
        if let Ok(client_infos) = self.check_attached_clients() {
            for (client_info, read_only) in client_infos {
                events.push(InterferenceEvent::HumanAttached {
                    client_info,
                    read_only,
                });
            }
        }
        
        // Update state for next check
        self.last_known_state = current_state;
        
        // Call callbacks
        for event in &events {
            for callback in &self.interference_callbacks {
                callback(event);
            }
        }
        
        Ok(events)
    }
    
    /// Check who's attached to the session
    fn check_attached_clients(&self) -> io::Result<Vec<(String, bool)>> {
        let output = Command::new("tmux")
            .args([
                "list-clients",
                "-t", &self.proxy.inner.session,
                "-F", "#{client_tty}:#{client_flags}:#{client_readonly}"
            ])
            .output()?;
            
        let clients: Vec<(String, bool)> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 {
                    let tty = parts[0];
                    let flags = parts[1];
                    let readonly = parts[2];
                    
                    // Check if read-only flag is set
                    let is_readonly = readonly == "1" || flags.contains("read-only");
                    
                    Some((format!("tty: {}", tty), is_readonly))
                } else {
                    None
                }
            })
            .collect();
            
        Ok(clients)
    }
    
    /// Add callback for interference events
    pub fn on_interference<F>(&mut self, callback: F)
    where
        F: Fn(&InterferenceEvent) + Send + 'static,
    {
        self.interference_callbacks.push(Box::new(callback));
    }
    
    /// Attempt to recover from interference
    pub fn recover(&mut self) -> io::Result<RecoveryReport> {
        let mut report = RecoveryReport::default();
        let current_state = Self::capture_state(&self.proxy)?;
        
        // Handle too many panes
        if current_state.pane_count > 1 {
            report.extra_panes_found = current_state.pane_count - 1;
            
            if self.proxy.config.auto_cleanup_panes {
                let cleaned = self.proxy.cleanup_extra_panes()?;
                report.panes_cleaned = cleaned;
            }
        }
        
        // Switch back to expected pane
        if current_state.active_pane != self.proxy.inner.pane {
            Command::new("tmux")
                .args([
                    "select-pane",
                    "-t", &format!("{}:{}.{}", 
                        self.proxy.inner.session,
                        self.proxy.inner.window,
                        self.proxy.inner.pane
                    )
                ])
                .output()?;
            report.active_pane_restored = true;
        }
        
        // Clear any unexpected input
        if report.extra_panes_found == 0 {  // Only clear if we're in control
            self.proxy.clear()?;
            report.pane_cleared = true;
        }
        
        Ok(report)
    }
    
    /// Create a lock message to warn humans
    pub fn display_lock_message(&self) -> io::Result<()> {
        let message = concat!(
            "\\033[1;33m",  // Yellow bold
            "╔══════════════════════════════════════╗\\n",
            "║  ⚠️  AGENT CONTROLLED SESSION  ⚠️   ║\\n", 
            "║                                      ║\\n",
            "║  This tmux session is managed by     ║\\n",
            "║  an automated agent.                 ║\\n",
            "║                                      ║\\n",
            "║  Manual changes may disrupt agent    ║\\n",
            "║  operations!                         ║\\n",
            "║                                      ║\\n",
            "║  👁️  Monitoring: tmux attach -r -t", 
        );
        
        let message2 = concat!(
            "  ║\\n",
            "╚══════════════════════════════════════╝",
            "\\033[0m"  // Reset
        );
        
        self.proxy.send_line(&format!("echo -e \"{}{}{}\"", 
            message, self.proxy.inner.session, message2))?;
        Ok(())
    }
    
    /// Display a welcome message for read-only monitors
    pub fn display_monitor_welcome(&self) -> io::Result<()> {
        let message = concat!(
            "\\033[1;32m",  // Green bold
            "╔══════════════════════════════════════╗\\n",
            "║  👁️  MONITORING MODE DETECTED  👁️   ║\\n", 
            "║                                      ║\\n",
            "║  Welcome! You are in read-only mode. ║\\n",
            "║  You can safely watch the agent      ║\\n",
            "║  without disrupting operations.      ║\\n",
            "║                                      ║\\n",
            "║  Agent will continue normally.       ║\\n",
            "╚══════════════════════════════════════╝",
            "\\033[0m"  // Reset
        );
        
        self.proxy.send_line(&format!("echo -e \"{}\"", message))?;
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct RecoveryReport {
    pub extra_panes_found: u32,
    pub panes_cleaned: u32,
    pub active_pane_restored: bool,
    pub pane_cleared: bool,
}

impl RecoveryReport {
    pub fn summary(&self) -> String {
        let mut actions = Vec::new();
        
        if self.panes_cleaned > 0 {
            actions.push(format!("Cleaned {} extra panes", self.panes_cleaned));
        }
        if self.active_pane_restored {
            actions.push("Restored active pane".to_string());
        }
        if self.pane_cleared {
            actions.push("Cleared pane content".to_string());
        }
        
        if actions.is_empty() {
            "No recovery needed".to_string()
        } else {
            format!("Recovery actions: {}", actions.join(", "))
        }
    }
}

// Extension trait for SafeTmuxProxy
impl SafeTmuxProxy {
    /// Create an interference detector for this proxy
    pub fn with_interference_detection(self) -> io::Result<InterferenceDetector> {
        InterferenceDetector::new(self)
    }
}