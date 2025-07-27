# AANY-HUB Terminal Broadcasting System

## Overview

The aany-hub now includes a terminal broadcasting system that allows you to access tmux sessions through a web browser using gRPC communication (not SSH).

## Quick Test

Run the complete test:
```bash
./test_terminal_system.sh
```

This will:
1. Start the hub server (if not running)
2. Create a test agent with tmux session
3. Start the terminal monitor
4. Show you the URLs and commands to test

## Manual Setup

### 1. Start the Hub Server
```bash
./start_hub_background.sh
```

### 2. Connect an Agent
```bash
./connect_agent_to_hub.sh <agent-name>
```

### 3. Access the Web Terminal
1. Open http://localhost:8090/terminal
2. Click on a terminal session card
3. You should see the terminal output

### 4. Send Commands to Tmux
```bash
tmux send-keys -t agent-<agent-name> "echo Hello World" Enter
```

## Architecture

1. **Agent Registration**: Agents register with hub via gRPC
2. **Terminal Monitor**: `monitor_tmux_pane_v2.py` captures tmux output
3. **gRPC Streaming**: Bidirectional stream for terminal I/O
4. **WebSocket Bridge**: Real-time communication with browser
5. **Web Terminal**: xterm.js for terminal emulation

## Files Created/Modified

### New Files:
- `websocket_terminal.py` - WebSocket handler
- `terminal_ui.py` - Web terminal interface  
- `monitor_tmux_pane.py` - Original tmux monitor
- `monitor_tmux_pane_v2.py` - Enhanced monitor with full updates
- `connect_agent_to_hub.sh` - Agent connection script
- `test_terminal_system.sh` - Complete test script

### Modified Files:
- `proto/agent.proto` - Added terminal RPCs
- `grpc_server.py` - Terminal session management
- `main.py` - WebSocket endpoint

## Troubleshooting

### No output in web terminal?
1. Check if monitor is running: `ps aux | grep monitor_tmux_pane`
2. Check hub logs: `tail -f logs/aany-hub.log`
3. Check monitor logs: `tail -f /tmp/monitor_*.log`

### WebSocket disconnecting?
- This is normal behavior when switching between terminals
- The connection is re-established automatically

### Commands to check status:
```bash
# List terminal sessions
curl http://localhost:8090/api/terminals | jq

# List connected agents  
curl http://localhost:8090/api/agents | jq

# View hub logs
tail -f logs/aany-hub.log
```

## Current Status

✅ Hub server with gRPC and HTTP
✅ Agent registration via gRPC
✅ Terminal session creation
✅ Tmux output monitoring
✅ gRPC bidirectional streaming
✅ WebSocket communication
✅ Web terminal UI with xterm.js
✅ Output buffering for late connections
✅ Full terminal updates (v2 monitor)

## Next Steps

The system is functional but could be enhanced with:
1. Input handling from web terminal to tmux
2. Terminal resize support
3. Authentication/authorization
4. Multiple concurrent sessions per agent
5. Session recording/playback