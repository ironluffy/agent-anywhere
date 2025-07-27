#!/bin/bash
# Test script that creates a tmux agent and simulates keyboard typing

echo "🎹 Agent Anywhere Keyboard Simulation Test"
echo "=========================================="

# Check dependencies
if ! command -v tmux &> /dev/null; then
    echo "❌ Error: tmux is not installed"
    exit 1
fi

# Kill any existing test sessions
tmux kill-session -t keyboard-test 2>/dev/null
tmux kill-session -t aany-hub-test 2>/dev/null

# Start aany-hub server in background tmux session
echo "1. Starting aany-hub server..."
tmux new-session -d -s aany-hub-test -c /Users/hsuh/Gitrepo/agent-anywhere/aany-hub \
    "source .venv/bin/activate && pip install -q grpcio grpcio-tools && python src/aany_hub/main.py 2>&1 | tee hub.log"

# Wait for hub to start
echo "   Waiting for hub to start..."
sleep 3

# Verify hub is running
if curl -s http://localhost:8000 > /dev/null; then
    echo "✅ Hub is running at http://localhost:8000"
else
    echo "❌ Hub failed to start. Check logs:"
    tmux capture-pane -t aany-hub-test -p
    exit 1
fi

# Create test tmux session for keyboard simulation
echo ""
echo "2. Creating tmux session 'keyboard-test'..."
tmux new-session -d -s keyboard-test

# Create simulation script
cat > /tmp/run_agent_simulation.rs << 'EOF'
use aany_tmux::{ModularTmuxProxy, ModularProxyConfig, TmuxProxy};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Starting keyboard simulation agent...");
    
    // Configure hub connector
    let mut modules_config = HashMap::new();
    let hub_config = serde_json::json!({
        "enabled": true,
        "hub_url": "localhost:50052",
    });
    modules_config.insert("hub_connector".to_string(), hub_config);
    
    let config = ModularProxyConfig {
        session_name: "keyboard-test".to_string(),
        window: 0,
        pane: 0,
        agent_id: "keyboard-agent-001".to_string(),
        modules_config,
    };
    
    // Create modular proxy with hub connection
    let mut proxy = ModularTmuxProxy::new(config)?;
    let tmux = proxy.proxy();
    
    println!("📝 Simulating keyboard typing...");
    
    // Simulate typing character by character
    let messages = vec![
        ("Hello from Agent Anywhere!", 50),
        ("I am typing like a human...", 75),
        ("Watch me execute commands:", 60),
    ];
    
    for (message, delay_ms) in messages {
        println!("   Typing: {}", message);
        
        // Type each character with delay
        for ch in message.chars() {
            tmux.send_keys(&ch.to_string())?;
            thread::sleep(Duration::from_millis(delay_ms));
        }
        
        // Press Enter
        tmux.send_keys("Enter")?;
        thread::sleep(Duration::from_millis(500));
        
        // Capture and log the output
        let output = tmux.capture_pane()?;
        println!("   Output captured: {} chars", output.len());
    }
    
    // Execute some commands
    println!("\n🚀 Executing commands...");
    
    let commands = vec![
        "date",
        "echo 'Agent Anywhere is awesome!'",
        "ls -la | head -5",
        "ps aux | grep tmux | head -3",
    ];
    
    for cmd in commands {
        println!("   Running: {}", cmd);
        proxy.send_command(cmd)?;
        thread::sleep(Duration::from_millis(1000));
    }
    
    // Simulate an error
    println!("\n⚠️  Simulating error condition...");
    proxy.report_error("Simulated keyboard input error!");
    
    // Final message
    thread::sleep(Duration::from_millis(500));
    proxy.send_command("echo '✅ Simulation complete!'")?;
    
    println!("\n✨ Keyboard simulation finished!");
    println!("Check the hub dashboard for all logged events!");
    
    Ok(())
}
EOF

# Build and run the simulation
echo ""
echo "3. Building simulation agent..."
cd /Users/hsuh/Gitrepo/agent-anywhere/aany-tmux
source ~/.cargo/env

