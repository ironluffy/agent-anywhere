use crate::{TmuxProxy, TmuxSession};
use std::process::Command;
use std::io;

/// Safety configuration for tmux operations
#[derive(Clone, Debug)]
pub struct SafetyConfig {
    /// Maximum allowed panes per window (default: 1)
    pub max_panes_per_window: u32,
    /// Warn when interacting with multi-pane windows
    pub warn_on_multi_pane: bool,
    /// Automatically clean up extra panes
    pub auto_cleanup_panes: bool,
    /// Require explicit confirmation for split operations
    pub require_split_confirmation: bool,
    /// Allow read-only monitoring without warnings
    pub allow_readonly_monitoring: bool,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        SafetyConfig {
            max_panes_per_window: 1,
            warn_on_multi_pane: true,
            auto_cleanup_panes: false,
            require_split_confirmation: true,
            allow_readonly_monitoring: true,
        }
    }
}

/// Safe wrapper around TmuxProxy with pane control
pub struct SafeTmuxProxy {
    pub(crate) inner: TmuxProxy,
    pub(crate) config: SafetyConfig,
}

impl SafeTmuxProxy {
    /// Create new safe proxy with default safety settings
    pub fn new(session: String, window: u32, pane: u32) -> io::Result<Self> {
        Self::with_config(session, window, pane, SafetyConfig::default())
    }
    
    /// Create new safe proxy with custom safety settings
    pub fn with_config(session: String, window: u32, pane: u32, config: SafetyConfig) -> io::Result<Self> {
        let proxy = TmuxProxy::new(session.clone(), window, pane);
        
        // Validate current state
        let pane_count = Self::count_panes(&session, window)?;
        
        if config.warn_on_multi_pane && pane_count > 1 {
            eprintln!("⚠️  WARNING: Window {} has {} panes (expected 1)", window, pane_count);
            eprintln!("⚠️  Consider using dedicated windows instead of splitting panes");
        }
        
        if pane_count > config.max_panes_per_window && config.auto_cleanup_panes {
            eprintln!("🧹 Auto-cleanup: Window has {} panes, maximum allowed is {}", 
                     pane_count, config.max_panes_per_window);
            // TODO: Implement cleanup
        }
        
        Ok(SafeTmuxProxy {
            inner: proxy,
            config,
        })
    }
    
    /// Count panes in a window
    fn count_panes(session: &str, window: u32) -> io::Result<u32> {
        let output = Command::new("tmux")
            .args([
                "list-panes",
                "-t", &format!("{}:{}", session, window),
                "-F", "#{pane_index}"
            ])
            .output()?;
            
        if !output.status.success() {
            return Ok(0);
        }
        
        let count = String::from_utf8_lossy(&output.stdout)
            .lines()
            .count() as u32;
            
        Ok(count)
    }
    
    /// Get all pane indices in a window
    fn get_pane_indices(session: &str, window: u32) -> io::Result<Vec<u32>> {
        let output = Command::new("tmux")
            .args([
                "list-panes",
                "-t", &format!("{}:{}", session, window),
                "-F", "#{pane_index}"
            ])
            .output()?;
            
        if !output.status.success() {
            return Ok(vec![]);
        }
        
        let indices: Vec<u32> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| line.trim().parse().ok())
            .collect();
            
