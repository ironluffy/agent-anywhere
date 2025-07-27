# aany-hub Terminal Broadcaster

This implements a web-based terminal interface for aany agents, allowing you to view and interact with tmux sessions through your browser.

## Quick Start

### 1. Start the Hub Server

```bash
cd aany-hub
./start_hub_background.sh
```

The server will start on:
- Dashboard: http://localhost:8090
- Terminal UI: http://localhost:8090/terminal
- gRPC: localhost:50052

### 2. Create a Hub-Connected Agent

```bash
cd ~/Gitrepo/agent-anywhere
./aany pool create my-agent --template hub-connected
./aany pool start my-agent
```

### 3. Access the Terminal UI

Open http://localhost:8090/terminal in your browser. You should see:
- Your agent listed as a terminal session
- Click on it to open the terminal interface
- Type commands and see output in real-time

## How It Works

1. **Agent Creation**: When you create an agent with the `hub-connected` template, it includes initialization commands that:
   - Register the agent with the hub
   - Start a monitor process that streams tmux pane content

2. **gRPC Communication**: The agent and hub communicate via gRPC:
   - Agent registers itself on startup
   - Creates a terminal session
   - Streams terminal output to hub
   - Receives input commands from hub

3. **WebSocket Bridge**: The hub bridges gRPC to WebSocket:
   - Terminal data from agents is forwarded to web clients
   - User input from web is sent back to agents

4. **Web Terminal**: Uses xterm.js for a full terminal experience

## Testing

Run the ultimate test to verify everything works:

```bash
cd aany-hub
./ultimate_test.sh
```

This will:
1. Start the hub server
2. Create a test agent
3. Verify registration
4. Send test commands
5. Guide you to verify the output

## Manual Testing

1. Start hub server:
   ```bash
   cd aany-hub
   ./start_hub_background.sh
   ```

2. Create agent:
   ```bash
   cd ~/Gitrepo/agent-anywhere
   ./aany pool create test-agent --template hub-connected
   ```

3. Start agent:
   ```bash
   ./aany pool start test-agent
   ```

4. Open terminal UI:
   - Go to http://localhost:8090/terminal
   - Click on the test-agent session

5. Attach to agent locally:
   ```bash
   ./aany pool attach test-agent
   ```

6. Type commands in the local tmux session and watch them appear in the web terminal!

## Stopping Services

```bash
# Stop hub server
cd aany-hub
./stop_hub.sh

# Stop agent
cd ~/Gitrepo/agent-anywhere
./aany pool stop test-agent
```

## Troubleshooting

1. **Agent not showing in Connected Agents**:
   - Check hub server logs: `tail -f aany-hub/logs/aany-hub.log`
   - Verify agent init script ran: Check tmux session for connection messages

2. **Terminal not updating**:
   - Check monitor process: `ps aux | grep monitor_tmux_pane`
   - View monitor logs: `tail -f /tmp/monitor_<agent-name>.log`

3. **Can't connect to hub**:
   - Verify hub is running: `curl http://localhost:8090`
   - Check gRPC port: `lsof -i :50052`

## Architecture

```
┌─────────────────┐     gRPC      ┌─────────────────┐    WebSocket    ┌─────────────────┐
│   tmux-agent    │ ◄──────────► │    aany-hub     │ ◄─────────────► │   Web Browser   │
│                 │               │                 │                 │                 │
│ - tmux session  │               │ - gRPC server   │                 │ - xterm.js      │
│ - monitor.py    │               │ - WebSocket srv │                 │ - Terminal UI   │
└─────────────────┘               └─────────────────┘                 └─────────────────┘
```