#!/bin/bash
# Script to connect an agent to the hub
# This should be run inside the tmux session when an agent starts

# Extract agent name from tmux session name (format: agent-<name>)
TMUX_SESSION=$(tmux display-message -p '#S' 2>/dev/null)
if [[ $TMUX_SESSION == agent-* ]]; then
    AGENT_NAME="${TMUX_SESSION#agent-}"
else
    AGENT_NAME="${1:-unknown}"
fi

HUB_URL="${2:-localhost:50052}"

echo "Connecting agent '$AGENT_NAME' to hub at $HUB_URL..."

# Export environment variables for the agent
export AANY_AGENT_ID="$AGENT_NAME"
export AANY_HUB_URL="$HUB_URL"

# Create a simple Python script to register with hub
cat > /tmp/register_agent.py << 'EOF'
import grpc
import sys
import os
import time
import socket

# Add the aany-hub path to import proto files
script_path = os.path.expanduser('~/Gitrepo/agent-anywhere/aany-hub/src')
if os.path.exists(script_path):
    sys.path.insert(0, script_path)
else:
    print(f"Warning: Path {script_path} not found")
    sys.exit(1)

from aany_hub import agent_pb2, agent_pb2_grpc

def register_with_hub():
    agent_id = os.environ.get('AANY_AGENT_ID', 'unknown')
    hub_url = os.environ.get('AANY_HUB_URL', 'localhost:50052')
    
    try:
        # Connect to hub
        channel = grpc.insecure_channel(hub_url)
        stub = agent_pb2_grpc.AgentHubStub(channel)
        
        # Register agent
        agent_info = agent_pb2.AgentInfo(
            agent_id=agent_id,
            version="1.0.0",
            hostname=socket.gethostname(),
            capabilities={
                "type": "tmux-agent",
                "terminal": "true"
            }
        )
        
        response = stub.RegisterAgent(agent_info)
        if response.success:
            print(f"✓ Agent '{agent_id}' registered with hub")
            print(f"  Session ID: {response.session_id}")
            
            # Create terminal session
            terminal_request = agent_pb2.CreateTerminalRequest(
                agent_id=agent_id,
                tmux_session_name=f"agent-{agent_id}",
                window=0,
                pane=0,
                metadata={}
            )
            
            terminal_response = stub.CreateTerminalSession(terminal_request)
            if terminal_response.success:
                print(f"✓ Terminal session created: {terminal_response.session_id}")
            else:
                print(f"✗ Failed to create terminal session: {terminal_response.message}")
                
        else:
            print(f"✗ Failed to register: {response.message}")
            
    except Exception as e:
        print(f"✗ Error connecting to hub: {e}")
        return False
    
    return True

if __name__ == "__main__":
    register_with_hub()
EOF

# Run the registration script using hub's virtual environment
HUB_DIR="$(dirname "$0")"
if [ -f "$HUB_DIR/.venv/bin/python" ]; then
    "$HUB_DIR/.venv/bin/python" /tmp/register_agent.py
else
    python3 /tmp/register_agent.py
fi

# Clean up
rm -f /tmp/register_agent.py

# Start the tmux pane monitor in background
echo "Starting tmux pane monitor..."
if [ -f "$HUB_DIR/.venv/bin/python" ]; then
    nohup "$HUB_DIR/.venv/bin/python" "$HUB_DIR/monitor_tmux_pane.py" "$AGENT_NAME" "$HUB_URL" > /tmp/monitor_${AGENT_NAME}.log 2>&1 &
else
    nohup python3 ~/Gitrepo/agent-anywhere/aany-hub/monitor_tmux_pane.py "$AGENT_NAME" "$HUB_URL" > /tmp/monitor_${AGENT_NAME}.log 2>&1 &
fi
MONITOR_PID=$!
echo "Monitor started with PID: $MONITOR_PID"

# Save PID for cleanup (create directory if needed)
AGENT_DIR="$HOME/.aany/pool/agents/${AGENT_NAME}"
if [ -d "$AGENT_DIR" ]; then
    echo $MONITOR_PID > "$AGENT_DIR/.monitor.pid"
fi

echo "Agent connection completed."