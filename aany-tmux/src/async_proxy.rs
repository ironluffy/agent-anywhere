use tokio::process::Command;
use tokio::time::{sleep, timeout, Duration};
use std::sync::Arc;
use tokio::sync::mpsc;

// Async version of TmuxProxy
pub struct AsyncTmuxProxy {
    session: String,
    window: u32,
    pane: u32,
}

impl AsyncTmuxProxy {
    pub fn new(session: String, window: u32, pane: u32) -> Self {
        AsyncTmuxProxy {
            session,
            window,
            pane,
        }
    }
    
    fn target(&self) -> String {
        format!("{}:{}.{}", self.session, self.window, self.pane)
    }
    
    // Async send line
    pub async fn send_line(&self, line: &str) -> tokio::io::Result<()> {
        Command::new("tmux")
            .args(["send-keys", "-t", &self.target(), line, "Enter"])
            .output()
            .await?;
        Ok(())
    }
    
    // Async capture pane
    pub async fn capture_pane(&self) -> tokio::io::Result<String> {
        let output = Command::new("tmux")
            .args(["capture-pane", "-t", &self.target(), "-p"])
            .output()
            .await?;
        
        String::from_utf8(output.stdout)
            .map_err(|e| tokio::io::Error::new(tokio::io::ErrorKind::InvalidData, e))
    }
    
    // Async wait for text with timeout
    pub async fn wait_for_text(&self, text: &str, timeout_ms: u64) -> tokio::io::Result<bool> {
        let duration = Duration::from_millis(timeout_ms);
        
        match timeout(duration, self.wait_for_text_loop(text)).await {
            Ok(result) => result,
            Err(_) => Ok(false), // Timeout occurred
        }
    }
    
    async fn wait_for_text_loop(&self, text: &str) -> tokio::io::Result<bool> {
        loop {
            let content = self.capture_pane().await?;
            if content.contains(text) {
                return Ok(true);
            }
            sleep(Duration::from_millis(100)).await;
        }
    }
    
    // Run command with streaming output callback
    pub async fn run_with_callback<F>(
        &self,
        command: &str,
        mut callback: F,
    ) -> tokio::io::Result<()>
    where
        F: FnMut(String) + Send + 'static,
    {
        // Send the command
        self.send_line(command).await?;
        
        // Capture initial state
        let mut last_content = self.capture_pane().await?;
        
        // Monitor for changes
        loop {
            sleep(Duration::from_millis(100)).await;
            
            let current_content = self.capture_pane().await?;
            
            // Check if content changed
            if current_content != last_content {
                // Extract the new part
                let new_part = current_content
                    .strip_prefix(&last_content)
                    .unwrap_or(&current_content)
                    .to_string();
                
                // Call the callback with new content
                callback(new_part);
            }
            
            // Check if command completed (look for prompt)
            if current_content.ends_with("$ ") || current_content.ends_with("# ") {
                break;
            }
            
            last_content = current_content;
        }
        
        Ok(())
    }
    
    // Run multiple commands concurrently in different panes
    pub async fn run_concurrent(
        proxies: Vec<Arc<AsyncTmuxProxy>>,
        commands: Vec<String>,
    ) -> Vec<tokio::io::Result<String>> {
        let mut handles = vec![];
        
        for (proxy, command) in proxies.into_iter().zip(commands.into_iter()) {
            let handle = tokio::spawn(async move {
                proxy.send_line(&command).await?;
                sleep(Duration::from_millis(500)).await;
                proxy.capture_pane().await
            });
            handles.push(handle);
        }
        
        let mut results = vec![];
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(tokio::io::Error::new(
                    tokio::io::ErrorKind::Other,
                    e.to_string(),
                ))),
            }
        }
        
        results
    }
}

// Event stream for monitoring pane changes
pub struct PaneMonitor {
    proxy: Arc<AsyncTmuxProxy>,
    tx: mpsc::Sender<String>,
    rx: mpsc::Receiver<String>,
}

impl PaneMonitor {
    pub fn new(proxy: AsyncTmuxProxy) -> Self {
        let (tx, rx) = mpsc::channel(100);
        PaneMonitor {
            proxy: Arc::new(proxy),
            tx,
            rx,
        }
    }
    
    // Start monitoring in background
    pub fn start(&self) {
        let proxy = Arc::clone(&self.proxy);
        let tx = self.tx.clone();
        
        tokio::spawn(async move {
            let mut last_content = String::new();
            
            loop {
                if let Ok(content) = proxy.capture_pane().await {
                    if content != last_content {
                        let new_part = content
                            .strip_prefix(&last_content)
                            .unwrap_or(&content)
                            .to_string();
                        
                        if !new_part.is_empty() {
                            let _ = tx.send(new_part).await;
                        }
                        
                        last_content = content;
                    }
                }
                
                sleep(Duration::from_millis(100)).await;
            }
        });
    }
    
    // Get receiver for events
    pub fn receiver(&mut self) -> &mut mpsc::Receiver<String> {
        &mut self.rx
    }
}