        Ok(indices)
    }
    
    /// Safely split pane (with warnings and confirmations)
    pub fn safe_split_pane(&self, vertical: bool, force: bool) -> io::Result<u32> {
        let session = &self.inner.session;
        let window = self.inner.window;
        let pane = self.inner.pane;
        
        // Check current pane count
        let current_count = Self::count_panes(session, window)?;
        
        if current_count >= self.config.max_panes_per_window && !force {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "Window already has {} panes (max: {}). Use force=true to override or create a new window instead.",
                    current_count, self.config.max_panes_per_window
                )
            ));
        }
        
        if self.config.require_split_confirmation && !force {
            eprintln!("⚠️  SPLIT CONFIRMATION REQUIRED");
            eprintln!("⚠️  Window {} currently has {} pane(s)", window, current_count);
            eprintln!("⚠️  Splitting panes can make agent control more complex");
            eprintln!("⚠️  Consider using a new window instead");
            eprintln!("⚠️  Use force=true to proceed with split");
            
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Split confirmation required. Use force=true to proceed."
            ));
        }
        
        // Proceed with split
        TmuxSession::split_pane(session, window, pane, vertical)
    }
    
    /// Create a new window (recommended over splitting panes)
    pub fn create_window(&self, name: Option<&str>) -> io::Result<(u32, SafeTmuxProxy)> {
        let window_idx = TmuxSession::new_window(&self.inner.session, name)?;
        let new_proxy = SafeTmuxProxy::with_config(
            self.inner.session.clone(),
            window_idx,
            0,
            self.config.clone()
        )?;
        
        Ok((window_idx, new_proxy))
    }
    
    /// Clean up extra panes in a window, keeping only the first one
    pub fn cleanup_extra_panes(&self) -> io::Result<u32> {
        let session = &self.inner.session;
        let window = self.inner.window;
        
        let panes = Self::get_pane_indices(session, window)?;
        let mut cleaned = 0;
        
        // Keep only pane 0, kill others
        for &pane_idx in panes.iter() {
            if pane_idx > 0 {
                let target = format!("{}:{}.{}", session, window, pane_idx);
                let result = Command::new("tmux")
                    .args(["kill-pane", "-t", &target])
                    .output()?;
                    
                if result.status.success() {
                    cleaned += 1;
                }
            }
        }
        
        if cleaned > 0 {
            println!("🧹 Cleaned up {} extra pane(s) in window {}", cleaned, window);
        }
        
        Ok(cleaned)
    }
    
    /// Get pane health status
    pub fn get_pane_health(&self) -> io::Result<PaneHealth> {
        let session = &self.inner.session;
        let window = self.inner.window;
        
        let pane_count = Self::count_panes(session, window)?;
        let (width, height) = self.inner.get_size()?;
        
        let status = if pane_count > self.config.max_panes_per_window {
            HealthStatus::Critical
        } else if pane_count > 1 {
            HealthStatus::Warning
        } else if width < 40 || height < 10 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };
        
        Ok(PaneHealth {
            pane_count,
            window_index: window,
            dimensions: (width, height),
            status,
        })
    }
    
    // Delegate safe methods to inner proxy
    pub fn send_keys(&self, keys: &str) -> io::Result<std::process::Output> {
        self.inner.send_keys(keys)
    }
    
    pub fn send_line(&self, line: &str) -> io::Result<std::process::Output> {
        self.inner.send_line(line)
    }
    
    pub fn capture_pane(&self) -> io::Result<String> {
        self.inner.capture_pane()
    }
    
    pub fn wait_for_text(&self, text: &str, timeout_ms: u64) -> io::Result<bool> {
        self.inner.wait_for_text(text, timeout_ms)
    }
    
    pub fn clear(&self) -> io::Result<std::process::Output> {
        self.inner.clear()
    }
    
    pub fn interrupt(&self) -> io::Result<std::process::Output> {
        self.inner.interrupt()
    }
    
    pub fn exists(&self) -> bool {
        self.inner.exists()
    }
}

#[derive(Debug, Clone)]
pub struct PaneHealth {
    pub pane_count: u32,
    pub window_index: u32,
    pub dimensions: (u32, u32),
    pub status: HealthStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

impl PaneHealth {
    pub fn report(&self) {
        let icon = match self.status {
            HealthStatus::Healthy => "✅",
            HealthStatus::Warning => "⚠️",
            HealthStatus::Critical => "❌",
        };
        
        println!("{} Window {} Health Report:", icon, self.window_index);
        println!("   Panes: {}", self.pane_count);
        println!("   Dimensions: {}x{}", self.dimensions.0, self.dimensions.1);
        
        match self.status {
            HealthStatus::Critical => {
                println!("   ⚠️  CRITICAL: Too many panes! Agent control may be compromised.");
                println!("   ⚠️  Recommend: Use cleanup_extra_panes() or create new windows.");
            }
            HealthStatus::Warning => {
                if self.pane_count > 1 {
                    println!("   ⚠️  WARNING: Multiple panes detected. Consider using separate windows.");
                }
                if self.dimensions.0 < 40 || self.dimensions.1 < 10 {
                    println!("   ⚠️  WARNING: Pane size is very small ({}x{}).", 
                           self.dimensions.0, self.dimensions.1);
                }
            }
            HealthStatus::Healthy => {
                println!("   ✅ Healthy: Single pane with good dimensions.");
            }
        }
    }
}