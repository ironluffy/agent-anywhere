#!/bin/bash
# Complete test script for aany-hub terminal broadcasting system

set -e

echo "=== AANY-HUB TERMINAL TEST SYSTEM ==="
echo

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Step 1: Check if hub is running
echo -e "${YELLOW}Step 1: Checking hub status...${NC}"
if [ -f .hub.pid ]; then
    PID=$(cat .hub.pid)
    if ps -p $PID > /dev/null; then
        echo -e "${GREEN}✓ Hub is running (PID: $PID)${NC}"
    else
        echo -e "${RED}✗ Hub is not running. Starting hub...${NC}"
        ./start_hub_background.sh
    fi
else
    echo -e "${RED}✗ Hub is not running. Starting hub...${NC}"
    ./start_hub_background.sh
fi

# Step 2: Kill any existing monitors
echo -e "\n${YELLOW}Step 2: Cleaning up old monitors...${NC}"
pkill -f monitor_tmux_pane || true
sleep 1
echo -e "${GREEN}✓ Cleaned up${NC}"

# Step 3: Create test agent
AGENT_NAME="test-agent-$(date +%s)"
echo -e "\n${YELLOW}Step 3: Creating test agent: $AGENT_NAME${NC}"

# Register agent
echo "Registering agent with hub..."
python3 << EOF
import grpc
import sys
sys.path.insert(0, 'src')
from aany_hub import agent_pb2, agent_pb2_grpc

channel = grpc.insecure_channel('localhost:50052')
stub = agent_pb2_grpc.AgentHubStub(channel)

request = agent_pb2.AgentInfo(
    agent_id='$AGENT_NAME',
    version='1.0.0',
    hostname='test-host',
    capabilities={}
)
response = stub.RegisterAgent(request)
print(f"Registration: {response.success} - {response.message}")
channel.close()
EOF

# Create tmux session
tmux new-session -d -s "agent-$AGENT_NAME" -c "$PWD"
echo -e "${GREEN}✓ Created tmux session: agent-$AGENT_NAME${NC}"

# Step 4: Start monitor v2
echo -e "\n${YELLOW}Step 4: Starting terminal monitor...${NC}"
nohup ./.venv/bin/python ./monitor_tmux_pane_v2.py "$AGENT_NAME" localhost:50052 > /tmp/monitor_${AGENT_NAME}.log 2>&1 &
MONITOR_PID=$!
echo -e "${GREEN}✓ Monitor started (PID: $MONITOR_PID)${NC}"

# Wait for monitor to connect
sleep 2

# Step 5: Check available terminals
echo -e "\n${YELLOW}Step 5: Available terminal sessions:${NC}"
curl -s http://localhost:8090/api/terminals | jq -r '.terminals[] | "  - Session: \(.session_id)\n    Agent: \(.agent_id)\n    Status: \(.status)"'

echo -e "\n${GREEN}=== SETUP COMPLETE ===${NC}"
echo
echo "To test the system:"
echo "1. Open your browser to: http://localhost:8090/terminal"
echo "2. Click on the terminal card for agent: $AGENT_NAME"
echo "3. In another terminal, send commands to the tmux session:"
echo "   tmux send-keys -t agent-$AGENT_NAME 'echo Hello World' Enter"
echo "4. The output should appear in the web terminal!"
echo
echo "Useful commands:"
echo "  - Send text: tmux send-keys -t agent-$AGENT_NAME 'your command' Enter"
echo "  - Attach to tmux: tmux attach -t agent-$AGENT_NAME"
echo "  - View monitor log: tail -f /tmp/monitor_${AGENT_NAME}.log"
echo "  - View hub log: tail -f logs/aany-hub.log"
echo "  - Stop everything: ./stop_hub.sh && tmux kill-session -t agent-$AGENT_NAME"
echo
echo "Agent Name: $AGENT_NAME"
echo "Monitor PID: $MONITOR_PID"