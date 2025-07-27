# aany-hub Terminal Broadcaster

A web-based terminal interface for Agent Anywhere that allows you to view and interact with agent tmux sessions through your browser.

## Quick Start

### 1. Start the Hub Server (Background)

```bash
cd aany-hub
./start_hub_background.sh
```

This starts the hub server with:
- Dashboard: http://localhost:8090
- Terminal UI: http://localhost:8090/terminal  
- gRPC API: localhost:50052

### 2. Create and Connect an Agent

From the main agent-anywhere directory:

```bash
# Build aany if needed
cargo build --bin aany

# Create a new agent
./target/debug/aany pool create my-agent

# Start the agent
./target/debug/aany pool start my-agent

# Connect agent to hub (run inside the agent's tmux session)
./target/debug/aany pool attach my-agent
# Then inside tmux:
bash ~/Gitrepo/agent-anywhere/aany-hub/connect_agent_to_hub.sh
```

### 3. Access the Web Terminal

1. Open http://localhost:8090/terminal in your browser
2. Click on your agent's terminal session
3. Start typing commands in the tmux session - they appear in real-time!

## Test Script

Run the complete test to verify everything works:

```bash
cd aany-hub
./test_complete.sh
```

This will:
- Start the hub server
- Create a demo agent
- Connect it to the hub
- Send test commands
- Open the terminal UI

## How It Works

1. **Hub Server**: A Python FastAPI server that provides:
   - gRPC API for agent communication
   - WebSocket API for browser clients
   - Web dashboard and terminal UI

2. **Agent Connection**: When an agent connects:
   - Registers with hub via gRPC
   - Creates a terminal session
   - Starts a monitor that streams tmux pane content

3. **Terminal Streaming**:
   - Monitor captures tmux pane output every 500ms
   - Sends updates to hub via gRPC streaming
   - Hub broadcasts to connected WebSocket clients
   - Browser renders using xterm.js

## Manual Steps Summary

1. **Start Hub**:
   ```bash
   cd aany-hub
   ./start_hub_background.sh
   ```

2. **Use aany UI**:
   ```bash
   cd agent-anywhere
   ./target/debug/aany pool ui
   ```
   - Press 'n' to create new agent
   - Enter a name
   - Select agent type
   - Press Enter to attach

3. **Connect Agent to Hub** (inside tmux):
   ```bash
   bash ~/Gitrepo/agent-anywhere/aany-hub/connect_agent_to_hub.sh
   ```

4. **View in Browser**:
   - Open http://localhost:8090/terminal
   - Click on your agent session
   - Watch your commands appear!

## Stopping Services

```bash
# Stop agent
./target/debug/aany pool stop <agent-name>

# Stop hub
cd aany-hub
./stop_hub.sh
```

## Troubleshooting

**Agent not showing in hub:**
- Check hub logs: `tail -f aany-hub/logs/aany-hub.log`
- Verify agent connected: Look for "✓ Agent registered" in tmux

**Terminal not updating:**
- Check monitor process: `ps aux | grep monitor_tmux_pane`
- View monitor logs: `tail -f /tmp/monitor_<agent-name>.log`

**Connection errors:**
- Ensure hub is running: `curl http://localhost:8090`
- Check gRPC port: `lsof -i :50052`

## Architecture

```
Agent (tmux)          Hub Server           Browser
    │                     │                   │
    ├──register──────────►│                   │
    ├──create session────►│                   │
    │                     │◄──────GET─────────┤
    ├──stream output─────►├──WebSocket──────►│
    │                     │                   │
    └─────────────────────┴───────────────────┘
```

## Files

- `start_hub_background.sh` - Start hub as background service
- `stop_hub.sh` - Stop the hub
- `connect_agent_to_hub.sh` - Connect agent to hub
- `monitor_tmux_pane.py` - Terminal output monitor
- `test_complete.sh` - Full test script
- `terminal_ui.py` - Web terminal interface