# Create a simple Rust project for the simulation
cat > examples/keyboard_simulation.rs << 'EOF'
use aany_tmux::{ModularTmuxProxy, ModularProxyConfig, TmuxProxy};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Starting keyboard simulation agent...");
    
    // Configure hub connector
    let mut modules_config = HashMap::new();
    let hub_config = serde_json::json!({
        "enabled": true,
        "hub_url": "localhost:50052",
    });
    modules_config.insert("hub_connector".to_string(), hub_config);
    
    let config = ModularProxyConfig {
        session_name: "keyboard-test".to_string(),
        window: 0,
        pane: 0,
        agent_id: "keyboard-agent-001".to_string(),
        modules_config,
    };
    
    // Create modular proxy with hub connection
    let mut proxy = ModularTmuxProxy::new(config)?;
    let tmux = proxy.proxy();
    
    println!("📝 Simulating keyboard typing...");
    
    // Clear the pane first
    tmux.clear()?;
    thread::sleep(Duration::from_millis(500));
    
    // Simulate typing character by character
    let messages = vec![
        ("Hello from Agent Anywhere!", 50),
        ("I am typing like a human...", 75),
        ("Watch me execute commands:", 60),
    ];
    
    for (message, delay_ms) in messages {
        println!("   Typing: {}", message);
        
        // Type each character with delay
        for ch in message.chars() {
            tmux.send_keys(&ch.to_string())?;
            thread::sleep(Duration::from_millis(delay_ms));
        }
        
        // Press Enter
        tmux.send_keys("Enter")?;
        thread::sleep(Duration::from_millis(500));
    }
    
    // Execute some commands using the proxy (with logging)
    println!("\n🚀 Executing commands...");
    
    let commands = vec![
        "date",
        "echo 'Agent Anywhere is awesome!'",
        "ls -la | head -5",
        "ps aux | grep tmux | head -3",
    ];
    
    for cmd in commands {
        println!("   Running: {}", cmd);
        let output = proxy.send_command(cmd)?;
        
        // Show a preview of the output
        let preview = output.lines()
            .skip_while(|line| line.is_empty() || line.contains(cmd))
            .take(3)
            .collect::<Vec<_>>()
            .join("\n");
        
        if !preview.is_empty() {
            println!("   Preview: {}", preview);
        }
        
        thread::sleep(Duration::from_millis(1000));
    }
    
    // Simulate an error
    println!("\n⚠️  Simulating error condition...");
    proxy.report_error("Simulated keyboard input error!");
    thread::sleep(Duration::from_millis(500));
    
    // Final message with typing simulation
    println!("\n📝 Typing final message...");
    let final_msg = "echo '✅ Simulation complete! Check the hub logs!'";
    for ch in final_msg.chars() {
        tmux.send_keys(&ch.to_string())?;
        thread::sleep(Duration::from_millis(30));
    }
    tmux.send_keys("Enter")?;
    thread::sleep(Duration::from_millis(1000));
    
    println!("\n✨ Keyboard simulation finished!");
    
    Ok(())
}
EOF

echo "4. Running keyboard simulation..."
echo "   (This will type slowly to simulate human input)"
echo ""

cargo run --example keyboard_simulation

echo ""
echo "5. Fetching results from hub..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Show hub logs
echo "📊 Hub Server Logs:"
tmux capture-pane -t aany-hub-test -p | tail -20

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Show tmux session content
echo "🖥️  Tmux Session Content:"
tmux capture-pane -t keyboard-test -p | tail -30

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "📌 Test Complete!"
echo ""
echo "View live sessions:"
echo "  - Hub logs:    tmux attach -t aany-hub-test"
echo "  - Agent view:  tmux attach -t keyboard-test"
echo "  - Dashboard:   http://localhost:8000"
echo ""
echo "Cleanup:"
echo "  ./cleanup_test.sh"

# Create cleanup script
cat > /Users/hsuh/Gitrepo/agent-anywhere/cleanup_test.sh << 'EOF'
#!/bin/bash
echo "🧹 Cleaning up test sessions..."
tmux kill-session -t keyboard-test 2>/dev/null
tmux kill-session -t aany-hub-test 2>/dev/null
rm -f /Users/hsuh/Gitrepo/agent-anywhere/aany-hub/hub.log
echo "✅ Cleanup complete!"
EOF

chmod +x /Users/hsuh/Gitrepo/agent-anywhere/cleanup_test.sh